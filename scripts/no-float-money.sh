#!/usr/bin/env bash
# no-float-money.sh — refuse a monetary quantity declared as binary floating point.
#
# The interim control for `[R:money-is-not-a-float]`. It reads the CONCEPT, not
# a surface string: an identifier in a monetary role — `price`, `amount`,
# `notional`, `pnl` — declared as `f32`/`f64` in Rust source. Three declaration
# shapes, because the defect wears all three:
#
#     amount: f64                struct field, function parameter, let binding
#     struct Price(f64);         the tuple newtype the rule body names by name
#     type Notional = f64;       the alias that hides the representation
#
# The newtype shape is here deliberately. `Price(f64)` satisfies
# `[R:newtype-liberally]` completely — that rule's own worked example is
# `Miles(f64)` — and is still the exact defect, because the rules compose rather
# than overlap: the newtype stops two concepts being swapped, and says nothing
# about the representation inside one of them. A scan that only read struct
# fields would pass the example the rule was written to forbid.
#
# WHY A SCRIPT AND NOT A `cargo test`. `scripts/verify-dependencies.sh` argues
# the opposite way and is right to: its subject is this repository's manifests,
# so a test is wired by construction. This check's subject is a FINANCIAL
# application's source, which is not this tree and never will be. A test inside
# `relearn` could only ever scan `relearn`, where a monetary field will never
# appear, and would report green by measuring nothing — a control placed where
# the thing it controls is absent (`[R:wired-artifact]`). So the roots are
# arguments, and the script is meant to be COPIED INTO the tree it is about,
# alongside a `.githooks/pre-push` that calls it. See FEATURES.md for the
# install, which is two commands.
#
# WHY THE PUSH AND NOT A COMMAND. A gate someone has to remember to invoke is
# the defect one level up — `[R:signal-needs-a-consequence]`, whose own incident
# is a gate that existed and was not run. The consequence attaches to an event
# that must happen anyway.
#
# THE DETECTOR HAS ITS OWN READER. `--self-test` runs the matcher over
# `scripts/fixtures/money-scan/`, whose two files are a probe in both
# directions: `caught.rs` must produce exactly the declarations listed in
# `caught.expected`, and `clean.rs` must produce none. A detector that has
# stopped detecting cannot then report a clean tree
# (`[R:guarantee-needs-a-reader]`). The hook and CI both run it before the scan.
#
# `[R:detector-excludes-own-definitions]`. The vocabulary is in this file, which
# is `.sh`, and in the fixtures, which are excluded by path; only `.rs` files
# are read, so neither this header nor `rules/money-is-not-a-float.md` can match
# the check they describe.
#
# THE HIT IS PRINTED IN FULL — path, line, identifier, type. That is not
# `[R:report-the-hit-not-the-match]` being ignored; that rule governs a search
# whose subject is a thing whose whole problem is that it exists. A field name
# is not a private identifier, and naming it is the entire point: the reader has
# to know WHICH declaration to change.
#
# AN ESCAPE EXISTS, ON PURPOSE. A measured quantity may legitimately be a float
# and may legitimately be called `spread` or `total`. Marking the line
#
#     spread_ratio: f64,  // money-scan: measured
#
# excludes it. Without an escape the check goes always-red on the first honest
# false positive and is then muted, which is worse than the gap. The marker is
# read BEFORE comments are stripped, which is why it can live in a comment.
#
# THIS IS THE INTERIM, NOT THE FIX. It reads Rust only; it cannot see a
# greenfield decision that has not been written yet; it reads a name, and a
# money field called `x` is invisible to it. The fix is a scaffolding crate that
# ships `Money<C>` with an exact representation and no `Div`, so the types are
# present before the first line is written and are SELECTED rather than
# invented. TODO.md carries it, and the rule's `uncovered` says the same.
#
# EXIT CODES, and the middle two are the point:
#   0   the scan ran and found nothing
#   1   a monetary declaration is in binary floating point — named below
#   2   DISARMED — a missing tool, or a root that does not exist; nothing ran
#   3   the detector itself is broken — `--self-test` did not behave
#
# 2 and 3 are failures, never passes: a check that reports the same green
# whether it ran or could not run is worse than none.
#
# The output is NOT filtered and this script pipes nothing into anything. A
# gate's verdict must reach its reader intact. `[R:verdict-survives-the-channel]`
#
# Usage:
#   scripts/no-float-money.sh [ROOT ...]     # default: the repository root
#   scripts/no-float-money.sh --self-test
#   scripts/no-float-money.sh --help

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURES="$ROOT/scripts/fixtures/money-scan"

# The vocabulary from the rule. A component of the identifier must equal one of
# these, with a trailing plural `s` removed first — so `unit_price`,
# `total_value`, `fees` and `NotionalAmount` all match, and `costume` does not.
# Substring matching was rejected: it reads the spelling, not the concept.
VOCAB="price amount qty quantity notional value balance fee commission pnl spread cost total"

usage() {
    sed -n '2,86p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

# The matcher, over ONE file. `--self-test` and the tree scan both come through
# here, so the self-test exercises the same code the hook runs rather than a
# second copy of the rule. Prints `path:line: identifier: why` per hit, and
# nothing at all when the file is clean.
scan_file() {
    local file="$1"
    local shown="$2"
    awk -v vocab="$VOCAB" -v shown="$shown" '
                BEGIN {
                    n = split(vocab, words, / +/)
                    for (i = 1; i <= n; i++) VOCAB[words[i]] = 1
                }

                # Lowercase, and break camel case into underscore components, so
                # `NotionalAmount` and `notional_amount` read the same.
                function norm(name,   i, c, out) {
                    out = ""
                    for (i = 1; i <= length(name); i++) {
                        c = substr(name, i, 1)
                        if (c ~ /[A-Z]/ && i > 1 && substr(name, i - 1, 1) ~ /[a-z0-9]/)
                            out = out "_"
                        out = out tolower(c)
                    }
                    return out
                }

                function is_money(name,   k, parts, i, w) {
                    k = split(norm(name), parts, /_+/)
                    for (i = 1; i <= k; i++) {
                        w = parts[i]
                        sub(/s$/, "", w)
                        if (w != "" && (w in VOCAB)) return 1
                    }
                    return 0
                }

                function report(name, kind) {
                    printf "%s:%d: %s: %s\n", shown, FNR, name, kind
                    found = 1
                }

                {
                    # Read the escape before stripping comments — it lives in one.
                    if ($0 ~ /money-scan:[ \t]*(measured|allow)/) next

                    line = $0
                    gsub(/\/\*[^*]*\*\//, " ", line)
                    i = index(line, "//")
                    if (i > 0) line = substr(line, 1, i - 1)
                    if (line !~ /f32|f64/) next

                    # `type Notional = f64;`
                    if (match(line, /(^|[^A-Za-z0-9_])type[ \t]+[A-Za-z_][A-Za-z0-9_]*[ \t]*=/)) {
                        seg = substr(line, RSTART, RLENGTH)
                        sub(/^[^A-Za-z0-9_]*type[ \t]+/, "", seg)
                        sub(/[ \t]*=$/, "", seg)
                        if (is_money(seg)) report(seg, "type alias over a binary float")
                    }

                    # `struct Price(f64);` — the newtype the rule names.
                    if (match(line, /(^|[^A-Za-z0-9_])struct[ \t]+[A-Za-z_][A-Za-z0-9_]*[ \t]*\(/)) {
                        seg = substr(line, RSTART, RLENGTH)
                        sub(/^[^A-Za-z0-9_]*struct[ \t]+/, "", seg)
                        sub(/[ \t]*\($/, "", seg)
                        if (is_money(seg)) report(seg, "tuple newtype over a binary float")
                    }

                    # `amount: f64` — field, parameter or binding, and through
                    # the wrappers a money field is routinely hidden behind.
                    rest = line
                    while (match(rest, /[A-Za-z_][A-Za-z0-9_]*[ \t]*:[ \t]*(&[ \t]*)?((Option|Vec|Box|Cell|Arc|Rc|VecDeque)[ \t]*<[ \t]*)*f(32|64)([^A-Za-z0-9_]|$)/)) {
                        seg = substr(rest, RSTART, RLENGTH)
                        rest = substr(rest, RSTART + RLENGTH)
                        name = seg
                        sub(/[ \t]*:.*$/, "", name)
                        if (is_money(name)) report(name, "declared f32/f64")
                    }
                }

                END { exit(found ? 1 : 0) }
    ' "$file"
}

# Every `.rs` file under a root, in sorted order, excluding the build tree, the
# git directory, and this repository's own fixtures — which contain deliberate
# violations and would otherwise make the gate permanently red against itself
# (`[R:detector-excludes-own-definitions]`).
#
# `scan_file`'s status is READ, not inferred from its output: 0 clean, 1 hits,
# anything else means awk itself failed. An awk that dies prints nothing, and a
# caller reading emptiness as cleanliness is a gate reporting green because it
# could not run. `[R:verdict-survives-the-channel]`
scan_tree() {
    local root="$1"
    local files file shown status=0 out

    files="$(
        find "$root" \
            -type d \( -name target -o -name .git -o -name node_modules \) -prune -o \
            -type f -name '*.rs' -print 2>/dev/null | sort
    )"

    [ -z "$files" ] && return 0

    while IFS= read -r file; do
        [ -z "$file" ] && continue
        case "$file" in "$FIXTURES"/*) continue ;; esac
        shown="${file#"$root"/}"
        out="$(scan_file "$file" "$shown")"
        case $? in
            0) ;;
            1)
                printf '%s\n' "$out"
                [ "$status" -eq 2 ] || status=1
                ;;
            *)
                echo "[money-scan] awk failed on $shown" >&2
                status=2
                ;;
        esac
    done <<EOF
$files
EOF

    return "$status"
}

self_test() {
    local expected="$FIXTURES/caught.expected"
    local status=0

    if [ ! -d "$FIXTURES" ] || [ ! -f "$expected" ]; then
        echo "[money-scan] BROKEN: fixtures missing at $FIXTURES" >&2
        return 3
    fi

    local caught clean caught_status clean_status
    caught="$(scan_file "$FIXTURES/caught.rs" "caught.rs")"
    caught_status=$?
    clean="$(scan_file "$FIXTURES/clean.rs" "clean.rs")"
    clean_status=$?

    # The statuses are read as well as the text. `caught.rs` must exit 1 and
    # `clean.rs` must exit 0; an awk that failed exits neither, and a self-test
    # that only compared output would call that a pass.
    if [ "$caught_status" -ne 1 ] || [ "$clean_status" -ne 0 ]; then
        echo "[money-scan] BROKEN: fixture exit codes were $caught_status (want 1)" >&2
        echo "[money-scan] and $clean_status (want 0) — the matcher did not run." >&2
        return 3
    fi

    if ! printf '%s\n' "$caught" | diff -u "$expected" - >&2; then
        echo "[money-scan] BROKEN: caught.rs did not produce the expected hits." >&2
        status=3
    fi

    if [ -n "$clean" ]; then
        echo "[money-scan] BROKEN: clean.rs produced hits, so the check is over-eager:" >&2
        printf '%s\n' "$clean" >&2
        status=3
    fi

    if [ "$status" -eq 0 ]; then
        echo "[money-scan] self-test ok — the detector catches what it must and nothing else."
    fi
    return "$status"
}

main() {
    case "${1-}" in
        --help | -h)
            usage
            return 0
            ;;
    esac

    # Every external tool the check depends on, named before anything runs.
    # `awk` is the matcher; `find` and `sort` build the file list, and a missing
    # `find` yields an EMPTY list, which the scan would otherwise report as a
    # clean tree — the exact false green this family of rules is about
    # (`[R:verdict-survives-the-channel]`). `diff` is what the self-test
    # compares with. Absence is exit 2, never 0.
    local tool
    for tool in awk find sort diff; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            echo "[money-scan] DISARMED: $tool not found; the gate did not run." >&2
            return 2
        fi
    done

    if [ "${1-}" = "--self-test" ]; then
        self_test
        return $?
    fi

    local roots=("$@")
    if [ "${#roots[@]}" -eq 0 ]; then
        roots=("$ROOT")
    fi

    local hits="" root found found_status blocked=0
    for root in "${roots[@]}"; do
        if [ ! -d "$root" ]; then
            echo "[money-scan] DISARMED: '$root' is not a directory; nothing was checked." >&2
            return 2
        fi
        found="$(scan_tree "$root")"
        found_status=$?
        case "$found_status" in
            0) ;;
            1)
                hits="${hits}${found}"$'\n'
                blocked=1
                ;;
            *)
                echo "[money-scan] DISARMED: the matcher failed under '$root'." >&2
                return 2
                ;;
        esac
    done

    if [ "$blocked" -eq 1 ]; then
        echo "[money-scan] BLOCKED: a monetary quantity is in binary floating point." >&2
        printf '%s' "$hits" >&2
        echo "[money-scan] Money is an exact decimal quantity: use integer minor units" >&2
        echo "[money-scan] or a fixed-scale decimal, and no Div. [R:money-is-not-a-float]" >&2
        echo "[money-scan] A genuinely MEASURED quantity is excused on its own line with" >&2
        echo "[money-scan]   // money-scan: measured" >&2
        return 1
    fi

    echo "[money-scan] clean — no monetary declaration in binary floating point."
    return 0
}

main "$@"
