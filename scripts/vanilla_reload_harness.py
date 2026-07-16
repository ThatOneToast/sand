#!/usr/bin/env python3
"""Run a generated datapack through vanilla startup and an actual reload."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import queue
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
import uuid
from pathlib import Path

STARTUP = re.compile(r"Done \([^)]+\)! For help")
RELOAD_START = re.compile(r"Reloading!|Reloading resources")
RELOAD_SUBMITTED = "__SAND_RELOAD_SUBMITTED__"
RELOAD_COMPLETE = "__SAND_RELOAD_COMPLETE__"
ERROR_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in (
        r"Failed to load (?:datapack|data packs?)",
        r"(?:Could not|Couldn't|Failed to) load (?:recipe|loot table|predicate|advancement|function|tag|item modifier)",
        r"(?:Could not|Couldn't|Failed to) parse",
        r"Unknown (?:command|function|tag)",
        r"JsonParseException|DatapackLoadFailedException",
        r"incompatible.*pack(?:_format| format)",
        r"Caused by: .*(?:Exception|Error)",
        r"Exception in server tick loop|Encountered an unexpected exception|\[Server thread/ERROR\]",
    )
]


class ValidationFailure(RuntimeError):
    def __init__(self, phase: str, message: str, matches: list[str] | None = None):
        super().__init__(message)
        self.phase = phase
        self.matches = matches or []


class ServerHarness:
    def __init__(self, args: argparse.Namespace):
        self.args = args
        self.output = Path(args.output).resolve() if args.output else Path(
            tempfile.mkdtemp(prefix=f"sand-vanilla-{args.version}-")
        )
        self.server = self.output / "server"
        self.log_path = self.output / "latest.log"
        self.lines: list[str] = []
        self.events: queue.Queue[str] = queue.Queue()
        self.process: subprocess.Popen[str] | None = None
        self.reader: threading.Thread | None = None
        self.client_cwd = Path.cwd()
        self.port = self._available_port()

    @staticmethod
    def _available_port() -> int:
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            return int(listener.getsockname()[1])

    @staticmethod
    def _offline_uuid(player: str) -> str:
        digest = bytearray(hashlib.md5(f"OfflinePlayer:{player}".encode()).digest())
        digest[6] = (digest[6] & 0x0F) | 0x30
        digest[8] = (digest[8] & 0x3F) | 0x80
        return str(uuid.UUID(bytes=bytes(digest)))

    def prepare(self) -> None:
        if self.output.exists() and self.args.output:
            shutil.rmtree(self.output)
        pack = Path(self.args.pack).resolve()
        destination = self.server / "world" / "datapacks" / pack.name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(pack, destination)
        (self.server / "eula.txt").write_text("eula=true\n", encoding="utf-8")
        if self.args.op_player:
            (self.server / "ops.json").write_text(
                json.dumps(
                    [
                        {
                            "uuid": self._offline_uuid(self.args.op_player),
                            "name": self.args.op_player,
                            "level": 4,
                            "bypassesPlayerLimit": True,
                        }
                    ]
                ),
                encoding="utf-8",
            )
        (self.server / "server.properties").write_text(
            "\n".join(
                [
                    "level-name=world",
                    "online-mode=false",
                    "max-players=1",
                    "enable-rcon=false",
                    f"server-port={self.port}",
                    "spawn-npcs=false",
                    "spawn-animals=false",
                    "spawn-monsters=false",
                    "generate-structures=false",
                    "view-distance=2",
                    "simulation-distance=2",
                    "sync-chunk-writes=false",
                    "",
                ]
            ),
            encoding="utf-8",
        )

    def _read_output(self) -> None:
        assert self.process and self.process.stdout
        with self.log_path.open("w", encoding="utf-8") as log:
            for line in self.process.stdout:
                log.write(line)
                log.flush()
                text = line.rstrip("\n")
                self.lines.append(text)
                self.events.put(text)
                print(text, flush=True)

    def start(self) -> None:
        command = [self.args.java, "-jar", str(Path(self.args.jar).resolve()), "nogui"]
        self.process = subprocess.Popen(
            command,
            cwd=self.server,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
            env=os.environ.copy(),
        )
        self.reader = threading.Thread(target=self._read_output, daemon=True)
        self.reader.start()

    def send(self, command: str) -> None:
        if not self.process or self.process.poll() is not None or not self.process.stdin:
            raise ValidationFailure("reload", "server exited before command could be sent")
        self.process.stdin.write(command + "\n")
        self.process.stdin.flush()

    def wait_for(self, pattern: re.Pattern[str] | str, phase: str, start: int = 0) -> int:
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            for index in range(start, len(self.lines)):
                line = self.lines[index]
                matched = pattern in line if isinstance(pattern, str) else pattern.search(line)
                if matched:
                    return index
            if self.process and self.process.poll() is not None:
                raise ValidationFailure(
                    phase,
                    f"server exited with code {self.process.returncode}",
                    self.errors(start, len(self.lines)),
                )
            try:
                self.events.get(timeout=0.1)
            except queue.Empty:
                pass
        marker = pattern if isinstance(pattern, str) else pattern.pattern
        raise ValidationFailure(
            phase,
            f"timed out after {self.args.timeout}s waiting for {marker}",
            self.errors(start, len(self.lines)),
        )

    def errors(self, start: int, end: int) -> list[str]:
        return [
            line
            for line in self.lines[start:end]
            if any(pattern.search(line) for pattern in ERROR_PATTERNS)
        ]

    def stop(self) -> None:
        if not self.process or self.process.poll() is not None:
            return
        try:
            self.send("stop")
            self.process.wait(timeout=self.args.timeout)
        except (ValidationFailure, subprocess.TimeoutExpired):
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=5)
        if self.reader:
            self.reader.join(timeout=2)

    def run_client(self) -> None:
        if not self.args.client_command:
            return
        env = os.environ.copy()
        env["SAND_SERVER_HOST"] = "127.0.0.1"
        env["SAND_SERVER_PORT"] = str(self.port)
        env["SAND_MC_VERSION"] = self.args.version
        try:
            result = subprocess.run(
                self.args.client_command,
                cwd=self.client_cwd,
                env=env,
                text=True,
                capture_output=True,
                timeout=self.args.timeout,
            )
        except subprocess.TimeoutExpired as error:
            output = (error.stdout or "") + (error.stderr or "")
            raise ValidationFailure(
                "semantic-client",
                f"client timed out after {self.args.timeout}s",
                output.splitlines(),
            ) from error
        if result.stdout:
            print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
        if result.stderr:
            print(
                result.stderr,
                file=sys.stderr,
                end="" if result.stderr.endswith("\n") else "\n",
            )
        if result.returncode != 0:
            raise ValidationFailure(
                "semantic-client",
                f"client stopped with code {result.returncode}",
                [line for line in (result.stdout + result.stderr).splitlines() if line],
            )

    def run(self) -> None:
        self.prepare()
        self.start()
        try:
            startup = self.wait_for(STARTUP, "startup")
            initial_errors = self.errors(0, startup + 1)
            if initial_errors:
                raise ValidationFailure("initial-load", "datapack errors during startup", initial_errors)

            client_offset = len(self.lines)
            self.run_client()
            client_errors = self.errors(client_offset, len(self.lines))
            if client_errors:
                raise ValidationFailure(
                    "semantic-client",
                    "datapack/server errors during client execution",
                    client_errors,
                )

            reload_offset = len(self.lines)
            self.send(f"say {RELOAD_SUBMITTED}")
            self.wait_for(RELOAD_SUBMITTED, "reload-submit", reload_offset)
            self.send("reload")
            self.send(f"say {RELOAD_COMPLETE}")
            reload_started = self.wait_for(RELOAD_START, "reload-start", reload_offset)
            reload_complete = self.wait_for(RELOAD_COMPLETE, "reload-complete", reload_started + 1)
            reload_errors = self.errors(reload_offset, reload_complete + 1)
            if reload_errors:
                raise ValidationFailure("reload", "datapack errors during reload", reload_errors)

            self.send("stop")
            assert self.process
            try:
                code = self.process.wait(timeout=self.args.timeout)
            except subprocess.TimeoutExpired as error:
                raise ValidationFailure("shutdown", "server did not stop cleanly") from error
            if code != 0:
                raise ValidationFailure("shutdown", f"server stopped with code {code}")
            if self.reader:
                self.reader.join(timeout=2)
        finally:
            self.stop()


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--pack", required=True)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--output")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--java", default="java", help=argparse.SUPPRESS)
    parser.add_argument("--op-player")
    parser.add_argument("--client-command", nargs=argparse.REMAINDER)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv or sys.argv[1:])
    harness = ServerHarness(args)
    started = time.monotonic()
    try:
        harness.run()
    except ValidationFailure as error:
        print(f"FAILED: version={args.version} phase={error.phase}: {error}", file=sys.stderr)
        for line in error.matches:
            print(f"  > {line}", file=sys.stderr)
        print(f"log: {harness.log_path}", file=sys.stderr)
        print(f"pack: {Path(args.pack).resolve()}", file=sys.stderr)
        return 1
    print(
        f"PASSED: version={args.version} startup=ok reload-start=ok "
        f"reload-complete=ok shutdown=ok elapsed={time.monotonic() - started:.1f}s"
    )
    print(f"log: {harness.log_path}")
    print(f"pack: {Path(args.pack).resolve()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
