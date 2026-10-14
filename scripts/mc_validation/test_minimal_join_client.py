"""Protocol regressions; live presence/gameplay needs the vanilla harness."""
import contextlib
import io
import socket
import struct
import unittest
from unittest.mock import patch

from scripts.mc_validation import minimal_join_client as client


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


if __name__ == "__main__":
    unittest.main()
