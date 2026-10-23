"""The bridge calls the gala CLI; the PyTorch test runs only where torch is installed."""

import math
import os
import unittest
from pathlib import Path

from gala_agent.bridge import GalaKernel

ROOT = Path(__file__).resolve().parents[3]
CLASSIFIER = ROOT / "library" / "std" / "vqa.gala"
BIN = str(ROOT / "target" / "debug" / "gala")


def _binary_ready() -> bool:
    return os.path.exists(BIN)


@unittest.skipUnless(_binary_ready(), "build the gala binary first (cargo build --bin gala)")
class KernelTests(unittest.TestCase):
    def setUp(self) -> None:
        self.kernel = GalaKernel(CLASSIFIER, "classify", gala_bin=BIN)

    def test_expectation_matches_the_analytic_value(self) -> None:
        # Ry(x) then Ry(theta) on |0>: E = cos(x + theta) for a Z measurement, with
        # P(false) = (1 + E) / 2. Here x = 0 and theta = 0, so E = 1.
        self.assertAlmostEqual(self.kernel.expectation([0.0, 0.0], shots=2000), 1.0, places=6)

    def test_gradient_matches_the_analytic_derivative(self) -> None:
        x, theta = 0.4, 0.9
        # dE/dtheta = -sin(x + theta).
        self.assertLess(abs(self.kernel.gradient([x, theta], wrt=1) - -math.sin(x + theta)), 0.05)


try:
    import torch
except ImportError:  # pragma: no cover
    torch = None


@unittest.skipUnless(torch is not None and _binary_ready(), "needs torch and the gala binary")
class TorchTrainingTests(unittest.TestCase):
    def test_pytorch_loop_trains_a_gala_kernel(self) -> None:
        from gala_agent.bridge import expectation

        kernel = GalaKernel(CLASSIFIER, "classify", gala_bin=BIN)
        theta = torch.nn.Parameter(torch.tensor(0.3, dtype=torch.float64))
        opt = torch.optim.SGD([theta], lr=1.0)
        # x = 0 should give label 0 (P(true) = 0); x = pi should give label 1 (P(true) = 1).
        samples = [(0.0, 0.0), (math.pi, 1.0)]
        for _ in range(12):
            opt.zero_grad()
            loss = torch.zeros((), dtype=torch.float64)
            for x, y in samples:
                e = expectation(kernel, [x], theta, wrt=1, shots=2000)
                p_true = (1 - e) / 2
                loss = loss + (p_true - y) ** 2
            loss.backward()
            opt.step()
        correct = 0
        for x, y in samples:
            e = kernel.expectation([x, float(theta.detach())], shots=2000)
            correct += int((1 - e) / 2 > 0.5) == int(y)
        self.assertEqual(correct, len(samples))


if __name__ == "__main__":
    unittest.main()
