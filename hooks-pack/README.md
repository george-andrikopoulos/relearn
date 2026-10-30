# hooks-pack — the corpus's detectors as Claude Code hooks

Some rules in this repository carry a **detector**: a mechanical check precise enough to
run on every write, with fixtures that prove it fires where it should and stays quiet where
it should not. `relearn` compiles each one into a bash script that Claude Code runs before
a `Write`, `Edit` or `MultiEdit`. This folder is that emission, committed, so it can be
installed on a machine that has no Rust toolchain.

```
hooks-pack/
└── hooks/
    ├── <name>.sh             one per detector
    ├── test-detectors.sh     every fixture, piped through the scripts above
    └── settings-snippet.md   the registration to merge into settings.json
```

| Script | Rule | Action | Runs on | Install |
|---|---|---|---|---|
| `no-anyhow-in-lib.sh` | `[R:no-anyhow-in-libraries]` | block | Write, Edit, MultiEdit | yes |
| `no-case-collision.sh` | `[R:case-collision]` | block | Write | yes |
| `no-count-in-heading.sh` | `[R:revision-integrity]` | warn | Write, Edit, MultiEdit | draft |
| `no-embedded-local-llm.sh` | `[R:no-weak-model-for-judgment]` | warn | Write, Edit, MultiEdit | draft |
| `no-expect-empty-msg.sh` | `[R:no-unwrap-in-production]` | block | Write, Edit, MultiEdit | yes |
| `no-public-repo-create.sh` | `[R:repository-private-by-default]` | block | Write, Edit, MultiEdit | draft |
| `no-string-identity-compare.sh` | `[R:identity-is-not-equality-for-boxes]` | block | Write, Edit, MultiEdit | draft |
| `no-tool-attribution.sh` | `[R:no-tool-attribution-in-public-artefacts]` | block | Write, Edit, MultiEdit | draft |
| `no-unwrap-in-src.sh` | `[R:no-unwrap-in-production]` | block | Write, Edit, MultiEdit | yes |

That table is hand-written. `tests/pack_counts.rs` reads it back against the scripts and
fails the build when a script is missing from it, a row names a script that does not
exist, or a row's rule is not the one the script was generated for.

**A `draft` row is not for installing yet.** Its rule is still `active`: the detector was
drafted by the control triage (`docs/control-triage.md`) and its fixtures pass, but whether
it covers the rule, and whether its false positives are acceptable on real work, is a
graduation review that has not happened. The script is in the pack because the `hooks`
target emits every detector; the row says so because the emission cannot. A row moves to
`yes` in the same change that graduates or partly graduates its rule.

## Never edit a script

Every file under `hooks/` is generated and ends in a `# <!-- relearn:generated … -->`
line. To change a detector, change its rule's `[[detector]]` table and rebuild:

```bash
relearn build --targets hooks --out hooks-pack
relearn verify --targets hooks --out hooks-pack   # 0 = the pack matches the rules
```

A rule whose fixtures disagree with its pattern does not parse, so a broken detector stops
the build rather than the first tool call it misfires on.

## Requirements

`bash` and `jq` on `PATH`. On Windows that means Git Bash, which is what Claude Code runs
hook commands in. A script that cannot run, because `jq` is missing or the payload is not
JSON, prints one line naming itself and **exits 1**. Claude Code reports that as a
non-blocking error, so a broken hook is loud and never silently refuses a write.

Check the pack on the machine it will run on before installing it:

```bash
bash hooks-pack/hooks/test-detectors.sh   # 0 all agree / 1 a fixture disagrees / 2 no jq
```

## Install

relearn installs nothing. These scripts change what every session on the machine is
allowed to write, and no gate in this repository can see that layer, so a person copies
them.

1. Copy each `hooks/*.sh` whose row above reads `yes` into `~/.claude/hooks/relearn/`.
   That directory is the generated scripts' own. Nothing generated is ever written over a
   hand-written hook. `test-detectors.sh` is not a hook and is never copied.
2. Merge the JSON in `hooks/settings-snippet.md` into the `hooks` table of
   `~/.claude/settings.json`, keeping only the entries for the scripts you copied. The
   snippet registers every script in the pack, drafts included.

## Cutting over from a hand-written hook of the same name

A machine that already runs hand-written hooks with these names should move one hook at a
time, and only after the hand-written hook's own tests pass against the generated script:

1. Run the existing tests for that hook with the generated script substituted for it.
2. Copy the generated script into `~/.claude/hooks/relearn/`.
3. In `settings.json`, **replace** the old registration with the generated one rather than
   adding it beside it. Exactly one of the two is registered at any time.
4. Run `relearn controls --root <name>=<path> … --settings ~/.claude/settings.json`. The
   hook should now read `RESOLVED_AND_WIRED`, because a generated hook counts as wired only
   when it is registered at `hooks/relearn/<name>.sh`.
5. Delete the hand-written original in its own commit, in the repository that owns it.

## What the scripts never read

Every content detector ignores `**/rules/*.md`, `**/hooks-pack/**` and
`**/hooks/relearn/**`. Those files hold the patterns and the inputs that must hit them, so
a detector that read them would block the edit that maintains it. `relearn check` runs
every detector over every rule file and every emitted file, and fails if any detector fires
on one.
