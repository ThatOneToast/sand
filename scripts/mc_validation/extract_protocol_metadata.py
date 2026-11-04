#!/usr/bin/env python3
"""Extract packet registration order from an official bundled Minecraft server jar.

Requires javap from the server's supported JDK. No server startup, downloads,
source checkout mutation, or inferred packet offsets are involved.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile
import zipfile

REGISTRIES = {
    "login": "net.minecraft.network.protocol.login.LoginProtocols",
    "configuration": "net.minecraft.network.protocol.configuration.ConfigurationProtocols",
    "play": "net.minecraft.network.protocol.game.GameProtocols",
}
SHAPES = [
    "net.minecraft.network.protocol.ProtocolInfoBuilder",
    "net.minecraft.network.protocol.ProtocolCodecBuilder",
    "net.minecraft.server.level.ClientInformation",
    "net.minecraft.world.entity.PositionMoveRotation",
    "net.minecraft.world.entity.Relative",
    "net.minecraft.server.network.ServerGamePacketListenerImpl",
    "net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket",
    "net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket",
    "net.minecraft.network.protocol.game.ServerboundPlayerLoadedPacket",
    "net.minecraft.network.protocol.game.ServerboundClientCommandPacket",
]


def parse_registry(bytecode):
    result = {}
    for direction in ("CLIENTBOUND", "SERVERBOUND"):
        names = re.findall(r"// Field net/minecraft/network/protocol/\w+/\w+PacketTypes\.(" + direction + r"_\w+):Lnet/minecraft/network/protocol/PacketType;", bytecode)
        if not names or len(names) != len(set(names)):
            raise ValueError(f"missing or duplicate {direction} packet registration; inspect bytecode")
        result[direction.lower()] = {name.removeprefix(direction + "_").lower(): index
                                     for index, name in enumerate(names)}
    return result


def extract(jar, javap="javap"):
    jar = Path(jar)
    bytecodes = {}
    with zipfile.ZipFile(jar) as bundled:
        version = json.loads(bundled.read("version.json"))
        records = [line.split("\t") for line in bundled.read("META-INF/versions.list").decode().splitlines()]
        matching = [record for record in records if len(record) == 3 and record[1] == version["id"]]
        if len(matching) != 1:
            raise ValueError("bundled jar must contain exactly one matching server version")
        expected, _, member = matching[0]
        content = bundled.read("META-INF/versions/" + member)
        if hashlib.sha256(content).hexdigest() != expected:
            raise ValueError("nested server jar digest does not match official bundle metadata")
    with tempfile.TemporaryDirectory(prefix="sand-packet-metadata-") as directory:
        nested = Path(directory) / "server.jar"
        nested.write_bytes(content)
        classes = {}
        with zipfile.ZipFile(nested) as archive:
            for name in [*REGISTRIES.values(), *SHAPES]:
                classes[name] = hashlib.sha256(archive.read(name.replace(".", "/") + ".class")).hexdigest()
                bytecodes[name] = subprocess.check_output([javap, "-classpath", str(nested), "-c", "-p", name], text=True)
    metadata = {
        "schema_version": 1,
        "version": version,
        "server_jar_sha1": hashlib.sha1(jar.read_bytes()).hexdigest(),
        "server_jar_sha256": hashlib.sha256(jar.read_bytes()).hexdigest(),
        "nested_jar_sha256": expected,
        "class_sha256": classes,
        "derivation": "javap registration order; ProtocolInfoBuilder appends codec entries, ProtocolCodecBuilder assigns sequential IDs. Game clientbound bundle entry occupies delimiter ID zero.",
        "teleport_ack_fields": [{"type": kind, "name": name} for kind, name in re.findall(
            r"^  private final (int|double|float) (\w+);$",
            bytecodes["net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket"], re.MULTILINE)],
        "packets": {stage: parse_registry(bytecodes[name]) for stage, name in REGISTRIES.items()},
    }
    return metadata, bytecodes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--javap", default="javap")
    parser.add_argument("--bytecode-output", type=Path)
    args = parser.parse_args()
    metadata, bytecodes = extract(args.jar, args.javap)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(metadata, indent=2, sort_keys=True) + "\n")
    if args.bytecode_output:
        args.bytecode_output.parent.mkdir(parents=True, exist_ok=True)
        args.bytecode_output.write_text("\n".join(bytecodes.values()))


if __name__ == "__main__":
    main()
