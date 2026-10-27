#!/usr/bin/env bash
# check.sh — the local gate, replacing hosted CI.
#
#   bash scripts/check.sh
#
# Hosted CI was switched off on 2026-10-26: GitHub Actions is disabled for the
# repository and .github/workflows/ci.yml no longer runs. This script runs the
# stages that workflow ran, with the same flags, and .githooks/pre-push runs it
# on every push (Gate 4), so the push is still the event that grades a change
# `[R:signal-needs-a-consequence]`.
#
# Unlike CI, every stage runs even when an earlier one fails, so one failure
# never hides another. Each stage's own exit status is printed, nothing is
# piped, and the script exits 1 if any stage failed `[R:verdict-survives-the-channel]`.
#
# CI ran on Ubuntu and on Windows because a break that shows on one OS only is
# real here (`[R:xplat-fixtures]`). This script runs wherever it is invoked, so
# a push from Windows grades Windows only; run it from a Linux shell (WSL counts)
# before a change that touches paths, line endings or process spawning.
#
# The banned-name gate is not a stage: it needs a machine-local term list and
# stays a separate pre-push gate (Gate 1), exactly as it was never a CI step.

set -u

cd "$(dirname "$0")/.." || exit 2

failed=()

stage() {
    local name=$1
    shift
    echo "== $name: $*"
    "$@"
    local rc=$?
    echo "== $name exit $rc"
    if [ "$rc" -ne 0 ]; then
        failed+=("$name")
    fi
}

stage rustfmt cargo fmt --check
stage clippy cargo clippy --all-targets -- -D warnings
stage test cargo test
stage verify cargo run --quiet -- verify
stage verify-copilot-pack cargo run --quiet -- verify --targets copilot --out copilot-pack
stage verify-claude-pack cargo run --quiet -- verify --targets claude --out claude-pack
stage lint cargo run --quiet -- lint --deny error
stage money-self-test bash scripts/no-float-money.sh --self-test
stage money-scan bash scripts/no-float-money.sh
stage pack-sync-self-test bash scripts/pack-sync-reminder.sh --self-test

if [ "${#failed[@]}" -eq 0 ]; then
    echo "PASS: rustfmt, clippy, test, verify x3, lint, money-scan, pack-sync"
    exit 0
fi
echo "FAIL: ${failed[*]}"
exit 1
