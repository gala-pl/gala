"""End-to-end test of the Gala Jupyter kernel: start it, run cells, read the replies."""

import os
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BIN = ROOT / "target" / "debug" / "gala"


def _jupyter_available() -> bool:
    try:
        import jupyter_client  # noqa: F401
        import ipykernel  # noqa: F401
    except ImportError:
        return False
    return BIN.exists()


@unittest.skipUnless(_jupyter_available(), "needs jupyter_client, ipykernel and the gala binary")
class KernelTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        from jupyter_client.manager import start_new_kernel

        os.environ["GALA_BIN"] = str(BIN)
        cls.manager, cls.client = start_new_kernel(kernel_name="gala")
        cls.client.wait_for_ready(timeout=60)

    @classmethod
    def tearDownClass(cls) -> None:
        cls.client.stop_channels()
        cls.manager.shutdown_kernel()

    def run_cell(self, code: str):
        msg_id = self.client.execute(code)
        streams, displays, status = [], [], None
        while True:
            msg = self.client.get_iopub_msg(timeout=120)
            if msg["parent_header"].get("msg_id") != msg_id:
                continue
            kind = msg["msg_type"]
            if kind == "stream":
                streams.append(msg["content"]["text"])
            elif kind in ("display_data", "execute_result"):
                displays.append(msg["content"]["data"])
            elif kind == "status" and msg["content"]["execution_state"] == "idle":
                break
        # Shell replies include kernel_info replies too: wait for this cell's execute_reply.
        while True:
            reply = self.client.get_shell_msg(timeout=120)
            if reply["msg_type"] == "execute_reply" and reply["parent_header"].get("msg_id") == msg_id:
                return "".join(streams), displays, reply["content"]["status"]

    def test_definitions_persist_and_expressions_evaluate(self) -> None:
        out, _, status = self.run_cell("fn double(x: Int) -> Int pure {\n    x * 2\n}")
        self.assertEqual(status, "ok")
        out, _, status = self.run_cell("double(21)")
        self.assertEqual(status, "ok")
        self.assertIn("42", out)

    def test_bad_cell_reports_an_error(self) -> None:
        _, _, status = self.run_cell("fn broken() -> Int { missing }")
        self.assertEqual(status, "error")

    def test_run_magic_shows_a_histogram(self) -> None:
        self.run_cell("fn flip() -> Measured<Bool> prob {\n    let q = qubit()\n    x(q)\n    measure(q)\n}")
        _, displays, status = self.run_cell("%run flip --shots 200")
        self.assertEqual(status, "ok")
        html = "".join(d.get("text/html", "") for d in displays)
        self.assertIn("flip()", html)
        self.assertIn("200", html)


if __name__ == "__main__":
    unittest.main()
