# load-log-pack — which relearn layers your sessions actually load

A Claude Code hook that appends one JSON line to a local file every time a session loads
an instruction layer, and the fixture runner that proves it on your shell. Read the log
with `relearn loads`.

Nothing here is installed by relearn. Once registered, the hook writes a line on every
skill and instruction-file load on the machine, so switching it on is your decision.

## What it records

Two Claude Code events, both from the hooks reference:

- **`PostToolUse` on `Skill`**: a skill the model opened. The payload names the skill as
  invoked (`global`, or `<plugin>:global` for the copy synced from your claude.ai
  account) but carries no path. The hook therefore finds the file itself, in the order the
  skills reference gives: your personal `~/.claude/skills/` first, then the project's
  `.claude/skills/` in the session's directory and each directory above it. A
  `<plugin>:<skill>` name is looked up under `~/.claude/skills/synced/`.
- **`InstructionsLoaded`**: a `CLAUDE.md` or `.claude/rules/` file the loader read, at
  session start or later when a path-scoped rule attaches. This payload carries its own
  path and the loader's `memory_type`.

Each line holds the time, the session id, the skill name as invoked (for a skill), the
resolved path, its location class (`user`, `project`, `account-synced`, `local`,
`managed`, or `null` when nothing was found), and the sha256 from the file's
`relearn:generated` marker. The hash is read from disk, because no payload carries it.

## What it cannot see

- **Whether a rule inside a layer was read.** A load is a whole home arriving in a
  session. Rule-level evidence needs the probes.
- **An enterprise (managed) skill.** These outrank personal ones, but the skills reference
  gives no Windows path for them, so the hook does not look. If one ever shadows a
  relearn skill, the log records the personal copy instead.
- **A plugin skill outside `skills/synced/`.** Its install path is not documented.
- **Possibly a skill typed as `/name`.** The hooks reference says a typed skill reaches
  `UserPromptSubmit` as typed. It does not say whether a `Skill` tool call follows, so
  until a typed load is seen in the log, assume it may be missing.

## Install

1. Run the fixtures on the shell your hooks run under. This needs `bash` and `jq`:

   ```bash
   bash load-log-pack/hooks/test-load-log.sh
   ```

   It ends with one `PASS` or `FAIL` line, and exits 2 (`DISARMED`, not a pass) when
   `jq` is missing.

2. Copy the hook beside hooks-pack's scripts:

   ```bash
   mkdir -p ~/.claude/hooks/relearn
   cp load-log-pack/hooks/relearn-load-log.sh ~/.claude/hooks/relearn/
   ```

3. Merge `hooks/load-log-settings-snippet.md`'s JSON into the `hooks` table of
   `~/.claude/settings.json`.

The log goes to `$RELEARN_LOAD_LOG`, or to `~/.local/state/relearn/load-log.jsonl`. That
is outside `~/.claude`, so nothing loads it back into a session. It holds session ids and
paths, so keep it out of any repository.

## Read it

```bash
relearn loads --log ~/.local/state/relearn/load-log.jsonl --since 2026-11-01 --until 2026-11-30
```

The report covers:

- **COLD**: a home that did not load at all in the window.
- **STALE**: a home that loaded with a hash `relearn build` would not write now. The
  report names the location class of the stale copy, which is how an account-synced copy
  that was never refreshed shows up.
- **SHADOWED**: one home loaded from two location classes in the window.
- **LOADED**: counts per home and location class.

The hook stamps each line in UTC, so `--since` and `--until` are UTC days. No relearn
load at all in the window exits 2. That usually means the hook is not
registered, and the report cannot tell that from silence.

## Rebuild

Generated, never transcribed:

```bash
relearn build --targets load-log --out load-log-pack
```

`scripts/check.sh` verifies the pack and runs its fixtures.
