#!/usr/bin/env bash
# Local development: reads dev/allow.txt directly, skipping deploy/env.
cd "$(dirname "$0")/.." || exit 2
. lib/allowlist.sh
allowlist_check "$1" dev/allow.txt
