+++
tag = "R:guarantee-needs-a-reader"
title = "A stated guarantee names what enforces it, or is deleted"
error_class = "A safety claim written in prose -- a header comment, a docstring, a promise in a readme, a ledger line -- with nothing in the system reading the state it asserts, so the sentence stops people looking at the very thing it fails to protect"
home = { kind = "global" }
created = "2026-08-16"
origin = "mined"
status = { kind = "partial", by = "test:tests/ledger.rs::every_enforced_by_row_names_an_artefact_or_declares_itself_exposed + test:tests/pack_counts.rs", uncovered = "any guarantee stated outside FEATURES.md and the pack READMEs", date = "2026-09-16" }
incident = "Four instances in two days across three languages, one cause. A bash `destructive` parameter bound and never read, while the file header promised 'every deletion is confirmed even under --yes'. A db.rs comment justifying a nil-UUID sentinel with 'validation flags dangling references' when no such check existed. A ledger entry reading 'Inventory confirmed: no agent remains on a cost-chosen cheaper tier' when the sweep had consumed agent model: fields and never read rules prose -- leaving a price-tier recommendation live in every session for four weeks. And a restore chain that put its verifier inside the && , so a verifier failure fired the || branch and announced 'REFUSED -- settings.json untouched' *after* the file had already been replaced, one hour after this rule was installed. Ported into this library 2026-08-25."
published_incident = "A one-time sweep answered a question about a repository, and a sentence recording the all-clear was written into a document. Six weeks later the thing the sweep had looked for was present both in the working tree and through most of the repository's history. Nothing in the system read the state that sentence asserted, and it was believed precisely because it was written down, which is what stopped anyone looking again. The sweep did not produce a control and the sentence was not one; the control that eventually held it reads the state on every push."

[[recurrence]]
date = "2026-09-05"
incident = "Design-Architecture-Tool M33. An employer-owned product name was found in the working tree and in 77 of the repository's 106 commit trees. M17-A6 had recorded an all-clear for exactly that question six weeks earlier -- a one-time sweep plus a sentence saying it was clean -- and the sentence was believed for six weeks precisely because it was written down, which is what stopped anyone looking again. Nothing in the system read the state the claim asserted. The sweep did not produce a control and the sentence was not one; the control is `the_repository_contains_no_banned_name`, which reads a committed digest list on every push and arrived only after the incident."

[[recurrence]]
date = "2026-09-19"
incident = "stochos-lab, scripts/verify-deploy.sh. Its Cowork section printed `ok  Cowork export matches repo sources` -- and the check behind that sentence hashed one file per skill, SKILL.md, while the bundle it certifies archives the whole skill directory: evals/, references/, assets/. An eval case was added to skills/relearn/evals/evals.json, the bundle was not rebuilt, and the gate reported the export fresh. This is the narrow-check-wide-language half of the rule rather than the no-check-at-all half: something did read the world, and the sentence describing what it read was wider than the read. The claim was also credited in FEATURES.md, so it was a guarantee with a named enforcing artefact that held part of what its own words claimed. Found by asking whether a change just made would actually reach the surface it was made for, which is the question the rule asks in the other direction. Fixed by scripts/skill-hash-lib.sh -- one definition of a skill's hash, shared by the exporter that writes the manifest and the gate that checks it -- and probed both ways: before, an evals-only change reported fresh; after, it reports exactly one stale skill and no other."

[[recurrence]]
date = "2026-09-20"
incident = "The always-loaded git-workflow layer carried the sentence `Attribution disabled globally via ~/.claude/settings.json`. The key it names -- `includeCoAuthoredBy` -- is not in that file; the default is on. Measured the same day: 72 commits across two repositories carry an assistant `Co-Authored-By` trailer and 67 are already pushed, against a standing house rule of never. This is the purest instance of the class so far and the most expensive, because the artefact is published rather than merely wrong: the sentence asserted a configuration state, nothing read that state, and the sentence is what stopped anyone looking -- for as long as it had been written. An audit on 2026-08-16 had found zero trailers, true of that day, and was then carried forward as if it described the setting. Found only because a rehoming task made someone open the file the sentence named, which is not a control."
+++

Every safety claim in prose names the line, test, or check that enforces it -- or the sentence is deleted. A guarantee with no reader is worse than no guarantee, because it is read as coverage and it ends the inquiry.

A claim of completeness must also state what the check actually consumed. Not "verified", but "verified by grepping `model:` across N agent files" -- so the gap between the scope of the check and the scope of the claim is visible on the face of the entry rather than reconstructable only by rerunning it. Most false completeness claims are not lies; they are a narrow check reported in wide language.

**Error paths are where these hide.** A message asserting a state must be produced by *checking that state*, not by which branch printed it. The failure path is the one nobody exercises, so a confident sentence with nothing behind it survives there longest, and it is read at exactly the moment the reader is least able to question it. Ask of every error message: *did anything read the world before this printed?*

Where a mechanical reader exists, use it and leave the prose to what it cannot see: an unread binding is already a hard error under a compiler run with warnings denied. What no linter can see is a true-looking sentence with nothing behind it, and that is what this rule is for.

The test: *what would have to be true for this sentence to be false, and what reads that?* R:wired-artifact is the neighbouring failure -- there a check exists and accepts forgeable evidence; here there is no check at all.
