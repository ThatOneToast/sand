"""Opt-in, single-session-loop controls for the protocol-776 audit client.

The harness atomically replaces command.json, then waits for status.json's
sent_id before issuing another command. This acknowledges bytes sent, never a
server-accepted action. No thread or control socket can write Minecraft packets.
"""
import json
import math
from pathlib import Path
import struct
import time
import uuid

MAX_COMMAND_BYTES = 4096
MAX_COMMANDS = 32
MAX_DEADLINE_SECONDS = 30


def atomic_json(path, value):
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(value, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(path)


def packet_varint(data, offset=0):
    """Read a bounded nonnegative wire ID/count, rejecting truncated packets."""
    value = 0
    for shift in range(0, 35, 7):
        if offset >= len(data):
            raise ValueError("truncated entity packet")
        byte = data[offset]
        offset += 1
        value |= (byte & 127) << shift
        if not byte & 128:
            if value > 0x7fffffff:
                raise ValueError("invalid entity ID/count")
            return value, offset
    raise ValueError("oversized entity VarInt")


class ScriptedInput:
    def __init__(self, directory, protocol):
        if protocol != 776:
            raise ValueError("scripted input supports protocol 776 only")
        self.directory = Path(directory)
        self.ready = False
        self.teleports = 0
        self.entities = {}
        self.sent_id = 0
        self.error = None
        self.publish()

    def publish(self):
        atomic_json(self.directory / "status.json", {
            "ready": self.ready, "teleports": self.teleports,
            "entities": self.entities, "sent_id": self.sent_id, "error": self.error,
        })

    def reset(self):
        self.ready = False
        self.entities.clear()
        self.publish()

    def teleported(self):
        # Called only after the session loop sent teleport/loading replies.
        self.teleports += 1
        self.ready = True
        self.publish()

    def observe(self, packet_id, data):
        if packet_id == 0x01:  # ClientboundAddEntityPacket
            entity_id, offset = packet_varint(data)
            if len(data) < offset + 16:
                raise ValueError("truncated entity UUID")
            entity_uuid = str(uuid.UUID(bytes=data[offset:offset + 16]))
            # A reused wire ID must not retain a previous UUID binding.
            self.entities = {key: value for key, value in self.entities.items()
                             if value != entity_id and key != entity_uuid}
            if len(self.entities) >= 1024:
                raise ValueError("scripted entity tracking limit exceeded")
            self.entities[entity_uuid] = entity_id
            self.publish()
        elif packet_id == 0x4d:  # ClientboundRemoveEntitiesPacket
            count, offset = packet_varint(data)
            if count > 1024:
                raise ValueError("entity removal limit exceeded")
            removed = set()
            for _ in range(count):
                entity_id, offset = packet_varint(data, offset)
                removed.add(entity_id)
            if offset != len(data):
                raise ValueError("trailing entity removal bytes")
            self.entities = {key: value for key, value in self.entities.items()
                             if value not in removed}
            self.publish()

    def poll(self, conn, write_varint):
        path = self.directory / "command.json"
        if not path.exists():
            return
        with path.open("rb") as source:
            raw = source.read(MAX_COMMAND_BYTES + 1)
        if len(raw) > MAX_COMMAND_BYTES:
            raise ValueError("command exceeds byte limit")
        command = json.loads(raw)
        if not isinstance(command, dict):
            raise ValueError("command must be an object")
        sequence = command.get("id")
        if type(sequence) is not int or not 1 <= sequence <= MAX_COMMANDS:
            raise ValueError("invalid command ID/count")
        if sequence == self.sent_id:
            return  # Atomic mailbox still contains the acknowledged command.
        if sequence != self.sent_id + 1:
            raise ValueError("commands must be sequential")
        deadline = command.get("deadline")
        now = time.monotonic()
        if type(deadline) not in (int, float) or not math.isfinite(deadline) or not now < deadline <= now + MAX_DEADLINE_SECONDS:
            raise ValueError("command deadline expired or exceeds 30 seconds")
        if not self.ready:
            raise ValueError("client is not ready")
        action = command.get("action")
        if action == "attack":
            if set(command) != {"id", "deadline", "action", "uuid"}:
                raise ValueError("invalid attack fields")
            if not isinstance(command["uuid"], str):
                raise ValueError("target UUID must be a string")
            target = str(uuid.UUID(command["uuid"]))
            if target not in self.entities:
                raise ValueError("target UUID is not currently tracked")
            packet_id, payload = 0x01, write_varint(self.entities[target])
        elif action == "move_pos_rot":
            if set(command) != {"id", "deadline", "action", "position"}:
                raise ValueError("invalid movement fields")
            position = command["position"]
            if not isinstance(position, list) or len(position) != 5:
                raise ValueError("position must contain x y z yaw pitch")
            limits = (30000000, 20000000, 30000000, 360, 90)
            if any(type(value) not in (int, float) or not math.isfinite(value) or abs(value) > limit
                   for value, limit in zip(position, limits)):
                raise ValueError("invalid movement coordinates")
            # Harness only supports grounded movement, no horizontal collision.
            packet_id, payload = 0x1f, struct.pack(">dddffB", *position, 1)
        else:
            raise ValueError("unsupported scripted action")
        conn.send_packet(packet_id, payload)
        self.sent_id = sequence
        self.publish()
