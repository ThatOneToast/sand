"""Registry fixture contracts; regenerate the fixtures from official jars with javap."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

from scripts.mc_validation.extract_protocol_metadata import extract, parse_registry
from scripts.mc_validation.protocols import profile_for_protocol, profile_for_version


class ProtocolMetadataTests(unittest.TestCase):
    def test_profiles_are_pinned_to_the_actual_official_artifact_versions(self):
        for version, protocol, artifact in (
            ("26.2", 776, "823e2250d24b3ddac457a60c92a6a941943fcd6a"),
            ("26.3", 777, "33680f5f2ac32864d6d7cf5e56a705fdb3e05f4c"),
        ):
            with self.subTest(version=version):
                metadata = profile_for_version(version)
                self.assertIs(metadata, profile_for_protocol(protocol))
                self.assertEqual(metadata["server_jar_sha1"], artifact)
                self.assertEqual(metadata["version"]["protocol_version"], protocol)
                self.assertEqual(len(metadata["nested_jar_sha256"]), 64)
                for stage in metadata["packets"].values():
                    for packets in stage.values():
                        self.assertEqual(sorted(packets.values()), list(range(len(packets))))

    def test_26_3_packet_changes_are_explicit_not_a_uniform_offset(self):
        older = profile_for_protocol(776)["packets"]
        newer = profile_for_protocol(777)["packets"]
        self.assertEqual(older["play"]["clientbound"]["keep_alive"], 0x2c)
        self.assertEqual(newer["play"]["clientbound"]["keep_alive"], 0x2d)
        self.assertEqual(older["play"]["clientbound"]["respawn"], 0x52)
        self.assertEqual(newer["play"]["clientbound"]["respawn"], 0x54)
        self.assertEqual(newer["play"]["clientbound"]["disconnect"], 0x20)
        self.assertEqual(newer["configuration"]["clientbound"]["select_known_packs"], 0x0f)
        self.assertEqual(newer["configuration"]["serverbound"]["select_known_packs"], 0x07)
        self.assertEqual(profile_for_protocol(776)["teleport_ack_fields"], [{"type": "int", "name": "id"}])
        self.assertEqual([field["name"] for field in profile_for_protocol(777)["teleport_ack_fields"]], ["id", "x", "y", "z", "yRot", "xRot"])

    def test_unknown_versions_and_protocols_never_fall_back(self):
        for value in (775, 778, "777"):
            with self.assertRaises(ValueError):
                profile_for_protocol(value)
        for value in ("26.1", "26.3.1", "26.4"):
            with self.assertRaises(ValueError):
                profile_for_version(value)

    def test_extractor_rejects_nested_jar_digest_mismatch_before_running_javap(self):
        with tempfile.TemporaryDirectory() as directory:
            jar = Path(directory) / "bundle.jar"
            with zipfile.ZipFile(jar, "w") as archive:
                archive.writestr("version.json", json.dumps({"id": "fixture"}))
                archive.writestr("META-INF/versions.list", "0" * 64 + "\tfixture\tfixture/server.jar")
                archive.writestr("META-INF/versions/fixture/server.jar", b"wrong nested bytes")
            with patch("scripts.mc_validation.extract_protocol_metadata.subprocess.check_output") as javap:
                with self.assertRaisesRegex(ValueError, "digest"):
                    extract(jar)
                javap.assert_not_called()

    def test_registry_parser_uses_directional_order_and_rejects_duplicates(self):
        def field(name):
            return f" // Field net/minecraft/network/protocol/game/GamePacketTypes.{name}:Lnet/minecraft/network/protocol/PacketType;"
        text = "\n".join(field(name) for name in ("SERVERBOUND_Z", "SERVERBOUND_A", "CLIENTBOUND_Z", "CLIENTBOUND_A"))
        self.assertEqual(parse_registry(text), {"serverbound": {"z": 0, "a": 1}, "clientbound": {"z": 0, "a": 1}})
        for invalid in ("", text + field("CLIENTBOUND_A")):
            with self.assertRaises(ValueError):
                parse_registry(invalid)


if __name__ == "__main__":
    unittest.main()
