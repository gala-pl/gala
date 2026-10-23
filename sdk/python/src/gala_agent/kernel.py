"""A Jupyter kernel for Gala, built on the `gala` CLI.

Cells are Gala source. Function definitions (`fn`, `import`) accumulate across cells, and an
expression is evaluated against them and printed. The magic `%run <function> [args...]` runs a
probabilistic function many times and shows its outcome histogram as an inline bar chart.

Install the kernel spec with `python -m gala_agent.kernel --install`. Set GALA_BIN to the `gala`
binary if it is not at target/debug/gala in this repository.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from html import escape
from pathlib import Path
from typing import Dict, List, Optional, Tuple

from ipykernel.kernelapp import IPKernelApp
from ipykernel.kernelbase import Kernel

from .bridge import _gala_binary

KERNEL_NAME = "gala"


def _histogram_html(histogram: Dict[str, int], title: str) -> str:
    total = sum(histogram.values()) or 1
    rows = []
    for outcome in sorted(histogram):
        count = histogram[outcome]
        pct = 100.0 * count / total
        rows.append(
            "<tr><td>{}</td><td style='width:60%'><div style='background:#4a7fd6;height:14px;"
            "width:{:.1f}%'></div></td><td>{} ({:.1f}%)</td></tr>".format(
                escape(outcome), pct, count, pct
            )
        )
    return (
        "<div><b>{}</b><table style='font-family:monospace;font-size:13px'>{}</table></div>".format(
            escape(title), "".join(rows)
        )
    )


class GalaKernel(Kernel):
    implementation = "gala"
    implementation_version = "0.1.0"
    language = "gala"
    language_version = "0.1.0"
    language_info = {
        "name": "gala",
        "mimetype": "text/x-gala",
        "file_extension": ".gala",
    }
    banner = "Gala kernel: definitions persist across cells; %run <fn> shows a histogram."

    def __init__(self, **kwargs) -> None:
        super().__init__(**kwargs)
        self.definitions: List[str] = []
        self.gala = _gala_binary(os.environ.get("GALA_BIN"))

    # ----- helpers -----

    def _program(self, extra: str = "") -> Path:
        """Write the accumulated definitions plus `extra` to a temporary source file."""
        text = "\n\n".join(self.definitions) + "\n" + extra
        handle = tempfile.NamedTemporaryFile("w", suffix=".gala", delete=False)
        handle.write(text)
        handle.close()
        return Path(handle.name)

    def _gala(self, args: List[str]) -> Tuple[int, str, str]:
        proc = subprocess.run([self.gala, *args], capture_output=True, text=True, timeout=600)
        return proc.returncode, proc.stdout, proc.stderr

    def _say(self, text: str) -> None:
        self.send_response(self.iopub_socket, "stream", {"name": "stdout", "text": text})

    def _error(self, text: str) -> None:
        self.send_response(self.iopub_socket, "stream", {"name": "stderr", "text": text})

    # ----- execution -----

    def do_execute(self, code, silent, store_history=True, user_expressions=None, allow_stdin=False):  # type: ignore[override]
        status, payload = "ok", []
        for line in code.split("\n"):
            line = line.strip()
            if not line or line.startswith("//"):
                continue
            if line.startswith("%run"):
                ok = self._run_magic(line)
            elif line.startswith(("fn ", "import ")):
                # A definition cell is one unit: it is checked and stored as a whole.
                ok = self._define(code)
                if not ok:
                    status = "error"
                break
            else:
                ok = self._evaluate(line)
            if not ok:
                status = "error"
                break
        return {"status": status, "execution_count": self.execution_count, "payload": payload,
                "user_expressions": {}}

    def _define(self, code: str) -> bool:
        """Add definitions from a cell after checking that the program still compiles."""
        candidate = "\n\n".join(self.definitions + [code])
        path = self._program()
        path.write_text(candidate, encoding="utf-8")
        rc, out, err = self._gala(["check", str(path)])
        if rc != 0:
            self._error(out or err)
            return False
        self.definitions.append(code)
        return True

    def _evaluate(self, expr: str) -> bool:
        source = "fn main() -> Int prob {\n    print(" + expr + ")\n    0\n}\n"
        path = self._program(source)
        rc, out, err = self._gala(["run", str(path)])
        if rc != 0:
            self._error(err or out)
            return False
        lines = [l for l in out.splitlines() if not l.startswith("results over") and not l.startswith("result:")]
        for l in lines:
            if l.startswith("  "):
                continue
            self._say(l + "\n")
        return True

    def _run_magic(self, line: str) -> bool:
        parts = line.split()
        if len(parts) < 2:
            self._error("usage: %run <function> [args...] [--shots N]\n")
            return False
        function = parts[1]
        shots = 4000
        args: List[str] = []
        rest = parts[2:]
        i = 0
        while i < len(rest):
            if rest[i] == "--shots" and i + 1 < len(rest):
                shots = int(rest[i + 1])
                i += 2
                continue
            args.append(rest[i])
            i += 1
        path = self._program()
        rc, out, err = self._gala(["call", str(path), function, "--args", *args, "--shots", str(shots)])
        if rc != 0:
            self._error(err or out)
            return False
        doc = json.loads(out)
        histogram = doc.get("histogram", {})
        self.send_response(
            self.iopub_socket,
            "display_data",
            {
                "data": {
                    "text/html": _histogram_html(histogram, f"{function}({', '.join(args)}), {shots} shots"),
                    "text/plain": json.dumps(histogram, sort_keys=True),
                },
                "metadata": {},
            },
        )
        return True


def install_kernelspec(prefix: Optional[str] = None) -> str:
    """Write the kernel spec so Jupyter can find the Gala kernel."""
    from jupyter_client.kernelspec import KernelSpecManager

    package_root = str(Path(__file__).resolve().parents[1])
    spec = {
        "argv": [sys.executable, "-m", "gala_agent.kernel", "-f", "{connection_file}"],
        "display_name": "Gala",
        "language": "gala",
        "env": {"PYTHONPATH": package_root, "GALA_BIN": _gala_binary(os.environ.get("GALA_BIN"))},
    }
    directory = tempfile.mkdtemp(prefix="gala-kernel-")
    Path(directory, "kernel.json").write_text(json.dumps(spec, indent=2), encoding="utf-8")
    return KernelSpecManager().install_kernel_spec(directory, kernel_name=KERNEL_NAME, prefix=prefix, user=prefix is None)


if __name__ == "__main__":
    if "--install" in sys.argv:
        print(install_kernelspec())
    else:
        IPKernelApp.launch_instance(kernel_class=GalaKernel)
