+++
tag = "R:case-collision"
title = "Check for a case-differing sibling before creating a file"
error_class = "Creating a file whose name differs from an existing one only by case or Unicode normalisation, which are two files on a case-sensitive filesystem and one file on a case-insensitive one -- so a repository synced across both silently merges or shadows content with no error"
home = { kind = "global" }
created = "2026-07-20"
origin = "mined"
status = { kind = "graduated", to = "hook:no-case-collision", date = "2026-07-21" }
incident = "Ferridis, 2026-07-20: the code architecture and its decisions log were written to ARCHITECTURE.md while a public design document architecture.md already existed. On ext4 these are two files; on NTFS they are one, and the repository syncs between both, so case-insensitive semantics are the binding constraint. Resolved by merging the code architecture into the existing file as a section rather than keeping a colliding name. Graduated the same day to hooks/no-case-collision.sh, which blocks the collision deterministically at Write time. Ported into this library 2026-08-25 with its graduated status intact, so the annotation names the control that now holds it."
published_incident = "Two files were created whose names differed only in case: a design document written in lower case, and a code-architecture document written in upper. On a case-sensitive filesystem these are two files; on a case-insensitive one they are a single file that silently takes whichever content was written last. The repository synchronised between both kinds, so the case-insensitive semantics were the binding constraint and one document was quietly overwriting the other. Resolved by merging the two rather than keeping a colliding name, and the class was later closed by a write-time check that refuses a name differing only in case from an existing one."
+++

Before creating any file, check for an existing sibling whose name differs only by case or by Unicode normalisation. Where a repository is synced between a case-sensitive filesystem and a case-insensitive one, the case-insensitive semantics are the binding constraint: two such names are one file, and the loser is whichever was written second.

Merge into the existing file, with a section heading, rather than creating the colliding name. The instinct to create a properly-capitalised new file is the failure -- the convention is not worth a silent overwrite on the other machine.

The damage is invisible from where the work was done. On the authoring filesystem both files exist and everything looks correct; the collision appears only after a sync, as content that vanished with no diff and no error to attribute it to.

This rule has graduated: a write-time hook now blocks the collision deterministically, so the instruction layer no longer has to carry it for that path. It still applies to every creation path the hook does not see -- a shell redirect, a git operation, another tool -- which is why the guidance is kept rather than retired.
