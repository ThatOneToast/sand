#!/usr/bin/env python3
"""Focused unit tests for rcon_client.py's packet framing and auth flow.

These run against an in-process fake RCON server (a background thread
speaking the same wire protocol over a loopback `socket.socketpair`), not a
real Minecraft server — they validate the *implementation* (framing,
auth-failure handling, multi-command ordering), not runtime behavior against
a live 26.2 server. See `README.md` for what is/isn't runtime-verified.

Run: python3 -m unittest scripts.mc_validation.test_rcon_client -v
     (from the repo root), or `python3 test_rcon_client.py` from this dir.
"""

from __future__ import annotations

import socket
import struct
import threading
import unittest

import rcon_client


def _read_packet_serverside(sock: socket.socket) -> tuple[int, int, str]:
    (length,) = struct.unpack("<i", sock.recv(4))
    data = b""
    while len(data) < length:
        chunk = sock.recv(length - len(data))
        if not chunk:
            raise ConnectionError("closed mid-packet")
        data += chunk
    request_id, packet_type = struct.unpack("<ii", data[:8])
    payload = data[8:-2].decode("utf-8", errors="replace")
    return request_id, packet_type, payload


def _send_packet_serverside(sock: socket.socket, request_id: int, packet_type: int, payload: str) -> None:
    body = struct.pack("<ii", request_id, packet_type) + payload.encode("utf-8") + b"\x00\x00"
    sock.sendall(struct.pack("<i", len(body)) + body)


def _fake_server(sock: socket.socket, password: str, responses: dict[str, str]) -> None:
    """Speak just enough SERVERDATA_AUTH/EXECCOMMAND to exercise the client."""
    request_id, _packet_type, auth_payload = _read_packet_serverside(sock)
    if auth_payload != password:
        _send_packet_serverside(sock, -1, 2, "")
        sock.close()
        return
    _send_packet_serverside(sock, request_id, 2, "")  # SERVERDATA_AUTH_RESPONSE
    try:
        while True:
            try:
                request_id, _packet_type, command = _read_packet_serverside(sock)
            except (ConnectionError, OSError, struct.error):
                return
            _send_packet_serverside(
                sock, request_id, 0, responses.get(command, f"unknown: {command}")
            )
    finally:
        sock.close()


class RconClientTests(unittest.TestCase):
    def _run_with_fake_server(self, password_used: str, password_expected: str, commands, responses):
        client_sock, server_sock = socket.socketpair()
        thread = threading.Thread(
            target=_fake_server, args=(server_sock, password_expected, responses), daemon=True
        )
        thread.start()
        try:
            return self._drive_client(client_sock, password_used, commands)
        finally:
            client_sock.close()
            thread.join(timeout=2)

    @staticmethod
    def _drive_client(sock: socket.socket, password: str, commands):
        rcon_client._packet(sock, 1, 3, password)
        request_id, _packet_type, _payload = rcon_client._read_packet(sock)
        if request_id == -1:
            raise PermissionError("RCON authentication failed")
        out = []
        for index, command in enumerate(commands, start=2):
            rcon_client._packet(sock, index, 2, command)
            _request_id, _packet_type, payload = rcon_client._read_packet(sock)
            out.append(payload)
        return out

    def test_successful_auth_and_single_command(self):
        responses = self._run_with_fake_server(
            password_used="secret",
            password_expected="secret",
            commands=["list"],
            responses={"list": "There are 0 of a max of 20 players online"},
        )
        self.assertEqual(responses, ["There are 0 of a max of 20 players online"])

    def test_multiple_commands_preserve_order(self):
        responses = self._run_with_fake_server(
            password_used="secret",
            password_expected="secret",
            commands=["function paudit:init", "data get storage paudit:audit"],
            responses={
                "function paudit:init": "ok-init",
                "data get storage paudit:audit": "ok-data",
            },
        )
        self.assertEqual(responses, ["ok-init", "ok-data"])

    def test_auth_failure_raises_permission_error(self):
        with self.assertRaises(PermissionError):
            self._run_with_fake_server(
                password_used="wrong",
                password_expected="secret",
                commands=["list"],
                responses={},
            )

    def test_packet_roundtrip_framing(self):
        client_sock, server_sock = socket.socketpair()
        try:
            rcon_client._packet(client_sock, 7, 2, "say hi")
            request_id, packet_type, payload = _read_packet_serverside(server_sock)
            self.assertEqual((request_id, packet_type, payload), (7, 2, "say hi"))
        finally:
            client_sock.close()
            server_sock.close()


if __name__ == "__main__":
    unittest.main()
