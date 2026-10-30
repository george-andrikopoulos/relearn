#!/usr/bin/env bash
# Unit tests. About 20 seconds.
cd "$(dirname "$0")/.." || exit 2
. src/greet.sh
out=$(greet Ada)
case "$out" in
    *Ada*) echo "quick: 1 passed" ;;
    *) echo "quick: FAILED: $out"; exit 1 ;;
esac
