"""Shared passive-player lifecycle for verified Minecraft 26.2/26.3 audits."""
from contextlib import contextmanager, ExitStack
from pathlib import Path
import re
import subprocess
import sys
import time

from validate_world_gameplay import WorldGameplayHarness
from vanilla_reload_harness import ValidationFailure
from mc_validation.protocols import profile_for_version


class PlayerHarness(WorldGameplayHarness):
    players = ("SandAlpha", "SandBeta")
    auto_respawn = False

    def client_options(self, player):
        """Additional opt-in client arguments; normal audits remain passive."""
        return []

    def prepare(self):
        profile_for_version(self.args.version)
        super().prepare()
        with (self.server / "server.properties").open("a", encoding="utf-8") as properties:
            properties.write(f"max-players={len(self.players)}\n")

    def eventually(self, name, condition):
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            marker = f"__SAND_WORLD_{name}_OK__"
            if self.passed(name, f"execute {condition} run say {marker}"):
                return
            time.sleep(0.1)
        raise ValidationFailure(name, f"condition did not become true: {condition}")

    @contextmanager
    def clients(self, *, players=None):
        """Connect selected players; nested sessions retain independent logs."""
        protocol = profile_for_version(self.args.version)["version"]["protocol_version"]
        players = self.players if players is None else tuple(players)
        sessions = getattr(self, "_client_sessions", {})
        self._client_sessions = sessions
        with ExitStack() as stack:
            processes = []
            try:
                for player in players:
                    sessions[player] = sessions.get(player, 0) + 1
                    suffix = "" if sessions[player] == 1 else f"-{sessions[player]}"
                    log = stack.enter_context((self.output / f"{player}{suffix}.log").open("w"))
                    start = len(self.lines)
                    process = subprocess.Popen([
                        sys.executable, "-u", str(Path(__file__).parent / "mc_validation/minimal_join_client.py"),
                        "127.0.0.1", str(self.port), str(protocol), player, str(self.args.timeout * 4 + 90),
                    ] + (["--auto-respawn"] if self.auto_respawn else []) + self.client_options(player),
                        stdout=log, stderr=subprocess.STDOUT)
                    processes.append(process)
                    self.wait_for(re.compile(re.escape(player) + r" joined the game$"), "player-join", start)
                yield processes
                if any(process.poll() is not None for process in processes):
                    raise ValidationFailure("player-client", "a client exited during the assertions")
                # Let the server close each connection before terminating the
                # passive client. Abrupt socket teardown can race queued chat
                # packets and produce server-side broken-pipe exceptions.
                for player in players:
                    start = len(self.lines)
                    self.send(f"kick {player} Validation complete")
                    self.wait_for(re.compile(re.escape(player) + r" left the game$"),
                                  "player-leave", start)
            finally:
                for process in processes:
                    process.terminate()
                for process in processes:
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
