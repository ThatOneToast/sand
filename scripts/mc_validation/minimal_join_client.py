"""Minimal, best-effort Minecraft Java Edition client for #265 runtime
validation — enough of the login → configuration → play handshake to make a
real `ServerPlayerEntity` join a live server, driven by an offline-mode
username (no Mojang auth).

# Status (honest, as of this tooling's introduction)

- Handshake, Login, and the full Configuration phase are implemented and
  **empirically confirmed working** against a real Minecraft Java 26.2
  server: the server log shows `<name> logged in with entity id N` and
  `<name> joined the game` on every run.
- The Play phase is **not stable**: the connection is consistently dropped
  by the server within roughly one tick of entering Play, before any
  scripted follow-up (e.g. an RCON-driven damage event) can reliably land
  inside the connection's live window. The exact missing serverbound
  packet was not identified with confidence in the time available — no
  official protocol documentation exists yet for this version (protocol
  776); every packet id below was derived empirically against the real
  server plus cross-referenced against `minecraft-data`'s closest published
  reference (`pc/1.21.11/protocol.json` — 26.2 itself is not yet in that
  dataset). Treat `PLAY_CLIENTBOUND_KEEP_ALIVE`/`PLAY_CLIENTBOUND_LOGIN`
  below as best-effort, not authoritative.
- Net effect: this client is real evidence that a player *can* join a real
  26.2 server (useful on its own — see
  `docs/testing/participant-role-evidence.md`), but is not yet a reliable
  driver for scripted in-game combat scenarios. Extending it to a stable
  Play-phase connection is tracked as follow-up scope, not attempted
  further here per the "do not redesign, only smallest correctness fixes"
  scope for this PR.

Usage:
    python3 minimal_join_client.py <host> <port> <protocol_version> \\
        <username> [<run_seconds>] [<on_join_shell_command>]

`<protocol_version>` for 26.2 is 776 (confirm via a status ping / the
`minecraft-data` `protocolVersions.json` table if targeting a different
version). `<on_join_shell_command>`, if given, runs via `bash -c` the
instant the Play-phase join packet is observed — used to race an RCON
command into the brief connected window.
"""

import socket
import struct
import sys
import time
import uuid
import zlib


def write_varint(value):
    out = b""
    while True:
        byte = value & 0x7F
        value >>= 7
        if value != 0:
            byte |= 0x80
        out += bytes([byte])
        if value == 0:
            return out


def read_varint_bytes(data, idx):
    value = 0
    shift = 0
    while True:
        b = data[idx]
        idx += 1
        value |= (b & 0x7F) << shift
        if not (b & 0x80):
            break
        shift += 7
    return value, idx


def write_string(s):
    data = s.encode("utf-8")
    return write_varint(len(data)) + data


class Conn:
    def __init__(self, host, port):
        self.sock = socket.create_connection((host, port), timeout=10)
        self.compression = -1

    def read_n(self, n):
        data = b""
        while len(data) < n:
            chunk = self.sock.recv(n - len(data))
            if not chunk:
                raise ConnectionError("closed")
            data += chunk
        return data

    def read_varint_sock(self):
        value = 0
        shift = 0
        while True:
            b = self.sock.recv(1)
            if not b:
                raise ConnectionError("closed reading varint")
            byte = b[0]
            value |= (byte & 0x7F) << shift
            if not (byte & 0x80):
                break
            shift += 7
        return value

    def send_packet(self, packet_id, payload):
        body = write_varint(packet_id) + payload
        if self.compression >= 0:
            if len(body) >= self.compression:
                compressed = zlib.compress(body)
                framed = write_varint(len(body)) + compressed
            else:
                framed = write_varint(0) + body
            self.sock.sendall(write_varint(len(framed)) + framed)
        else:
            self.sock.sendall(write_varint(len(body)) + body)

    def recv_packet(self, timeout=10):
        self.sock.settimeout(timeout)
        length = self.read_varint_sock()
        raw = self.read_n(length)
        if self.compression >= 0:
            idx = 0
            data_len, idx = read_varint_bytes(raw, idx)
            body = raw[idx:]
            if data_len != 0:
                body = zlib.decompress(body)
        else:
            body = raw
        pid, idx = read_varint_bytes(body, 0)
        return pid, body[idx:]


# Verified against minecraft-data pc/1.21.11/protocol.json (protocol 776 /
# 26.2 itself is not yet published there, but empirical testing against a
# live 26.2 server confirmed identical packet ids for every packet observed
# below).
CONFIG_CLIENTBOUND = {
    0x00: "cookie_request",
    0x01: "custom_payload",
    0x02: "disconnect",
    0x03: "finish_configuration",
    0x04: "keep_alive",
    0x05: "ping",
    0x06: "reset_chat",
    0x07: "registry_data",
    0x08: "remove_resource_pack",
    0x09: "add_resource_pack",
    0x0A: "store_cookie",
    0x0B: "transfer",
    0x0C: "feature_flags",
    0x0D: "tags",
    0x0E: "select_known_packs",
    0x0F: "custom_report_details",
    0x10: "server_links",
}
PLAY_CLIENTBOUND_KEEP_ALIVE = 0x4C
PLAY_CLIENTBOUND_LOGIN = 0x31
PLAY_SERVERBOUND_KEEP_ALIVE = 0x1C


def main():
    host, port, protocol_version, username = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
    conn = Conn(host, port)

    payload = write_varint(protocol_version) + write_string(host) + struct.pack(">H", port) + write_varint(2)
    conn.send_packet(0x00, payload)

    offline_uuid = uuid.uuid3(uuid.NAMESPACE_OID, f"OfflinePlayer:{username}")
    payload = write_string(username) + offline_uuid.bytes
    conn.send_packet(0x00, payload)

    run_seconds = float(sys.argv[5]) if len(sys.argv) > 5 else 30
    state = "login"
    deadline = time.monotonic() + run_seconds
    joined = False
    while time.monotonic() < deadline:
        try:
            pid, data = conn.recv_packet(timeout=8)
        except socket.timeout:
            continue
        except ConnectionError as e:
            print(f"stopped: {e}")
            break

        if state == "login":
            if pid == 0x03:
                threshold, _ = read_varint_bytes(data, 0)
                conn.compression = threshold
                print(f"[login] compression threshold={threshold}")
            elif pid == 0x02:
                print("[login] LOGIN SUCCESS")
                conn.send_packet(0x03, b"")  # login_acknowledged
                state = "configuration"
                # Proactively send Client Information (configuration
                # serverbound 0x00) -- stable shape since 1.20.2, some
                # servers expect it promptly after login_acknowledged.
                info = (
                    write_string("en_us")
                    + bytes([10])  # view distance
                    + write_varint(0)  # chat mode: enabled
                    + bytes([1])  # chat colors: true
                    + bytes([0x7F])  # displayed skin parts: all
                    + write_varint(1)  # main hand: right
                    + bytes([0])  # text filtering: false
                    + bytes([1])  # allow server listings: true
                    + write_varint(0)  # particle status: all
                )
                conn.send_packet(0x00, info)
                print("  sent client information")
            elif pid == 0x00:
                print(f"[login] DISCONNECT: {data!r}")
                break
        elif state == "configuration":
            name = CONFIG_CLIENTBOUND.get(pid, f"unknown_0x{pid:02x}")
            print(f"[config] {name} len={len(data)}")
            if name == "select_known_packs":
                # Echo the exact same known-packs list back (serverbound id 0x07).
                conn.send_packet(0x07, data)
                print("  -> replied select_known_packs (echo)")
            elif name == "finish_configuration":
                conn.send_packet(0x03, b"")  # serverbound finish_configuration ack
                print("  -> acked finish_configuration, entering play")
                state = "play"
            elif name == "disconnect":
                print(f"  DISCONNECT payload: {data!r}")
                break
        elif state == "play":
            if pid == PLAY_CLIENTBOUND_KEEP_ALIVE and len(data) == 8:
                conn.send_packet(PLAY_SERVERBOUND_KEEP_ALIVE, data)
                print("[play] keep_alive echoed")
            elif pid == PLAY_CLIENTBOUND_LOGIN:
                joined = True
                print("[play] LOGIN (join game) packet received -- player entity should now exist")
                if len(sys.argv) > 6:
                    import subprocess
                    print(f"  firing on-join command: {sys.argv[6]}")
                    subprocess.run(["bash", "-c", sys.argv[6]])
            else:
                print(f"[play] packet id=0x{pid:02x} len={len(data)}")

    print(f"final state={state} joined_play={joined}")
    return joined


if __name__ == "__main__":
    ok = main()
    sys.exit(0 if ok else 1)
