"""Observable network/credential boundaries for disposable RCON audits."""
import contextlib
import io
import os
from pathlib import Path
import secrets
import stat
import tempfile
import unittest
from unittest.mock import patch

from scripts.mc_validation.local_rcon_server import write_properties


class LocalRconServerTests(unittest.TestCase):
    def test_written_properties_bind_both_vanilla_listeners_to_loopback(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            secret = secrets.token_urlsafe(32)
            output = io.StringIO()
            with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
                write_properties(root, 25577, 25578, secret)
            properties = dict(line.split("=", 1) for line in (root / "server.properties").read_text().splitlines())
            self.assertEqual(properties["server-ip"], "127.0.0.1")
            self.assertEqual(properties["server-port"], "25577")
            self.assertEqual(properties["rcon.port"], "25578")
            self.assertEqual(properties["enable-rcon"], "true")
            self.assertEqual(properties["rcon.password"], secret)
            self.assertNotIn(secret, output.getvalue())
            if os.name == "posix":
                self.assertEqual(stat.S_IMODE((root / "server.properties").stat().st_mode), 0o600)

    def test_audit_rcon_keeps_credentials_in_process_and_preserves_transcripts(self):
        from scripts.mc_validation import run_audit
        with patch.object(run_audit.rcon_client, "run_commands", return_value=["first\nsecond", "done"]) as commands:
            with patch.object(run_audit.subprocess, "run") as subprocess_run:
                result = run_audit.rcon("127.0.0.1", 25578, "list", "say hello")
        commands.assert_called_once_with("127.0.0.1", 25578, run_audit.RCON_PASSWORD, ["list", "say hello"])
        subprocess_run.assert_not_called()
        self.assertEqual(result, [">>> list", "first", "second", ">>> say hello", "done"])
        self.assertNotIn(run_audit.RCON_PASSWORD, "\n".join(result))


if __name__ == "__main__":
    unittest.main()
