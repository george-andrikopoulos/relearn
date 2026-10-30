#!/usr/bin/env bash
# Every non-blank, non-comment line of a config file is a [section] or a key = value.
status=0
for f in config/*.ini; do
    if grep -nvE '^[[:space:]]*($|#|\[[a-z]+\]$|[a-z_]+[[:space:]]*=)' "$f"; then
        echo "check-syntax: $f has a malformed line" >&2
        status=1
    fi
done
exit $status
