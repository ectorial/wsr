"""A deterministic native CI reference, with no third-party dependencies."""
import os
import unittest


class ArithmeticTest(unittest.TestCase):
    def test_addition(self):
        expected = 5 if os.environ.get("WSR_EXAMPLE_MODE", "pass") == "fail" else 4
        self.assertEqual(2 + 2, expected, "controlled WSR example assertion")
