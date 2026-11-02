#!/usr/bin/env bash
# pack-sync-reminder.sh — after a push that changes a pack someone installs by
# hand, say that the installed copies are now behind.
#
#   bash scripts/pack-sync-reminder.sh < <pre-push stdin>   # one line per pack touched
#   bash scripts/pack-sync-reminder.sh --self-test         # the probe, both directions
#
# Two packs are installed by hand and refreshed by nothing:
#
#   claude-pack/  the claude.ai account copies of the skills. A cloud scheduled
#                 task cannot save an account skill, and neither can a session
#                 without the save tool. The route that worked on 2026-11-02 is
#                 to upload each changed home's .skill bundle in the Claude app
#                 (stochos-lab's export-cowork-bundle.sh writes them to
#                 ~/Phd/skills). Nothing is retyped, so the copy is exact.
#   rules-pack/   the user-level domain layers in ~/.claude/rules/, copied there
#                 by hand per rules-pack/README.md. Added 2026-11-02: a pushed
#                 change to a domain layer lagged on every machine until someone
#                 noticed, with nothing saying so.
#
# A copy lags a rule change until someone acts, and the push of a changed pack
# is the moment the lag starts, so .githooks/pre-push hands this script the
# lines git gives a pre-push hook and it prints one line for each pack any
# pushed commit touches.
#
# A reminder, not a gate: it always exits 0 in normal mode and cannot refuse a
# push. It refreshes nothing; it only turns "lags until someone asks" into
# "lags until the reminder is acted on".
#
# --self-test builds a throwaway repository and checks that each pack's push
# prints that pack's line and only that line, that a push touching both prints
# both, and that a push touching neither, a branch delete and empty input print
# nothing. scripts/check.sh runs it, so a reminder that has stopped reminding
# fails the gate rather than going quiet `[R:guarantee-needs-a-reader]`.

set -uo pipefail

# Pack directory, in the order their lines print.
PACKS=(claude-pack rules-pack)

reminder_for() {
    case "$1" in
        claude-pack) echo '[pre-push] claude-pack/ changed: upload the changed homes from ~/Phd/skills/<home>.skill in the Claude app (export-cowork-bundle.sh writes them), or the account copies stay behind.' ;;
        rules-pack) echo '[pre-push] rules-pack/ changed: cp rules-pack/.claude/rules/domain-*.md ~/.claude/rules/, or the installed domain layers stay behind.' ;;
    esac
}

# Reads pre-push lines (<local ref> <local sha> <remote ref> <remote sha>) on
# stdin; succeeds if any commit being pushed touches the pack directory $1.
pushes_pack() {
    local pack=$1 local_ref local_sha remote_ref remote_sha hit
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
        hit=$(git log -1 --format=%H "${range[@]}" -- "$pack/" 2>/dev/null)
        [ -n "$hit" ] && return 0
    done
    return 1
}

remind() {
    # stdin can be read once; every pack is asked about the same push.
    local input pack
    input=$(cat)
    for pack in "${PACKS[@]}"; do
        if pushes_pack "$pack" <<<"$input"; then
            reminder_for "$pack"
        fi
    done
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
        # Literal paths, not $PACKS: a fixture built from the detector's own
        # variable agrees with a wrong value and the probe passes on a broken
        # reminder `[R:detector-excludes-own-definitions]`.
        mkdir -p claude-pack/skills && echo b > claude-pack/skills/x.md \
            && g add claude-pack && g commit -q -m pack || exit 2
        mkdir -p rules-pack/.claude/rules && echo r > rules-pack/.claude/rules/domain-x.md \
            && g add rules-pack && g commit -q -m rules || exit 2
        echo c >> other.txt && g commit -q -am other || exit 2
    ) || { echo "[pack-sync] self-test could not build its fixture repository" >&2; return 2; }

    local base pack rules other zero claude_line rules_line
    base=$(git -C "$tmp/repo" rev-parse HEAD~3)
    pack=$(git -C "$tmp/repo" rev-parse HEAD~2)
    rules=$(git -C "$tmp/repo" rev-parse HEAD~1)
    other=$(git -C "$tmp/repo" rev-parse HEAD)
    zero=0000000000000000000000000000000000000000
    claude_line=$(reminder_for claude-pack)
    rules_line=$(reminder_for rules-pack)

    # expect <name> <want: exact output> <stdin>
    expect() {
        local name=$1 want=$2 input=$3 got
        got=$(cd "$tmp/repo" && printf '%s' "$input" | remind)
        if [ "$got" != "$want" ]; then
            echo "[pack-sync] self-test FAILED: $name printed [$got], wanted [$want]" >&2
            failed=1
        fi
    }

    expect "a push including both pack commits" "$claude_line"$'\n'"$rules_line" \
        "refs/heads/main $other refs/heads/main $base"$'\n'
    expect "a push of only the claude-pack commit" "$claude_line" \
        "refs/heads/main $pack refs/heads/main $base"$'\n'
    expect "a push of only the rules-pack commit" "$rules_line" \
        "refs/heads/main $rules refs/heads/main $pack"$'\n'
    expect "a push of only a non-pack commit" "" \
        "refs/heads/main $other refs/heads/main $rules"$'\n'
    expect "a new branch carrying both pack commits" "$claude_line"$'\n'"$rules_line" \
        "refs/heads/main $other refs/heads/main $zero"$'\n'
    expect "a branch delete" "" \
        "(delete) $zero refs/heads/gone $other"$'\n'
    expect "empty input" "" ""

    if [ "$failed" -ne 0 ]; then
        return 1
    fi
    echo "[pack-sync] self-test ok — each pack's push prints its own line, and nothing else does."
    return 0
}

case "${1:-}" in
    --self-test) self_test; exit $? ;;
    "") remind; exit 0 ;;
    *) echo "usage: pack-sync-reminder.sh [--self-test]" >&2; exit 2 ;;
esac
