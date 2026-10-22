"""Run with: cd sdk/python && python3 -m unittest discover -s tests -v"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from gala_agent import GalaAgent, parse_run_stdout

REPO = Path(__file__).resolve().parents[3]


def read(rel: str) -> str:
    return (REPO / rel).read_text(encoding="utf-8")


class GalaAgentTests(unittest.TestCase):
    def setUp(self) -> None:
        self.agent = GalaAgent()

    def test_check_reports_e0412(self) -> None:
        result = self.agent.check(read("tests/conformance/linearity/use_after_consume.gala"))
        self.assertEqual(result.schema, "gala.check.v1")
        self.assertFalse(result.ok)
        codes = [d["code"] for d in result.diagnostics]
        self.assertIn("E0412", codes)

    def test_run_prints_true(self) -> None:
        result = self.agent.run(read("tests/conformance/run/bell_run.gala"), seed=1)
        self.assertTrue(result.ok, result.error)
        self.assertEqual(result.output[0], "true")
        self.assertEqual(result.result, "0")

    def test_grad_is_unsupported(self) -> None:
        result = self.agent.grad("f", "theta")
        self.assertFalse(result.supported)
        self.assertTrue(result.reason)

    def test_verify_properties_runs_runnable_and_reports_the_rest(self) -> None:
        source = (
            "#[property(effect_honesty, unitary)]\n"
            "fn prep() -> Qubit quantum {\n    qubit()\n}\n\n"
            "fn main() -> Int { 0 }\n"
        )
        result = self.agent.verify_properties(source, ["effect_honesty", "unitary"])
        self.assertEqual([r["property"] for r in result.results], ["effect_honesty"])
        self.assertTrue(result.results[0]["passed"])
        self.assertEqual([s["property"] for s in result.skipped], ["unitary"])
        self.assertEqual(result.status, "unsupported")

    def test_verify_properties_reports_compile_errors(self) -> None:
        result = self.agent.verify_properties("fn main() -> Int { missing }", ["effect_honesty"])
        self.assertEqual(result.status, "error")
        self.assertTrue(result.diagnostics)

    def test_missing_binary_is_reported(self) -> None:
        result = GalaAgent(gala_bin="/nonexistent/gala").run("fn main() -> Int { 0 }")
        self.assertFalse(result.ok)
        self.assertIn("could not run", result.error or "")

    def test_parse_run_stdout_histogram(self) -> None:
        parsed = parse_run_stdout("true\nresults over 4 shots:\n  0: 4\n")
        self.assertEqual(parsed["output"], ["true"])
        self.assertEqual(parsed["histogram"], {"0": 4})
        self.assertEqual(parsed["shots"], 4)


if __name__ == "__main__":
    unittest.main()
