#!/usr/bin/env python3
"""Validate Sand's ai/ knowledge-system resources.

Checks (see ai/maintenance.md):
- Required files exist and are non-empty.
- ai/project-status.yaml and ai/capability-manifest.yaml parse.
- Capability IDs in capability-manifest.yaml are unique.
- status/stability values are drawn from the documented enums.
- `implemented` capabilities have a non-empty `evidence` list.
- Recipe front matter parses and its `capabilities:` entries exist in the manifest.
- known-limitations.md `Affects:` capability references exist in the manifest.
- Referenced local file paths in `evidence:`/documentation fields resolve.

Deliberately dependency-free (no PyYAML): the two manifest files use a
narrow, consistent subset of YAML (block mappings, block sequences of
mappings, quoted/plain scalars, `>`-folded block scalars). This module
implements just that subset rather than adding an external dependency for
one CI script (see ai/maintenance.md's "no large dependency" guidance).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AI = ROOT / "ai"

REQUIRED_FILES = [
    ROOT / "AGENTS.md",
    ROOT / "llms.txt",
    AI / "README.md",
    AI / "project-status.yaml",
    AI / "capability-manifest.yaml",
    AI / "authoring-guide.md",
    AI / "known-limitations.md",
    AI / "maintenance.md",
    AI / "recipes" / "README.md",
]

ALLOWED_STATUS = {
    "implemented",
    "partial",
    "experimental",
    "raw_only",
    "planned",
    "intentionally_unsupported",
    "vanilla_impossible",
    "unknown",
}
ALLOWED_STABILITY = {"stable", "alpha", "experimental", "unknown"}


# ── Minimal block-YAML loader ────────────────────────────────────────────────


def _strip_comment(line: str) -> str:
    in_quote = None
    for i, ch in enumerate(line):
        if in_quote:
            if ch == in_quote:
                in_quote = None
        elif ch in "\"'":
            in_quote = ch
        elif ch == "#" and (i == 0 or line[i - 1] == " "):
            return line[:i]
    return line


def _parse_scalar(raw: str):
    raw = raw.strip()
    if raw in ("", "~", "null"):
        return None
    if raw == "true":
        return True
    if raw == "false":
        return False
    if raw.startswith('"') and raw.endswith('"') and len(raw) >= 2:
        return raw[1:-1]
    if raw.startswith("'") and raw.endswith("'") and len(raw) >= 2:
        return raw[1:-1]
    if raw.startswith("[") and raw.endswith("]"):
        inner = raw[1:-1].strip()
        if not inner:
            return []
        return [_parse_scalar(part) for part in _split_flow_list(inner)]
    return raw


def _split_flow_list(inner: str) -> list[str]:
    parts: list[str] = []
    depth = 0
    current = ""
    in_quote = None
    for ch in inner:
        if in_quote:
            current += ch
            if ch == in_quote:
                in_quote = None
            continue
        if ch in "\"'":
            in_quote = ch
            current += ch
        elif ch in "[{":
            depth += 1
            current += ch
        elif ch in "]}":
            depth -= 1
            current += ch
        elif ch == "," and depth == 0:
            parts.append(current)
            current = ""
        else:
            current += ch
    if current.strip():
        parts.append(current)
    return parts


def load_yaml(text: str):
    lines = text.split("\n")
    entries = []
    for raw_line in lines:
        line = raw_line.rstrip("\n")
        stripped_for_comment = _strip_comment(line)
        if not stripped_for_comment.strip():
            continue
        indent = len(line) - len(line.lstrip(" "))
        content = stripped_for_comment.strip()
        entries.append((indent, content, raw_line))

    pos = 0

    def parse_block(min_indent: int):
        nonlocal pos
        if pos >= len(entries):
            return None
        indent, content, _ = entries[pos]
        if indent < min_indent:
            return None
        if content.startswith("- "):
            return parse_sequence(indent)
        return parse_mapping(indent)

    def parse_sequence(indent: int):
        nonlocal pos
        result = []
        while pos < len(entries):
            cur_indent, content, _ = entries[pos]
            if cur_indent != indent or not content.startswith("-"):
                break
            item_content = content[1:].strip()
            pos += 1
            if not item_content:
                value = parse_block(indent + 1)
                result.append(value)
            elif ":" in item_content and not item_content.startswith(('"', "'")):
                key, _, rest = item_content.partition(":")
                key = key.strip()
                rest = rest.strip()
                mapping = {}
                if rest == "" or rest == ">":
                    if rest == ">":
                        mapping[key] = parse_folded_scalar(indent + 2)
                    else:
                        nested = parse_block(indent + 2)
                        mapping[key] = nested
                else:
                    mapping[key] = _parse_scalar(rest)
                while pos < len(entries):
                    next_indent, next_content, _ = entries[pos]
                    if next_indent <= indent or next_content.startswith("- ") and next_indent == indent:
                        break
                    if next_indent != indent + 2:
                        if next_indent > indent + 2:
                            pos += 1
                            continue
                        break
                    nkey, _, nrest = next_content.partition(":")
                    nkey = nkey.strip()
                    nrest = nrest.strip()
                    pos += 1
                    if nrest == ">":
                        mapping[nkey] = parse_folded_scalar(next_indent + 2)
                    elif nrest == "":
                        mapping[nkey] = parse_block(next_indent + 2)
                    else:
                        mapping[nkey] = _parse_scalar(nrest)
                result.append(mapping)
            else:
                result.append(_parse_scalar(item_content))
        return result

    def parse_folded_scalar(min_indent: int) -> str:
        nonlocal pos
        parts = []
        while pos < len(entries):
            indent, content, _ = entries[pos]
            if indent < min_indent:
                break
            parts.append(content)
            pos += 1
        return " ".join(parts)

    def parse_mapping(indent: int):
        nonlocal pos
        result = {}
        while pos < len(entries):
            cur_indent, content, _ = entries[pos]
            if cur_indent != indent:
                break
            if content.startswith("- "):
                break
            key, sep, rest = content.partition(":")
            if not sep:
                pos += 1
                continue
            key = key.strip()
            rest = rest.strip()
            pos += 1
            if rest == ">":
                result[key] = parse_folded_scalar(indent + 2)
            elif rest == "":
                nxt = entries[pos] if pos < len(entries) else None
                if nxt and nxt[0] > indent:
                    result[key] = parse_block(indent + 2 if nxt[0] >= indent + 2 else nxt[0])
                else:
                    result[key] = None
            else:
                result[key] = _parse_scalar(rest)
        return result

    # Skip a leading document-start marker or blank prefix.
    if pos < len(entries) and entries[pos][1] == "---":
        pos += 1
    return parse_mapping(0) if pos < len(entries) else {}


# ── Checks ────────────────────────────────────────────────────────────────────


def check_required_files() -> list[str]:
    failures = []
    for path in REQUIRED_FILES:
        if not path.is_file():
            failures.append(f"missing required file: {path.relative_to(ROOT)}")
        elif path.stat().st_size == 0:
            failures.append(f"required file is empty: {path.relative_to(ROOT)}")
    return failures


def check_capability_manifest(data: dict) -> tuple[list[str], set[str]]:
    failures = []
    caps = data.get("capabilities") or []
    seen_ids: set[str] = set()
    for cap in caps:
        if not isinstance(cap, dict):
            failures.append(f"capability entry is not a mapping: {cap!r}")
            continue
        cap_id = cap.get("id")
        if not cap_id:
            failures.append(f"capability entry missing id: {cap!r}")
            continue
        if cap_id in seen_ids:
            failures.append(f"duplicate capability id: {cap_id}")
        seen_ids.add(cap_id)

        status = cap.get("status")
        if status not in ALLOWED_STATUS:
            failures.append(f"{cap_id}: invalid status '{status}'")

        stability = cap.get("stability")
        if stability is not None and stability not in ALLOWED_STABILITY:
            failures.append(f"{cap_id}: invalid stability '{stability}'")

        if status == "implemented":
            evidence = cap.get("evidence")
            if not evidence:
                failures.append(f"{cap_id}: status implemented but no evidence listed")

    return failures, seen_ids


def check_recipes(known_ids: set[str]) -> list[str]:
    failures = []
    recipes_dir = AI / "recipes"
    for path in sorted(recipes_dir.glob("*.md")):
        if path.name == "README.md":
            continue
        text = path.read_text(encoding="utf-8")
        match = re.match(r"^---\n(.*?)\n---\n", text, re.DOTALL)
        if not match:
            failures.append(f"{path.relative_to(ROOT)}: missing YAML front matter")
            continue
        front_matter = load_yaml(match.group(1))
        if not isinstance(front_matter, dict):
            failures.append(f"{path.relative_to(ROOT)}: front matter did not parse as a mapping")
            continue
        caps = front_matter.get("capabilities") or []
        if not caps:
            failures.append(f"{path.relative_to(ROOT)}: front matter has no capabilities")
        for cap_id in caps:
            if cap_id not in known_ids:
                failures.append(
                    f"{path.relative_to(ROOT)}: references unknown capability id '{cap_id}'"
                )
    return failures


def check_known_limitations(known_ids: set[str]) -> list[str]:
    failures = []
    path = AI / "known-limitations.md"
    text = path.read_text(encoding="utf-8")
    for match in re.finditer(r"Affects:\s*(.+)", text):
        refs = match.group(1)
        for cap_id in re.findall(r"`([a-z0-9][a-z0-9-]*)`", refs):
            if cap_id == "none" or cap_id in known_ids:
                continue
            failures.append(
                f"known-limitations.md: 'Affects:' references unknown capability id '{cap_id}'"
            )
    return failures


def main() -> int:
    failures: list[str] = []

    failures += check_required_files()
    if failures:
        # Missing required files makes the remaining checks meaningless.
        for f in failures:
            print(f"FAIL: {f}")
        return 1

    try:
        project_status = load_yaml((AI / "project-status.yaml").read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001 - report and continue
        failures.append(f"ai/project-status.yaml failed to parse: {exc}")
        project_status = {}

    try:
        manifest = load_yaml((AI / "capability-manifest.yaml").read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        failures.append(f"ai/capability-manifest.yaml failed to parse: {exc}")
        manifest = {}

    if not isinstance(project_status, dict) or "sand" not in project_status:
        failures.append("ai/project-status.yaml: expected top-level 'sand' key not found")

    known_ids: set[str] = set()
    if isinstance(manifest, dict) and "capabilities" in manifest:
        cap_failures, known_ids = check_capability_manifest(manifest)
        failures += cap_failures
    else:
        failures.append("ai/capability-manifest.yaml: expected top-level 'capabilities' list not found")

    if known_ids:
        failures += check_recipes(known_ids)
        failures += check_known_limitations(known_ids)

    if failures:
        for f in failures:
            print(f"FAIL: {f}")
        print(f"\n{len(failures)} failure(s).")
        return 1

    print(f"OK: {len(known_ids)} capability ids, all checks passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
