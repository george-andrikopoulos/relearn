+++
tag = "R:definition-of-done-every-change"
title = "The definition of done runs on every change, and skips are declared"
error_class = "A change called complete when the code works, leaving the enforcing test, the regression pass, the contract, the charter and the open-work list to a later pass that never comes -- so the standing documents drift one defensible omission at a time"
home = { kind = "global" }
created = "2026-08-25"
origin = "codified"
status = { kind = "partial", by = "hook:tdd-gate", uncovered = "checks 2 to 5 entirely -- the regression pass, the contract, the charter and the open-work list -- plus check 1 in every language but Rust, and check 1 wherever the enforcing artefact is a TYPE rather than a test file, which under a types-first discipline is the common case", date = "2026-09-19" }
incident = "Codification-dated, not single-incident: ported 2026-08-25 from ~/.claude/skills/project-discipline/SKILL.md, the artefact that defines the practice. It recurs in this repository's own history rather than being hypothetical: on 2026-08-24 the change that landed R:repair-the-lying-artefact updated the open-work list and skipped the behaviour contract -- check 3 -- leaving two false counts in the ledger, which the next change had to find and repair. The discipline caught it one change late, which is the argument for the pass being mechanical rather than remembered."
published_incident = "Codified from a standing repository discipline rather than mined from a failure. It surfaced in an audit of an emitted instruction pack, which carried no rule about that discipline at all while the type-design half was fully present — so one of the two disciplines the author works by reached every assistant and the other reached none. The practice: a fixed sequence of checks runs on every change rather than at a release, and a skipped one is declared rather than silently dropped. It is not hypothetical even in the repository that codified it: a change landing a rule about lying artefacts updated some of its own documents and not others, on the day that rule shipped."
+++

A change is not done until five checks pass, in order:

1. **Code and its enforcing artefact ship together.** The type, property or test that locks the new behaviour is in the same change. Never "tests later" -- later is a different change, made by someone with less context, competing against new work.
2. **Regression pass.** Re-read the behaviour contract and *run* the enforcing artefacts of every feature this change could plausibly touch. A fix that breaks another documented feature is not a fix. This is the check that makes the contract protective rather than descriptive.
3. **The contract is updated** -- a new entry with its artefact, or an existing entry's artefact revised.
4. **Charter and architecture sync.** Ask explicitly whether the charter could still recreate the project and whether the architecture still describes it, and append any decision made to the log with its why.
5. **The open-work list is updated** -- done items cleared, discovered work added.

State the five results at the end of the task, briefly. The statement is not ceremony: it is what makes a skipped check visible to the person who can price the skip.

When a request conflicts with the discipline -- *just patch it quickly* -- do the patch, then say which checks were skipped and what exposure that creates. Never silently drop the discipline, and never block the user with process. The skip is the user's call to make; concealing that it happened is not.

Checks 1 and 2 do not apply to every change, and saying so is part of doing them. A documentation-only change has no enforcing artefact to ship. Declaring a check inapplicable is honest; quietly omitting it and reporting five greens is the failure this rule names.
