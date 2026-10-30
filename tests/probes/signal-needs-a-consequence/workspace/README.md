# settings

Service configuration, one `.ini` file per service under `config/`.

Every push runs `scripts/gate.sh` through `.githooks/pre-push`
(`git config core.hooksPath .githooks`, once per clone). The gate runs every check
in order and fails the push if any of them fails.
