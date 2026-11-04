#!/usr/bin/env python3
"""Prove versioned passive-player join, loading, keepalive and reconnect transport.

This fixture uses native commands rather than a Sand gameplay pack. It verifies
the client/harness transport prerequisite; gameplay scenarios remain separate.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import time
import zipfile

from mc_validation.protocols import profile_for_version
from player_validation_harness import PlayerHarness
from vanilla_reload_harness import STARTUP, ValidationFailure


class ProtocolHarness(PlayerHarness):
    def wait_client_trace(self, player, marker, count=1, session=1):
        suffix = "" if session == 1 else f"-{session}"
        path = self.output / f"{player}{suffix}.log"
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                raise ValidationFailure("protocol-server", "server exited while waiting for client trace")
            if path.exists() and path.read_text().count(marker) >= count:
                return
            time.sleep(0.05)
        raise ValidationFailure("protocol-client", f"missing {marker!r} x{count} in {path.name}")


    def confirm_native_damage(self):
        # Socket-send completion is not server acceptance. Retry only while no
        # native damage command has succeeded, and prove both its success result
        # and the corresponding health delta. Never infer success from a health
        # value that environmental damage could have produced independently.
        self.eventually("full_health_before_damage", "if entity @a[name=SandAlpha,nbt={Health:20.0f}]")
        self.send("scoreboard players set #damaged protocol_proof 0")
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            self.send("execute unless score #damaged protocol_proof matches 1 store success score #damaged protocol_proof run damage SandAlpha 1 minecraft:generic")
            marker = "__SAND_WORLD_controlled_damage_OK__"
            if self.passed("controlled_damage", f"execute if score #damaged protocol_proof matches 1 if entity @a[name=SandAlpha,nbt={{Health:19.0f}}] run say {marker}"):
                return
            time.sleep(0.05)
        raise ValidationFailure("protocol-damage", "native damage command never succeeded after dimension acknowledgment")


def validate_jar(jar, profile):
    actual = hashlib.sha1(Path(jar).read_bytes()).hexdigest()
    if actual != profile["server_jar_sha1"]:
        raise ValueError("server jar SHA-1 does not match the authoritative protocol metadata")
    with zipfile.ZipFile(jar) as archive:
        version = json.loads(archive.read("version.json"))
    if version != profile["version"]:
        raise ValueError("server jar version.json does not match the protocol profile")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="new evidence directory")
    parser.add_argument("--version", choices=("26.2", "26.3"), default="26.3")
    parser.add_argument("--timeout", type=float, default=90)
    args = parser.parse_args(argv)
    profile = profile_for_version(args.version)
    validate_jar(args.jar, profile)
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    pack = root / "pack"
    pack.mkdir()
    major = profile["version"]["pack_version"]["data_major"]
    (pack / "pack.mcmeta").write_text(json.dumps({"pack": {"description": "Passive player protocol proof", "min_format": major, "max_format": major}}))
    settings = argparse.Namespace(version=args.version, pack=str(pack), jar=str(args.jar.resolve()), output=str(root / "runtime"), timeout=args.timeout, java="java", op_player=None, client_command=None)
    h = ProtocolHarness(settings)
    checks = []
    result = {"passed": False, "version": args.version, "protocol": profile["version"]["protocol_version"],
              "server_jar_sha1": profile["server_jar_sha1"], "players": 2, "checks": checks}
    try:
        h.prepare()
        h.start()
        h.wait_for(STARTUP, "startup")
        if h.errors(0, len(h.lines)):
            raise ValidationFailure("startup", "server load errors", h.errors(0, len(h.lines)))
        h.send("scoreboard objectives add protocol_proof dummy")
        h.send("forceload add 0 0")
        h.send("execute in minecraft:the_nether run forceload add 0 0")
        h.load_test_chunk("minecraft:overworld")
        h.load_test_chunk("minecraft:the_nether")
        h.send("fill 0 199 0 6 199 6 minecraft:stone")
        h.send("execute in minecraft:the_nether run fill 0 99 0 6 99 6 minecraft:stone")
        h.send("fill 0 200 0 6 204 6 minecraft:air")
        h.send("execute in minecraft:the_nether run fill 0 100 0 6 104 6 minecraft:air")
        h.send("difficulty peaceful")
        with h.clients(players=("SandBeta",)):
            with h.clients(players=("SandAlpha",)):
                for player in ("SandAlpha", "SandBeta"):
                    h.wait_client_trace(player, "[play] LOGIN (join game)")
                    h.wait_client_trace(player, "[play] player loaded")
                    h.wait_client_trace(player, "replied select_known_packs")
                checks.append("two-player login/configuration/play and initial loaded acknowledgments")
                h.send("gamemode survival @a")
                h.send("tp @a 2 200 2")
                h.send("scoreboard players set SandAlpha protocol_proof 11")
                h.send("scoreboard players set SandBeta protocol_proof 22")
                h.send("item replace entity SandAlpha hotbar.0 with minecraft:diamond")
                h.send("item replace entity SandBeta hotbar.0 with minecraft:emerald")
                h.check("owner_isolation", "if score SandAlpha protocol_proof matches 11 if score SandBeta protocol_proof matches 22 if items entity SandAlpha hotbar.0 minecraft:diamond if items entity SandBeta hotbar.0 minecraft:emerald")
                checks.append("independent player score and inventory commands")
                for player in ("SandAlpha", "SandBeta"):
                    h.wait_client_trace(player, "[play] keep_alive echoed")
                checks.append("native keepalive challenges echoed by both players")
                h.send("execute in minecraft:the_nether run tp SandAlpha 2 100 2")
                h.wait_client_trace("SandAlpha", "[play] respawn/dimension change")
                h.wait_client_trace("SandAlpha", "[play] player loaded", 2)
                h.eventually("nether", 'if entity @a[name=SandAlpha,nbt={Dimension:"minecraft:the_nether"}]')
                h.confirm_native_damage()
                h.send("execute in minecraft:overworld run tp SandAlpha 2 200 2")
                h.wait_client_trace("SandAlpha", "[play] respawn/dimension change", 2)
                h.wait_client_trace("SandAlpha", "[play] player loaded", 3)
                h.eventually("returned", 'if entity @a[name=SandAlpha,nbt={Dimension:"minecraft:overworld"}]')
                h.check("other_owner_dimension", 'if entity @a[name=SandBeta,nbt={Dimension:"minecraft:overworld"}] if score SandBeta protocol_proof matches 22')
                checks.append("dimension transfer in both directions with teleport/loading acknowledgments and native damage acceptance")
            with h.clients(players=("SandAlpha",)):
                h.wait_client_trace("SandAlpha", "[play] player loaded", session=2)
                h.check("reconnect_state", "if score SandAlpha protocol_proof matches 11 if items entity SandAlpha hotbar.0 minecraft:diamond if score SandBeta protocol_proof matches 22 if items entity SandBeta hotbar.0 minecraft:emerald")
                h.wait_client_trace("SandBeta", "[play] LOGIN (join game)")
                checks.append("disconnect/reconnect preserves player identity and does not disconnect the other owner")
        errors = h.errors(0, len(h.lines))
        if errors:
            raise ValidationFailure("runtime", "server errors", errors)
        result["passed"] = True
    except (ValidationFailure, OSError, ValueError) as error:
        result["failure"] = {"phase": getattr(error, "phase", "protocol"), "message": str(error)}
        print(str(error), file=sys.stderr)
    finally:
        h.stop()
        (root / "evidence.json").write_text(json.dumps(result, indent=2) + "\n")
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
