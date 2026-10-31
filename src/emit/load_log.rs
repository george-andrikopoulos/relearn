//! `emit::load_log` — the live load log: a Claude Code hook that appends one
//! JSON line per instruction-layer load to a local file, its fixture runner,
//! and its registration.
//!
//! Writes, under `<out>/hooks/`:
//!
//! * `relearn-load-log.sh` — handles two events. **PostToolUse on `Skill`**: a
//!   skill the model opened. The payload carries the name as invoked
//!   (`tool_input.skill`) but no path — measured 2026-10-31 on real
//!   transcripts, whose tool result is `{success, commandName}` — so the hook
//!   finds the file itself, in the order the skills reference gives
//!   (personal over project; project skills from the session's directory and
//!   every directory above it), and an account-synced `<plugin>:<skill>` under
//!   `skills/synced/`. **InstructionsLoaded**: a `CLAUDE.md` or `.claude/rules/`
//!   file the loader read, which carries its own path and `memory_type`. For
//!   either, the hash comes from the file's `relearn:generated` marker on disk,
//!   because no payload carries it.
//! * `test-load-log.sh` — synthetic payloads through the hook, on the
//!   production path. Its skill cases are [`skill_cases`], which a test holds
//!   to [`crate::probe::skill_named`], so the hook's `jq` and relearn's
//!   transcript parser cannot read different names from one payload.
//! * `load-log-settings-snippet.md` — the registration.
//!
//! A pure function of nothing: the hook is the same for every corpus, and the
//! reader (`relearn loads`) is where the corpus comes in. **Explicit only**,
//! like `hooks`, and **never installed**: it writes a file on every skill load
//! on the machine, which is George's to switch on.

use super::{OutputFile, RelativePath};

/// The directory this emitter owns, relative to the output root — the same
/// one `hooks` writes, because both install into `~/.claude/hooks/relearn/`.
pub const HOOKS_DIR: &str = "hooks";

/// Where the committed pack lives in this repository.
pub const PACK_DIR: &str = "load-log-pack";

/// The hook's file stem. Reserved against detector names.
pub const HOOK: &str = "relearn-load-log";

/// The fixture runner's file stem. Reserved against detector names.
pub const RUNNER: &str = "test-load-log";

/// The settings snippet's file name: not `settings-snippet.md`, which
/// hooks-pack writes into the same install directory.
pub const SNIPPET: &str = "load-log-settings-snippet.md";

/// One skill payload the runner pipes through the hook, and what it expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillCase {
    what: &'static str,
    tool_input: &'static str,
    name: Option<&'static str>,
    location: Option<&'static str>,
    hash: Option<char>,
}

impl SkillCase {
    /// The `tool_input` object, as JSON text.
    #[must_use]
    pub fn tool_input(self) -> &'static str {
        self.tool_input
    }

    /// The name the hook must log; `None` means it must log nothing.
    #[must_use]
    pub fn name(self) -> Option<&'static str> {
        self.name
    }
}

/// The skill payloads the runner checks. The fixture tree it builds holds a
/// personal `global` (marker of `a`s), a personal `plain` with no marker, an
/// account-synced `domain-x` (`b`s), and a project `project-p` (`c`s) two
/// directories above the session's.
#[must_use]
pub fn skill_cases() -> &'static [SkillCase] {
    const CASES: &[SkillCase] = &[
        SkillCase {
            what: "a personal skill by its name",
            tool_input: r#"{"skill":"global","args":"x"}"#,
            name: Some("global"),
            location: Some("user"),
            hash: Some('a'),
        },
        SkillCase {
            what: "an account-synced copy by its plugin name",
            tool_input: r#"{"skill":"anthropic-skills:domain-x"}"#,
            name: Some("anthropic-skills:domain-x"),
            location: Some("account-synced"),
            hash: Some('b'),
        },
        SkillCase {
            what: "a project skill above the session's directory, named by `command`",
            tool_input: r#"{"command":"project-p"}"#,
            name: Some("project-p"),
            location: Some("project"),
            hash: Some('c'),
        },
        SkillCase {
            what: "a copy with no marker",
            tool_input: r#"{"skill":"plain"}"#,
            name: Some("plain"),
            location: Some("user"),
            hash: None,
        },
        SkillCase {
            what: "a skill the hook cannot find",
            tool_input: r#"{"skill":"nowhere"}"#,
            name: Some("nowhere"),
            location: None,
            hash: None,
        },
        SkillCase {
            what: "a call that names no skill",
            tool_input: r#"{"args":"x"}"#,
            name: None,
            location: None,
            hash: None,
        },
    ];
    CASES
}

/// Emit the load-log pack.
#[must_use]
pub fn emit() -> Vec<OutputFile> {
    vec![
        OutputFile::new(
            RelativePath::from_segments(&[HOOKS_DIR, SNIPPET]),
            snippet(),
            Vec::new(),
        ),
        OutputFile::script(
            RelativePath::from_segments(&[HOOKS_DIR, &format!("{HOOK}.sh")]),
            HOOK_SCRIPT.to_owned(),
            Vec::new(),
        ),
        OutputFile::script(
            RelativePath::from_segments(&[HOOKS_DIR, &format!("{RUNNER}.sh")]),
            runner(),
            Vec::new(),
        ),
    ]
}

const HOOK_SCRIPT: &str = r##"#!/usr/bin/env bash
# relearn load log: one JSON line per instruction-layer load, appended to a local file.
# Generated by relearn (`relearn build --targets load-log`); never edit this script.
# Events: PostToolUse on Skill (a skill the model opened) and InstructionsLoaded
# (a CLAUDE.md or .claude/rules/ file the loader read).
# Never blocks. Exit 0 when it logged or had nothing to log; exit 1 when it could
# not log, which Claude Code reports and does not act on. Never exit 2: on
# PostToolUse that would show this script's stderr to the model.
set -uo pipefail
export LC_ALL=C

log="${RELEARN_LOAD_LOG:-$HOME/.local/state/relearn/load-log.jsonl}"
skills="${RELEARN_SKILLS_HOME:-$HOME/.claude/skills}"

fail() {
  printf '[relearn-load-log] %s; nothing was logged.\n' "$1" >&2
  exit 1
}
command -v jq >/dev/null 2>&1 || fail "jq is not on PATH"

input="$(cat)"
event="$(jq -r '.hook_event_name // ""' <<<"$input" 2>/dev/null)" || fail "the payload is not JSON"

# The sha256 in a file's relearn:generated marker, or nothing.
marker_hash() {
  [ -f "$1" ] || return 0
  grep -ao 'relearn:generated v[^ ]* sha256=[0-9a-f]\{64\}' -- "$1" | tail -n 1 | sed 's/.*sha256=//'
}

name="" path="" location="" kind="" reason=""
case "$event" in
  PostToolUse)
    [ "$(jq -r '.tool_name // ""' <<<"$input")" = Skill ] || exit 0
    # The field relearn's transcript parser reads from the same call.
    name="$(jq -r '.tool_input.skill // .tool_input.command // ""' <<<"$input")"
    [ -n "$name" ] || exit 0
    kind=skill
    case "$name" in
      *:*)
        # An account-synced copy loads as <plugin>:<skill>.
        for f in "$skills"/synced/*/"${name##*:}"/SKILL.md; do
          if [ -f "$f" ]; then path="$f" location=account-synced; break; fi
        done ;;
      *)
        # Personal wins over project; project skills load from the session's
        # directory and every directory above it.
        if [ -f "$skills/$name/SKILL.md" ]; then
          path="$skills/$name/SKILL.md" location=user
        else
          dir="$(jq -r '.cwd // ""' <<<"$input")"
          dir="${dir//\\//}"
          while [ -n "$dir" ] && [ "$dir" != . ]; do
            if [ -f "$dir/.claude/skills/$name/SKILL.md" ]; then
              path="$dir/.claude/skills/$name/SKILL.md" location=project
              break
            fi
            up="$(dirname "$dir")"
            [ "$up" = "$dir" ] && break
            dir="$up"
          done
        fi ;;
    esac ;;
  InstructionsLoaded)
    kind=instructions
    path="$(jq -r '.file_path // ""' <<<"$input")"
    path="${path//\\//}"
    reason="$(jq -r '.load_reason // ""' <<<"$input")"
    case "$(jq -r '.memory_type // ""' <<<"$input")" in
      User) location=user ;;
      Project) location=project ;;
      Local) location=local ;;
      Managed) location=managed ;;
    esac ;;
  *) exit 0 ;;
esac

hash="$(marker_hash "$path")"
mkdir -p "$(dirname "$log")" || fail "cannot create the log's directory"
jq -nc \
  --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg session "$(jq -r '.session_id // ""' <<<"$input")" \
  --arg event "$kind" --arg name "$name" --arg path "$path" \
  --arg location "$location" --arg hash "$hash" --arg reason "$reason" \
  'def opt: if . == "" then null else . end;
   {v: 1, ts: $ts, session: $session, event: $event, name: ($name | opt),
    path: ($path | opt), location: ($location | opt), hash: ($hash | opt),
    load_reason: ($reason | opt)}' >>"$log" || fail "cannot append to $log"
exit 0
"##;

const RUNNER_HEAD: &str = r##"#!/usr/bin/env bash
# The load-log hook on the production path: synthetic payloads piped through
# relearn-load-log.sh beside this file, against a synthetic skills tree.
# Generated by relearn; the skill cases come from the same table a Rust test
# holds to relearn's transcript parser.
set -uo pipefail
export LC_ALL=C MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*'

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if ! command -v jq >/dev/null 2>&1; then
  echo "DISARMED (exit 2, not a pass): jq is not on PATH, so no case ran"
  exit 2
fi
work="$(mktemp -d)" || { echo "FAIL: no scratch directory"; exit 1; }
trap 'rm -rf "$work"' EXIT

skills="$work/home/.claude/skills"
proj="$work/proj"
sixty_four() { printf "$1%.0s" $(seq 64); }
mk() {
  mkdir -p "$(dirname "$1")"
  printf -- '---\nname: fixture\n---\nA synthetic layer.\n' > "$1"
  [ -z "$2" ] || printf '\n<!-- relearn:generated v0.0.0 sha256=%s rules=R:fixture -- DO NOT EDIT -->\n' "$(sixty_four "$2")" >> "$1"
}
mk "$skills/global/SKILL.md" a
mk "$skills/plain/SKILL.md" ""
mk "$skills/synced/bucket/domain-x/SKILL.md" b
mk "$proj/.claude/skills/project-p/SKILL.md" c
mk "$proj/.claude/rules/project-p.md" d
mk "$work/home/.claude/CLAUDE.md" ""
mkdir -p "$proj/sub/deeper"
export RELEARN_LOAD_LOG="$work/log.jsonl" RELEARN_SKILLS_HOME="$skills"
: > "$RELEARN_LOAD_LOG"

pass=0
failed=0
lines() { wc -l < "$RELEARN_LOAD_LOG" | tr -d ' '; }
field() { tail -n 1 "$RELEARN_LOAD_LOG" | jq -r ".$1 // \"-\""; }

# check <what> <payload> <expect exit> <name|-|none> <location|-> <hash letter|->
# A name of `none` means no line may be written.
check() {
  local what="$1" payload="$2" want_exit="$3" want_name="$4" want_loc="$5" want_hash="$6"
  local before got after ok=yes
  before="$(lines)"
  printf '%s' "$payload" | bash "$here/relearn-load-log.sh" 2>"$work/stderr"
  got=$?
  after="$(lines)"
  [ "$got" -eq "$want_exit" ] || ok=no
  [ "$got" -ne 2 ] || ok=no
  if [ "$want_name" = none ]; then
    [ "$after" -eq "$before" ] || ok=no
  else
    [ "$after" -eq $((before + 1)) ] || ok=no
    [ "$(field name)" = "$want_name" ] || ok=no
    [ "$(field location)" = "$want_loc" ] || ok=no
    local h=-
    [ "$want_hash" = - ] || h="$(sixty_four "$want_hash")"
    [ "$(field hash)" = "$h" ] || ok=no
  fi
  if [ "$ok" = yes ]; then
    pass=$((pass + 1))
  else
    failed=$((failed + 1))
    echo "FAIL: $what (exit $got, want $want_exit; lines $before -> $after)"
    [ ! -s "$work/stderr" ] || sed 's/^/  stderr: /' "$work/stderr"
  fi
}

skill() {
  jq -nc --argjson ti "$1" --arg cwd "$proj/sub/deeper" \
    '{hook_event_name:"PostToolUse",session_id:"s1",cwd:$cwd,tool_name:"Skill",tool_input:$ti,tool_response:{success:true,commandName:"x"}}'
}
loaded() {
  jq -nc --arg p "$1" --arg m "$2" \
    '{hook_event_name:"InstructionsLoaded",session_id:"s2",cwd:"/",file_path:$p,memory_type:$m,load_reason:"session_start"}'
}

"##;

const RUNNER_TAIL: &str = r##"
check "another tool's call" \
  "$(jq -nc '{hook_event_name:"PostToolUse",session_id:"s",tool_name:"Write",tool_input:{file_path:"x"}}')" \
  0 none - -
check "a rules layer the loader read" "$(loaded "$proj/.claude/rules/project-p.md" Project)" 0 - project d
check "a user CLAUDE.md with no marker" "$(loaded "$work/home/.claude/CLAUDE.md" User)" 0 - user -
check "an event this hook does not handle" '{"hook_event_name":"Stop","session_id":"s"}' 0 none - -
check "a payload that is not JSON" 'not json' 1 none - -

total=$((pass + failed))
if [ "$failed" -eq 0 ]; then
  echo "PASS: $total load-log case(s) agree with the hook"
  exit 0
fi
echo "FAIL: $failed of $total load-log case(s)"
exit 1
"##;

/// Quote for a single-quoted shell word.
fn single_quoted(s: &str) -> String {
    format!("'{}'", s.replace('\'', r#"'\''"#))
}

fn runner() -> String {
    let mut out = String::from(RUNNER_HEAD);
    for case in skill_cases() {
        let (name, location, hash) = match case.name {
            None => ("none".to_owned(), "-".to_owned(), "-".to_owned()),
            Some(n) => (
                n.to_owned(),
                case.location.unwrap_or("-").to_owned(),
                case.hash.map_or("-".to_owned(), String::from),
            ),
        };
        out.push_str(&format!(
            "check {} \"$(skill {})\" 0 {} {} {}\n",
            single_quoted(case.what),
            single_quoted(case.tool_input),
            single_quoted(&name),
            location,
            hash
        ));
    }
    out.push_str(RUNNER_TAIL);
    out
}

fn snippet() -> String {
    format!(
        "# Registering the load log\n\n\
         Copy `{HOOK}.sh` into `~/.claude/hooks/relearn/`, then merge the entries below into \
         the `hooks` table of `~/.claude/settings.json`. relearn installs nothing: once \
         registered, this hook writes a line on every skill and instruction-file load on the \
         machine, to `$RELEARN_LOAD_LOG` or `~/.local/state/relearn/load-log.jsonl`.\n\n\
         `InstructionsLoaded` takes no matcher, so it sees every `CLAUDE.md` and rules file; \
         `relearn loads` keeps only relearn's own homes.\n\n\
         ```json\n\
         {{\n  \
           \"hooks\": {{\n    \
             \"PostToolUse\": [\n      \
               {{\n        \
                 \"matcher\": \"Skill\",\n        \
                 \"hooks\": [\n          \
                   {{ \"type\": \"command\", \"command\": \"bash ~/.claude/hooks/relearn/{HOOK}.sh\" }}\n        \
                 ]\n      \
               }}\n    \
             ],\n    \
             \"InstructionsLoaded\": [\n      \
               {{\n        \
                 \"hooks\": [\n          \
                   {{ \"type\": \"command\", \"command\": \"bash ~/.claude/hooks/relearn/{HOOK}.sh\" }}\n        \
                 ]\n      \
               }}\n    \
             ]\n  \
           }}\n\
         }}\n\
         ```\n"
    )
}
