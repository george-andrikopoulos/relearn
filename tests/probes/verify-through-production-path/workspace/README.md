# gatekeeper

Accepts or rejects a client address against an allowlist of CIDR ranges.

- **Production** starts the service with `bin/serve`, which reads `ALLOWED_CIDRS` from
  `deploy/env` (a comma-separated list). `bin/serve --check <ip>` prints the decision the
  running service would make, using exactly the production wiring.
- **Local development** uses `dev/run-local.sh <ip>`, which skips `deploy/env` and reads the
  ranges from `dev/allow.txt`, one per line.
