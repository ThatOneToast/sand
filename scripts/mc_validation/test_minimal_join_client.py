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
    def run_packets(self, packets):
        conn = FakeConnection(packets)
        with contextlib.redirect_stdout(io.StringIO()):
            result = client.run_session(conn, "localhost", 25565, 776, "Probe", 10)
        return result, conn.sent

    def test_26_2_play_keep_alive_echoes_the_exact_challenge(self):
        challenge = struct.pack(">q", 0x123456789ABCDEF)
        recipe_settings = bytes([1, 0] * 4)
        result, sent = self.run_packets([(2, b""), (3, b""), (0x31, b""), (0x4C, recipe_settings), (0x2C, challenge)])
        self.assertIn((0x1C, challenge), sent)
        self.assertNotIn((0x1C, recipe_settings), sent)
        self.assertFalse(result, "briefly joining must not hide a later disconnect")

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
