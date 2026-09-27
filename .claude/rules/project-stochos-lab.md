# Project rules

## Keep secret-bearing, machine-local config out of a shared config repo [R:no-secrets-in-config-repo]

When a repository versions configuration that is meant to be shared, keep the secret-bearing and machine-local parts out of it: credentials, permission allowlists, and per-machine settings are not shareable configuration and do not belong in shared history, even (especially) when the repo is otherwise a config repo. Version the shareable wiring separately from the secrets, so tracking the former never commits the latter. A secret in history is a secret to rotate, not a secret to delete.

## Verify a moved file is still tracked in a whitelist-gitignore repo [R:verify-tracked-after-move]

After moving or renaming a tracked file in a repository whose .gitignore is a whitelist, verify the file is still tracked before considering the change done. A whitelist ignore silently drops anything outside its re-included paths, so a relocation can remove a file from version control with no error and no diff line to notice. Run the repo's tracking/deploy verification as the gate: the failure mode is invisible precisely when you most assume the move was safe.

This is the class where a *correct* change has a silent side effect outside its own frame. On 2026-07-21 executables were moved out of an auto-writable directory for sound security reasons, the destination had never been re-included, and the two scripts that made the rules monitor run at all existed on one disk with no history for a day. `git status` showed two deletions, which read as intended.

Failure-mode check: *what did this correct change quietly stop tracking?*

(Clause recovered 2026-09-27. It survived only in the rules ledger's 2026-07-22 entry, in neither rule home, until the 2026-08-16 audit found it and the port to this corpus did not carry it.)

<!-- relearn:generated v0.1.0 sha256=e2ab02f8287f2f4a0b6dacf3f3da47e9f15f28c92b804f22de801667ee01c6da rules=R:no-secrets-in-config-repo,R:verify-tracked-after-move -- DO NOT EDIT; regenerate with `relearn build` -->
