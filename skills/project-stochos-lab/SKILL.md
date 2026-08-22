---
name: project-stochos-lab
description: "Rules for project: stochos-lab. Covers: Keep secret-bearing, machine-local config out of a shared config repo (Committing machine-local, secret-bearing files (credentials, a permission allowlist, local settings) into a repository that also versions shareable configuration, leaking secrets and pinning machine-specific state into shared history); Verify a moved file is still tracked in a whitelist-gitignore repo (Relocating a file in a repository whose .gitignore is a whitelist (ignore-all, then re-include named paths), so the moved file silently falls outside the re-included set and leaves version control unnoticed)"
---

# project: stochos-lab rules

## Keep secret-bearing, machine-local config out of a shared config repo [R:no-secrets-in-config-repo]

When a repository versions configuration that is meant to be shared, keep the secret-bearing and machine-local parts out of it: credentials, permission allowlists, and per-machine settings are not shareable configuration and do not belong in shared history, even (especially) when the repo is otherwise a config repo. Version the shareable wiring separately from the secrets, so tracking the former never commits the latter. A secret in history is a secret to rotate, not a secret to delete.

## Verify a moved file is still tracked in a whitelist-gitignore repo [R:verify-tracked-after-move]

After moving or renaming a tracked file in a repository whose .gitignore is a whitelist, verify the file is still tracked before considering the change done. A whitelist ignore silently drops anything outside its re-included paths, so a relocation can remove a file from version control with no error and no diff line to notice. Run the repo's tracking/deploy verification as the gate: the failure mode is invisible precisely when you most assume the move was safe.

<!-- relearn:generated v0.1.0 sha256=04c481e45a3e06a5593d3e295fc20d67d157eada12e41e967eb729d44add1802 rules=R:no-secrets-in-config-repo,R:verify-tracked-after-move -- DO NOT EDIT; regenerate with `relearn build` -->
