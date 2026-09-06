# A `SessionStart` hook that surfaces unheld recurrences

**Status: design note only. Deliberately NOT installed.** Installing it edits
`~/.claude`, the layer loaded into every session on the machine — a different repository,
a different blast radius, and a change that wants its own commit and its own review. This
file exists so the idea is written down rather than re-invented.

## Why it is the right companion to `relearn lint`

`UnheldRecurrence` is a `Warning`, so `relearn lint` exits non-zero on it and CI notices.
But CI notices at the end, and the fact it is reporting — *this rule is held by prose
alone and prose has already been shown to fail* — is most useful at the moment work
starts, when the choice of which control to reach for is still open. That is a
`SessionStart` concern, not a CI one.

## The snippet

```bash
#!/usr/bin/env bash
# unheld-recurrences — SessionStart hook.
# Reports rules that have fired again since they were written, so the session
# starts knowing which parts of the instruction layer are not holding.
set -uo pipefail

repo="${CLAUDE_PROJECT_DIR:-$PWD}"
[ -d "$repo/rules" ] || exit 0
command -v relearn >/dev/null 2>&1 || exit 0

# `lint` exits 1 on any actionable finding; that is expected here and is not an
# error for the hook, so its status is read rather than propagated.
findings="$(relearn lint --rules "$repo/rules" 2>/dev/null | grep '^warning: unheld recurrence' || true)"
[ -n "$findings" ] || exit 0

echo "[recurrence] rules that have fired again and are still held by prose alone:"
printf '%s\n' "$findings"
```

## Two things it must not become

**It must not infer a recurrence from session text.** The relearn skill already fires when
a human corrects something, and already decides whether the answer is a new rule or a
repair to a control that exists. The human is the detector — P4. A hook that guessed
"this looks like the same error class" from keywords would be exactly the heuristic
dressed up as certainty that this repository refuses elsewhere, and the same 728 lines of
it were deleted from DAT at M20-C.

**It must not become a gate.** `relearn lint` is the gate, and it already fails CI. A
second thing that can block on the same finding, in a different repository, with a
different exit code, is one more place for the verdict to be lost in transit.
