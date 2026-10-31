# rules-pack — the domain layers for `~/.claude/rules/`

The **user-level rules layer**: one file per language-scoped domain home, each carrying a
`paths:` list, so Claude Code attaches it when a matching file is read and costs nothing in
a session that never reads one.

```
rules-pack/
└── .claude/rules/
    ├── domain-java.md    paths: **/*.java
    └── domain-rust.md    paths: **/*.rs
```

The layout mirrors the destination: `rules-pack/.claude/rules/` maps onto `~/.claude/rules/`.

## Why it is a pack and not a repository's own files

A domain belongs to the developer, not to a repository. It attaches by file type wherever
that developer works, so **one** user-level copy serves every repository. Until 2026-10-31
this repository also emitted the domain layers into its own `.claude/rules/`, and a session
here then loaded the rust layer twice — once from `~/.claude/rules/` and once from the
repository — every time it read a `.rs` file. `relearn budget` reports that as
`DUPLICATE_LAYER`. A repository's own `.claude/rules/` now carries its own project layer and
nothing else, and the domain layers live here.

Domain homes with no language glob (`low-latency`, `disclosure`, `project-files`,
`agent-tooling`) are not here: a rules file can only be always-resident or path-scoped,
and "load when the task is about this" is what a skill's description does. They ship in
`claude-pack/` as skills.

## Install

Copy the files you want into your user rules directory:

```bash
cp rules-pack/.claude/rules/domain-*.md ~/.claude/rules/
```

Or build them there directly from a clone, which writes the same bytes:

```bash
relearn build --targets claude-rules --rules-layer user --out ~
```

Either way, each file carries a `relearn:generated` marker; `relearn build` refuses to
overwrite a file without one, so a hand-written `~/.claude/rules/domain-rust.md` is never
clobbered.

**Then remove any repository copy.** If a repository you work in still carries
`.claude/rules/domain-*.md` from an older relearn build, a session there loads that layer
twice. Check with:

```bash
relearn budget --project <repo> --user ~/.claude
```

## Rebuilding

Generated, never transcribed: `relearn build --targets claude-rules --rules-layer user --out
rules-pack`. `scripts/check.sh` verifies it as a second output root, so a rule added,
changed or retired without rebuilding the pack fails the gate.
