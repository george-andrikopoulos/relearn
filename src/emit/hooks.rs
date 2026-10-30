//! `emit::hooks` — every rule's `[[detector]]` tables, compiled to Claude Code
//! hook scripts.
//!
//! Writes, under `<out>/hooks/`:
//!
//! * `<name>.sh` per detector — bash with `jq`, the runtime every stochos-lab
//!   hook already uses. Exit 2 with the reason on **stderr** blocks; exit 0
//!   allows, and a `warn` puts its text in `hookSpecificOutput.additionalContext`
//!   on stdout; any other exit is the script itself failing, which Claude Code
//!   reports and does not block on. That split is from the current hooks
//!   reference, checked 2026-10-30, and is the reason no failure path here may
//!   exit 2.
//! * `test-detectors.sh` — every fixture of every detector, piped through the
//!   generated scripts. `relearn check` already ran the fixtures in Rust; this is
//!   the same claim on the production path (`[R:verify-through-production-path]`),
//!   because a pattern meaning one thing to the `regex` crate and another to
//!   `grep -E` is the defect this target exists to rule out.
//! * `settings-snippet.md` — the registration, as JSON inside markdown, because a
//!   JSON file cannot carry the generated-by marker and so could be neither
//!   guarded nor verified.
//!
//! **Explicit only.** Not in `build`'s default targets, so bare `build` and
//! `verify` are unchanged, and the first defect of a shell emitter cannot block
//! an unrelated push. **Never installed:** the pack is copied by a person
//! (`docs/session-start-poke.md` is the precedent).
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::{OutputFile, RelativePath, emittable};
use crate::library::{Library, Validated};
use crate::rule::{
    Action, Detection, Detector, Engine, Event, FixtureInput, Rule, RuleTag, Subject,
};

/// The directory this emitter owns, relative to the output root.
pub const HOOKS_DIR: &str = "hooks";

/// Where the committed pack lives in this repository.
pub const PACK_DIR: &str = "hooks-pack";

/// Where the pack's README tells a person to install the scripts, relative to
/// the Claude Code configuration directory. Beside the hand-written hooks, never
/// over them, so a machine mid-migration can hold both and register one.
pub const INSTALL_DIR: &str = "hooks/relearn";

/// The fixture runner's file stem, which no detector may take.
pub const RUNNER: &str = "test-detectors";

/// Emit the hook pack. A library with no detector produces no file.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    // By name: the name is the file, so this is the order a reader lists them.
    let mut detectors: BTreeMap<&str, (&Rule, &Detector)> = BTreeMap::new();
    for rule in emittable(library) {
        for detector in rule.detectors() {
            detectors.insert(detector.name().as_str(), (rule, detector));
        }
    }
    if detectors.is_empty() {
        return Vec::new();
    }

    let mut files = Vec::with_capacity(detectors.len() + 2);
    for (name, (rule, detector)) in &detectors {
        let file = format!("{name}.sh");
        files.push(OutputFile::script(
            RelativePath::from_segments(&[HOOKS_DIR, &file]),
            script(rule.tag(), detector),
            vec![rule.tag().clone()], // allow:clone: the OutputFile owns its provenance, outliving the &Library borrow
        ));
    }

    let mut sources: Vec<RuleTag> = detectors.values().map(|(r, _)| r.tag().clone()).collect(); // allow:clone: same
    sources.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    sources.dedup();
    let runner = format!("{RUNNER}.sh");
    files.push(OutputFile::script(
        RelativePath::from_segments(&[HOOKS_DIR, &runner]),
        test_runner(&detectors),
        sources.clone(), // allow:clone: two generated files share one provenance list
    ));
    files.push(OutputFile::new(
        RelativePath::from_segments(&[HOOKS_DIR, "settings-snippet.md"]),
        settings_snippet(&detectors),
        sources,
    ));
    files
}

/// A detector that would fire on a file relearn itself reads or writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelfMatch {
    detector: String,
    file: String,
}

impl SelfMatch {
    /// The detector that fired.
    #[must_use]
    pub fn detector(&self) -> &str {
        &self.detector
    }

    /// The file it fired on, relative to where it lives.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }
}

/// Every detector, run as a `Write` of each given file
/// (`[R:detector-excludes-own-definitions]`).
///
/// `files` is every rule file at `rules/<name>.md` and every file every target
/// emits, at the path it is emitted to. A rule file holds a pattern and the
/// inputs that must hit it, and an emitted instruction layer quotes what a rule
/// forbids; a detector that fires on either has a scope wide enough to read its
/// own definitions, and would block the edit that maintains it. The implicit
/// excludes are meant to make this empty; this is what shows they do, for the
/// corpus as it is rather than as the excludes assumed it would be.
///
/// Siblings for the case engine are the other given files in the same
/// directory, so a case collision **among** relearn's own files is a hit too.
#[must_use]
pub fn self_matches(library: &Library<Validated>, files: &[(String, &str)]) -> Vec<SelfMatch> {
    let dir_of = |p: &str| p.rsplit_once('/').map_or("", |(d, _)| d).to_owned();
    let mut found = Vec::new();
    for rule in library.rules() {
        for detector in rule.detectors() {
            for (path, text) in files {
                let dir = dir_of(path);
                let siblings: Vec<String> = files
                    .iter()
                    .filter(|(other, _)| other != path && dir_of(other) == dir)
                    .map(|(other, _)| other.rsplit('/').next().unwrap_or(other).to_owned())
                    .collect();
                let subject = Subject::new(path, text, &siblings);
                match detector.evaluate(&subject) {
                    Detection::Hit => found.push(SelfMatch {
                        detector: detector.name().as_str().to_owned(),
                        file: path.clone(), // allow:clone: the finding owns the path it names, outliving the borrowed list
                    }),
                    Detection::Clean => {}
                }
            }
        }
    }
    found
}

/// `'…'` with every `'` closed, escaped and reopened: the one quoting a shell
/// never interprets, so a pattern or message reaches `grep` byte for byte.
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Escape a literal prefix for use inside an ERE.
fn ere_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if ".[]()*+?{}|^$\\".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// The Claude Code tools an engine reads. Creating a file is only ever a
/// `Write`; changing text is any of the three.
fn tools(engine: &Engine) -> &'static str {
    match engine {
        Engine::LineRegex { .. } => "Write|Edit|MultiEdit",
        Engine::SiblingCase => "Write",
    }
}

/// The prologue every script shares: identity, the dependency check, the tool
/// filter, the path, and the scope.
fn prologue(tag: &RuleTag, d: &Detector) -> String {
    let mut s = String::new();
    let events: Vec<&str> = d.events().iter().map(|e| event_label(*e)).collect();
    let _ = writeln!(s, "#!/usr/bin/env bash");
    let _ = writeln!(
        s,
        "# Detector `{name}` for [{tag}], generated by relearn from the rule file.\n\
         # Edit the rule, never this script: `relearn verify` reports a hand edit.\n\
         # Engine: {engine}. Event: {events}, on {tools}.\n\
         # Exit 2 blocks, and stderr is the reason Claude reads. Exit 0 allows.\n\
         # Any other exit is this script failing: reported, never a block.",
        name = d.name().as_str(),
        tag = tag.as_str(),
        engine = d.engine().as_str(),
        events = events.join(", "),
        tools = tools(d.engine()),
    );
    let _ = writeln!(s, "set -uo pipefail");
    let _ = writeln!(s, "export LC_ALL=C");
    let _ = writeln!(s);
    let _ = writeln!(s, "name={}", sh_quote(d.name().as_str()));
    let _ = writeln!(s, "tag={}", sh_quote(tag.as_str()));
    let _ = writeln!(s, "message={}", sh_quote(d.message()));
    let _ = writeln!(s, "files_ere={}", sh_quote(&d.scope().files_ere()));
    if let Some(exclude) = d.scope().exclude_ere() {
        let _ = writeln!(s, "exclude_ere={}", sh_quote(&exclude));
    }
    s.push_str(
        r#"
fail() {
  printf '[%s] %s; the detector did not run.\n' "$name" "$1" >&2
  exit 1
}
command -v jq >/dev/null 2>&1 || fail "jq is not on PATH"

input="$(cat)"
tool_name="$(jq -r '.tool_name // ""' <<<"$input")" || fail "the payload is not JSON"
"#,
    );
    let _ = writeln!(
        s,
        "case \"$tool_name\" in {}) ;; *) exit 0 ;; esac",
        tools(d.engine())
    );
    s.push_str(
        r#"file_path="$(jq -r '.tool_input.file_path // ""' <<<"$input")" || fail "the payload has no readable file_path"
file_path="${file_path//\\//}"
[ -n "$file_path" ] || exit 0

# Scope: one ERE per list, compiled by relearn from the rule's globs.
printf '%s\n' "$file_path" | grep -qE -- "$files_ere"
case $? in 0) ;; 1) exit 0 ;; *) fail "grep could not read the files pattern" ;; esac
"#,
    );
    if d.scope().exclude_ere().is_some() {
        s.push_str(
            r#"printf '%s\n' "$file_path" | grep -qE -- "$exclude_ere"
case $? in 0) exit 0 ;; 1) ;; *) fail "grep could not read the exclude pattern" ;; esac
"#,
        );
    }
    s
}

fn event_label(event: Event) -> &'static str {
    match event {
        Event::ClaudeCodePreToolUse => "Claude Code PreToolUse",
    }
}

/// The verdict: how a hit leaves the script.
fn verdict(d: &Detector, report: &str) -> String {
    match d.action() {
        Action::Block => format!("{{\n{report}}} >&2\nexit 2\n"),
        Action::Warn => format!(
            "context=\"$(\n{report})\"\n\
             jq -n --arg c \"$context\" \
             '{{hookSpecificOutput:{{hookEventName:\"PreToolUse\",additionalContext:$c}}}}' \
             || fail \"jq could not build the warning\"\n\
             exit 0\n"
        ),
    }
}

fn verdict_word(action: Action) -> &'static str {
    match action {
        Action::Block => "BLOCKED",
        Action::Warn => "WARNING",
    }
}

/// One detector's script.
fn script(tag: &RuleTag, d: &Detector) -> String {
    let mut s = prologue(tag, d);
    let word = verdict_word(d.action());
    match d.engine() {
        Engine::LineRegex {
            pattern,
            comment,
            allow,
        } => {
            let _ = writeln!(s, "\npattern={}", sh_quote(pattern.as_str()));
            s.push_str(
                r#"case "$tool_name" in
  Write) new="$(jq -r '.tool_input.content // ""' <<<"$input")" ;;
  Edit) new="$(jq -r '.tool_input.new_string // ""' <<<"$input")" ;;
  MultiEdit) new="$(jq -r '[.tool_input.edits[]?.new_string] | join("\n")' <<<"$input")" ;;
esac || fail "the payload's text could not be read"

hits="$(printf '%s\n' "$new" | grep -nE -- "$pattern")"
case $? in 0) ;; 1) exit 0 ;; *) fail "grep could not read the pattern" ;; esac
"#,
            );
            if let Some(comment) = comment {
                let _ = writeln!(
                    s,
                    "# A line whose first non-blank text is {} is a comment, not code.\n\
                     hits=\"$(printf '%s\\n' \"$hits\" | grep -vE -- {})\"",
                    sh_quote(comment.as_str()),
                    sh_quote(&format!(
                        "^[0-9]+:[[:space:]]*{}",
                        ere_literal(comment.as_str())
                    ))
                );
            }
            if let Some(allow) = allow {
                let _ = writeln!(
                    s,
                    "# A line carrying {0} is a deliberate, marked exception.\n\
                     hits=\"$(printf '%s\\n' \"$hits\" | grep -vF -- {0})\"",
                    sh_quote(allow.as_str())
                );
            }
            s.push_str("[ -n \"$hits\" ] || exit 0\n\n");
            let mut report = format!(
                "  printf '[%s] %s: {word}. %s\\n' \"$tag\" \"$name\" \"$message\"\n\
                 \x20 printf 'Offending line(s) in %s:\\n%s\\n' \"$file_path\" \"$hits\"\n"
            );
            if let Some(allow) = allow {
                let _ = writeln!(
                    report,
                    "  printf 'A deliberate exception is marked on its own line with: %s\\n' {}",
                    sh_quote(allow.as_str())
                );
            }
            s.push_str(&verdict(d, &report));
        }
        Engine::SiblingCase => {
            s.push_str(
                r#"
dir="$(dirname -- "$file_path")"
base="$(basename -- "$file_path")"
[ -d "$dir" ] || exit 0
lower() { printf '%s' "$1" | tr '[:upper:]' '[:lower:]'; }
base_lc="$(lower "$base")"
collision=""
for entry in "$dir"/* "$dir"/.[!.]* "$dir"/..?*; do
  [ -e "$entry" ] || continue
  sibling="${entry##*/}"
  # The exact name already exists: this write replaces that file, as intended.
  [ "$sibling" = "$base" ] && exit 0
  if [ -z "$collision" ] && [ "$(lower "$sibling")" = "$base_lc" ]; then
    collision="$sibling"
  fi
done
[ -n "$collision" ] || exit 0

"#,
            );
            let report = format!(
                "  printf '[%s] %s: {word}. %s differs only by case from the existing %s in %s. %s\\n' \
                 \"$tag\" \"$name\" \"$base\" \"$collision\" \"$dir\" \"$message\"\n"
            );
            s.push_str(&verdict(d, &report));
        }
    }
    s
}

/// The fixture runner: every fixture of every detector, through the scripts.
fn test_runner(detectors: &BTreeMap<&str, (&Rule, &Detector)>) -> String {
    let mut s = String::from(
        r#"#!/usr/bin/env bash
# Every detector fixture, piped through the generated scripts beside this file.
# `relearn check` ran the same fixtures in Rust; this runs them on the
# production path, because the hook is `grep -E` and the check is not.
# Generated by relearn; edit the rule files, never this script.
set -uo pipefail
export LC_ALL=C MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*'

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if ! command -v jq >/dev/null 2>&1; then
  echo "DISARMED (exit 2, not a pass): jq is not on PATH, so no fixture ran"
  exit 2
fi
work="$(mktemp -d)" || { echo "FAIL: no scratch directory"; exit 1; }
trap 'rm -rf "$work"' EXIT
pass=0
failed=0

# run <script> <expect hit|clean> <action block|warn> <path> <text> [sibling...]
run() {
  local script="$1" expect="$2" action="$3" path="$4" text="$5"
  shift 5
  local case_dir="$work/$((pass + failed))"
  local target="$case_dir/$path"
  mkdir -p "$(dirname "$target")"
  local sibling
  for sibling in "$@"; do : > "$(dirname "$target")/$sibling"; done
  local out got
  out="$(jq -nc --arg p "$target" --arg c "$text" \
    '{tool_name:"Write",tool_input:{file_path:$p,content:$c}}' \
    | bash "$here/$script" 2>"$work/stderr")"
  got=$?
  local err
  err="$(cat "$work/stderr")"
  local ok=no
  case "$expect:$action" in
    clean:*) [ "$got" -eq 0 ] && [ -z "$out" ] && [ -z "$err" ] && ok=yes ;;
    hit:block) [ "$got" -eq 2 ] && [ -n "$err" ] && ok=yes ;;
    hit:warn) [ "$got" -eq 0 ] && case "$out" in *additionalContext*) true ;; *) false ;; esac && ok=yes ;;
  esac
  if [ "$ok" = yes ]; then
    pass=$((pass + 1))
  else
    failed=$((failed + 1))
    printf 'FAIL  %s  %s expected %s (%s): exit %s, stdout %s bytes, stderr %s bytes\n' \
      "$script" "$path" "$expect" "$action" "$got" "${#out}" "${#err}"
  fi
}

"#,
    );
    for (name, (_, detector)) in detectors {
        let _ = writeln!(s, "# {name}");
        for fixture in detector.fixtures() {
            let expect = match fixture.expect() {
                Detection::Hit => "hit",
                Detection::Clean => "clean",
            };
            let (text, siblings) = match fixture.input() {
                FixtureInput::Text(text) => (text.as_str(), Vec::new()),
                FixtureInput::Siblings(names) => ("", names.iter().map(|n| sh_quote(n)).collect()),
            };
            let _ = write!(
                s,
                "run {} {expect} {} {} {}",
                sh_quote(&format!("{name}.sh")),
                detector.action().as_str(),
                sh_quote(fixture.path().as_str()),
                sh_quote(text),
            );
            for sibling in siblings {
                let _ = write!(s, " {sibling}");
            }
            s.push('\n');
        }
    }
    s.push_str(
        r#"
if [ "$failed" -eq 0 ]; then
  echo "PASS: $pass detector fixture(s) agree with the generated scripts"
  exit 0
fi
echo "FAIL: $failed of $((pass + failed)) detector fixture(s) disagree with the generated scripts"
exit 1
"#,
    );
    s
}

/// The registration, grouped by matcher, inside a markdown file.
fn settings_snippet(detectors: &BTreeMap<&str, (&Rule, &Detector)>) -> String {
    let mut by_matcher: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (name, (_, detector)) in detectors {
        by_matcher
            .entry(tools(detector.engine()))
            .or_default()
            .push(name);
    }
    let mut groups = Vec::new();
    for (matcher, names) in &by_matcher {
        let commands: Vec<String> = names
            .iter()
            .map(|n| {
                format!(
                    "          {{ \"type\": \"command\", \"command\": \"bash ~/.claude/{INSTALL_DIR}/{n}.sh\" }}"
                )
            })
            .collect();
        groups.push(format!(
            "      {{\n        \"matcher\": \"{matcher}\",\n        \"hooks\": [\n{}\n        ]\n      }}",
            commands.join(",\n")
        ));
    }
    format!(
        "# Registering the generated hooks\n\n\
         Copy every `*.sh` in this directory except `{RUNNER}.sh` into \
         `~/.claude/{INSTALL_DIR}/`, then merge the entries below into the `hooks` \
         table of `~/.claude/settings.json`. relearn installs nothing: these files \
         edit the layer every session on the machine loads, and no gate in this \
         repository can see that layer.\n\n\
         Where a hand-written hook with the same name is registered today, replace \
         its entry with the one below rather than adding a second: one hook per \
         detector, so a block is never reported twice and a retired one is never \
         left running.\n\n\
         ```json\n{{\n  \"hooks\": {{\n    \"PreToolUse\": [\n{}\n    ]\n  }}\n}}\n```\n",
        groups.join(",\n")
    )
}
