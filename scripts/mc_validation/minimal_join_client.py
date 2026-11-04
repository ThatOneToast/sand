"""Minimal offline-mode client for Minecraft 26.2/776 and 26.3/777 audits.

Keeps a real ServerPlayer connected for the requested duration by handling the
login/configuration handshake, keep-alive challenges, and teleport acknowledgments. Packet IDs were read
from each official server jar's protocol registries, not inferred from older IDs.
By default this is a passive test client. Opt-in --control-dir enables bounded
grounded movement and melee attack packets; it is not a gameplay bot or renderer.
An early disconnect is a failure even if the player briefly joined.

Usage:
    python3 minimal_join_client.py <host> <port> <776|777> <username> [<run_seconds>]

An optional final argument runs a shell command when the Play login arrives.
Pass --auto-respawn for death/respawn lifecycle audits; other audits leave it off.
"""

from contextlib import closing
import argparse
import math
import select

import socket
import struct
import sys
import time
import uuid
import zlib


if __package__:
    from .protocols import profile_for_protocol
else:
    from protocols import profile_for_protocol


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




def teleport_reply(data, protocol, previous):
    """Encode the versioned acknowledgment from the server's teleport codec.

    Protocol 777 adds absolute xyz/yaw/pitch to the reply. Its clientbound packet
    contains xyz, velocity xyz, yaw/pitch and a four-byte Relative bitmask. The
    position/rotation addition and pitch clamp follow PositionMoveRotation's
    calculateAbsolute bytecode. Passive sessions only change position here.
    """
    teleport_id, offset = read_varint_bytes(data, 0)
    if protocol == 776:
        return teleport_id, write_varint(teleport_id), previous
    if protocol != 777:
        profile_for_protocol(protocol)
    if len(data) != offset + 60:
        raise ValueError("protocol 777 teleport must contain position, velocity, rotation and relative flags")
    x, y, z, dx, dy, dz, yaw, pitch, flags = struct.unpack(">ddddddffI", data[offset:])
    if flags & ~0x1ff or not all(math.isfinite(value) for value in (x, y, z, dx, dy, dz, yaw, pitch)):
        raise ValueError("invalid protocol 777 teleport coordinates or relative flags")
    position = tuple(value + (previous[index] if flags & (1 << index) else 0)
                     for index, value in enumerate((x, y, z, yaw, pitch)))
    position = (*position[:4], max(-90, min(90, position[4])))
    if not all(math.isfinite(value) for value in position):
        raise ValueError("nonfinite absolute teleport coordinates")
    return teleport_id, write_varint(teleport_id) + struct.pack(">dddff", *position), position


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("host")
    parser.add_argument("port", type=int)
    parser.add_argument("protocol_version", type=int)
    parser.add_argument("username")
    parser.add_argument("run_seconds", nargs="?", type=float, default=30)
    parser.add_argument("on_join", nargs="?")
    parser.add_argument("--auto-respawn", action="store_true")
    parser.add_argument("--control-dir", help="existing private directory for atomic JSON commands/status")
    args = parser.parse_args()
    host, port, protocol_version, username = args.host, args.port, args.protocol_version, args.username
    profile_for_protocol(protocol_version)
    run_seconds, on_join = args.run_seconds, args.on_join
    if run_seconds <= 0:
        raise ValueError("run_seconds must be positive")
    controls = None
    if args.control_dir:
        if __package__:
            from .scripted_input import ScriptedInput
        else:
            from scripted_input import ScriptedInput
        controls = ScriptedInput(args.control_dir, protocol_version)
    conn = Conn(host, port)
    with closing(conn.sock):
        return run_session(conn, host, port, protocol_version, username, run_seconds, on_join,
                           auto_respawn=args.auto_respawn, controls=controls)


def run_session(conn, host, port, protocol_version, username, run_seconds, on_join=None, *, auto_respawn=False, controls=None):
    profile = profile_for_protocol(protocol_version)
    packets = profile["packets"]
    login_in, login_out = packets["login"]["clientbound"], packets["login"]["serverbound"]
    config_out = packets["configuration"]["serverbound"]
    config_in = {packet_id: name for name, packet_id in packets["configuration"]["clientbound"].items()}
    play_in, play_out = packets["play"]["clientbound"], packets["play"]["serverbound"]
    payload = write_varint(protocol_version) + write_string(host) + struct.pack(">H", port) + write_varint(2)
    conn.send_packet(0x00, payload)

    offline_uuid = uuid.uuid3(uuid.NAMESPACE_OID, f"OfflinePlayer:{username}")
    payload = write_string(username) + offline_uuid.bytes
    conn.send_packet(login_out["hello"], payload)

    state = "login"
    deadline = time.monotonic() + run_seconds
    joined = False
    loading = False
    position = (0.0, 0.0, 0.0, 0.0, 0.0)
    while time.monotonic() < deadline:
        try:
            if controls is not None:
                # Drain incoming entity lifecycle packets before processing input.
                # One loop owns every outbound write, including keep-alive and
                # teleport responses. Never shorten a partially read frame's
                # socket timeout merely to poll the local control mailbox.
                readable, _, _ = select.select([conn.sock], [], [], 0.05)
                if not readable:
                    controls.poll(conn, write_varint)
                    continue
            pid, data = conn.recv_packet(timeout=min(8, max(0.001, deadline - time.monotonic())))
        except socket.timeout:
            if controls is not None:
                # select already reported incoming bytes. A stalled partial
                # frame cannot safely be discarded and read as a new packet.
                controls.error = "timed out reading a Minecraft packet"
                controls.publish()
                return False
            continue
        except ConnectionError as e:
            print(f"stopped: {e}")
            return False
        except (ValueError, OSError, OverflowError, RecursionError) as error:
            if controls is None:
                raise
            controls.error = str(error)
            controls.publish()
            return False

        if state == "login":
            if pid == login_in["login_compression"]:
                threshold, _ = read_varint_bytes(data, 0)
                conn.compression = threshold
                print(f"[login] compression threshold={threshold}")
            elif pid == login_in["login_finished"]:
                print("[login] LOGIN SUCCESS")
                conn.send_packet(login_out["login_acknowledged"], b"")
                state = "configuration"
                # Proactively send Client Information (configuration
                # serverbound client_information). Both verified versions have
                # the same ClientInformation codec; some
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
                conn.send_packet(config_out["client_information"], info)
                print("  sent client information")
            elif pid == login_in["login_disconnect"]:
                print(f"[login] DISCONNECT: {data!r}")
                return False
        elif state == "configuration":
            name = config_in.get(pid, f"unknown_0x{pid:02x}")
            print(f"[config] {name} len={len(data)}")
            if name == "keep_alive":
                conn.send_packet(config_out["keep_alive"], data)
            elif name == "ping":
                conn.send_packet(config_out["pong"], data)
            elif name == "select_known_packs":
                # Echo the exact same known-packs list back (serverbound id 0x07).
                conn.send_packet(config_out["select_known_packs"], data)
                print("  -> replied select_known_packs (echo)")
            elif name == "finish_configuration":
                conn.send_packet(config_out["finish_configuration"], b"")
                print("  -> acked finish_configuration, entering play")
                state = "play"
            elif name == "disconnect":
                print(f"  DISCONNECT payload: {data!r}")
                return False
        elif state == "play":
            if controls is not None:
                try:
                    response = controls.observe(pid, data)
                    if response is not None:
                        conn.send_packet(*response)
                        print("[play] chunk batch acknowledged")
                except ValueError as error:
                    controls.error = str(error)
                    controls.publish()
                    return False
            if pid == play_in["keep_alive"] and len(data) == 8:
                conn.send_packet(play_out["keep_alive"], data)
                print("[play] keep_alive echoed")
            elif pid == play_in["player_position"]:
                # ClientboundPlayerPositionPacket starts with the teleport
                # ID. Acknowledging it clears the server's changing-dimension
                # state; without this, a teleported player remains invulnerable.
                teleport_id, acknowledgment, position = teleport_reply(data, protocol_version, position)
                conn.send_packet(play_out["accept_teleportation"], acknowledgment)
                print(f"[play] acknowledged teleport {teleport_id}")
                if loading:
                    conn.send_packet(play_out["player_loaded"], b"")
                    loading = False
                    print("[play] player loaded")
                if controls is not None:
                    controls.teleported()
            elif pid == play_in["player_combat_kill"] and auto_respawn:
                # Verified ServerboundClientCommandPacket codecs use the
                # PERFORM_RESPAWN enum action zero on both versions.
                conn.send_packet(play_out["client_command"], write_varint(0))
                print("[play] requested respawn after death")
            elif pid == play_in["respawn"]:
                loading = True
                if controls is not None:
                    controls.reset()
                print("[play] respawn/dimension change")
            elif pid == play_in["login"]:
                loading = True
                if controls is not None:
                    controls.reset()
                joined = True
                print("[play] LOGIN (join game) packet received -- player entity should now exist")
                if on_join is not None:
                    import subprocess
                    print(f"  firing on-join command: {on_join}")
                    subprocess.run(["bash", "-c", on_join], check=True)
            elif pid == play_in["disconnect"]:
                print(f"[play] DISCONNECT payload: {data!r}")
                return False
            else:
                print(f"[play] packet id=0x{pid:02x} len={len(data)}")

    print(f"final state={state} joined_play={joined}")
    return joined


if __name__ == "__main__":
    ok = main()
    sys.exit(0 if ok else 1)
