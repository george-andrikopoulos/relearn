+++
tag = "R:measure-the-claim-not-a-subset"
title = "Match the scope before contradicting a recorded figure"
error_class = "Measuring a subset, comparing it to a claim about the superset, and overwriting a true recorded figure with a false one that now carries the authority of having been measured"
home = { kind = "global" }
created = "2026-08-30"
origin = "mined"
status = { kind = "active" }
incident = "Incident 2026-08-30 (Design-Architecture-Tool v0.5.5): FEATURES.md, the declared single source of truth for the test count, read '30 PDF tests'. Measured with `cargo test -p dat-shared --features pdf pdf::pdf_tests::` -> 23, and the row was 'corrected' to 23 in the release commit, the release notes and the pushed tree. The crate holds 7 more in `pdf::paint::paint_tests::`; 23 + 7 = 30, exactly as written. `-- --list` was one flag away. Ten green gates, none of which read the row -- now closed for that instance by `scripts/verify.sh` `test_suite_and_count()`, which defines both denominators itself. Ported into this library 2026-09-16 from ~/.claude/CLAUDE.md, where it reached no instruction layer but the one hand-maintained file and no rule could cite it."
published_incident = "A feature ledger, declared to be the single source of truth for a test count, recorded thirty tests in a subsystem. A measurement taken with a module-path filter returned twenty-three, and the row was 'corrected' to twenty-three — in the commit, in the release notes, and in the published tree. The filter had excluded a second module holding the remaining seven; the original figure had been right all along, and the correction replaced it with a false one carrying the authority of a measurement. Listing the tests rather than counting them was one flag away, and would have shown the excluded set at a glance."
+++

A figure is already written down, your measurement disagrees, and you are about to declare
the document wrong. Usually it is. When it is not, you replace a true figure with a false
one -- and the false one now carries the authority of having been measured, pushed there
by the very rules that tell you to keep documents current (`[R:doc-currency]`,
`[R:repair-the-lying-artefact]`). This is their failure mode.

State the scope the recorded figure claims, then show that your measurement covers exactly
that scope -- no wider, no narrower. A filter, an include pattern, a package flag, a module
path, a date range: each one silently defines a subset, and comparing a subset's count to a
superset's claim manufactures a discrepancy out of nothing.

**Prefer an enumeration you can read to a count you can only trust.** A list flag, printing
the matches, thirty lines you can eyeball -- all beat one integer you believe, because the
excluded set is usually visible at a glance in the enumeration and invisible in the total.

Failure-mode check: **what would my measurement miss that the claim includes?** If you
cannot name the excluded set, you have not established the scope, and you are not entitled
to overwrite the number.

Where a figure is load-bearing enough to be worth arguing about, it is worth a check that
owns its own denominator. The durable fix moves the scope definition out of the reader's
hands and into a gate, so the claim and the measurement can no longer drift apart --
`[R:prefer-by-construction]` applied to a number.
