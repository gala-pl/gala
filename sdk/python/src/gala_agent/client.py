"""GalaAgent: wraps the gala CLI. Each call writes the source to a temp file, runs the CLI,
and parses its output."""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Union

# <repo>/sdk/python/src/gala_agent/client.py -> parents[4] is the repo root.
_DEFAULT_GALA_BIN = Path(__file__).resolve().parents[4] / "target" / "debug" / "gala"



class GalaError(RuntimeError):
    """Raised when the gala CLI cannot produce a usable result."""


@dataclass
class CheckResult:
    ok: bool
    errors: int
    warnings: int
    diagnostics: List[Dict[str, Any]]
    schema: str = "gala.check.v1"


@dataclass
class RunResult:
    ok: bool
    exit_code: Optional[int]
    output: List[str] = field(default_factory=list)
    result: Optional[str] = None
    shots: Optional[int] = None
    histogram: Dict[str, int] = field(default_factory=dict)
    stderr: str = ""
    error: Optional[str] = None


@dataclass
class GradResult:
    """supported=True with `value` (the parameter-shift gradient) when the CLI computed it."""

    supported: bool
    reason: Optional[str] = None
    value: Optional[float] = None
    occurrences: Optional[int] = None
    submissions: Optional[int] = None


@dataclass
class PropertyVerification:
    """status: "passed" (all requested ran and held), "failed" (one did not hold), "unsupported"
    (a requested kind has no runner and is listed in `skipped`), or "error" (does not compile)."""

    status: str
    requested: List[str]
    results: List[Dict[str, Any]]
    skipped: List[Dict[str, Any]]
    diagnostics: List[Dict[str, Any]]


def parse_run_stdout(stdout: str) -> Dict[str, Any]:
    """Splits `gala run` stdout into print lines, the result line, and the histogram."""
    text = stdout[:-1] if stdout.endswith("\n") else stdout
    lines = text.split("\n") if text else []
    output: List[str] = []
    histogram: Dict[str, int] = {}
    result: Optional[str] = None
    shots: Optional[int] = None
    for line in lines:
        result_match = re.fullmatch(r"result: (.*)", line)
        if result_match:
            result = result_match.group(1)
            continue
        header = re.fullmatch(r"results over (\d+) shots:", line.strip())
        if header:
            shots = int(header.group(1))
            continue
        bucket = re.fullmatch(r"\s+(.+): (\d+)", line) if shots is not None else None
        if bucket:
            histogram[bucket.group(1)] = int(bucket.group(2))
            continue
        output.append(line)
    return {"output": output, "result": result, "shots": shots, "histogram": histogram}


def _sanitize_name(name: str) -> str:
    base = re.split(r"[\\/]", name)[-1]
    safe = re.sub(r"[^A-Za-z0-9_.-]", "_", base) or "input.gala"
    return safe if safe.endswith(".gala") else f"{safe}.gala"


class GalaAgent:
    """Agent-facing wrapper around the gala CLI.

    The binary is taken from ``gala_bin``, then $GALA_BIN, then <repo>/target/debug/gala.
    """

    def __init__(self, gala_bin: Optional[str] = None, timeout_s: float = 60.0) -> None:
        self.gala_bin = gala_bin or os.environ.get("GALA_BIN") or str(_DEFAULT_GALA_BIN)
        self.timeout_s = timeout_s

    def check(self, code: str, file_name: Optional[str] = None) -> CheckResult:
        """Type-checks Gala source and returns structured diagnostics."""
        with self._source_file(code, file_name) as path:
            proc = self._exec(["check", str(path), "--json"])
        doc = _parse_json_object(proc.stdout)
        if doc is None or doc.get("schema") != "gala.check.v1" or not isinstance(
            doc.get("diagnostics"), list
        ):
            raise GalaError(self._describe_failure("check", proc))
        return CheckResult(
            ok=bool(doc.get("ok")),
            errors=int(doc.get("errors", 0)),
            warnings=int(doc.get("warnings", 0)),
            diagnostics=list(doc["diagnostics"]),
            schema=str(doc["schema"]),
        )

    def run(self, code: str, shots: Optional[int] = None, seed: Optional[int] = None) -> RunResult:
        """Runs the entry point on the simulator. Failures are reported in the result, not raised."""
        extra: List[str] = []
        if shots is not None:
            extra += ["--shots", str(shots)]
        if seed is not None:
            extra += ["--seed", str(seed)]
        with self._source_file(code, None) as path:
            proc = self._exec(["run", str(path), *extra])
        stderr = proc.stderr.strip()
        if proc.spawn_error or proc.timed_out or proc.returncode != 0:
            return RunResult(
                ok=False,
                exit_code=proc.returncode,
                stderr=stderr,
                error=self._describe_failure("run", proc),
            )
        return RunResult(ok=True, exit_code=0, stderr=stderr, **parse_run_stdout(proc.stdout))

    def grad(
        self,
        code: str,
        fn: str,
        wrt: int,
        args: Sequence[float] = (),
        shots: int = 20000,
        seed: int = 0,
    ) -> GradResult:
        """Parameter-shift gradient of `fn`'s measured expectation with respect to argument `wrt`.

        Runs `gala grad`; the value is shot-estimated, so it carries sampling noise.
        """
        with self._source_file(code, None) as path:
            proc = self._exec(
                ["grad", str(path), fn, "--wrt", str(wrt), "--shots", str(shots),
                 "--seed", str(seed), "--args", *[str(a) for a in args]]
            )
        doc = _parse_json_object(proc.stdout)
        if proc.returncode != 0 or doc is None or not isinstance(doc.get("value"), (int, float)):
            return GradResult(supported=False, reason=self._describe_failure("grad", proc))
        return GradResult(
            supported=True,
            value=float(doc["value"]),
            occurrences=doc.get("occurrences"),
            submissions=doc.get("submissions"),
        )

    def verify_properties(self, code: str, properties: Sequence[str]) -> PropertyVerification:
        """Runs the `#[property(...)]` functions through `gala test --property --json` and reports
        only the requested kinds. Kinds the CLI cannot run are "unsupported", never passing."""
        check = self.check(code)
        if not check.ok:
            return PropertyVerification(
                status="error",
                requested=list(properties),
                results=[],
                skipped=[],
                diagnostics=check.diagnostics,
            )
        with self._source_file(code, None) as path:
            proc = self._exec(["test", str(path), "--property", "--json"])
        doc = _parse_json_object(proc.stdout)
        if doc is None:
            raise RuntimeError(self._describe_failure("test --property", proc))
        wanted = set(properties)
        results = [r for r in doc.get("results", []) if r.get("property") in wanted]
        skipped = [s for s in doc.get("skipped", []) if s.get("property") in wanted]
        if any(not r.get("passed") for r in results):
            status = "failed"
        elif skipped:
            status = "unsupported"
        else:
            status = "passed"
        return PropertyVerification(
            status=status,
            requested=list(properties),
            results=results,
            skipped=skipped,
            diagnostics=[],
        )

    # ----- internals -----

    def _source_file(self, code: str, file_name: Optional[str]) -> "_SourceFile":
        return _SourceFile(code, _sanitize_name(file_name or "input.gala"))

    def _exec(self, args: List[str]) -> "_Proc":
        if shutil.which(self.gala_bin) is None and not Path(self.gala_bin).exists():
            return _Proc(returncode=None, stdout="", stderr="", spawn_error=f"{self.gala_bin} not found")
        try:
            done = subprocess.run(
                [self.gala_bin, *args],
                capture_output=True,
                text=True,
                timeout=self.timeout_s,
                check=False,
            )
        except subprocess.TimeoutExpired:
            return _Proc(returncode=None, stdout="", stderr="", timed_out=True)
        except OSError as err:
            return _Proc(returncode=None, stdout="", stderr="", spawn_error=str(err))
        return _Proc(returncode=done.returncode, stdout=done.stdout, stderr=done.stderr)

    def _describe_failure(self, what: str, proc: "_Proc") -> str:
        if proc.spawn_error:
            return f"gala {what}: could not run {self.gala_bin}: {proc.spawn_error}"
        if proc.timed_out:
            return f"gala {what}: timed out after {self.timeout_s} s"
        detail = (proc.stderr or proc.stdout).strip()
        return f"gala {what} failed (exit {proc.returncode})" + (f": {detail}" if detail else "")


@dataclass
class _Proc:
    returncode: Optional[int]
    stdout: str
    stderr: str
    spawn_error: Optional[str] = None
    timed_out: bool = False


class _SourceFile:
    """Context manager: writes source into a fresh temp directory and removes it on exit."""

    def __init__(self, code: str, name: str) -> None:
        self._code = code
        self._name = name
        self._dir: Optional[tempfile.TemporaryDirectory] = None

    def __enter__(self) -> Path:
        self._dir = tempfile.TemporaryDirectory(prefix="gala-agent-")
        path = Path(self._dir.name) / self._name
        path.write_text(self._code, encoding="utf-8")
        return path

    def __exit__(self, *exc: object) -> None:
        if self._dir is not None:
            self._dir.cleanup()


def _parse_json_object(text: str) -> Optional[Dict[str, Any]]:
    try:
        value = json.loads(text)
    except (ValueError, TypeError):
        return None
    return value if isinstance(value, dict) else None
