#!/usr/bin/env python3
"""Validate the portable counter on 26.2 with real players and explicit ticks.

The minimal join clients only stay connected briefly. Freeze automatic ticks,
then run a separate audit helper while both real player entities are online.
This records explicit lifecycle/tick invocations, not automatic tick evidence.
"""
import argparse
import json
import queue
import re
import secrets
import shlex
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path
from rcon_client import run_commands
from run_unified_state_audit import available_port, wait_for_startup

ROOT = Path(__file__).resolve().parents[2]
VALUE = "sffe99bc0114fb8b"
PRESENCE = "s048b722efdc0917"
SUPPRESSION = "sd5409a75751a99c"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/sand")
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--initial-value", type=int, default=0,
                        help="Player State default; this increment workload needs four points of i32 headroom")
    args = parser.parse_args()
    if not -(2**31) <= args.initial_value <= 2**31 - 5:
        parser.error("--initial-value must fit i32 with four increments of headroom")
    checks, logs = {}, []
    token = secrets.token_urlsafe(24)
    port, rcon_port = available_port(), available_port()
    server = Path(tempfile.mkdtemp(prefix="sand-portable-audit-")).resolve()
    pack = server / "world/datapacks/demo"
    document = json.loads((ROOT / "tests/fixtures/program/counter.sand.json").read_text())
    document["modules"][0]["states"][0]["fields"][0]["default"] = args.initial_value
    input_path = server / "counter.sand.json"
    input_path.write_text(json.dumps(document))
    compile_result = subprocess.run([str(args.binary.resolve()), "program", "compile", "--input",
        str(input_path), "--output", str(pack)],
        capture_output=True, text=True, timeout=30, check=True)
    response = json.loads(compile_result.stdout)
    assert response["success"]
    audit = server / "world/datapacks/portable_audit"
    (audit / "data/portable_audit/function").mkdir(parents=True)
    shutil.copyfile(pack / "pack.mcmeta", audit / "pack.mcmeta")
    commands = [
        "execute if score #done portable_audit matches 1 run return 0",
        "execute unless entity @a[name=CounterOne] run return 0",
        "execute unless entity @a[name=CounterTwo] run return 0",
        f"scoreboard players reset @a {VALUE}",
        f"scoreboard players reset @a {PRESENCE}",
        f"scoreboard players reset @a {SUPPRESSION}",
        "execute as @a[name=CounterOne] run function demo:increment",
        f"scoreboard players operation #direct portable_audit = CounterOne {VALUE}",
        f"scoreboard players operation #presence portable_audit = CounterOne {PRESENCE}",
    ]
    for _ in range(3):
        commands.extend(["function demo:__sand_lifecycle_tick", "function demo:tick"])
    commands.extend([
        f"scoreboard players operation #one portable_audit = CounterOne {VALUE}",
        f"scoreboard players operation #two portable_audit = CounterTwo {VALUE}",
        "scoreboard players set #done portable_audit 1",
    ])
    (audit / "data/portable_audit/function/check.mcfunction").write_text("\n".join(commands))
    (server / "eula.txt").write_text("eula=true\n")
    (server / "server.properties").write_text("\n".join([
        "level-name=world", "online-mode=false", "enable-rcon=true", f"rcon.password={token}",
        f"rcon.port={rcon_port}", f"server-port={port}", "server-ip=127.0.0.1", "view-distance=2",
        "simulation-distance=2", "generate-structures=false", "spawn-protection=0", "sync-chunk-writes=false", ""]))
    process = subprocess.Popen([args.java, "-Xmx1G", "-jar", str(args.jar.resolve()), "nogui"],
        cwd=server, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)
    lines = queue.Queue()
    def drain():
        for line in process.stdout:
            logs.append(line); lines.put(line)
    threading.Thread(target=drain, daemon=True).start()
    def command(*commands):
        return run_commands("127.0.0.1", rcon_port, token, list(commands))
    def score(holder, objective="portable_audit"):
        output = command(f"scoreboard players get {holder} {objective}")[0]
        match = re.search(r"has (-?\d+) \[", output)
        return int(match.group(1)) if match else None
    def check(name, passed, evidence):
        checks[name] = {"passed": bool(passed), "evidence": evidence}
        print(f"{'PASS' if passed else 'FAIL'} {name}: {evidence}", flush=True)
    clients = []
    passed = False
    try:
        # World readiness can precede the RCON listener. Reuse the readiness
        # barrier exercised by test_unified_state_audit instead of racing it.
        ready, startup = wait_for_startup(process, lines, 120)
        check("server_startup", ready, "".join(logs[-5:]))
        if not ready: return 1
        version = any(re.search(r"Starting minecraft server version 26\.2$", line.strip()) for line in startup)
        check("verified_server_version", version, "Minecraft 26.2 startup banner")
        if not version: return 1
        check("pack_loaded", "file/demo" in command("datapack list")[0], command("datapack list"))
        frozen = command("tick freeze", "scoreboard objectives add portable_audit dummy", "scoreboard players set #done portable_audit 0")
        check("automatic_ticks_frozen", "frozen" in frozen[0].lower(), frozen[0])
        def join(name, command_text):
            callback = shlex.join([sys.executable, str(ROOT / "scripts/mc_validation/rcon_client.py"), "127.0.0.1", str(rcon_port), token, command_text])
            client = subprocess.Popen([sys.executable, str(ROOT / "scripts/mc_validation/minimal_join_client.py"),
                "127.0.0.1", str(port), "776", name, "8", callback], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            clients.append(client)
            return client
        # Repeat a bounded number of joins to overlap the clients' brief play windows.
        for _ in range(3):
            pair = [join(name, "function portable_audit:check") for name in ["CounterOne", "CounterTwo"]]
            for client in pair:
                try: client.communicate(timeout=12)
                except subprocess.TimeoutExpired: client.kill(); client.communicate()
            if score("#done") == 1: break
        observed = {name: score(name) for name in ["#done", "#direct", "#presence", "#one", "#two"]}
        expected = {"#done":1,"#direct":args.initial_value + 1,"#presence":1,
                    "#one":args.initial_value + 4,"#two":args.initial_value + 3}
        check("two_real_players_explicit_ticks", observed == expected, {"observed":observed,"expected":expected})
        before = [score("CounterOne",VALUE), score("CounterTwo",VALUE)]
        command("reload")
        time.sleep(1)
        after = [score("CounterOne",VALUE), score("CounterTwo",VALUE)]
        check("reload_preserves_scores", before == after == [args.initial_value + 4,args.initial_value + 3], {"before":before,"after":after})
        client = join("CounterNew", "execute as @a[name=CounterNew] run function demo:increment")
        try: client.communicate(timeout=12)
        except subprocess.TimeoutExpired: client.kill(); client.communicate()
        joined = [score("CounterNew",VALUE),score("CounterOne",VALUE),score("CounterTwo",VALUE)]
        check("joining_player_isolated", joined == [args.initial_value + 1,args.initial_value + 4,args.initial_value + 3], joined)
        passed = all(check["passed"] for check in checks.values())
        return 0 if passed else 1
    finally:
        for client in clients:
            if client.poll() is None: client.kill(); client.wait(timeout=5)
        if process.poll() is None:
            try: command("stop"); process.wait(timeout=20)
            except Exception:
                process.terminate()
                try: process.wait(timeout=5)
                except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=5)
        args.evidence.parent.mkdir(parents=True, exist_ok=True)
        args.evidence.write_text(json.dumps({"minecraft":"26.2", "passed":passed, "initial_value":args.initial_value, "tick_mode":"frozen automatic ticks; three explicitly invoked lifecycle/tick pairs", "checks":checks,
            "server_log_tail":logs[-30:]},indent=2)+"\n")
        shutil.rmtree(server)


if __name__ == "__main__":
    raise SystemExit(main())
