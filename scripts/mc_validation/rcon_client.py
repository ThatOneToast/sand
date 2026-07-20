#!/usr/bin/env python3
"""Minimal Minecraft RCON client (#265 runtime-validation tooling).

The RCON wire protocol (Source RCON, reused unmodified by Minecraft since
1.9) has been stable for over a decade — this implementation is not a
version-specific guess, it is the well-documented, version-independent
protocol. Used to drive real server commands (`summon`, `data get`,
`scoreboard`, `reload`, ...) against a live server without a connected
player — real evidence for anything that does not require an actual
`ServerPlayerEntity` (see `README.md` in this directory for what does).

Usage:
    python3 rcon_client.py <host> <port> <password> <command> [<command> ...]

Each command's response is printed to stdout, prefixed with the command.
Exits non-zero if authentication fails.
"""

from __future__ import annotations

import socket
import struct
import sys


def _packet(sock: socket.socket, request_id: int, packet_type: int, payload: str) -> None:
    body = struct.pack("<ii", request_id, packet_type) + payload.encode("utf-8") + b"\x00\x00"
    sock.sendall(struct.pack("<i", len(body)) + body)


def _read_packet(sock: socket.socket) -> tuple[int, int, str]:
    (length,) = struct.unpack("<i", sock.recv(4))
    data = b""
    while len(data) < length:
        chunk = sock.recv(length - len(data))
        if not chunk:
            raise ConnectionError("RCON connection closed mid-packet")
        data += chunk
    request_id, packet_type = struct.unpack("<ii", data[:8])
    payload = data[8:-2].decode("utf-8", errors="replace")
    return request_id, packet_type, payload


def run_commands(host: str, port: int, password: str, commands: list[str]) -> list[str]:
    """Authenticate and run `commands` in order, returning each response."""
    sock = socket.create_connection((host, port), timeout=10)
    try:
        _packet(sock, 1, 3, password)  # SERVERDATA_AUTH
        request_id, _packet_type, _payload = _read_packet(sock)
        if request_id == -1:
            raise PermissionError("RCON authentication failed")
        responses = []
        for index, command in enumerate(commands, start=2):
            _packet(sock, index, 2, command)  # SERVERDATA_EXECCOMMAND
            _request_id, _packet_type, payload = _read_packet(sock)
            responses.append(payload)
        return responses
    finally:
        sock.close()


def main(argv: list[str]) -> int:
    if len(argv) < 4:
        print(__doc__, file=sys.stderr)
        return 2
    host, port, password = argv[0], int(argv[1]), argv[2]
    commands = argv[3:]
    try:
        responses = run_commands(host, port, password, commands)
    except PermissionError as error:
        print(f"AUTH FAILED: {error}", file=sys.stderr)
        return 1
    for command, response in zip(commands, responses):
        print(f">>> {command}\n{response}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
