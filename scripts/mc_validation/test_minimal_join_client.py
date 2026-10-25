"""Protocol regressions; live presence/gameplay needs the vanilla harness."""
import contextlib
import io
import json
from pathlib import Path
import socket
import struct
import tempfile
import time
import unittest
import uuid
from unittest.mock import patch

from scripts.mc_validation import minimal_join_client as client
from scripts.mc_validation.scripted_input import ScriptedInput, atomic_json


class FakeConnection:
    def __init__(self, packets):
        self.packets = iter(packets)
        self.sent = []
        self.compression = -1

    def send_packet(self, packet_id, payload):
        self.sent.append((packet_id, payload))

    def recv_packet(self, timeout):
        item = next(self.packets, ConnectionError("server closed"))
        if isinstance(item, Exception):
            raise item
        return item


class MinimalClientTests(unittest.TestCase):
    def run_packets(self, packets, **options):
        conn = FakeConnection(packets)
        with contextlib.redirect_stdout(io.StringIO()):
            result = client.run_session(conn, "localhost", 25565, 776, "Probe", 10, **options)
        return result, conn.sent

    def test_26_2_play_keep_alive_echoes_the_exact_challenge(self):
        challenge = struct.pack(">q", 0x123456789ABCDEF)
        recipe_settings = bytes([1, 0] * 4)
        result, sent = self.run_packets([(2, b""), (3, b""), (0x31, b""), (0x4C, recipe_settings), (0x2C, challenge)])
        self.assertIn((0x1C, challenge), sent)
        self.assertNotIn((0x1C, recipe_settings), sent)
        self.assertFalse(result, "briefly joining must not hide a later disconnect")

    def test_teleports_are_acknowledged_after_join_and_dimension_change(self):
        # Position packets start with a VarInt teleport ID in protocol 776.
        # The remaining position/relative fields are not part of the reply.
        positions = [client.write_varint(value) + b"position payload" for value in (1, 300, 301)]
        _, sent = self.run_packets([
            (2, b""), (3, b""), (0x31, b""), (0x48, positions[0]),
            (0x52, b"respawn payload"), (0x48, positions[1]), (0x48, positions[2]),
        ])
        ids = [client.write_varint(i) for i in (1, 300, 301)]
        replies = [(packet, payload) for packet, payload in sent
                   if (packet == 0 and payload in ids) or packet == 0x2C]
        self.assertEqual(replies, [
            (0, ids[0]), (0x2C, b""), (0, ids[1]), (0x2C, b""), (0, ids[2]),
        ], "acknowledge every position; complete loading once per login or respawn")

    def test_respawn_request_is_opt_in(self):
        _, sent = self.run_packets([(2, b""), (3, b""), (0x31, b""), (0x44, b"death")])
        self.assertNotIn((0x0C, b"\x00"), sent)

    def test_passive_client_does_not_ack_chunk_batches(self):
        _, sent = self.run_packets([(2, b""), (3, b""), (0x31, b""),
                                    (0x0c, b""), (0x0b, b"\x09")])
        self.assertNotIn(0x0b, [packet for packet, _ in sent])

    def test_each_death_requests_one_respawn_and_completes_loading(self):
        position = client.write_varint(42) + b"position"
        _, sent = self.run_packets([
            (2, b""), (3, b""), (0x31, b""),
            (0x44, b"first death"), (0x68, struct.pack(">f", 0.0)),
            (0x52, b"respawn"), (0x48, position), (0x44, b"second death"),
        ], auto_respawn=True)
        self.assertEqual([packet for packet in sent if packet[0] == 0x0C],
                         [(0x0C, b"\x00"), (0x0C, b"\x00")])
        self.assertIn((0x00, client.write_varint(42)), sent)
        self.assertIn((0x2C, b""), sent)

    def test_configuration_keep_alive_and_ping_are_answered(self):
        challenge, ping = struct.pack(">q", -45), struct.pack(">i", 19)
        result, sent = self.run_packets([(2, b""), (4, challenge), (5, ping)])
        self.assertIn((4, challenge), sent)
        self.assertIn((5, ping), sent)
        self.assertFalse(result)

    def test_disconnect_packet_after_join_is_failure(self):
        result, _ = self.run_packets([(2, b""), (3, b""), (0x31, b""), (0x20, b"reason")])
        self.assertFalse(result)

    def test_success_requires_join_and_requested_duration(self):
        for packets, expected in [([(2, b""), (3, b""), (0x31, b"")], True), ([(2, b""), (3, b""), socket.timeout()], False)]:
            with self.subTest(expected=expected):
                conn = FakeConnection(packets)
                # Start, three receive iterations, then the duration expires.
                with patch.object(client.time, "monotonic", side_effect=[0, 0, 0, 1, 1, 2, 2, 10]), contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(client.run_session(conn, "localhost", 25565, 776, "Probe", 10), expected)

    def test_unsupported_protocol_is_rejected_before_connecting(self):
        with patch.object(client.sys, "argv", ["client", "localhost", "25565", "775", "Probe"]), patch.object(client, "Conn") as connect:
            with self.assertRaisesRegex(ValueError, "776"):
                client.main()
            connect.assert_not_called()


class ScriptedInputTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.controls = ScriptedInput(self.directory, 776)
        self.conn = FakeConnection([])
        self.target = uuid.UUID("01234567-89ab-cdef-0123-456789abcdef")

    def add_entity(self, wire_id=300, target=None):
        self.controls.observe(1, client.write_varint(wire_id) + (target or self.target).bytes)

    def command(self, **fields):
        command = {"id": 1, "deadline": time.monotonic() + 10,
                   "action": "attack", "uuid": str(self.target)}
        command.update(fields)
        atomic_json(self.directory / "command.json", command)
        self.controls.poll(self.conn, client.write_varint)

    def test_attack_codec_and_sent_ack_do_not_claim_acceptance_or_retry(self):
        self.add_entity()
        self.controls.teleported()
        self.command()
        self.controls.poll(self.conn, client.write_varint)
        self.assertEqual(self.conn.sent, [(0x01, b"\xac\x02")])
        status = json.loads((self.directory / "status.json").read_text())
        self.assertEqual(status["sent_id"], 1)
        self.assertNotIn("accepted", status)

    def test_move_codec_is_big_endian_position_rotation_and_ground_flag(self):
        self.controls.teleported()
        atomic_json(self.directory / "command.json", {
            "id": 1, "deadline": time.monotonic() + 10,
            "action": "move_pos_rot", "position": [1.25, 80, -2, -90, 0],
        })
        self.controls.poll(self.conn, client.write_varint)
        self.assertEqual(self.conn.sent, [(0x1f, struct.pack(">dddffB", 1.25, 80, -2, -90, 0, 1))])

    def test_removed_and_reused_wire_ids_cannot_attack_stale_uuid(self):
        self.controls.teleported()
        for lifecycle in ("remove", "reuse", "respawn"):
            with self.subTest(lifecycle=lifecycle):
                self.add_entity()
                if lifecycle == "remove":
                    self.controls.observe(0x4d, b"\x01\xac\x02")
                elif lifecycle == "reuse":
                    self.add_entity(target=uuid.UUID(int=1))
                else:
                    self.controls.reset()
                    self.controls.teleported()
                with self.assertRaisesRegex(ValueError, "not currently tracked"):
                    self.command()
        self.assertEqual(self.conn.sent, [])

    def test_readded_uuid_uses_new_wire_id(self):
        self.add_entity(300)
        self.add_entity(400)
        self.controls.teleported()
        self.command()
        self.assertEqual(self.conn.sent, [(1, b"\x90\x03")])

    def test_not_ready_expired_out_of_order_and_excess_commands_send_nothing(self):
        self.add_entity()
        with self.assertRaisesRegex(ValueError, "not ready"):
            self.command()
        self.controls.teleported()
        for fields in ({"deadline": 0}, {"deadline": time.monotonic() + 100},
                       {"deadline": float("nan")}, {"id": 2}, {"id": 33},
                       {"id": True}, {"action": "use_item"}, {"uuid": 5}):
            with self.subTest(fields=fields), self.assertRaises(ValueError):
                self.command(**fields)
        self.assertEqual(self.conn.sent, [])

    def test_oversized_input_and_invalid_coordinates_send_nothing(self):
        (self.directory / "command.json").write_bytes(b" " * 4097)
        with self.assertRaisesRegex(ValueError, "byte limit"):
            self.controls.poll(self.conn, client.write_varint)
        self.controls.teleported()
        for position in ([0, 80, 0, 0], [0, 80, float("inf"), 0, 0], [0, 80, 0, 0, 91]):
            atomic_json(self.directory / "command.json", {
                "id": 1, "deadline": time.monotonic() + 10,
                "action": "move_pos_rot", "position": position,
            })
            with self.assertRaises(ValueError):
                self.controls.poll(self.conn, client.write_varint)
        self.assertEqual(self.conn.sent, [])

    def test_bad_protocol_and_truncated_entity_packets_fail_closed(self):
        with self.assertRaisesRegex(ValueError, "776"):
            ScriptedInput(self.directory, 775)
        for packet_id, payload in ((1, b"\x80"), (1, b"\x01short"),
                                   (0x4d, b"\x01"), (0x4d, b"\x00extra")):
            with self.subTest(packet_id=packet_id, payload=payload), self.assertRaises(ValueError):
                self.controls.observe(packet_id, payload)

    def test_chunk_batch_ack_requires_one_valid_matching_finished_packet(self):
        for invalid in (b"", b"\x09", b"\x80"):
            with self.assertRaises(ValueError):
                self.controls.observe(0x0b, invalid)
        self.assertIsNone(self.controls.observe(0x0c, b""))
        for invalid in (b"", b"\x00", b"\x09extra", b"\x80", b"\x81\x08"):
            with self.assertRaises(ValueError):
                self.controls.observe(0x0b, invalid)
        self.assertEqual(self.controls.observe(0x0b, b"\x09"), (0x0b, struct.pack(">f", 9)))
        with self.assertRaises(ValueError):
            self.controls.observe(0x0b, b"\x09")
        with self.assertRaises(ValueError):
            self.controls.observe(0x0c, b"extra")
        self.controls.observe(0x0c, b"")
        with self.assertRaises(ValueError):
            self.controls.observe(0x0c, b"")
        self.controls.reset()
        with self.assertRaises(ValueError):
            self.controls.observe(0x0b, b"\x09")

    def test_session_sends_exactly_one_ack_per_finished_batch(self):
        conn = FakeConnection([(2, b""), (3, b""), (0x31, b""),
                               (0x0c, b""), (0x27, b"chunk"), (0x0b, b"\x01"),
                               (0x0c, b""), (0x0b, b"\x09")])
        conn.sock = object()
        with patch.object(client.select, "select", return_value=([conn.sock], [], [])), contextlib.redirect_stdout(io.StringIO()):
            self.assertFalse(client.run_session(conn, "localhost", 25565, 776, "Probe", 10,
                                                controls=self.controls))
        self.assertEqual([packet for packet in conn.sent if packet[0] == 0x0b],
                         [(0x0b, struct.pack(">f", 9.0))] * 2)

    def test_session_drains_keepalive_and_teleport_before_serial_attack(self):
        self.add_entity()
        atomic_json(self.directory / "command.json", {
            "id": 1, "deadline": time.monotonic() + 10,
            "action": "attack", "uuid": str(self.target),
        })
        position = client.write_varint(42) + b"position"
        challenge = struct.pack(">q", 123)
        conn = FakeConnection([(2, b""), (3, b""), (0x31, b""),
                               (1, b"\xac\x02" + self.target.bytes),
                               (0x48, position), (0x2c, challenge)])
        conn.sock = object()
        # Six incoming packets, one idle command poll, then server close.
        readiness = [([conn.sock], [], [])] * 6 + [([], [], []), ([conn.sock], [], [])]
        with patch.object(client.select, "select", side_effect=readiness), contextlib.redirect_stdout(io.StringIO()):
            self.assertFalse(client.run_session(conn, "localhost", 25565, 776, "Probe", 10,
                                                controls=self.controls))
        self.assertEqual(conn.sent[-4:], [(0, b"\x2a"), (0x2c, b""), (0x1c, challenge), (1, b"\xac\x02")])


if __name__ == "__main__":
    unittest.main()
