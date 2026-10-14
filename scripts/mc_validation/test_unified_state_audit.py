"""Regression coverage for preflight evidence and semantic system discovery."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

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
