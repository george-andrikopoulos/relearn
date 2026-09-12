#!/usr/bin/env bash
# verify-dependencies.sh — a hand-invocable front door to the dependency gate.
#
# THIS SCRIPT CONTAINS NO CHECK. The gate is `tests/dependencies.rs`, and it is
# deliberately a test rather than a script: `cargo test` already runs in CI on
# Linux and Windows, so the check is wired by construction and cannot arrive
# disarmed the way a script needing an installed hook or a remembered CI step
# can. Everything here is a convenience for a person who wants to run that one
# check without running the whole suite, and a place for `--help` to say what
# the check is. Put no logic in it: a second implementation of the rule is a
# second home for it, which is the failure this repository exists to prevent.
#
# WHAT THE GATE CHECKS. Every dependency in every Cargo.toml — runtime, dev,
# build and per-target — must be named by a row in ARCHITECTURE.md's decisions
# log whose Decision cell OPENS with `Dependency:` or `Dependencies:`, naming
# the crate in backticks:
#
#   | 2026-09-12 | **Dependency: `crate`** | what for; why this one | refused |
#
# Nine crates predating the rule are exempted by name in the test's
# GRANDFATHERED list, which may only shrink. `[R:price-every-dependency]`
#
# WHAT NO GATE CAN CHECK. A dependency *refused* leaves no manifest line and no
# artifact of any kind, so there is nothing to compare the log against. That
# half of the rule is on the person who refused it. Do not read a green run
# here as covering it.
#
# EXIT CODES, and the middle one is the point:
#   0   the gate ran and passed
#   2   DISARMED — no cargo, so nothing was checked
#   101 the gate ran and did not pass — cargo's output names the crates
#
# 2 is a failure, never a pass: a check that reports the same green whether it
# ran or could not run is worse than none. `[R:guarantee-needs-a-reader]`
#
# The codes above are what the script was OBSERVED to return on 2026-09-12, not
# what it was assumed to return: 101 is cargo's own failing-test status, passed
# through rather than normalised to 1, because a wrapper that rewrites a
# verdict is a wrapper that can get it wrong. A compile error exits non-zero
# too, with cargo's status for that.
#
# The output is NOT filtered. A gate's verdict must reach the reader intact,
# and a pipeline reports its last stage rather than the check.
# `[R:verdict-survives-the-channel]`

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v cargo >/dev/null 2>&1; then
    echo "[dependencies] DISARMED: cargo not found; the gate did not run." >&2
    exit 2
fi

cd "$ROOT" || exit 2
cargo test --test dependencies "$@"
