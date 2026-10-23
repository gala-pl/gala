"""Agent-facing SDK for the Gala language. Wraps the `gala` CLI."""

from .client import (
    CheckResult,
    GalaAgent,
    GalaError,
    GradResult,
    PropertyVerification,
    RunResult,
    parse_run_stdout,
)

__all__ = [
    "CheckResult",
    "GalaAgent",
    "GalaError",
    "GradResult",
    "PropertyVerification",
    "RunResult",
    "parse_run_stdout",
]
