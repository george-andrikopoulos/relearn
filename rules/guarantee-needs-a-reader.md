+++
tag = "R:guarantee-needs-a-reader"
title = "A stated guarantee names what enforces it, or is deleted"
error_class = "A safety claim written in prose -- a header comment, a docstring, a promise in a readme, a ledger line -- with nothing in the system reading the state it asserts, so the sentence stops people looking at the very thing it fails to protect"
home = { kind = "global" }
created = "2026-08-16"
status = { kind = "active" }
incident = "Four instances in two days across three languages, one cause. A bash `destructive` parameter bound and never read, while the file header promised 'every deletion is confirmed even under --yes'. A db.rs comment justifying a nil-UUID sentinel with 'validation flags dangling references' when no such check existed. A ledger entry reading 'Inventory confirmed: no agent remains on a cost-chosen cheaper tier' when the sweep had consumed agent model: fields and never read rules prose -- leaving a price-tier recommendation live in every session for four weeks. And a restore chain that put its verifier inside the && , so a verifier failure fired the || branch and announced 'REFUSED -- settings.json untouched' *after* the file had already been replaced, one hour after this rule was installed. Ported into this library 2026-08-25."
+++

Every safety claim in prose names the line, test, or check that enforces it -- or the sentence is deleted. A guarantee with no reader is worse than no guarantee, because it is read as coverage and it ends the inquiry.

A claim of completeness must also state what the check actually consumed. Not "verified", but "verified by grepping `model:` across N agent files" -- so the gap between the scope of the check and the scope of the claim is visible on the face of the entry rather than reconstructable only by rerunning it. Most false completeness claims are not lies; they are a narrow check reported in wide language.

**Error paths are where these hide.** A message asserting a state must be produced by *checking that state*, not by which branch printed it. The failure path is the one nobody exercises, so a confident sentence with nothing behind it survives there longest, and it is read at exactly the moment the reader is least able to question it. Ask of every error message: *did anything read the world before this printed?*

Where a mechanical reader exists, use it and leave the prose to what it cannot see: an unread binding is already a hard error under a compiler run with warnings denied. What no linter can see is a true-looking sentence with nothing behind it, and that is what this rule is for.

The test: *what would have to be true for this sentence to be false, and what reads that?* R:wired-artifact is the neighbouring failure -- there a check exists and accepts forgeable evidence; here there is no check at all.
