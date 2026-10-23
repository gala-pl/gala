"""Call Gala kernels from Python, and from PyTorch autograd.

A Gala function returning `Measured<Bool>` defines an expectation value E = P(false) - P(true)
(the Pauli-Z expectation of its measured qubit). `GalaKernel.expectation` estimates E from shots
with `gala call`. `GalaKernel.gradient` returns the parameter-shift gradient from `gala grad`.

`GalaExpectation` wires both into PyTorch, so a PyTorch optimizer can train a Gala kernel:

    kernel = GalaKernel("classifier.gala", "classify")
    theta = torch.nn.Parameter(torch.tensor(0.3))
    e = GalaExpectation.apply(kernel, [x], theta, 1)   # theta is argument 1
    loss = ((1 - e) / 2 - y) ** 2

PyTorch is optional: importing this module without it still gives `GalaKernel`.
"""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Optional, Sequence

from .client import GalaError

try:  # PyTorch is optional.
    import torch
except ImportError:  # pragma: no cover - exercised only without torch
    torch = None  # type: ignore[assignment]


def _describe_error_message(command: str, stderr: str, code: int) -> str:
    detail = stderr.strip() or f"exit code {code}"
    return f"gala {command} failed: {detail}"


def _gala_binary(gala_bin: Optional[str]) -> str:
    if gala_bin:
        return gala_bin
    if os.environ.get("GALA_BIN"):
        return os.environ["GALA_BIN"]
    return str(Path(__file__).resolve().parents[3] / "target" / "debug" / "gala")


class GalaKernel:
    """A Gala function, called through the `gala` CLI."""

    def __init__(self, source: str | Path, function: str, gala_bin: Optional[str] = None) -> None:
        self.source = str(source)
        self.function = function
        self.gala_bin = _gala_binary(gala_bin)

    def _run(self, args: Sequence[str]) -> dict:
        proc = subprocess.run(
            [self.gala_bin, *args], capture_output=True, text=True, timeout=600, check=False
        )
        try:
            doc = json.loads(proc.stdout)
        except json.JSONDecodeError:
            raise GalaError(_describe_error_message(args[0], proc.stderr, proc.returncode)) from None
        if proc.returncode != 0:
            raise GalaError(_describe_error_message(args[0], proc.stderr, proc.returncode))
        return doc

    def expectation(self, args: Sequence[float], shots: int = 4000, seed: int = 0) -> float:
        """E = P(false) - P(true), estimated from `shots` runs."""
        doc = self._run(
            ["call", self.source, self.function, "--args", *map(str, args),
             "--shots", str(shots), "--seed", str(seed)]
        )
        hist = doc.get("histogram", {})
        total = sum(hist.values()) or 1
        return (hist.get("false", 0) - hist.get("true", 0)) / total

    def gradient(self, args: Sequence[float], wrt: int, shots: int = 20000, seed: int = 0) -> float:
        """Parameter-shift derivative of E with respect to argument `wrt`."""
        doc = self._run(
            ["grad", self.source, self.function, "--wrt", str(wrt),
             "--args", *map(str, args), "--shots", str(shots), "--seed", str(seed)]
        )
        return float(doc["value"])


if torch is not None:

    class GalaExpectation(torch.autograd.Function):
        """E(args) as a differentiable PyTorch op. `theta` is the argument at position `wrt`."""

        @staticmethod
        def forward(ctx, kernel: GalaKernel, fixed: list, theta, wrt: int, shots: int):  # type: ignore[override]
            args = list(fixed)
            args.insert(wrt, float(theta))
            ctx.kernel = kernel
            ctx.args = args
            ctx.wrt = wrt
            ctx.shots = shots
            return torch.tensor(kernel.expectation(args, shots=shots), dtype=torch.float64)

        @staticmethod
        def backward(ctx, grad_output):  # type: ignore[override]
            d = ctx.kernel.gradient(ctx.args, ctx.wrt)
            return None, None, grad_output * d, None, None

    def expectation(kernel: GalaKernel, fixed: Sequence[float], theta, wrt: int, shots: int = 4000):
        """Torch-friendly wrapper: `fixed` are the other arguments, `theta` is the trainable one."""
        return GalaExpectation.apply(kernel, list(fixed), theta, wrt, shots)
