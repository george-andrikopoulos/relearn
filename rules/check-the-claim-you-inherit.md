+++
tag = "R:check-the-claim-you-inherit"
title = "Check an inherited claim about the world before repeating it"
error_class = "A document's factual assertion about observable live state -- an absence, a status, a count, a capability -- repeated as a fresh finding without querying the state, so a stale or never-true claim is laundered into a current report and acted on, with the repetition adding false corroboration"
home = { kind = "global" }
created = "2026-09-20"
origin = "mined"
status = { kind = "active" }
incident = "Home Assistant heating config, 2026-09-20. Asked what remained on a project, Claude read /config/TODO.md and reported to its owner that the master bedroom thermometer was \"still absent\", so that room's valve ran uncalibrated. The owner replied: \"master bedroom thermometer is not absent. check again.\" One query settled it -- the sensor had reported continuously across the whole 168-hour recorder window on a 100% battery, with no unavailable gap. The claim was not merely stale; nothing in the live system had ever agreed with it. It had been copied into four documents -- the automation's own description, CLAUDE.md's hardware table, FEATURES.md's R6 row and TODO.md -- and on the strength of it that room had been deliberately excluded from the calibration sync for months. The document was the only evidence anyone had ever had, and every reader inherited it rather than checking. Cost: the wrong remaining-work item was reported, and the real defect underneath it (every calibration write silently failing) was found only because the owner pushed back on the one claim he knew to be false."
+++

Before repeating a document's factual claim about live state, query the state.

A document can tell you two different kinds of thing, and they do not deserve the same
trust. A claim about *intent* -- why a threshold is 18, what was rejected and why, which
invariant a file exists to hold -- is only recorded in prose, and the document is the
authoritative source. A claim about *the world* -- a sensor is absent, a service is
disabled, a count is 23, a capability is unenforced -- describes something that can be
observed directly, and the document is a cached reading with no expiry date.

Repeating the second kind without observing is how a stale or never-true claim gets
laundered into a fresh report. It is worse than the original error, because the claim
now arrives with today's date and the authority of having just been "checked", and
because the report is the thing the reader acts on. Whoever wrote it may have been right
at the time, may have been guessing, or may have copied it from somewhere else; none of
that is visible by the time you are reading it, and a claim copied into four documents
looks four times corroborated while resting on one unverified assertion.

This is the reader's half of `[R:doc-currency]`. That rule tells the writer to update
the document in the same change as the thing it describes, and it will sometimes fail --
so the reader is the last position where the error can still be caught. It is also the
mirror of `[R:measure-the-claim-not-a-subset]`: that one guards the moment you are about
to *overwrite* a recorded figure, this one the moment you are about to *repeat* it.
Between them, a recorded number is never simply passed along unexamined in either
direction.

The check is cheap and it is bounded. You are not re-deriving the document; you are
spot-checking the specific assertions you are about to put your own name to, and only
those that name observable state.

Failure-mode check: **is this a claim about the world, and can I observe the world?**
If both, the document is a hypothesis and your report needs the observation, not the
quotation. If you cannot observe it, repeat it with its provenance attached -- "TODO.md
records X, unverified" -- rather than asserting it flat.
