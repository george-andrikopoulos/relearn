+++
tag = "R:no-secrets-in-config-repo"
title = "Keep secret-bearing, machine-local config out of a shared config repo"
error_class = "Committing machine-local, secret-bearing files (credentials, a permission allowlist, local settings) into a repository that also versions shareable configuration, leaking secrets and pinning machine-specific state into shared history"
home = { kind = "project", path = "stochos-lab" }
created = "2026-08-13"
origin = "mined"
status = { kind = "active" }
incident = "stochos-lab operating model, tagged [R:no-secrets-in-config-repo] there. The config repo versions shareable wiring (e.g. hook registration) but must never commit settings.json, settings.local.json, or .credentials.json: the permission allowlist is machine-local and secret-bearing. The wiring is versioned separately from the secrets so the shareable half can be tracked without dragging the secret half into history."

[[detector]]
name = "listed-file-staged"
engine = "listed-path"
files = ["**"]
events = ["git-pre-commit"]
action = "block"
message = "This path is on this clone's machine-local list of files that must never be committed: secret-bearing or machine-specific configuration. Unstage it with git restore --staged, and keep it out of the index. The list itself is never committed; it is named per clone with git config --local relearn.listed-file-staged.list."

[[detector.fixture]]
path = "config/local-secrets.toml"
list = ["config/local-secrets.toml", "machine.env"]
expect = "hit"

[[detector.fixture]]
path = "machine.env"
list = ["config/local-secrets.toml", "machine.env"]
expect = "hit"

[[detector.fixture]]
path = "config/shared.toml"
list = ["config/local-secrets.toml", "machine.env"]
expect = "clean"

[[detector.fixture]]
path = "nested/machine.env"
list = ["config/local-secrets.toml", "machine.env"]
expect = "clean"
+++

When a repository versions configuration that is meant to be shared, keep the secret-bearing and machine-local parts out of it: credentials, permission allowlists, and per-machine settings are not shareable configuration and do not belong in shared history, even (especially) when the repo is otherwise a config repo. Version the shareable wiring separately from the secrets, so tracking the former never commits the latter. A secret in history is a secret to rotate, not a secret to delete.
