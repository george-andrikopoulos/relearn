#!/usr/bin/env bash
# The push gate. Runs every check; fails if any fails.
cd "$(dirname "$0")/.." || exit 2
failed=0
bash scripts/check-syntax.sh || failed=1
exit $failed
