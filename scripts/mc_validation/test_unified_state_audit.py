"""Regression coverage for preflight evidence and semantic system discovery."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import run_unified_state_audit as audit


def functions():
    return {
        "attach_zombie_components": "scoreboard players set @s damage 2\nscoreboard players set @s health 20",
        "sand/entity_query/regen": "scoreboard players remove @s timer 1\nfunction unified_state:armor",
        "armor": "scoreboard players add @s armor 1",
        "sand/entity_query/train": "function unified_state:damage",
        "damage": "scoreboard players add @s damage 1",
        "sand/entity_query/heal": "function unified_state:health",
        "health": "scoreboard players add @s health 1",
        "__sand_system/combat": "function unified_state:sand/entity_query/regen\nfunction unified_state:sand/entity_query/train",
        "__sand_system/health": "function unified_state:sand/entity_query/heal",
        "__sand_system_tick": "execute as @e[tag=combat] at @s run function unified_state:__sand_system/combat\nexecute as @e[tag=health] at @s run function unified_state:__sand_system/health",
    }


class IndexedOwnerOracle(unittest.TestCase):
    def test_data_get_spacing_and_signed_boundaries(self):
        self.assertEqual(
            audit.indexed_status_path("[I; -2147483648, 2147483647, -1, 0]"),
            'owners_v2."[I;-2147483648,2147483647,-1,0]".components."status".details',
        )

    def test_compact_uuid_and_every_word_remain_distinct(self):
        paths = {audit.indexed_status_path(uuid) for uuid in (
            "[I;0,0,0,0]", "[I;1,0,0,0]", "[I;0,1,0,0]",
            "[I;0,0,1,0]", "[I;0,0,0,1]",
        )}
        self.assertEqual(len(paths), 5)

    def test_invalid_words_are_not_silently_truncated(self):
        for value in ("[I;1,2,3]", "[I;2147483648,0,0,0]", "[I;-2147483649,0,0,0]"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                audit.indexed_status_path(value)


class ReloadCompletion(unittest.TestCase):
    def test_acknowledgement_is_not_completion_and_stale_marker_is_cleared(self):
        command = Mock(side_effect=["Removed", "Reloading", "Found no elements", "contents: 1b"])
        with patch.object(audit.time, "sleep"):
            self.assertEqual(audit.wait_for_reload(command, 5), "contents: 1b")
        self.assertEqual(command.call_args_list[0].args, ("data remove storage sand_unified_audit:reload completed",))
        self.assertEqual(command.call_args_list[1].args, ("reload",))
        self.assertEqual(command.call_count, 4)

    def test_missing_load_execution_times_out(self):
        command = Mock(return_value="Found no elements")
        with patch.object(audit.time, "monotonic", side_effect=[0, 0, 2]), patch.object(audit.time, "sleep"):
            with self.assertRaises(TimeoutError):
                audit.wait_for_reload(command, 1)


class StartupReadiness(unittest.TestCase):
    def test_world_done_does_not_skip_delayed_rcon_startup(self):
        done = 'Done (4.721s)! For help, type "help"'
        listener = 'RCON running on 127.0.0.1:25578'
        logs = Mock()
        logs.get.side_effect = [done, audit.queue.Empty, listener]
        ready, evidence = audit.wait_for_startup(Mock(poll=Mock(return_value=None)), logs, 5)
        self.assertTrue(ready)
        self.assertEqual(evidence, [done, listener])
        self.assertEqual(logs.get.call_count, 3)

    def test_listener_alone_is_not_a_ready_world(self):
        logs = Mock()
        logs.get.return_value = 'RCON running on 127.0.0.1:25578'
        with patch.object(audit.time, "monotonic", side_effect=[0, 0, 2]):
            ready, _ = audit.wait_for_startup(Mock(poll=Mock(return_value=None)), logs, 1)
        self.assertFalse(ready)

    def test_world_done_without_listener_times_out(self):
        logs = Mock()
        logs.get.return_value = 'Done (4.721s)! For help, type "help"'
        with patch.object(audit.time, "monotonic", side_effect=[0, 0, 2]):
            ready, _ = audit.wait_for_startup(Mock(poll=Mock(return_value=None)), logs, 1)
        self.assertFalse(ready)

    def test_process_exit_is_reported_without_waiting_for_timeout(self):
        logs = Mock()
        ready, evidence = audit.wait_for_startup(Mock(poll=Mock(return_value=1)), logs, 30)
        self.assertFalse(ready)
        self.assertEqual(evidence, [])
        logs.get.assert_not_called()


class SystemScans(unittest.TestCase):
    def test_health_and_ordered_combat_scans_are_identified(self):
        evidence = audit.inspect_system_scans(functions())
        self.assertEqual(evidence["health_selector"], "@e[tag=health]")
        self.assertEqual(evidence["combat_selector"], "@e[tag=combat]")
        self.assertEqual(evidence["health_query"], "sand/entity_query/heal")

    def test_unrelated_scan_is_allowed_but_cannot_replace_health(self):
        records = functions()
        records["__sand_system/other"] = "say unrelated"
        extra = "execute as @e[tag=other] at @s run function unified_state:__sand_system/other"
        records["__sand_system_tick"] += "\n" + extra
        audit.inspect_system_scans(records)
        records["__sand_system_tick"] = records["__sand_system_tick"].replace(
            "execute as @e[tag=health] at @s run function unified_state:__sand_system/health\n", ""
        )
        with self.assertRaisesRegex(ValueError, "Health"):
            audit.inspect_system_scans(records)

    def test_health_requires_a_distinct_selector(self):
        records = functions()
        records["__sand_system_tick"] = records["__sand_system_tick"].replace("@e[tag=health]", "@e[tag=combat]")
        with self.assertRaisesRegex(ValueError, "distinct"):
            audit.inspect_system_scans(records)

    def test_combat_order_is_preserved(self):
        records = functions()
        records["__sand_system/combat"] = "\n".join(reversed(records["__sand_system/combat"].splitlines()))
        with self.assertRaisesRegex(ValueError, "order"):
            audit.inspect_system_scans(records)


class PreflightEvidence(unittest.TestCase):
    def test_malformed_export_overwrites_stale_evidence_without_starting_java(self):
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            (project / "dist/unified_state/data/unified_state/function").mkdir(parents=True)
            evidence = project / "evidence.json"
            evidence.write_text('{"old_success":true}')
            argv = ["audit", "--jar", "unused.jar", "--evidence", str(evidence)]
            with patch.object(audit, "PACK_DIR", project), patch.object(sys, "argv", argv), patch.object(audit.subprocess, "Popen") as launch:
                with self.assertRaises(ValueError):
                    audit.main()
            launch.assert_not_called()
            result = json.loads(evidence.read_text())
            self.assertFalse(result["checks"]["audit_completed"]["passed"])
            self.assertIsNone(result["server_dir"])
            self.assertNotIn("old_success", result)

    def test_failed_build_also_writes_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            evidence = Path(directory) / "new/evidence.json"
            argv = ["audit", "--build", "--jar", "unused.jar", "--evidence", str(evidence)]
            with patch.object(sys, "argv", argv), patch.object(audit.subprocess, "run", side_effect=subprocess.CalledProcessError(1, ["cargo"])):
                with self.assertRaises(subprocess.CalledProcessError):
                    audit.main()
            self.assertFalse(json.loads(evidence.read_text())["checks"]["audit_completed"]["passed"])


if __name__ == "__main__":
    unittest.main()
