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

MARKDOWN_LINK = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")
REFERENCE_LINK = re.compile(r"^\s*\[[^\]]+\]:\s+(\S+)", re.MULTILINE)


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


def main() -> int:
    failures: list[str] = []

    for source in iter_markdown_files():
        text = source.read_text(encoding="utf-8")
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

    if failures:
        print("Broken local Markdown links found:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("Markdown local links are valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
