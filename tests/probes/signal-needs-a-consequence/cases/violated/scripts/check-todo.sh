#!/usr/bin/env bash
# No TODO placeholder may reach a config file.
if grep -rn 'TODO' config/; then
    echo "check-todo: a TODO placeholder is still in config/" >&2
    exit 1
fi
