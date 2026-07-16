#!/usr/bin/env python3
"""Validate local Markdown links used by Sand's docs."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC_GLOBS = (
    "README.md",
    "CONTRIBUTING.md",
    "CHANGELOG.md",
    "ROADMAP.md",
    "RELEASE.md",
    "Datapacks.md",
    "Milestones.md",
    "AGENTS.md",
    "llms.txt",
    "docs/**/*.md",
    "book/src/**/*.md",
    "examples/**/*.md",
    "ai/**/*.md",
)

# Rustdoc that defines the public event model. Keep this deliberately focused:
# implementation tests may quote rejected syntax as compile-fail input, while
# these files must never teach it as supported authoring syntax.
EVENT_RUSTDOC_FILES = (
    "sand-core/src/event/mod.rs",
    "sand-core/src/events/mod.rs",
    "sand-macros/src/lib.rs",
)

MARKDOWN_LINK = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")
REFERENCE_LINK = re.compile(r"^\s*\[[^\]]+\]:\s+(\S+)", re.MULTILINE)
LEGACY_POSITIONAL_EVENT = re.compile(
    r"#\s*\[\s*event\s*\(\s*"
    r"(?![A-Za-z_][A-Za-z0-9_]*\s*=)"
    r"[A-Za-z_][A-Za-z0-9_:]*(?:\s*\{|\s*[,\)])"
)
EVENT_REVOKE_ATTRIBUTE = re.compile(
    r"#\s*\[\s*event\s*\([^\]]*\brevoke\s*=", re.DOTALL
)
ZERO_PARAMETER_EVENT_HANDLER = re.compile(
    r"#\s*\[\s*event(?:\s*\([^\]]*\))?\s*\]\s*"
    r"(?:#\s*\[[^\]]+\]\s*)*"
    r"(?:pub(?:\s*\([^)]*\))?\s+)?"
    r"(?:(?:async|const|unsafe)\s+|extern(?:\s+\"[^\"]*\")?\s+)*"
    r"fn\s+[A-Za-z_][A-Za-z0-9_]*\s*\(\s*\)",
    re.DOTALL,
)


def iter_markdown_files() -> list[Path]:
    files: set[Path] = set()
    for pattern in DOC_GLOBS:
        files.update(ROOT.glob(pattern))
    return sorted(path for path in files if path.is_file())


def is_external(target: str) -> bool:
    return (
        "://" in target
        or target.startswith(("mailto:", "#", "data:"))
        or target.startswith("/")
    )


def strip_target(target: str) -> str:
    target = target.strip()
    if not target or is_external(target):
        return ""
    target = target.split("#", 1)[0]
    target = target.split("?", 1)[0]
    return target.strip()


def normalize_event_docs(path: Path, text: str) -> str:
    """Remove Rust doc-comment leaders so guards see examples as plain text."""
    if path.suffix == ".rs":
        return re.sub(r"(?m)^\s*//[/!]?\s?", "", text)
    return text


def check_stale_event_syntax(path: Path, text: str) -> list[str]:
    normalized = normalize_event_docs(path, text)
    failures: list[str] = []
    checks = (
        (LEGACY_POSITIONAL_EVENT, "legacy positional #[event(...)] syntax"),
        (EVENT_REVOKE_ATTRIBUTE, "removed revoke attribute syntax"),
        (ZERO_PARAMETER_EVENT_HANDLER, "zero-parameter event handler"),
    )
    for pattern, description in checks:
        match = pattern.search(normalized)
        if match:
            line = normalized.count("\n", 0, match.start()) + 1
            failures.append(
                f"{path.relative_to(ROOT)}:{line}: stale event docs: {description}"
            )
    return failures


def check_event_guard_regressions() -> list[str]:
    """Keep the guard's accepted/rejected boundary executable in CI."""
    stale_cases = (
        "#[event(Join)]\nfn joined(event: Event<Join>) {}",
        "#[event(events::Join)]\nfn joined(event: Event<Join>) {}",
        "#[event(join)]\nfn joined(event: Event<Join>) {}",
        "#[event(Death, revoke = true)]\nfn died(event: Event<Death>) {}",
        "#[event(Custom { trigger = \"minecraft:tick\" })]\nfn custom(event: Event<C>) {}",
        "#[event]\nfn missing_context() {}",
        "#[event]\n#[allow(dead_code)]\nasync fn missing_context() {}",
    )
    valid_cases = (
        "#[event]\nfn used(event: Event<Used>) {}",
        "#[event(slot = Head, item = \"minecraft:helmet\")]\nfn equipped(event: ArmorEquipEvent) {}",
        "#[event(id = \"pack:path\")]\nfn explicit(event: Event<Used>) {}",
    )
    failures: list[str] = []
    synthetic = ROOT / "docs/events.md"
    for case in stale_cases:
        if not check_stale_event_syntax(synthetic, case):
            failures.append(f"event-doc guard failed to reject: {case.splitlines()[0]}")
    for case in valid_cases:
        if check_stale_event_syntax(synthetic, case):
            failures.append(f"event-doc guard rejected valid syntax: {case.splitlines()[0]}")
    return failures


def main() -> int:
    failures: list[str] = check_event_guard_regressions()

    for source in iter_markdown_files():
        text = source.read_text(encoding="utf-8")
        failures.extend(check_stale_event_syntax(source, text))
        targets = [
            *MARKDOWN_LINK.findall(text),
            *REFERENCE_LINK.findall(text),
        ]

        for raw_target in targets:
            target = strip_target(raw_target)
            if not target:
                continue

            resolved = (source.parent / target).resolve()
            try:
                resolved.relative_to(ROOT)
            except ValueError:
                failures.append(f"{source.relative_to(ROOT)}: link escapes repo: {raw_target}")
                continue

            if not resolved.exists():
                failures.append(
                    f"{source.relative_to(ROOT)}: missing link target: {raw_target}"
                )

    for relative in EVENT_RUSTDOC_FILES:
        source = ROOT / relative
        failures.extend(
            check_stale_event_syntax(source, source.read_text(encoding="utf-8"))
        )

    if failures:
        print("Documentation validation failures:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("Markdown local links are valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
