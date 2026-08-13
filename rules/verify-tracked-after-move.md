+++
tag = "R:verify-tracked-after-move"
title = "Verify a moved file is still tracked in a whitelist-gitignore repo"
error_class = "Relocating a file in a repository whose .gitignore is a whitelist (ignore-all, then re-include named paths), so the moved file silently falls outside the re-included set and leaves version control unnoticed"
home = { kind = "project", path = "stochos-lab" }
created = "2026-08-13"
status = { kind = "active" }
incident = "stochos-lab operating model, tagged [R:verify-tracked-after-move] there. The repo IS the live ~/.claude tree, guarded by a whitelist .gitignore, so a file moved out of a re-included path silently leaves version control -- the loss is invisible until something that depended on it is missing. The standing mitigation is to run the deploy/track verification (`verify-deploy`) after any relocation, which catches an artifact that dropped out of tracking."
+++

After moving or renaming a tracked file in a repository whose .gitignore is a whitelist, verify the file is still tracked before considering the change done. A whitelist ignore silently drops anything outside its re-included paths, so a relocation can remove a file from version control with no error and no diff line to notice. Run the repo's tracking/deploy verification as the gate: the failure mode is invisible precisely when you most assume the move was safe.
