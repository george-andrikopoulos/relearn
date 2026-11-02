#!/usr/bin/env bash
# loader-constants.sh — does the installed Claude Code still size its
# instruction loader the way `relearn budget` assumes?
#
#   bash scripts/loader-constants.sh --binary <claude binary> [--source <loadset.rs>]
#       0 the loader's three constants match / 1 one moved / 2 could not check
#   bash scripts/loader-constants.sh --find        # print the installed binary's path; 2 if none
#   bash scripts/loader-constants.sh --self-test   # every verdict on a synthetic snippet
#
# `relearn budget` encodes the loader's limits: per file
# max(40_000, round(window * 0.05 * k)), combined max(120_000, per-file). They
# were read once, from Claude Code 2.1.316 on 2026-10-31, and nothing noticed
# if an upgrade changed them (TODO.md). This re-reads them.
#
# **One home for the numbers.** The expected values are parsed out of
# src/loadset.rs — its doc comment and DEFAULT_BUDGET — never restated here, so
# the script and the code cannot disagree about what was assumed.
#
# **Anchored on shape, not names.** The binary embeds its JavaScript minified,
# and the identifiers change every release. The anchor is the code's structure:
# a function returning Math.max(F, Math.round(w * P * k(...))) and, directly
# after it, `var C=N;` and a function returning Math.max(C, <that function>).
# F and P are then resolved by name inside the match. Exactly one match is
# required: none or several is "could not check", exit 2, which is never a
# pass `[R:verdict-survives-the-channel]`.

set -uo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
I='[A-Za-z0-9_$]+'
SHAPE="var $I=[0-9.e]+,($I=[0-9.e]+,)*$I=[0-9.e]+;function $I\([^{]*\)\{[^}]*return Math\.max\($I,Math\.round\($I\*$I\*$I\([^)]*\)\)\)\}var $I=[0-9.e]+;function $I\($I\)\{return Math\.max\($I,$I\($I\)\)\}"

# A number as awk reads it, so `4e4` and `40000` and `40_000` compare equal.
norm() { awk -v v="${1//_/}" 'BEGIN { printf "%.10g\n", v + 0 }'; }

# expected <loadset.rs>: prints "per_file_floor fraction combined_floor", or fails.
expected() {
    local src=$1 line floor frac combined budget
    line=$(grep -o -E 'max\([0-9_]+, round\(window \* [0-9.]+ \* k\)\)' "$src" | head -1)
    floor=$(sed -E 's/^max\(([0-9_]+),.*/\1/' <<<"$line")
    frac=$(sed -E 's/.*window \* ([0-9.]+) \* k.*/\1/' <<<"$line")
    combined=$(grep -o -E 'max\([0-9_]+, per-file\)' "$src" | head -1 | sed -E 's/^max\(([0-9_]+),.*/\1/')
    budget=$(grep -o -E 'pub const DEFAULT_BUDGET: usize = [0-9_]+;' "$src" | sed -E 's/.* = ([0-9_]+);/\1/')
    if [ -z "$line" ] || [ -z "$combined" ] || [ -z "$budget" ]; then
        echo "[loader-constants] cannot read the assumed constants from $src" >&2
        return 2
    fi
    if [ "$(norm "$combined")" != "$(norm "$budget")" ]; then
        echo "[loader-constants] $src disagrees with itself: the combined floor is $combined, DEFAULT_BUDGET is $budget" >&2
        return 1
    fi
    echo "$(norm "$floor") $(norm "$frac") $(norm "$combined")"
}

# found <file>: prints "per_file_floor fraction combined_floor" read from the
# one match of SHAPE, or fails 2.
found() {
    local file=$1 matches count snippet fname pname floor frac combined
    matches=$(LC_ALL=C grep -a -o -E "$SHAPE" "$file" 2>/dev/null | tr -c '[:print:]\n' '?')
    count=$(grep -c . <<<"$matches")
    if [ -z "$matches" ] || [ "$count" -ne 1 ]; then
        echo "[loader-constants] the loader's limit formula matched ${count:-0} time(s) in $file, not once — the code changed shape, and nothing was checked" >&2
        return 2
    fi
    snippet=$matches
    fname=$(sed -E "s/.*return Math\.max\(($I),Math\.round\($I\*$I\*.*/\1/" <<<"$snippet")
    pname=$(sed -E "s/.*return Math\.max\($I,Math\.round\($I\*($I)\*.*/\1/" <<<"$snippet")
    combined=$(sed -E "s/.*\)\)\)\}var $I=([0-9.e]+);function .*/\1/" <<<"$snippet")
    floor=$(grep -o -E "(^var |,)${fname//\$/\\\$}=[0-9.e]+" <<<"$snippet" | head -1 | sed -E 's/.*=//')
    frac=$(grep -o -E "(^var |,)${pname//\$/\\\$}=[0-9.e]+" <<<"$snippet" | head -1 | sed -E 's/.*=//')
    if [ -z "$floor" ] || [ -z "$frac" ] || [ -z "$combined" ]; then
        echo "[loader-constants] the formula was found but its constants were not assigned beside it — nothing was checked" >&2
        return 2
    fi
    echo "$(norm "$floor") $(norm "$frac") $(norm "$combined")"
}

# check <binary> <loadset.rs>: the verdict, printed and returned.
check() {
    local bin=$1 src=$2 want got rc
    [ -f "$bin" ] || { echo "[loader-constants] no file at $bin — nothing was checked" >&2; return 2; }
    want=$(expected "$src"); rc=$?; [ "$rc" -ne 0 ] && return "$rc"
    got=$(found "$bin"); rc=$?; [ "$rc" -ne 0 ] && return "$rc"
    read -r wf wp wc <<<"$want"
    read -r gf gp gc <<<"$got"
    if [ "$want" = "$got" ]; then
        echo "[loader-constants] ok — per-file max($gf, window * $gp * k), combined max($gc, per-file), as src/loadset.rs assumes."
        return 0
    fi
    echo "[loader-constants] MOVED — the installed loader sizes files differently from what relearn budget assumes:" >&2
    [ "$wf" = "$gf" ] || echo "  per-file floor: assumed $wf, installed $gf" >&2
    [ "$wp" = "$gp" ] || echo "  window fraction: assumed $wp, installed $gp" >&2
    [ "$wc" = "$gc" ] || echo "  combined floor: assumed $wc, installed $gc" >&2
    echo "  Update src/loadset.rs (its doc comment, DEFAULT_LIMIT and DEFAULT_BUDGET) and the decisions log." >&2
    return 1
}

# The installed Claude Code binary: the resolved `claude` when it is the binary
# itself, or the native binary an npm install's shim launches.
find_binary() {
    local c real dir candidate
    c=$(command -v claude 2>/dev/null) || return 2
    real=$(readlink -f "$c" 2>/dev/null || echo "$c")
    if [ -f "$real" ] && [ "$(wc -c < "$real")" -gt 10000000 ]; then
        echo "$real"; return 0
    fi
    dir=$(dirname "$c")
    for candidate in "$dir/node_modules/@anthropic-ai/claude-code/bin/claude.exe" \
                     "$dir/node_modules/@anthropic-ai/claude-code/bin/claude" \
                     "$dir/../lib/node_modules/@anthropic-ai/claude-code/bin/claude"; do
        [ -f "$candidate" ] && { echo "$candidate"; return 0; }
    done
    return 2
}

self_test() {
    local tmp failed=0
    tmp=$(mktemp -d) || return 2
    trap 'rm -rf "$tmp"' RETURN
    # A synthetic loadset.rs carrying the assumed values in the forms the real
    # one uses, and synthetic snippets of the loader's shape with invented
    # identifiers. Literal numbers, not values read from the real source: a
    # fixture built from the detector's own reading agrees with a wrong one
    # `[R:detector-excludes-own-definitions]`.
    cat > "$tmp/loadset.rs" <<'RS'
/// combined limit there: `max(40_000, round(window * 0.05 * k))` per file and
/// `max(120_000, per-file)` combined, read from a fixture.
pub const DEFAULT_BUDGET: usize = 120_000;
RS
    sed 's/DEFAULT_BUDGET: usize = 120_000/DEFAULT_BUDGET: usize = 100_000/' "$tmp/loadset.rs" > "$tmp/loadset-split.rs"
    shape() { printf 'junk\x00\x01var Aa=%s,bb=4194304,Cc=%s;function Fn(e=q()){let n=g(e),r=n>0?n:d;return Math.max(Cc,Math.round(r*Aa*Kk(e)))}var Dd=%s;function Gg(e){return Math.max(Dd,Fn(e))}\x02tail' "$1" "$2" "$3"; }
    shape 0.05 4e4 120000 > "$tmp/held"
    shape 0.04 4e4 120000 > "$tmp/fraction-moved"
    shape 0.05 40000 100000 > "$tmp/combined-moved"
    printf 'junk with no loader in it\n' > "$tmp/absent"
    { shape 0.05 4e4 120000; printf '\n'; shape 0.05 4e4 120000; } > "$tmp/twice"

    # expect <name> <want exit> <binary> <source>
    expect() {
        check "$3" "$4" >/dev/null 2>&1
        local got=$?
        if [ "$got" -ne "$2" ]; then
            echo "[loader-constants] self-test FAILED: $1 exited $got, wanted $2" >&2
            failed=1
        fi
    }
    expect "the assumed constants" 0 "$tmp/held" "$tmp/loadset.rs"
    expect "a moved window fraction" 1 "$tmp/fraction-moved" "$tmp/loadset.rs"
    expect "a moved combined floor" 1 "$tmp/combined-moved" "$tmp/loadset.rs"
    expect "no formula in the binary" 2 "$tmp/absent" "$tmp/loadset.rs"
    expect "the formula twice" 2 "$tmp/twice" "$tmp/loadset.rs"
    expect "a missing binary" 2 "$tmp/nothing-here" "$tmp/loadset.rs"
    expect "a source that disagrees with itself" 1 "$tmp/held" "$tmp/loadset-split.rs"
    # The real source must still be readable, or every real run is exit 2.
    if ! expected "$ROOT/src/loadset.rs" >/dev/null; then
        echo "[loader-constants] self-test FAILED: src/loadset.rs no longer states the assumed constants in a form this script reads" >&2
        failed=1
    fi
    [ "$failed" -eq 0 ] || return 1
    echo "[loader-constants] self-test ok — held, moved twice, absent, duplicated, missing and self-contradicting all come back as named."
}

case "${1:-}" in
    --self-test) self_test; exit $? ;;
    --find) find_binary; exit $? ;;
    --binary)
        [ -n "${2:-}" ] || { echo "usage: loader-constants.sh --binary <path> [--source <loadset.rs>]" >&2; exit 2; }
        src="$ROOT/src/loadset.rs"
        [ "${3:-}" = "--source" ] && src=${4:-$src}
        check "$2" "$src"; exit $? ;;
    *) echo "usage: loader-constants.sh --binary <path> [--source <loadset.rs>] | --find | --self-test" >&2; exit 2 ;;
esac
