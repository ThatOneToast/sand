#!/usr/bin/env python3
"""Build timing and isolated, reproducible Cargo-check scenarios.

The suite never edits the source checkout or removes caches. Its snapshot and all
Cargo targets live in a fresh output directory outside the checkout.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import signal
import stat
import statistics
import subprocess
import sys
import tempfile
import time

EXCLUDED_PARTS = {".git", "target", "dist"}


def utc_now():
    return datetime.now(timezone.utc).isoformat()


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args])


def selected_files(root):
    paths = git(root, "ls-files", "-z", "--cached", "--others", "--exclude-standard")
    return sorted({os.fsdecode(value) for value in paths.split(b"\0") if value
                   and not EXCLUDED_PARTS.intersection(Path(os.fsdecode(value)).parts)})


def tree_entries(root):
    """Read current worktree bytes, including dirty and nonignored untracked files."""
    entries = []
    for name in selected_files(root):
        path = root / name
        # Tracked deletions are part of the current dirty tree, not copy errors.
        if not path.exists() and not path.is_symlink():
            continue
        mode = path.lstat().st_mode
        if stat.S_ISLNK(mode):
            try:
                target = path.resolve(strict=True).relative_to(root)
            except (ValueError, FileNotFoundError, RuntimeError) as error:
                raise ValueError(f"unsafe external or dangling symlink: {name}") from error
            if EXCLUDED_PARTS.intersection(target.parts):
                raise ValueError(f"symlink targets excluded build output: {name}")
            # Normalize internal absolute links to snapshot-relative links.
            link = os.path.relpath(root / target, path.parent)
            entries.append({"path": name, "kind": "symlink", "target": link})
        elif stat.S_ISREG(mode):
            # A symlink in an ancestor must not escape the source tree either.
            if not path.resolve().is_relative_to(root):
                raise ValueError(f"file escapes source tree through symlink: {name}")
            entries.append({"path": name, "kind": "file", "executable": bool(mode & 0o111),
                            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
        else:
            raise ValueError(f"unsupported snapshot entry (submodule or special file): {name}")
    return entries


def fingerprint(entries):
    return hashlib.sha256(json.dumps(entries, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def snapshot_tree(root, destination):
    """Copy bytes, never hardlink; detect concurrent source edits before measuring."""
    entries = tree_entries(root)
    destination.mkdir()
    for entry in entries:
        source = root / entry["path"]
        target = destination / entry["path"]
        target.parent.mkdir(parents=True, exist_ok=True)
        if entry["kind"] == "symlink":
            target.symlink_to(entry["target"])
        else:
            content = source.read_bytes()
            if hashlib.sha256(content).hexdigest() != entry["sha256"]:
                raise ValueError(f"source changed during snapshot: {entry['path']}; retry when edits stop")
            target.write_bytes(content)
            target.chmod(0o755 if entry["executable"] else 0o644)
    if tree_entries(root) != entries:
        raise ValueError("source tree changed during snapshot; retry when edits stop")
    # Included links must resolve within the copied snapshot, not depend on an
    # ignored/untracked-excluded target that happened to exist in the checkout.
    for entry in entries:
        if entry["kind"] == "symlink":
            target = destination / entry["path"]
            try:
                resolved = target.resolve(strict=True)
            except (FileNotFoundError, RuntimeError) as error:
                raise ValueError(f"symlink target is absent from snapshot: {entry['path']}") from error
            if not resolved.is_relative_to(destination):
                raise ValueError(f"symlink escapes snapshot: {entry['path']}")
    return entries


def measure(command, cwd, env, log):
    """Retain output and a structured failure, including a missing executable."""
    started = time.perf_counter()
    record = {"command": list(command), "cwd": str(cwd), "log": str(log)}
    with log.open("w", encoding="utf-8") as output:
        try:
            process = subprocess.Popen(command, cwd=cwd, env=env, stdout=output,
                                       stderr=subprocess.STDOUT, start_new_session=True)
        except OSError as error:
            output.write(f"{type(error).__name__}: {error}\n")
            record.update(returncode=127, error=str(error))
        else:
            try:
                record["returncode"] = process.wait()
            except KeyboardInterrupt:
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
                record.update(returncode=130, error="interrupted")
    record["seconds"] = time.perf_counter() - started
    return record


def summary(samples):
    successful = [sample["seconds"] for sample in samples if sample["returncode"] == 0]
    if not successful:
        return {"successful_samples": 0}
    values = sorted(successful)
    result = {"successful_samples": len(values), "median_seconds": statistics.median(values),
              "min_seconds": values[0], "max_seconds": values[-1]}
    # Small samples cannot support a useful tail estimate. Nearest-rank p95 is
    # explicitly defined when enough samples have actually completed.
    if len(values) >= 20:
        result["p95_seconds"] = values[math.ceil(0.95 * len(values)) - 1]
    return result


def write_result(output, result):
    path = output / "results.json"
    temporary = output / "results.json.tmp"
    temporary.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    temporary.replace(path)


def output_directory(value, root=None):
    if value:
        output = Path(value).expanduser().resolve()
        if root and output.is_relative_to(root):
            raise ValueError("benchmark output must be outside the source checkout")
        output.mkdir(parents=True, exist_ok=False)
    else:
        temporary_root = Path(tempfile.gettempdir()).resolve()
        if root and temporary_root.is_relative_to(root):
            raise ValueError("temporary output is inside source checkout; choose --output outside it")
        output = Path(tempfile.mkdtemp(prefix="sand-build-bench-", dir=temporary_root)).resolve()
    return output


def tool_version(command):
    try:
        completed = subprocess.run(command, text=True, stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT, check=False)
        return {"command": command, "returncode": completed.returncode, "output": completed.stdout.strip()}
    except OSError as error:
        return {"command": command, "returncode": 127, "error": str(error)}


def suite(args):
    root = Path(args.source).resolve()
    output = output_directory(args.output, root)
    print(f"Build benchmark artifacts: {output}", file=sys.stderr, flush=True)
    env = dict(os.environ, SAND_MC_VERSION=args.version, SAND_STRICT_CODEGEN="1", SAND_ALLOW_PLACEHOLDER_CODEGEN="0")
    # Resolve cache/executable paths before changing the subprocess working
    # directory into the snapshot, so a caller's relative setting stays stable.
    env["CARGO_HOME"] = str(Path(env.get("CARGO_HOME", "~/.cargo")).expanduser().resolve())
    cargo_executable = str(Path(args.cargo).expanduser().resolve()) if os.sep in args.cargo else args.cargo
    edit_scope = getattr(args, "edit_scope", "facade")
    edit_paths = {"facade": "sand/src/lib.rs", "core": "sand-core/src/lib.rs",
                  "contracts": "sand/src/api_contracts.rs", "codegen": "sand-build/src/codegen/mod.rs"}
    test_compile = getattr(args, "test_compile", False)
    command = [cargo_executable, "test", "--workspace", "--locked", "--all-features", "--no-run"] if test_compile else [cargo_executable, "check", "--workspace", "--locked"]
    result = {"schema_version": 1, "started_at": utc_now(), "passed": False, "scenarios": [],
              "source": str(root), "output": str(output), "minecraft_version": args.version,
              "method": "isolated compilation; external caches primed by unmeasured warmup",
              "command": command, "edit_scope": edit_scope, "edited_path": edit_paths[edit_scope],
              "external_caches": {"cargo_home": str(Path(env.get("CARGO_HOME", "~/.cargo")).expanduser().resolve()),
                                  "sand_cache": str(Path.home() / ".sand/cache")},
              "environment": {key: env[key] for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER",
                  "RUSTC_WORKSPACE_WRAPPER", "RUSTC", "CARGO_BUILD_RUSTC", "CARGO_BUILD_TARGET", "CARGO_BUILD_JOBS", "CARGO_INCREMENTAL", "RUSTUP_TOOLCHAIN",
                  "CARGO_PROFILE_DEV_DEBUG", "CARGO_PROFILE_TEST_DEBUG", "CARGO_PROFILE_DEV_OPT_LEVEL", "CARGO_PROFILE_TEST_OPT_LEVEL",
                  "SAND_MC_VERSION", "SAND_STRICT_CODEGEN", "SAND_ALLOW_PLACEHOLDER_CODEGEN") if key in env},
              "platform": {"description": platform.platform(), "machine": platform.machine(), "cpu_count": os.cpu_count(), "python": sys.version}}
    original_source = None
    edited_source = output / "source" / edit_paths[edit_scope]
    try:
        if Path(result["external_caches"]["cargo_home"]).is_relative_to(root):
            raise ValueError("CARGO_HOME must be an external cache outside the source checkout")
        if Path(os.fsdecode(git(root, "rev-parse", "--show-toplevel")).strip()).resolve() != root:
            raise ValueError("--source must identify the Git checkout root")
        result["git"] = {"head": git(root, "rev-parse", "HEAD").decode().strip(),
                         "status_porcelain_v1": os.fsdecode(git(root, "status", "--porcelain=v1", "-z", "--untracked-files=all"))}
        entries = snapshot_tree(root, output / "source")
        result["tree_sha256"] = fingerprint(entries)
        (output / "tree-manifest.json").write_text(json.dumps(entries, indent=2) + "\n")
        result["snapshot"] = str(output / "source")
        result["toolchain"] = {"cargo": tool_version([cargo_executable, "--version"]),
                               "rustc": tool_version([env.get("RUSTC", "rustc"), "-vV"]), "java": tool_version(["java", "-version"])}
        cargo_home = Path(result["external_caches"]["cargo_home"])
        result["external_cargo_config"] = [
            {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
            for path in (cargo_home / "config", cargo_home / "config.toml") if path.is_file()
        ]
        original_source = edited_source.read_bytes()
        warm_target = output / "targets/warm"

        def run(name, sample, target, measured=True):
            target.mkdir(parents=True, exist_ok=name != "cold-target-warm-external-caches")
            log = output / "logs" / f"{name}-{sample}.log"
            log.parent.mkdir(exist_ok=True)
            sample_env = dict(env, CARGO_TARGET_DIR=str(target))
            record = measure(command, output / "source", sample_env, log)
            record.update(scenario=name, sample=sample, measured=measured, target=str(target))
            result["scenarios"].append(record)
            write_result(output, result)
            print(f"{name} [{sample}]: {record['seconds']:.3f}s (exit {record['returncode']}); {log}", file=sys.stderr, flush=True)
            if record["returncode"]:
                result["failure"] = {"scenario": name, "sample": sample, "returncode": record["returncode"]}
                raise RuntimeError(f"{name} sample {sample} failed (exit {record['returncode']}); see {log}")

        run("cache-warmup", 1, warm_target, measured=False)
        for sample in range(1, args.cold_samples + 1):
            run("cold-target-warm-external-caches", sample, output / f"targets/cold-{sample}")
        for sample in range(1, args.warm_samples + 1):
            edited_source.write_bytes(original_source + f"\n// Sand isolated {edit_scope} benchmark sample {sample}.\n".encode())
            run(f"warm-{edit_scope}-rebuild", sample, warm_target)
        edited_source.write_bytes(original_source)
        run("restore-warmup", 1, warm_target, measured=False)
        for sample in range(1, args.warm_samples + 1):
            run("no-op", sample, warm_target)
        result["passed"] = True
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError, KeyboardInterrupt) as error:
        result["error"] = f"{type(error).__name__}: {error}"
        print(result["error"], file=sys.stderr)
    finally:
        if original_source is not None:
            try:
                if not edited_source.exists() or edited_source.read_bytes() != original_source:
                    edited_source.write_bytes(original_source)
            except OSError as error:
                result["passed"] = False
                result["restoration_error"] = str(error)
        result["finished_at"] = utc_now()
        names = sorted({sample["scenario"] for sample in result["scenarios"] if sample["measured"]})
        result["summary"] = {name: summary([sample for sample in result["scenarios"] if sample["scenario"] == name]) for name in names}
        write_result(output, result)
    print(output / "results.json")
    return 0 if result["passed"] else 1


def single(args):
    output = output_directory(args.output)
    command = args.command
    if command and command[0] == "--":
        command = command[1:]
    if not command:
        raise ValueError("single measurement requires a command after --")
    result = {"schema_version": 1, "started_at": utc_now(), "passed": True, "scenario": args.name, "samples": []}
    for sample in range(1, args.samples + 1):
        record = measure(command, Path.cwd(), os.environ.copy(), output / f"sample-{sample}.log")
        result["samples"].append(record)
        if record["returncode"]:
            result["passed"] = False
            break
    result.update(finished_at=utc_now(), summary=summary(result["samples"]))
    write_result(output, result)
    print(f"\n### {args.name} (n={len(result['samples'])})\n")
    print("samples: " + " ".join(f"{sample['seconds']:.3f}s" for sample in result["samples"]))
    print(f"\nArtifacts: {output / 'results.json'}")
    if result["passed"]:
        values = result["summary"]
        print("\n| median | min | max |\n|---|---|---|")
        print(f"| {values['median_seconds']:.3f}s | {values['min_seconds']:.3f}s | {values['max_seconds']:.3f}s |")
    else:
        print(f"Sample failed (exit {result['samples'][-1]['returncode']}); see {result['samples'][-1]['log']}", file=sys.stderr)
    status = result["samples"][-1]["returncode"]
    return 0 if result["passed"] else status if 0 < status < 256 else 1


def positive(value):
    number = int(value)
    if number <= 0:
        raise argparse.ArgumentTypeError("sample count must be positive")
    return number


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="mode", required=True)
    command = commands.add_parser("suite", help="snapshot and measure Cargo checks without editing the checkout")
    command.add_argument("--source", default=str(Path(__file__).resolve().parents[1]))
    command.add_argument("--output", help="new artifact directory outside source; default is a unique temporary directory")
    command.add_argument("--version", default="26.3")
    command.add_argument("--cold-samples", type=positive, default=2)
    command.add_argument("--warm-samples", type=positive, default=5)
    command.add_argument("--cargo", default="cargo", help="Cargo executable (recorded in results)")
    command.add_argument("--edit-scope", choices=("facade", "core", "contracts", "codegen"), default="facade", help="snapshot source invalidation workload")
    command.add_argument("--test-compile", action="store_true", help="measure isolated all-feature test compilation/linking instead of Cargo check; tests are not executed")
    command.set_defaults(handler=suite)
    command = commands.add_parser("single", help="time an arbitrary command; compatible with bench_build.sh")
    command.add_argument("--output")
    command.add_argument("name")
    command.add_argument("samples", type=positive)
    command.add_argument("command", nargs=argparse.REMAINDER)
    command.set_defaults(handler=single)
    args = parser.parse_args(argv)
    try:
        return args.handler(args)
    except (OSError, ValueError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    raise SystemExit(main())
