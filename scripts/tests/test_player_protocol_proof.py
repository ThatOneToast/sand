"""Native transport evidence must include actual accepted commands and failures."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from validate_player_protocol import ProtocolHarness, main, validate_jar
from vanilla_reload_harness import ValidationFailure


class PlayerProtocolProofTests(unittest.TestCase):
    def test_jar_version_and_digest_must_match_the_profile(self):
        with tempfile.TemporaryDirectory() as directory:
            jar = Path(directory) / "server.jar"
            version = {"id": "fixture", "protocol_version": 999}
            with zipfile.ZipFile(jar, "w") as archive:
                archive.writestr("version.json", json.dumps(version))
            profile = {"version": version, "server_jar_sha1": hashlib.sha1(jar.read_bytes()).hexdigest()}
            validate_jar(jar, profile)
            with self.assertRaisesRegex(ValueError, "SHA-1"):
                validate_jar(jar, {**profile, "server_jar_sha1": "wrong"})
            with self.assertRaisesRegex(ValueError, "version.json"):
                validate_jar(jar, {**profile, "version": {"id": "other"}})

    def test_health_value_alone_cannot_establish_native_damage_acceptance(self):
        harness = ProtocolHarness.__new__(ProtocolHarness)
        harness.args = argparse.Namespace(timeout=1)
        harness.eventually = Mock()
        harness.send = Mock()
        harness.passed = Mock(return_value=False)
        with patch("validate_player_protocol.time.monotonic", side_effect=[0, 0, 2]), patch("validate_player_protocol.time.sleep"):
            with self.assertRaises(ValidationFailure) as raised:
                harness.confirm_native_damage()
        self.assertEqual(raised.exception.phase, "protocol-damage")
        self.assertIn("unless score #damaged protocol_proof matches 1", harness.send.call_args.args[0])
        self.assertIn("store success score #damaged protocol_proof", harness.send.call_args.args[0])
        self.assertIn("if score #damaged protocol_proof matches 1", harness.passed.call_args.args[1])
        self.assertIn("Health:19.0f", harness.passed.call_args.args[1])
        harness.eventually.assert_called_once_with("full_health_before_damage", "if entity @a[name=SandAlpha,nbt={Health:20.0f}]")

    def test_failed_native_stage_records_failure_and_never_reports_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "proof"
            harness = Mock()
            harness.lines = []
            harness.wait_for.side_effect = ValidationFailure("join", "missing player")
            with patch("validate_player_protocol.validate_jar"), patch("validate_player_protocol.ProtocolHarness", return_value=harness), patch("builtins.print"):
                status = main(["--jar", "unused", "--output", str(root)])
            self.assertEqual(status, 1)
            result = json.loads((root / "evidence.json").read_text())
            self.assertFalse(result["passed"])
            self.assertEqual(result["failure"], {"phase": "join", "message": "missing player"})
            self.assertEqual(result["checks"], [])
            harness.stop.assert_called_once()


if __name__ == "__main__":
    unittest.main()
