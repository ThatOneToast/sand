# Release Process

Before release:

1. Run formatting, clippy, workspace tests, macro trybuild tests, and rustdoc.
2. Run `mdbook build` if mdBook is installed.
3. Build representative example datapacks.
4. Review `ROADMAP.md` and version-support docs for accuracy.
5. Confirm escape hatches are documented and beginner docs remain
   attribute-first.

Stable surfaces should include typed command/state/component APIs and macro
diagnostics covered by tests. Experimental surfaces should be called out in the
book or reference docs.
