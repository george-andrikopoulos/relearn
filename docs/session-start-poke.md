# The poke at session start — the snippet, deliberately not installed

`relearn lint --upstream <clone>` prints the federation's pokes. This file holds the
hook snippet that would run it when an assistant session starts, and **this repository
does not install it.**

## Why it is written down rather than wired up

The programme names installing it as one of the things that goes wrong in this phase:

> **A `SessionStart` hook is installed.** The recurrence design already ruled this out of
> scope: anything under `~/.claude` edits the layer loaded into every session on the
> machine. Write the snippet into `docs/`; do not install it.

A hook under `~/.claude` is not configuration of this repository. It is an edit to the
instruction layer of every session on the machine, for every project, and installing one
from inside a phase that nobody asked to change their machine is exactly the
invented-requirement class this tool exists to retire. It is also the one change here
that no gate in this repository could see: `relearn verify` reads this tree, and a hook
lives somewhere else entirely.

So the snippet is a thing a person copies, having decided to.

## The snippet

`settings.json`, under `hooks.SessionStart`:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup",
        "hooks": [
          {
            "type": "command",
            "command": "relearn lint --rules rules --upstream ~/src/relearn-corpus || true"
          }
        ]
      }
    ]
  }
}
```

Three things about it are deliberate.

**`|| true`.** A poke never changes `lint`'s exit code, but a *finding* does, and a hook
that fails a session start because the corpus has an unheld recurrence has turned a
report into a gate. The `|| true` says so out loud rather than relying on the reader
knowing which of the two printed lines was fatal.

**`--upstream` names a path the person chose.** There is no default location, no home
directory lookup and no environment variable: the clone is a path given on the command
line, exactly as every other input to this tool is (`tests/solo_mode.rs` fails the build
on the alternative). Someone who has not cloned a corpus has nothing to put here, which
is the correct outcome — not an error, not a prompt.

**No `--poke` flags.** The default set is the reactive trigger and the two about a copy you
already hold — a stale cache, and one retired upstream.
Adding `--poke contributed` or `--poke high-recurrence` to a hook that fires on every
session is how the poke becomes a notification people dismiss, including the one that
mattered. If you want them, run `lint` by hand on the day you want them.

## What to run in CI instead

`lint` already runs on every change in CI, which is why §6 put the poke there rather than
in a `relearn news` command nobody would run. CI is the better home for the broadcast
triggers, precisely because nobody is watching a CI log for news:

```bash
relearn lint --rules rules --upstream "$CORPUS_CLONE" --poke class-covered --poke cache-behind
```

A stale cache is worth failing a *review* over, and it is worth noticing there rather
than at the moment somebody is trying to start work.
