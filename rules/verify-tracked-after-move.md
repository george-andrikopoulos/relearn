+++
tag = "R:verify-tracked-after-move"
title = "Verify a moved file is still tracked in a whitelist-gitignore repo"
error_class = "Relocating a file in a repository whose .gitignore is a whitelist (ignore-all, then re-include named paths), so the moved file silently falls outside the re-included set and leaves version control unnoticed"
home = { kind = "project", path = "stochos-lab" }
created = "2026-08-13"
origin = "mined"
status = { kind = "active" }
incident = "stochos-lab operating model, tagged [R:verify-tracked-after-move] there. The repo IS the live ~/.claude tree, guarded by a whitelist .gitignore, so a file moved out of a re-included path silently leaves version control -- the loss is invisible until something that depended on it is missing. The standing mitigation is to run the deploy/track verification (`verify-deploy`) after any relocation, which catches an artifact that dropped out of tracking."
+++

After moving or renaming a tracked file in a repository whose .gitignore is a whitelist, verify the file is still tracked before considering the change done. A whitelist ignore silently drops anything outside its re-included paths, so a relocation can remove a file from version control with no error and no diff line to notice. Run the repo's tracking/deploy verification as the gate: the failure mode is invisible precisely when you most assume the move was safe.

This is the class where a *correct* change has a silent side effect outside its own frame. On 2026-07-21 executables were moved out of an auto-writable directory for sound security reasons, the destination had never been re-included, and the two scripts that made the rules monitor run at all existed on one disk with no history for a day. `git status` showed two deletions, which read as intended.

Failure-mode check: *what did this correct change quietly stop tracking?*

(Clause recovered 2026-09-27. It survived only in the rules ledger's 2026-07-22 entry, in neither rule home, until the 2026-08-16 audit found it and the port to this corpus did not carry it.)
