# `ai/` — machine- and agent-facing Sand resources

This directory is Sand's AI knowledge system: a compact, evidence-backed set
of files that let a coding agent (or any LLM) determine what Sand supports,
pick the right typed API, and validate its work, without re-deriving
everything from the full source tree or book.

Start at repo-root `AGENTS.md` for operating rules; this directory holds the
detailed, retrievable references it points to.

## Retrieval guide

| Need | Read |
|---|---|
| Current project maturity, CLI/test/validation status | `project-status.yaml` |
| Is a feature supported, and how well? | `capability-manifest.yaml` |
| How should an agent implement a specific kind of pack? | `authoring-guide.md` |
| Known boundaries, workarounds, doc contradictions | `known-limitations.md` |
| Complete, compilable examples | `recipes/` |
| How to keep these resources current | `maintenance.md` |

## Load only what the task needs

These files are split by purpose so an agent can load one without pulling in
the rest of the context window:

- Answering "does Sand support X" → `capability-manifest.yaml` alone,
  filtered to the relevant capability ID(s). Don't load `authoring-guide.md`
  for a pure support-check question.
- Writing new pack code → `authoring-guide.md` for the decision tree, plus
  the one matching file in `recipes/`.
- Judging whether a claim is stale → `known-limitations.md` first; it
  records where `docs/`/`book/src/`/`ROADMAP.md` disagree with source.
- Updating these files after a code change → `maintenance.md`.

## What this directory is not

It does not replace `book/src/` (tutorials/concepts) or `docs/` (detailed
reference prose), and it does not duplicate `sand-components/src/
registry_coverage.rs` or `.../trigger_coverage.rs`, which remain the
ground-truth, compile-checked coverage tables. `capability-manifest.yaml`
points at those tables for exhaustive per-registry/per-trigger detail instead
of copying them.

## Currency

Every file here is handwritten and reviewed alongside source changes — none
are code-generated yet. `last_reviewed` dates appear in the YAML files.
Treat status claims older than a few weeks with more skepticism and prefer
re-checking source (`ai/maintenance.md` explains how, and lists what should
eventually be generated instead of handwritten).
