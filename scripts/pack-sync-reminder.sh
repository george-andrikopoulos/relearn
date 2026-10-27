#!/usr/bin/env bash
# pack-sync-reminder.sh — after a push that changes claude-pack/, say that the
# claude.ai account copies of the skills are now behind.
#
#   bash scripts/pack-sync-reminder.sh < <pre-push stdin>   # prints 0 or 1 line
#   bash scripts/pack-sync-reminder.sh --self-test         # the probe, both directions
#
# Nothing refreshes the account copies automatically. A cloud scheduled task
# cannot save an account skill (that tool exists only inside the Cowork desktop
# app), so the only working route is a manual sync in a Cowork session, and a
# copy lags a rule change until someone asks for one. The push of a changed
# pack is the moment the lag starts, so .githooks/pre-push hands this script
# the lines git gives a pre-push hook and it prints one line if any commit
# being pushed touches claude-pack/.
#
# A reminder, not a gate: it always exits 0 in normal mode and cannot refuse a
# push. It refreshes nothing; it only turns "lags until someone asks" into
# "lags until the reminder is acted on".
#
# --self-test builds a throwaway repository and checks that a push touching
# the pack prints the line and that one not touching it, a branch delete and
# empty input print nothing. scripts/check.sh runs it, so a reminder that has
# stopped reminding fails the gate rather than going quiet
# `[R:guarantee-needs-a-reader]`.

set -uo pipefail

PACK=claude-pack
REMINDER='[pre-push] claude-pack/ changed: run the claude.ai skill sync in Cowork, or the account copies stay behind.'

# Reads pre-push lines (<local ref> <local sha> <remote ref> <remote sha>) on
# stdin; succeeds if any commit being pushed touches $PACK/.
pushes_pack() {
    local local_ref local_sha remote_ref remote_sha hit
    local -a range
    while read -r local_ref local_sha remote_ref remote_sha; do
        # A delete pushes no commits.
        [[ -z "${local_sha:-}" || "$local_sha" =~ ^0+$ ]] && continue
        # A new branch, or a remote tip this clone has never seen: count every
        # commit no remote-tracking ref already has.
        if [[ "${remote_sha:-0}" =~ ^0+$ ]] || ! git cat-file -e "${remote_sha}^{commit}" 2>/dev/null; then
            range=("$local_sha" --not --remotes)
        else
            range=("$remote_sha..$local_sha")
        fi
        hit=$(git log -1 --format=%H "${range[@]}" -- "$PACK/" 2>/dev/null)
        [ -n "$hit" ] && return 0
    done
    return 1
}

remind() {
    if pushes_pack; then
        echo "$REMINDER"
    fi
    return 0
}

self_test() {
    local tmp failed=0
    tmp=$(mktemp -d) || return 2
    trap 'rm -rf "$tmp"' RETURN
    mkdir -p "$tmp/repo" "$tmp/nohooks"

    (
        cd "$tmp/repo" || exit 2
        g() { git -c core.hooksPath="$tmp/nohooks" -c core.autocrlf=false \
                  -c user.name=self-test -c user.email=self-test@invalid "$@"; }
        g init -q || exit 2
        echo a > other.txt && g add other.txt && g commit -q -m base || exit 2
        # The literal path, not $PACK: a fixture built from the detector's own
        # variable agrees with a wrong value and the probe passes on a broken
        # reminder `[R:detector-excludes-own-definitions]`.
        mkdir -p claude-pack/skills && echo b > claude-pack/skills/x.md \
            && g add claude-pack && g commit -q -m pack || exit 2
        echo c >> other.txt && g commit -q -am other || exit 2
    ) || { echo "[pack-sync] self-test could not build its fixture repository" >&2; return 2; }

    local base pack other zero
    base=$(git -C "$tmp/repo" rev-parse HEAD~2)
    pack=$(git -C "$tmp/repo" rev-parse HEAD~1)
    other=$(git -C "$tmp/repo" rev-parse HEAD)
    zero=0000000000000000000000000000000000000000

    # expect <name> <want: line|silent> <stdin>
    expect() {
        local name=$1 want=$2 input=$3 got
        got=$(cd "$tmp/repo" && printf '%s' "$input" | remind)
        if [ "$want" = line ] && [ "$got" != "$REMINDER" ]; then
            echo "[pack-sync] self-test FAILED: $name should print the reminder" >&2
            failed=1
        elif [ "$want" = silent ] && [ -n "$got" ]; then
            echo "[pack-sync] self-test FAILED: $name should print nothing" >&2
            failed=1
        fi
    }

    expect "a push including the pack commit" line \
        "refs/heads/main $other refs/heads/main $base"$'\n'
    expect "a push of only a non-pack commit" silent \
        "refs/heads/main $other refs/heads/main $pack"$'\n'
    expect "a new branch carrying the pack commit" line \
        "refs/heads/main $other refs/heads/main $zero"$'\n'
    expect "a branch delete" silent \
        "(delete) $zero refs/heads/gone $other"$'\n'
    expect "empty input" silent ""

    if [ "$failed" -ne 0 ]; then
        return 1
    fi
    echo "[pack-sync] self-test ok — the reminder fires on a pack push and on nothing else."
    return 0
}

case "${1:-}" in
    --self-test) self_test; exit $? ;;
    "") remind; exit 0 ;;
    *) echo "usage: pack-sync-reminder.sh [--self-test]" >&2; exit 2 ;;
esac
