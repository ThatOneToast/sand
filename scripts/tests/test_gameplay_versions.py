"""Native player proofs must export for the selected server version."""
import importlib
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))


class ExportReached(Exception):
    pass


class GameplayVersionTests(unittest.TestCase):
    def test_player_proofs_export_the_requested_version(self):
        for name, key in (("owned_players", "SAND_GAMEPLAY_PROOF_VERSION"),
                          ("inventory", "SAND_INVENTORY_PROOF_VERSION"),
                          ("hud", "SAND_PRESENTATION_PROOF_VERSION")):
            module = importlib.import_module(f"validate_{name}")
            for version in ("26.2", "26.3", None):
                with self.subTest(proof=name, version=version), tempfile.TemporaryDirectory() as root:
                    argv = [name, "--jar", "/unused/server.jar", "--output", root]
                    if version:
                        argv += ["--version", version]
                    with patch.object(sys, "argv", argv), patch.object(module.subprocess, "run", side_effect=ExportReached) as run:
                        with self.assertRaises(ExportReached):
                            module.main()
                    self.assertEqual(run.call_args.kwargs["env"][key], version or "26.3")


if __name__ == "__main__":
    unittest.main()
