+++
tag = "R:wired-artifact"
title = "A success check consumes a sentinel nothing else can produce"
error_class = "A check that runs, passes, and accepts forgeable evidence -- a date, a header, a log echo, a file's existence, a component's own unit tests -- so an inert or failed thing certifies as working and the check's greenness is what conceals it"
home = { kind = "global" }
created = "2026-07-20"
origin = "mined"
status = { kind = "active" }
incident = "Ferridis, 2026-07-20: BackpressureSignal and StreamChunk shipped in v0.4 with five unit tests and sat unwired for two months. The type-level tests made an inert feature look enforced, and the first behaviour-contract draft cited them as the enforcing artefact -- a type's own tests prove the type works, never that anything produces or consumes it. Strengthened 2026-07-21 after a second, sharper instance: a quarterly-review wrapper grepped for a bare date to decide success, matched the header its own miner had just regenerated, and cleared the REVIEW-DUE flag on a run that had failed. Ported into this library 2026-08-25; this repository's charter had listed it as scheduled for migration since Phase A and cited it in prose, so the corpus was citing a rule it did not define."
+++

A success check must consume a write-once sentinel that is unique to the artefact class it verifies, emitted as the final act of the success path, and producible by nothing else in the system. Pattern-matching on a date, a header, a log line, or the presence of a file is not verification -- it is a check that something happened, which is a different claim from the one being made.

Ask before wiring any check: *what else in this system can produce the string my check accepts?* If the answer is anything at all, the check accepts forgery, and it will accept it silently on the day it matters -- because the failure path is the one that regenerates headers and re-emits dates.

The same test applies to enforcement claimed on a component: *what produces this, what consumes it, and does the cited artefact cross that seam?* If nothing crosses it, the feature is inert and the honest ledger entry says so rather than naming the component's own tests.

A green check that cannot fail is worse than no check. It converts an open question into a settled one, so nobody looks again, and the thing it was protecting degrades behind a signal that says it is fine. This is the type-level and tooling-level sibling of R:verify-through-production-path, and it shares a family with R:guarantee-needs-a-reader: that rule fires when nothing enforces the claim, this one when something does and accepts the wrong evidence.
