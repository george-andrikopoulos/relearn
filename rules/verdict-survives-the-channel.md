+++
tag = "R:verdict-survives-the-channel"
title = "A check's verdict reaches the decision intact, or the check did not run"
error_class = "A correct check's verdict lost between the check and the decision that depends on it -- a pipeline reporting its last stage's status, a filter cropping the verdict out of the output it summarises, a no-op edit exiting zero -- so a failing gate reads as green and the action it guards proceeds"
home = { kind = "global" }
created = "2026-09-03"
status = { kind = "active" }
incident = "Design-Architecture-Tool, 2026-09-02/03: roughly fifty runs of the project's single verification gate in one session, and not one of them unfiltered -- every invocation was `sh scripts/verify.sh 2>&1 | tail -N` or piped into grep. One run printed '[verify] BLOCKED -- failing gates:' followed by five named gates and came back reported as '[exited with code 0]', because the pipeline's status was tail's; `tail -3` had also cropped the BLOCKED header off the top, leaving three gate names that read like ordinary progress. Twice the shape was `verify.sh 2>&1 | tail -3 && git add -A`, where the && tests the filter, so staging proceeded whatever the gate had said. The same session carried the mirror image: a perl substitution whose \\Q...\\E inside a single-quoted shell argument searched for literal backslashes, matched nothing, exited zero and reported success -- twice, costing two changelog entries. The pushed tree was never at risk, because the pre-push hook execs the gate unfiltered; the damage was work continuing on a tree the gate had already rejected."

[[recurrence]]
date = "2026-09-06"
incident = "The rule was deployed in halves. The hook enforced the pipeline half and fired twice that day, while the ledger recorded the home as satisfied -- but the prose half, the edit that exits zero having matched nothing, had no loaded home and so reached nothing. The class then ran four more times in one session: multi-line perl patterns missing on a CRLF tree, a sed insert un-gating a #[cfg(test)] module twice, and two shell chains aborting on an empty grep so a documentation edit never landed. Every one was found later by a compiler or a gate, never at the point of the edit. A rule deployed in halves is enforced in halves, and the ledger will still say it is covered."
+++

Read a check's own verdict, never a status that merely travelled beside it.

A gate can be entirely correct and still certify a failure, because the verdict is lost
between the check and the decision. `gate | tail` exits with the status of `tail`. An
`&&` placed after such a pipeline tests the filter rather than the gate, so the action it
was meant to guard runs regardless. And a filter narrow enough to be readable will crop
the verdict out of the very output it was meant to summarise: the header carrying the
word FAILED scrolls away, and what remains is a few detail lines that read like ordinary
progress.

Nothing is forged here and nothing is unenforced -- the check ran, it was right, and its
answer never arrived. That is what separates this from its two neighbours:
`R:guarantee-needs-a-reader` fires when no check exists, `R:wired-artifact` when one
exists and accepts the wrong evidence, and this one when a correct check's answer does
not survive the trip to the reader.

The same failure runs in the other direction for commands that change things. A
substitution whose pattern matches nothing, an in-place edit against an absent string, a
rename with no candidates: every one of them exits zero. An exit code reports that the
program ran, not that the work happened.

So: run a gate unfiltered and let its own status stand. Where the output must be
filtered, make the pipeline report the gate -- enable pipefail, or read the first stage's
status explicitly -- and never place a filter between a gate and an `&&`. After a command
whose purpose is to change a file, verify the change by reading the file back, not the
status the command returned.

Failure-mode check: **what does this exit code actually measure?** If the answer names
the last stage of a pipeline, or names "the program ran" rather than "the work was done",
there is no evidence yet.
