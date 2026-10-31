#!/usr/bin/env bash
# probe-run.sh — replay each rule's probe against a model, with the rule's home
# layer and without it, and report the violation rate of each arm.
#
#   bash scripts/probe-run.sh --self-test
#   bash scripts/probe-run.sh --dry-run --model <m> [--samples N] [--tag R:x ...]
#   bash scripts/probe-run.sh --model <m> --samples N --budget-usd X \
#        --out <results-dir> [--work-root <dir>] [--tag R:x ...]
#
# THIS IS THE ONLY THING IN THE REPOSITORY THAT STARTS A MODEL. relearn itself
# opens no socket and spawns no process (`tests/solo_mode.rs`); it owns what a
# probe is (`relearn probe list|task`) and how a session is judged (`relearn
# probe judge|tally`), and this script does the running in between. See the
# ARCHITECTURE decisions log, 2026-10-30, for why the split falls here.
#
# --self-test runs every probe's checker over its committed held and violated
# cases and fails unless each comes back as named. It makes no model call and
# is a stage of scripts/check.sh: a checker that cannot say `violated` about a
# known-bad session measures nothing (`[R:evidence-must-discriminate]`).
#
# COST IS NEVER SPENT SILENTLY (`[R:no-silent-spend]`). --dry-run prints the
# session count, an estimated token count and cost, and an estimated wall
# clock, and starts nothing. A real run needs --budget-usd, aborts before the
# next session once the reported cost reaches it, gives each session
# --max-budget-usd, and refuses --samples above --max-samples.
#
# ISOLATION. Both arms run in fresh directories under --work-root, which must lie
# outside this repository AND outside $HOME: on Windows the system temp
# directory sits under the home directory, so `~/.claude` would be an ancestor
# `.claude/` of every workspace. Each workspace is `git init`ed so discovery
# stops at it. The user layer is cut off with an empty CLAUDE_CONFIG_DIR,
# `--setting-sources project,local` and `--strict-mcp-config`. The docs do not
# promise that this removes everything under ~/.claude, so two things are
# checked rather than assumed: every session's `init` event (anything in it
# beyond the CLI's own baseline and the WITH arm's skill makes the session
# `contaminated`, excluded and counted), and one preflight session at the start
# of a real run that must not be able to quote the user's CLAUDE.md. The canary is read from that file at
# run time and is never printed (`[R:report-the-hit-not-the-match]`).
#
# Raw transcripts and workspaces go to --out, which must also lie outside the
# repository. They can carry private content and are never committed.
#
# Exit: 0 done / 1 a self-test case disagreed or a session failed the run /
# 2 refused before starting (bad arguments, missing tool, missing auth).

set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
rules="$repo/rules"
probes="$repo/tests/probes"

# --- the price and size figures behind --dry-run -----------------------------
# Per million tokens: input, output, cache write, cache read. Official pricing
# page, read 2026-10-30. The cache write is the 5-minute one, 1.25x input: every
# write in the 2026-10-31 pilot was `ephemeral_5m`, and at that price the
# pilot's token counts reproduce its reported cost exactly (the 1-hour price,
# assumed before it, put the cost about a third too high). A model not listed here
# has no estimate, and --dry-run says so rather than guess.
price_for() {
    case "$1" in
        claude-opus-5-5 | opus) echo "4 20 5 0.20" ;;
        claude-sonnet-5-5 | sonnet) echo "2 10 2.5 0.10" ;;
        *) return 1 ;;
    esac
}
# Per session, MEASURED on the 2026-10-31 pilot (claude-opus-5-5, CLI 2.1.316,
# the 137 sessions that left a transcript): cache read, cache written, output,
# uncached input. The low end is each arm's mean, the high end its costliest
# session. The WITH arm is dearer because a session that opens the skill reads
# it. Another model, or a probe set with longer tasks, may differ.
MEAN_WITHOUT="44647 6527 1365 9"
MEAN_WITH="73167 15004 1874 9"
MAX_WITHOUT="65630 7881 2391 12"
MAX_WITH="145569 30341 4017 12"
# Wall clock per session, judging included: 67 minutes for 140 sessions.
SECONDS_PER_SESSION=29

die() { echo "probe-run: $*" >&2; exit 2; }

relearn() { cargo run --quiet --manifest-path "$repo/Cargo.toml" -- "$@"; }

native() { if command -v cygpath > /dev/null; then cygpath -w "$1"; else printf '%s' "$1"; fi; }

# --- arguments ---------------------------------------------------------------
mode=run model="" samples=10 max_samples=20 budget="" out="" tags=()
per_session=2.00
case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) work_root=/c/relearn-probes ;;
    *) work_root=/tmp/relearn-probes ;;
esac
while [ $# -gt 0 ]; do
    case $1 in
        --self-test) mode=self-test ;;
        --dry-run) mode=dry-run ;;
        --model) model=${2:?--model needs a value}; shift ;;
        --samples) samples=${2:?--samples needs a value}; shift ;;
        --max-samples) max_samples=${2:?--max-samples needs a value}; shift ;;
        --budget-usd) budget=${2:?--budget-usd needs a value}; shift ;;
        --session-budget-usd) per_session=${2:?--session-budget-usd needs a value}; shift ;;
        --out) out=${2:?--out needs a value}; shift ;;
        --work-root) work_root=${2:?--work-root needs a value}; shift ;;
        --tag) tags+=("${2:?--tag needs a value}"); shift ;;
        *) die "unknown argument $1" ;;
    esac
    shift
done

command -v jq > /dev/null || die "jq is required"
plan=$(relearn probe list --rules "$rules") || die "relearn probe list failed"
plan=$(printf '%s\n' "$plan" | grep -v '^#')
if [ "${#tags[@]}" -gt 0 ]; then
    filtered=""
    for t in "${tags[@]}"; do
        line=$(printf '%s\n' "$plan" | awk -F'\t' -v t="$t" '$1 == t')
        [ -n "$line" ] || die "$t carries no probe"
        filtered+="$line"$'\n'
    done
    plan=${filtered%$'\n'}
fi
[ -n "$plan" ] || die "no rule carries a probe"

body_of() { printf '%s' "${1#R:}"; }

# --- self-test ---------------------------------------------------------------
self_test() {
    local failures=0 cases=0 tag home layer ws kind dir expect got rc tmp
    while IFS=$'\t' read -r tag home layer ws kind; do
        dir="$probes/$(body_of "$tag")"
        for expect in held violated; do
            case $kind in
                trace) got=$(relearn probe judge --rules "$rules" --tag "$tag" \
                    --transcript "$dir/cases/$expect.jsonl") ;;
                diff) got=$(relearn probe judge --rules "$rules" --tag "$tag" \
                    --fixture "$dir/workspace" --workspace "$dir/cases/$expect") ;;
                run)
                    tmp=$(mktemp -d)
                    cp -r "$dir/cases/$expect/." "$tmp/"
                    bash "$dir/check.sh" "$tmp" "$dir/workspace" 2> /dev/null
                    rc=$?
                    rm -rf "$tmp"
                    got=$(relearn probe judge --rules "$rules" --tag "$tag" --run-exit "$rc") ;;
                *) got="unknown checker $kind" ;;
            esac
            cases=$((cases + 1))
            if [ "${got%%$'\t'*}" = "$expect" ]; then
                echo "ok   $tag $expect"
            else
                echo "FAIL $tag: the $expect case came back ${got%%$'\t'*}"
                failures=$((failures + 1))
            fi
        done
    done <<< "$plan"
    if [ "$cases" -eq 0 ]; then
        echo "FAIL: no case ran"
        return 1
    fi
    if [ "$failures" -ne 0 ]; then
        echo "FAIL: $failures of $cases probe case(s) disagreed with their names"
        return 1
    fi
    echo "PASS: $cases probe case(s), every checker says both held and violated"
}

if [ "$mode" = self-test ]; then
    self_test
    exit $?
fi

# --- the plan and its estimate -----------------------------------------------
[ -n "$model" ] || die "--model is required: the arms are compared on one named model"
case $samples in '' | *[!0-9]*) die "--samples must be a whole number" ;; esac
case $max_samples in '' | *[!0-9]*) die "--max-samples must be a whole number" ;; esac
[ "$samples" -ge 1 ] || die "--samples must be at least 1"
[ "$samples" -le "$max_samples" ] || die "--samples $samples is above --max-samples $max_samples"

n_probes=$(printf '%s\n' "$plan" | grep -c .)
sessions=$((n_probes * 2 * samples))

estimate() {
    local p in_p out_p cw_p cr_p
    if ! p=$(price_for "$model"); then
        echo "estimated cost: none: no price is recorded for '$model' in this script"
        return
    fi
    read -r in_p out_p cw_p cr_p <<< "$p"
    awk -v s="$sessions" -v mw="$MEAN_WITH" -v mo="$MEAN_WITHOUT" \
        -v xw="$MAX_WITH" -v xo="$MAX_WITHOUT" \
        -v ip="$in_p" -v op="$out_p" -v wp="$cw_p" -v rp="$cr_p" '
    function tokens(f,   a) { split(f, a, " "); return a[1] + a[2] + a[3] + a[4] }
    function cost(f,   a) { split(f, a, " "); return (a[1] * rp + a[2] * wp + a[3] * op + a[4] * ip) / 1e6 }
    BEGIN {
        # Half the sessions are each arm.
        t  = (tokens(mw) + tokens(mo)) / 2
        lo = (cost(mw) + cost(mo)) / 2
        hi = (cost(xw) + cost(xo)) / 2
        printf "estimated tokens per session: about %d (WITH %d, WITHOUT %d, mean)\n", t, tokens(mw), tokens(mo)
        printf "estimated tokens in total: about %.1f million\n", s * t / 1e6
        printf "estimated cost: $%.2f to $%.2f ($%.3f to $%.3f per session)\n", s * lo, s * hi, lo, hi
    }'
}

if [ "$mode" = dry-run ]; then
    echo "probe-run --dry-run: nothing is started"
    echo "model: $model"
    echo "probes: $n_probes"
    printf '%s\n' "$plan" | awk -F'\t' '{ printf "  %-42s %-8s %-6s layer: %s\n", $1, $5, $4, $3 }'
    echo "samples per arm: $samples (hard cap --max-samples $max_samples)"
    echo "headless sessions: $sessions ($n_probes probes x 2 arms x $samples), plus 1 isolation preflight"
    estimate
    awk -v s="$sessions" -v t="$SECONDS_PER_SESSION" 'BEGIN {
        printf "estimated wall clock, serial: about %.1f hours\n", s * t / 3600 }'
    echo "per-session figures above were MEASURED on one pilot (2026-10-31, claude-opus-5-5); another model or probe set may differ"
    exit 0
fi

# --- a real run ----------------------------------------------------------------
[ -n "$budget" ] || die "--budget-usd is required for a real run: no run spends without a ceiling"
# Money is whole micro-dollars, never a float (`[R:money-is-not-a-float]`). This
# reads what a person types (`50`, `2.5`) and what `relearn probe judge` prints
# (`0.351234`, always six places), and refuses anything else.
micros() {
    local v=${1:-0} int frac=""
    case $v in *[!0-9.]* | *.*.* | . ) return 1 ;; esac
    int=${v%%.*}
    [ "$int" = "$v" ] || frac=${v#*.}
    [ "${#frac}" -le 6 ] || return 1
    frac=$(printf '%-6s' "$frac")
    echo $((10#${int:-0} * 1000000 + 10#${frac// /0}))
}
dollars() { printf '%d.%06d' $(($1 / 1000000)) $(($1 % 1000000)); }
budget_micros=$(micros "$budget") || die "--budget-usd $budget is not an amount in dollars"
session_micros=$(micros "$per_session") || die "--session-budget-usd $per_session is not an amount in dollars"
[ "$budget_micros" -gt 0 ] || die "--budget-usd must be above zero"
[ -n "$out" ] || die "--out is required: transcripts never go into the repository"
command -v claude > /dev/null || die "the claude CLI is not on PATH"
if [ -z "${CLAUDE_CODE_OAUTH_TOKEN:-}" ] && [ -z "${ANTHROPIC_API_KEY:-}" ]; then
    die "set CLAUDE_CODE_OAUTH_TOKEN (claude setup-token) or ANTHROPIC_API_KEY: an empty CLAUDE_CONFIG_DIR has no login"
fi

inside() { case "$1/" in "$2"/*) return 0 ;; *) return 1 ;; esac; }
mkdir -p "$out" "$work_root" || die "cannot create --out or --work-root"
out=$(cd "$out" && pwd)
work_root=$(cd "$work_root" && pwd)
home=$(cd "$HOME" && pwd)
for d in "$out" "$work_root"; do
    inside "$d" "$repo" && die "$d is inside the repository"
done
inside "$work_root" "$home" && die "--work-root $work_root is inside \$HOME, where ~/.claude is an ancestor of every workspace"


cli_flags=(--model "$model"
    --output-format stream-json --verbose
    --setting-sources project,local
    --strict-mcp-config
    --permission-mode dontAsk --permission-prompts none
    --tools "Bash,Read,Write,Edit,Glob,Grep,Skill"
    --allowedTools "Bash Read Write Edit Glob Grep Skill"
    --no-session-persistence)

session() { # <workspace> <task> <transcript> <session-budget>
    local cfg="$1.config"
    mkdir -p "$cfg"
    (cd "$1" && CLAUDE_CONFIG_DIR=$(native "$cfg") claude -p "$2" "${cli_flags[@]}" \
        --max-budget-usd "$4" < /dev/null > "$3" 2> "$3.stderr")
}

judge_loaded() { # <transcript> <an existing workspace>: the judge's cost and loaded fields
    local line
    line=$(relearn probe judge --rules "$rules" --tag "$(printf '%s\n' "$plan" | head -n 1 | cut -f1)" \
        --transcript "$1" --workspace "$2" --run-exit 2 2> "$1.judge.stderr") || return 1
    printf '%s\n' "$line" | cut -f2,4
}

# What the CLI loads by itself, measured rather than listed. A session with an
# empty home, an empty config directory and a key that cannot authenticate
# prints its `init` and fails at the first request, so it costs nothing. A
# session listing anything beyond this, other than the WITH arm's own skill,
# took it from somewhere else, and is contaminated. A list of the user's skill
# names could not decide this: Claude Code bundles `deep-research`, and an
# account-synced copy of Anthropic's skill uses the same name, which failed a
# clean preflight on 2026-10-31. A synced copy loads as `<plugin>:<skill>` besides.
base="$work_root/baseline-$$"
mkdir -p "$base/ws" "$base/home" "$base/cfg" && git -C "$base/ws" init -q
(cd "$base/ws" && env -u CLAUDE_CODE_OAUTH_TOKEN HOME="$(native "$base/home")" USERPROFILE="$(native "$base/home")" \
    CLAUDE_CONFIG_DIR="$(native "$base/cfg")" ANTHROPIC_API_KEY=probe-baseline-cannot-authenticate \
    claude -p "Answer NONE." "${cli_flags[@]}" --max-budget-usd 0.01 \
    < /dev/null > "$out/baseline.jsonl" 2> "$out/baseline.jsonl.stderr")
base_line=$(judge_loaded "$out/baseline.jsonl" "$base/ws")
base_rc=$?
rm -rf "${base:?}"
[ "$base_rc" -eq 0 ] || die "the baseline transcript could not be read (location: $out/baseline.jsonl)"
grep -q '"subtype":"init"' "$out/baseline.jsonl" ||
    die "the CLI's own baseline printed no init event, so contamination could not be judged (location: $out/baseline.jsonl)"
IFS=$'\t' read -r base_cost base_loaded <<< "$base_line"
[ "$(micros "${base_cost:-0}")" -eq 0 ] || die "the baseline session reported a cost, so its key authenticated; nothing else was run"
baseline=$(tr , '\n' <<< "$base_loaded")
echo "CLI baseline: $(grep -c . <<< "$baseline") item(s) the CLI loads by itself"

contaminated() { # <loaded> <allowed-skill or empty>; prints the first item from elsewhere
    local item
    IFS=, read -r -a items <<< "$1"
    for item in "${items[@]}"; do
        [ -n "$item" ] || continue
        [ -n "$2" ] && [ "$item" = "skill:$2" ] && continue
        grep -qxF "$item" <<< "$baseline" || { printf '%s' "$item"; return 0; }
    done
    return 1
}

spent=0
# The tally sums results.tsv, which has no row for the preflight; this is the
# figure the budget was enforced against.
spent_line() {
    echo "spent, including the isolation preflight: \$$(dollars "$spent")"
    # One model per run is a stated condition; one CLI is too, and an update
    # elsewhere on the machine can change it between sessions (seen 2026-10-31).
    echo "CLI version per session:"
    find "$out" -name transcript.jsonl -exec jq -r 'select(.type == "system" and .subtype == "init") | .claude_code_version // "unreported"' {} + |
        sort | uniq -c
}
# A probe session never updates the CLI itself.
export DISABLE_AUTOUPDATER=1
results="$out/results.tsv"
[ -f "$results" ] || printf '#tag\tarm\tsample\toutcome\tcost_usd\ttokens\tloaded\tskills_invoked\n' > "$results"

# The isolation preflight: an empty workspace, the WITHOUT arm's flags, and a
# question only a leaked user layer could answer. The canary is the longest
# ASCII run in the first heading of the user's CLAUDE.md: a transcript may
# escape any other character, and a canary that cannot match is a preflight
# that cannot fail (`[R:evidence-must-discriminate]`). It is never printed.
user_md="$HOME/.claude/CLAUDE.md"
canary=""
if [ -f "$user_md" ]; then
    canary=$(sed -n 's/^# //p' "$user_md" | head -n 1 | LC_ALL=C grep -o '[A-Za-z0-9 &,.:;()-]*' |
        awk '{ gsub(/^ +| +$/, "") } length > length(best) { best = $0 } END { print best }')
    [ "${#canary}" -ge 12 ] || die "cannot derive a canary from the first heading of $user_md, so the preflight could not fail"
fi
pre="$work_root/preflight-$$"
mkdir -p "$pre" && git -C "$pre" init -q
session "$pre" "Quote the first heading of every CLAUDE.md or other instruction file you were given, and list every skill you can see. If there are none, answer NONE." \
    "$out/preflight.jsonl" "$(dollars "$session_micros")"
pre_src=$?
pre_line=$(judge_loaded "$out/preflight.jsonl" "$pre")
pre_rc=$?
rm -rf "${pre:?}" "${pre:?}.config"
[ "$pre_rc" -eq 0 ] || die "isolation preflight: the transcript could not be read (location: $out/preflight.jsonl)"
IFS=$'\t' read -r pre_cost pre_loaded <<< "$pre_line"
spent=$((spent + $(micros "${pre_cost:-0}")))
# A preflight that failed and spent nothing never reached the model, usually a
# key that does not authenticate; every session after it would do the same.
if [ "$pre_src" -ne 0 ] && [ "$(micros "${pre_cost:-0}")" -eq 0 ]; then
    die "isolation preflight: the session exited $pre_src having spent nothing, so it never reached the model; check the key (location: $out/preflight.jsonl). Nothing else was run."
fi
if [ -n "$canary" ] && grep -qF "$canary" "$out/preflight.jsonl"; then
    die "isolation preflight FAILED: the session could quote the user's CLAUDE.md (location: $out/preflight.jsonl). Nothing else was run."
fi
if leaked=$(contaminated "$pre_loaded" ""); then
    die "isolation preflight FAILED: the session's init listed $leaked, which the CLI does not load by itself (location: $out/preflight.jsonl). Nothing else was run."
fi
echo "isolation preflight: no user CLAUDE.md canary, and nothing in init beyond the CLI's baseline"

failed=0
started=$(date +%s)
for ((i = 1; i <= samples; i++)); do
    # Alternate which arm goes first, so an arm is never always the earlier one.
    if ((i % 2)); then arms="with without"; else arms="without with"; fi
    while IFS=$'\t' read -r tag home layer wsk kind; do
        body=$(body_of "$tag")
        task=$(relearn probe task --rules "$rules" --tag "$tag")
        for arm in $arms; do
            if [ "$spent" -ge "$budget_micros" ]; then
                echo "probe-run: budget reached (\$$budget); stopping before the next session" >&2
                relearn probe tally --rules "$rules" --results "$results"
                spent_line
                exit 1
            fi
            ws="$work_root/$body-$arm-$i-$$"
            rec="$out/$body/$arm-$i"
            mkdir -p "$ws" "$rec"
            [ "$wsk" = fixture ] && cp -r "$probes/$body/workspace/." "$ws/"
            git -C "$ws" init -q
            allowed=""
            if [ "$arm" = with ]; then
                case $layer in
                    claude) relearn build --rules "$rules" --targets claude --home "$home" --out "$ws/.claude" > /dev/null ;;
                    # The workspace is named for no project, so the layer is explicit:
                    # a project home goes into its own project layer, a domain into
                    # the user layer, which is what a domain is outside a probe.
                    claude-rules)
                        case $home in
                            project-*) rl="project=${home#project-}" ;;
                            *) rl=user ;;
                        esac
                        relearn build --rules "$rules" --targets claude-rules --rules-layer "$rl" --home "$home" --out "$ws" > /dev/null ;;
                esac || { echo "probe-run: could not emit $home for $tag" >&2; failed=1; continue; }
                allowed=$home
            fi
            left=$((budget_micros - spent))
            [ "$left" -lt "$session_micros" ] || left=$session_micros
            session "$ws" "$task" "$rec/transcript.jsonl" "$(dollars "$left")"
            src=$?
            mkdir -p "$rec/workspace" && cp -r "$ws/." "$rec/workspace/"
            judge=(relearn probe judge --rules "$rules" --tag "$tag" --transcript "$rec/transcript.jsonl")
            case $kind in
                diff) judge+=(--workspace "$ws") ; [ "$wsk" = fixture ] && judge+=(--fixture "$probes/$body/workspace") ;;
                run)
                    scratch="$ws.check"
                    mkdir -p "$scratch" && cp -r "$ws/." "$scratch/"
                    bash "$probes/$body/check.sh" "$scratch" "$probes/$body/workspace" 2> "$rec/check.stderr"
                    judge+=(--run-exit $?) ;;
            esac
            if line=$("${judge[@]}" 2> "$rec/judge.stderr"); then
                IFS=$'\t' read -r verdict cost tokens loaded invoked <<< "$line"
                outcome=$verdict
                # A session that failed and spent nothing did nothing, so a verdict
                # on its empty transcript would be `inconclusive` for a reason that
                # is not the probe's. One that failed after spending, such as at its
                # cap, did work, and is judged on what it did.
                if [ "$src" -ne 0 ] && [ "$(micros "${cost:-0}")" -eq 0 ]; then outcome=error; fi
                contaminated "$loaded" "$allowed" > /dev/null && outcome=contaminated
            else
                outcome=error cost="" tokens="" loaded="" invoked=""
            fi
            [ "$outcome" = error ] && failed=1
            spent=$((spent + $(micros "$cost")))
            printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$tag" "$arm" "$i" "$outcome" "$cost" \
                "$tokens" "$loaded" "$invoked" >> "$results"
            echo "$tag $arm $i: $outcome (\$${cost:-?})"
            rm -rf "$ws" "$ws.config" "$ws.check"
        done
    done <<< "$plan"
done
echo "wall clock: $(( ($(date +%s) - started) / 60 )) minute(s)"
relearn probe tally --rules "$rules" --results "$results"
spent_line
exit $failed
