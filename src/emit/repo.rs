//! `emit::repo` — every rule's repository gates, compiled to bash scripts a
//! consuming repository's own gate calls.
//!
//! Writes, under `<out>/gates/`:
//!
//! * `<name>.sh` per gate. `bash <name>.sh <repository root>` runs the
//!   **collect** stage (`git ls-files`, `git check-attr`, the files themselves)
//!   into records and the **judge** stage over them, and exits 0 clean, 1 on a
//!   finding for a `block` gate (a `warn` gate prints and exits 0), and 2 when it
//!   could not check: not a checkout, `git` missing, a record the judge cannot
//!   read. `bash <name>.sh --judge` reads records on stdin and runs the judge
//!   alone, exiting 0 or 1: the fixture runner's door onto the same judge.
//! * `test-gates.sh` — every gate fixture's records through `--judge`, and
//!   with `--trees <dir>`, every gate run whole over a synthetic checkout built
//!   from `<dir>/<name>/hit` and `<dir>/<name>/clean`, so the collect stage is
//!   exercised on the production path (`[R:verify-through-production-path]`).
//!
//! * `relearn-pre-commit.sh` — every detector whose events include
//!   `git-pre-commit`, as one dispatcher a repository's own `pre-commit` calls
//!   (`bash relearn-pre-commit.sh [name ...]`). It reads the **index**:
//!   `git diff --cached` for the staged paths and `git cat-file blob :0:<path>`
//!   for staged content, so a file fixed on disk and not re-staged still
//!   blocks. It exits 0 clean, 1 when a `block` detector hit, and 2 when it
//!   could not check, including a `listed-path` detector with no readable list.
//!   The runner builds a repository per fixture, stages it, and runs the
//!   dispatcher on it.
//!
//! The judge is `awk` reading a tab-separated record per line; `rule::Gate`
//! holds the same judge in Rust, which is what runs the fixtures at parse.
//! Patterns reach `awk` through the environment, never `-v`, because `-v`
//! processes backslash escapes and would turn the glob ERE's `\.` into `.`.
//!
//! **Explicit only**, for the reason `hooks` is. **Never installed:** a person
//! copies the pack into a repository and calls the scripts from its own gate.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::hooks::{HookRole, SECTIONS_JUDGE, ere_literal};
use super::{OutputFile, RelativePath, emittable};
use crate::library::{Library, Validated};
use crate::rule::{
    Action, Detection, Detector, Engine, Event, FixtureInput, Gate, GateEngine, Glob, HeadingLevel,
    Pattern, Rule, RuleTag, Token,
};

/// The directory this emitter owns, relative to the output root.
pub const GATES_DIR: &str = "gates";

/// Where the committed pack lives in this repository.
pub const PACK_DIR: &str = "repo-pack";

/// The fixture runner's file stem, which no detector may take.
pub const RUNNER: &str = "test-gates";

/// The commit dispatcher's file stem, which no detector may take.
pub const DISPATCHER: &str = "relearn-pre-commit";

/// What the dispatcher can run over the index: staged content for
/// `line-regex` and `sections`, a staged path for `listed-path`. `sibling-case` lists a
/// directory, which a commit does not change, so a commit function for one is
/// unconstructible rather than emitted empty.
enum CommitEngine<'a> {
    LineRegex {
        pattern: &'a Pattern,
        comment: Option<&'a Token>,
        allow: Option<&'a Token>,
    },
    ListedPath,
    Sections {
        heading: HeadingLevel,
        require: &'a Pattern,
    },
}

impl<'a> CommitEngine<'a> {
    fn of(d: &'a Detector) -> Option<Self> {
        if !d.runs_at(Event::GitPreCommit) {
            return None;
        }
        match d.engine() {
            Engine::LineRegex {
                pattern,
                comment,
                allow,
            } => Some(CommitEngine::LineRegex {
                pattern,
                comment: comment.as_ref(),
                allow: allow.as_ref(),
            }),
            Engine::ListedPath => Some(CommitEngine::ListedPath),
            Engine::Sections { heading, require } => Some(CommitEngine::Sections {
                heading: *heading,
                require,
            }),
            Engine::SiblingCase => None,
        }
    }
}

/// One commit-time detector's inputs: the rule, the detector, what it runs.
type Committed<'a> = (&'a Rule, &'a Detector, CommitEngine<'a>);

/// Emit the repo pack. A library with no gate and no commit-time detector
/// produces no file.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    emit_with_roles(library)
        .into_iter()
        .map(|(file, _)| file)
        .collect()
}

/// [`emit`], with each file's role for the self-exclusion sweep. A gate's
/// script holds only its own definition; the dispatcher holds every
/// commit-time detector's; the runner holds every fixture.
#[must_use]
pub fn emit_with_roles(library: &Library<Validated>) -> Vec<(OutputFile, HookRole)> {
    let mut gates: BTreeMap<&str, (&Rule, &Gate)> = BTreeMap::new();
    let mut committed: BTreeMap<&str, Committed<'_>> = BTreeMap::new();
    for rule in emittable(library) {
        for gate in rule.gates() {
            gates.insert(gate.name().as_str(), (rule, gate));
        }
        for detector in rule.detectors() {
            if let Some(engine) = CommitEngine::of(detector) {
                committed.insert(detector.name().as_str(), (rule, detector, engine));
            }
        }
    }
    if gates.is_empty() && committed.is_empty() {
        return Vec::new();
    }
    let mut files = Vec::with_capacity(gates.len() + 2);
    let mut sources: Vec<RuleTag> = Vec::new();
    if !committed.is_empty() {
        let mut dispatched: Vec<RuleTag> = Vec::new();
        for (rule, _, _) in committed.values() {
            if !dispatched.contains(rule.tag()) {
                dispatched.push(rule.tag().clone()); // allow:clone: the OutputFile owns its provenance, outliving the &Library borrow
            }
            if !sources.contains(rule.tag()) {
                sources.push(rule.tag().clone()); // allow:clone: as above
            }
        }
        dispatched.sort();
        files.push((
            OutputFile::script(
                RelativePath::from_segments(&[GATES_DIR, &format!("{DISPATCHER}.sh")]),
                dispatcher(&committed),
                dispatched,
            ),
            HookRole::Dispatcher,
        ));
    }
    for (name, (rule, gate)) in &gates {
        files.push((
            OutputFile::script(
                RelativePath::from_segments(&[GATES_DIR, &format!("{name}.sh")]),
                script(rule.tag(), gate),
                vec![rule.tag().clone()], // allow:clone: the OutputFile owns its provenance, outliving the &Library borrow
            ),
            HookRole::Script(gate.name().clone()), // allow:clone: the role names its gate beyond the &Library borrow
        ));
        if !sources.contains(rule.tag()) {
            sources.push(rule.tag().clone()); // allow:clone: as above
        }
    }
    sources.sort();
    files.push((
        OutputFile::script(
            RelativePath::from_segments(&[GATES_DIR, &format!("{RUNNER}.sh")]),
            test_runner(&gates, &committed),
            sources,
        ),
        HookRole::Runner,
    ));
    files
}

/// The shell function a detector's name becomes: `d_` and the name with `-`
/// as `_`. Names are `[a-z0-9-]`, so two names never share a function.
fn function_name(name: &str) -> String {
    format!("d_{}", name.replace('-', "_"))
}

/// The commit dispatcher: one function per commit-time detector, run in name
/// order, or only those named on the command line.
fn dispatcher(committed: &BTreeMap<&str, Committed<'_>>) -> String {
    let mut s = String::from(
        r#"#!/usr/bin/env bash
# The commit-time detectors, generated by relearn from the rule files.
# Edit the rules, never this script: `relearn verify` reports a hand edit.
# A repository's own pre-commit calls it; relearn never installs it:
#   bash relearn-pre-commit.sh [name ...]   0 clean, 1 blocked, 2 could not check
# With no name, every detector below runs. It reads the index, never the
# working tree, so a file fixed on disk and not re-staged still blocks.
# A `listed-path` detector reads its list from the file this clone names with
#   git config --local relearn.<name>.list <file>
# and with no list, or one it cannot read, it cannot check: exit 2, never 0.
set -uo pipefail
export LC_ALL=C

disarmed() {
  printf '[relearn-pre-commit] DISARMED (exit 2, not a pass): %s\n' "$1" >&2
  exit 2
}
command -v git >/dev/null 2>&1 || disarmed "git is not on PATH"
top="$(git rev-parse --show-toplevel 2>/dev/null)" || disarmed "this is not a git checkout"
cd "$top" || disarmed "the checkout's top level cannot be entered"
work="$(mktemp -d)" || disarmed "no scratch directory"
trap 'rm -rf "$work"' EXIT
git diff --cached --name-only -z --diff-filter=ACMR > "$work/staged" \
  || disarmed "git could not list the staged paths"
blocked=0

# in_scope <path> <files ERE> <exclude ERE, or empty>: 0 in scope, 1 not.
in_scope() {
  printf '%s\n' "$1" | grep -qE -- "$2"
  case $? in 0) ;; 1) return 1 ;; *) disarmed "grep could not read a files pattern" ;; esac
  [ -n "$3" ] || return 0
  printf '%s\n' "$1" | grep -qE -- "$3"
  case $? in 0) return 1 ;; 1) return 0 ;; *) disarmed "grep could not read an exclude pattern" ;; esac
}

# verdict <action> <tag> <name> <message> <heading> <findings>
verdict() {
  local word=WARNING
  if [ "$1" = block ]; then word=BLOCKED; blocked=1; fi
  printf '[%s] %s: %s. %s\n%s\n%s' "$2" "$3" "$word" "$4" "$5" "$6" >&2
}
"#,
    );
    if committed
        .values()
        .any(|(_, _, e)| matches!(e, CommitEngine::Sections { .. }))
    {
        s.push('\n');
        s.push_str(SECTIONS_JUDGE);
    }
    for (name, (rule, detector, engine)) in committed {
        s.push('\n');
        s.push_str(&commit_function(name, rule.tag(), detector, engine));
    }
    let names: Vec<String> = committed.keys().map(|n| sh_quote(n)).collect();
    let _ = writeln!(
        s,
        "\n[ \"$#\" -gt 0 ] || set -- {}\nfor selected in \"$@\"; do\n  case \"$selected\" in",
        names.join(" ")
    );
    for name in committed.keys() {
        let _ = writeln!(s, "    {}) {} ;;", sh_quote(name), function_name(name));
    }
    s.push_str(
        "    *) disarmed \"no commit-time detector is named $selected\" ;;\n  esac\ndone\nexit \"$blocked\"\n",
    );
    s
}

/// One commit-time detector, as a shell function over `$work/staged`.
fn commit_function(name: &str, tag: &RuleTag, d: &Detector, engine: &CommitEngine<'_>) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "# {name}: {engine} for [{tag}].",
        engine = d.engine().as_str(),
        tag = tag.as_str()
    );
    let _ = writeln!(s, "{}() {{", function_name(name));
    let _ = writeln!(s, "  local name={}", sh_quote(name));
    let _ = writeln!(s, "  local tag={}", sh_quote(tag.as_str()));
    let _ = writeln!(s, "  local action={}", sh_quote(d.action().as_str()));
    let _ = writeln!(s, "  local message={}", sh_quote(d.message()));
    let _ = writeln!(s, "  local files_ere={}", sh_quote(&d.scope().files_ere()));
    let _ = writeln!(
        s,
        "  local exclude_ere={}",
        sh_quote(&d.scope().exclude_ere().unwrap_or_default())
    );
    s.push_str("  local p hits found=\"\"\n");
    match engine {
        CommitEngine::LineRegex {
            pattern,
            comment,
            allow,
        } => {
            let _ = writeln!(s, "  local pattern={}", sh_quote(pattern.as_str()));
            s.push_str(
                r#"  while IFS= read -r -d '' p; do
    in_scope "$p" "$files_ere" "$exclude_ere" || continue
    case "$(git cat-file -t ":0:$p" 2>/dev/null)" in
      blob) ;;
      commit) continue ;;
      *) disarmed "$name: git could not read the staged $p" ;;
    esac
    git cat-file blob ":0:$p" > "$work/blob" || disarmed "$name: git could not read the staged $p"
    hits="$(grep -anE -- "$pattern" "$work/blob")"
    case $? in 0) ;; 1) continue ;; *) disarmed "$name: grep could not read the pattern" ;; esac
"#,
            );
            if let Some(comment) = comment {
                let _ = writeln!(
                    s,
                    "    # A line whose first non-blank text is {} is a comment, not code.\n\
                     \x20   hits=\"$(printf '%s\\n' \"$hits\" | grep -vE -- {})\"",
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
                    "    # A line carrying {0} is a deliberate, marked exception.\n\
                     \x20   hits=\"$(printf '%s\\n' \"$hits\" | grep -vF -- {0})\"",
                    sh_quote(allow.as_str())
                );
            }
            s.push_str(
                r#"    [ -n "$hits" ] || continue
    found="$found$p:"$'\n'"$(printf '%s\n' "$hits" | sed 's/^/  /')"$'\n'
  done < "$work/staged"
  [ -n "$found" ] || return 0
  verdict "$action" "$tag" "$name" "$message" "Offending staged line(s):" "$found"
"#,
            );
            if let Some(allow) = allow {
                let _ = writeln!(
                    s,
                    "  printf 'A deliberate exception is marked on its own line with: %s\\n' {} >&2",
                    sh_quote(allow.as_str())
                );
            }
        }
        CommitEngine::ListedPath => {
            s.push_str(
                r#"  local list
  list="$(git config --local --type=path --get "relearn.$name.list")" \
    || disarmed "$name has no list: name one with git config --local relearn.$name.list <file>"
  [ -f "$list" ] && [ -r "$list" ] || disarmed "$name: the list this clone names cannot be read"
  # One relative forward-slash path per line; blank lines and lines starting
  # with # are skipped. Any other line that is not such a path disarms the
  # check, and only its line number is printed: the list names what it guards.
  awk '
{ sub(/\r$/, "") }
/^#/ || $0 == "" { next }
{
  ok = ($0 !~ /^\//) && index($0, "\\") == 0 && index($0, ":") == 0
  n = split($0, seg, "/")
  for (i = 1; i <= n; i++) if (seg[i] == "" || seg[i] == "..") ok = 0
  if (ok) print; else bad = bad " " NR
}
END { if (bad != "") { print "line(s)" bad > "/dev/stderr"; exit 3 } }
' "$list" > "$work/list" 2> "$work/list.err" \
    || disarmed "$name: the list has an entry that is not a relative path, $(cat "$work/list.err")"
  while IFS= read -r -d '' p; do
    in_scope "$p" "$files_ere" "$exclude_ere" || continue
    grep -qxF -- "$p" "$work/list"
    case $? in 0) found="$found  $p"$'\n' ;; 1) ;; *) disarmed "$name: grep could not read the list" ;; esac
  done < "$work/staged"
  [ -n "$found" ] || return 0
  verdict "$action" "$tag" "$name" "$message" "Staged path(s) on this clone's list:" "$found"
"#,
            );
        }
        CommitEngine::Sections { heading, require } => {
            let _ = writeln!(s, "  local require={}", sh_quote(require.as_str()));
            let _ = writeln!(s, "  local level={}", heading.get());
            s.push_str(
                r#"  while IFS= read -r -d '' p; do
    in_scope "$p" "$files_ere" "$exclude_ere" || continue
    case "$(git cat-file -t ":0:$p" 2>/dev/null)" in
      blob) ;;
      commit) continue ;;
      *) disarmed "$name: git could not read the staged $p" ;;
    esac
    git cat-file blob ":0:$p" > "$work/blob" || disarmed "$name: git could not read the staged $p"
    hits="$(sections_judge "$work/blob" "$require" "$level")"
    case $? in 0) continue ;; 1) ;; *) disarmed "$name: the sections judge could not read the staged $p" ;; esac
    found="$found$p:"$'\n'"$(printf '%s\n' "$hits" | sed 's/^/  /')"$'\n'
  done < "$work/staged"
  [ -n "$found" ] || return 0
  verdict "$action" "$tag" "$name" "$message" "Staged section(s) with no line matching the requirement:" "$found"
"#,
            );
        }
    }
    s.push_str("}\n");
    s
}

/// Single-quote for bash.
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// ANSI-C quote for bash, so a record's tabs are visible as `\t` in the file.
fn ansi_quote(s: &str) -> String {
    let mut out = String::from("$'");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// One ERE for a list of globs; empty when the list is.
fn ere(globs: &[Glob]) -> String {
    globs
        .iter()
        .map(|g| format!("({})", g.to_ere()))
        .collect::<Vec<_>>()
        .join("|")
}

/// One gate's script.
fn script(tag: &RuleTag, g: &Gate) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "#!/usr/bin/env bash");
    let _ = writeln!(
        s,
        "# Gate `{name}` for [{tag}], generated by relearn from the rule file.\n\
         # Edit the rule, never this script: `relearn verify` reports a hand edit.\n\
         # Engine: {engine}. Records: {fields}, tab-separated.\n\
         #   bash {name}.sh <repository root>   0 clean, 1 finding, 2 could not check\n\
         #   bash {name}.sh --judge < records   0 clean, 1 hit (the fixture runner)\n\
         # A `warn` gate prints its finding and exits 0.",
        name = g.name().as_str(),
        tag = tag.as_str(),
        engine = g.engine().as_str(),
        fields = g.engine().fields().join(", "),
    );
    let _ = writeln!(s, "set -uo pipefail");
    let _ = writeln!(s, "export LC_ALL=C");
    let _ = writeln!(s);
    let _ = writeln!(s, "name={}", sh_quote(g.name().as_str()));
    let _ = writeln!(s, "tag={}", sh_quote(tag.as_str()));
    let _ = writeln!(s, "message={}", sh_quote(g.message()));
    let _ = writeln!(s, "export RL_FILES={}", sh_quote(&g.scope().files_ere()));
    let _ = writeln!(
        s,
        "export RL_EXCLUDE={}",
        sh_quote(&g.scope().exclude_ere().unwrap_or_default())
    );
    match g.engine() {
        GateEngine::EolAttribute { always } => {
            let _ = writeln!(s, "export RL_ALWAYS={}", sh_quote(&ere(always)));
        }
        GateEngine::RootDocuments => {}
        GateEngine::LintEnabled {
            lint,
            groups,
            default,
        } => {
            let _ = writeln!(s, "export RL_LINT={}", sh_quote(lint.as_str()));
            let groups: Vec<&str> = groups.iter().map(|l| l.as_str()).collect();
            let _ = writeln!(s, "export RL_GROUPS={}", sh_quote(&groups.join(" ")));
            let _ = writeln!(s, "export RL_DEFAULT={}", sh_quote(default.as_str()));
        }
    }
    s.push('\n');
    s.push_str(&judge(g.engine()));
    s.push('\n');
    s.push_str(collect(g.engine()));
    s.push_str(
        r#"
if [ "${1:-}" = "--judge" ]; then
  judge
  exit $?
fi

disarmed() {
  printf '[%s] %s: DISARMED (exit 2, not a pass): %s\n' "$tag" "$name" "$1" >&2
  exit 2
}
root="${1:-.}"
command -v git >/dev/null 2>&1 || disarmed "git is not on PATH"
git -C "$root" rev-parse --is-inside-work-tree >/dev/null 2>&1 \
  || disarmed "$root is not a git checkout"
work="$(mktemp -d)" || disarmed "no scratch directory"
trap 'rm -rf "$work"' EXIT
collect "$root" > "$work/records" || disarmed "the collect stage failed"
judge < "$work/records" > "$work/findings"
case $? in
  0) exit 0 ;;
  1) ;;
  *) disarmed "the judge could not read a record" ;;
esac
count="$(wc -l < "$work/findings" | tr -d ' ')"
"#,
    );
    let word = match g.action() {
        Action::Block => "BLOCKED",
        Action::Warn => "WARNING",
    };
    let _ = writeln!(
        s,
        "printf '[%s] %s: {word}. %s\\n' \"$tag\" \"$name\" \"$message\"\n\
         printf '%s finding(s) in %s:\\n' \"$count\" \"$root\"\n\
         sed 's/^/  /' \"$work/findings\""
    );
    let _ = writeln!(
        s,
        "exit {}",
        match g.action() {
            Action::Block => 1,
            Action::Warn => 0,
        }
    );
    s
}

/// The judge: records on stdin, one finding per line on stdout; 0 clean, 1
/// hit, 3 a record it cannot read. Mirrors `rule::Gate::judge`.
fn judge(engine: &GateEngine) -> String {
    let (fields, body) = match engine {
        GateEngine::EolAttribute { .. } => (
            4,
            r#"  always = ENVIRON["RL_ALWAYS"]
}
$0 == "" { next }
NF != 4 { bad = 1; next }
$1 !~ files { next }
excl != "" && $1 ~ excl { next }
{
  shebang = ($2 ~ /^#!/)
  needs = shebang || (always != "" && $1 ~ always)
  pinned = ($4 == "lf") || ($3 == "unset") || ($4 == "crlf" && !shebang)
  if (needs && !pinned) { print $1; hits++ }
}
END { if (bad) exit 3; exit (hits > 0) }"#,
        ),
        GateEngine::RootDocuments => (
            1,
            r#"}
$0 == "" { next }
NF != 1 { bad = 1; next }
index($1, "/") { next }
$1 !~ files { next }
excl != "" && $1 ~ excl { next }
{ print $1; hits++ }
END { if (bad) exit 3; exit (hits > 0) }"#,
        ),
        GateEngine::LintEnabled { .. } => (
            3,
            r#"  lint = ENVIRON["RL_LINT"]
  n = split(ENVIRON["RL_GROUPS"], g, " ")
  for (i = 1; i <= n; i++) grp[g[i]] = 1
  def = ENVIRON["RL_DEFAULT"]
}
$0 == "" { next }
NF != 3 { bad = 1; next }
$1 !~ files { next }
excl != "" && $1 ~ excl { next }
{
  on = ($3 == "warn" || $3 == "deny" || $3 == "forbid")
  off = ($3 == "allow" || $3 == "expect")
  if ($2 == lint) { if (on) non = 1; if (off) noff = 1 }
  else if ($2 in grp) { if (on) gon = 1; if (off) goff = 1 }
}
END {
  if (bad) exit 3
  enabled = !noff && (non || gon || (def == "warn" && !goff))
  if (!enabled) { print lint " is not enabled"; exit 1 }
  exit 0
}"#,
        ),
    };
    format!(
        "# Judge: {fields} field(s) per record on stdin. 0 clean, 1 hit, 3 unreadable.\n\
         judge() {{\n  awk '\nBEGIN {{\n  FS = \"\\t\"\n  files = ENVIRON[\"RL_FILES\"]\n  excl = ENVIRON[\"RL_EXCLUDE\"]\n  hits = 0\n  bad = 0\n{body}\n'\n}}\n"
    )
}

/// The collect stage: one record per fact about the checkout at `$1`.
fn collect(engine: &GateEngine) -> &'static str {
    match engine {
        GateEngine::EolAttribute { .. } => {
            r#"# Collect: every tracked path, its first line (tabs and CR removed), and
# `git check-attr` for text and eol. The attributes are the repository's own:
# this machine's system and global config are switched off, and the per-user
# attributes file is pointed at an empty one, because a pin that lives only on
# one machine is not a pin (git 2.32 or later; `.git/info/attributes`, per
# clone, is still read).
collect() {
  (
    cd "$1" || exit 1
    : > "$work/empty" || exit 1
    export GIT_CONFIG_NOSYSTEM=1 GIT_ATTR_NOSYSTEM=1 GIT_CONFIG_GLOBAL="$work/empty"
    git ls-files -z > "$work/paths" || exit 1
    git -c core.attributesFile="$work/empty" check-attr -z --stdin text eol \
      < "$work/paths" > "$work/attrs" || exit 1
    : > "$work/first"
    if [ -s "$work/paths" ]; then
      xargs -0 awk 'FNR == 1 { l = $0; gsub(/[\t\r]/, "", l); print FILENAME "\t" l; nextfile }' \
        < "$work/paths" > "$work/first" 2>/dev/null
    fi
    tr '\0' '\n' < "$work/paths" > "$work/paths.nl"
    tr '\0' '\n' < "$work/attrs" > "$work/attrs.nl"
    awk '
BEGIN { FS = "\t" }
FILENAME == ARGV[1] { first[$1] = $2; next }
FILENAME == ARGV[2] { n++; if (n % 3 == 1) p = $0; else if (n % 3 == 2) a = $0; else v[p, a] = $0; next }
{ print $0 "\t" first[$0] "\t" v[$0, "text"] "\t" v[$0, "eol"] }
' "$work/first" "$work/attrs.nl" "$work/paths.nl"
  )
}
"#
        }
        GateEngine::RootDocuments => {
            r#"# Collect: every tracked path.
collect() {
  git -C "$1" ls-files -z | tr '\0' '\n'
}
"#
        }
        GateEngine::LintEnabled { .. } => {
            r#"# Collect: lint levels from `[lints.clippy]`, `[lints.rust]` and their
# `workspace.` forms in every TOML file in scope, and from single-line
# crate-root `#![level(lint, ...)]` attributes in every Rust file in scope.
# A level given on the command line (`cargo clippy -- -W ...`) is not seen.
collect() {
  (
    cd "$1" || exit 1
    git ls-files -z | tr '\0' '\n' > "$work/paths.nl" || exit 1
    grep -E -- "$RL_FILES" "$work/paths.nl" > "$work/scoped" || true
    if [ -n "$RL_EXCLUDE" ]; then
      grep -vE -- "$RL_EXCLUDE" "$work/scoped" > "$work/kept" || true
    else
      cp "$work/scoped" "$work/kept"
    fi
    while IFS= read -r p; do
      [ -f "$p" ] || continue
      RL_PATH="$p" awk '
BEGIN { p = ENVIRON["RL_PATH"]; toml = (p ~ /\.toml$/) }
{ sub(/\r$/, "") }
toml && /^[ \t]*#/ { next }
toml && /^[ \t]*\[/ { t = $0; sub(/#.*/, "", t); gsub(/[ \t]/, "", t); table = t; next }
toml && (table == "[lints.clippy]" || table == "[workspace.lints.clippy]" || table == "[lints.rust]" || table == "[workspace.lints.rust]") && /=/ {
  key = $0; sub(/=.*/, "", key); gsub(/[ \t"]/, "", key); gsub(/-/, "_", key)
  val = $0; sub(/^[^=]*=/, "", val)
  if (match(val, /"(allow|expect|warn|deny|forbid)"/)) {
    prefix = (table ~ /clippy/) ? "clippy::" : ""
    print p "\t" prefix key "\t" substr(val, RSTART + 1, RLENGTH - 2)
  }
  next
}
!toml && /^[ \t]*#!\[(allow|expect|warn|deny|forbid)\(/ {
  l = $0; sub(/^[ \t]*#!\[/, "", l)
  level = l; sub(/\(.*/, "", level)
  inner = l; sub(/^[a-z]+\(/, "", inner); sub(/\)\].*$/, "", inner)
  n = split(inner, parts, ",")
  for (i = 1; i <= n; i++) { x = parts[i]; gsub(/[ \t]/, "", x); if (x != "") print p "\t" x "\t" level }
}
' "$p"
    done < "$work/kept"
  )
}
"#
        }
    }
}

/// The fixture runner: every gate's records through `--judge`, with
/// `--trees` every gate whole over its synthetic checkouts, and every
/// commit-time fixture through the dispatcher over a staged repository.
fn test_runner(
    gates: &BTreeMap<&str, (&Rule, &Gate)>,
    committed: &BTreeMap<&str, Committed<'_>>,
) -> String {
    let mut s = String::from(
        r#"#!/usr/bin/env bash
# Every repository-gate fixture, fed to the generated scripts beside this file.
# `relearn check` ran the same records through the Rust judge; this runs them
# through the generated `awk` judge. With `--trees <dir>`, it also builds a
# checkout per gate from <dir>/<name>/hit and <dir>/<name>/clean, runs
# `git init` and `git add`, and runs the whole gate, collect stage included.
# A tree stores a dotfile as `dot-<name>`, so the repository committing the
# tree never reads its `.gitattributes` or `.gitignore`.
# Every commit-time fixture is staged in a fresh repository and run through
# relearn-pre-commit.sh, with this machine's git config switched off.
# Generated by relearn; edit the rule files, never this script.
set -uo pipefail
export LC_ALL=C

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
trees=""
case "${1:-}" in
  --trees) trees="${2:-}"; [ -d "$trees" ] || { echo "FAIL: --trees needs a directory"; exit 1; } ;;
  "") ;;
  *) echo "usage: test-gates.sh [--trees <dir>]"; exit 2 ;;
esac
"#,
    );
    let _ = writeln!(
        s,
        "need_git={}",
        if committed.is_empty() {
            "\"$trees\""
        } else {
            "yes"
        }
    );
    s.push_str(
        r#"if [ -n "$need_git" ] && ! command -v git >/dev/null 2>&1; then
  echo "DISARMED (exit 2, not a pass): git is not on PATH, so no tree or commit case ran"
  exit 2
fi
work="$(mktemp -d)" || { echo "FAIL: no scratch directory"; exit 1; }
trap 'rm -rf "$work"' EXIT
: > "$work/empty-config" || { echo "FAIL: no scratch config"; exit 1; }
pass=0
failed=0

# judge_case <script> <expect hit|clean> [record...]
judge_case() {
  local script="$1" expect="$2"
  shift 2
  printf '%s\n' "$@" | bash "$here/$script" --judge > /dev/null 2> "$work/stderr"
  local got=$? want=0
  [ "$expect" = hit ] && want=1
  if [ "$got" -eq "$want" ]; then
    pass=$((pass + 1))
  else
    failed=$((failed + 1))
    printf 'FAIL  %s  records expected %s: judge exit %s, stderr %s bytes\n' \
      "$script" "$expect" "$got" "$(wc -c < "$work/stderr" | tr -d ' ')"
  fi
}

# tree_case <script> <gate name> <expect hit|clean> <action block|warn>
tree_case() {
  local script="$1" gate="$2" expect="$3" action="$4"
  local src="$trees/$gate/$expect"
  if [ ! -d "$src" ]; then
    failed=$((failed + 1))
    printf 'FAIL  %s  no %s tree at %s\n' "$script" "$expect" "$src"
    return
  fi
  local repo="$work/tree-$gate-$expect"
  mkdir -p "$repo" && cp -R "$src/." "$repo/"
  local f
  find "$repo" -name 'dot-*' > "$work/dots"
  while IFS= read -r f; do
    mv "$f" "$(dirname "$f")/.$(basename "$f" | sed 's/^dot-//')"
  done < "$work/dots"
  git -C "$repo" init -q > /dev/null 2>&1 && git -C "$repo" add -A > /dev/null 2>&1 || {
    failed=$((failed + 1))
    printf 'FAIL  %s  git could not stage the %s tree\n' "$script" "$expect"
    return
  }
  local out got ok=no
  out="$(bash "$here/$script" "$repo" 2> "$work/stderr")"
  got=$?
  case "$expect:$action" in
    clean:*) [ "$got" -eq 0 ] && [ -z "$out" ] && ok=yes ;;
    hit:block) [ "$got" -eq 1 ] && case "$out" in *BLOCKED*) true ;; *) false ;; esac && ok=yes ;;
    hit:warn) [ "$got" -eq 0 ] && case "$out" in *WARNING*) true ;; *) false ;; esac && ok=yes ;;
  esac
  if [ "$ok" = yes ]; then
    pass=$((pass + 1))
  else
    failed=$((failed + 1))
    printf 'FAIL  %s  %s tree expected %s (%s): exit %s, stdout %s bytes, stderr %s bytes\n' \
      "$script" "$gate" "$expect" "$action" "$got" "${#out}" "$(wc -c < "$work/stderr" | tr -d ' ')"
  fi
}

# commit_case <detector> <expect hit|clean|disarmed> <action> <path> <text> <none|given> [list line...]
commit_case() {
  local det="$1" expect="$2" action="$3" path="$4" text="$5" mode="$6"
  shift 6
  local n=$((pass + failed))
  local repo="$work/commit-$n"
  mkdir -p "$repo" || { failed=$((failed + 1)); printf 'FAIL  %s  no scratch repository\n' "$det"; return; }
  (
    export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL="$work/empty-config"
    cd "$repo" || exit 1
    git init -q . > /dev/null 2>&1 || exit 1
    mkdir -p "$(dirname "$path")" && printf '%s' "$text" > "$path" || exit 1
    if [ "$mode" = given ]; then
      { printf '# a synthetic list\n\n'; printf '%s\n' "$@"; } > "$work/list-$n" || exit 1
      git config --local "relearn.$det.list" "$work/list-$n" || exit 1
    fi
    git add -A > /dev/null 2>&1
  ) || {
    failed=$((failed + 1))
    printf 'FAIL  %s  %s: git could not stage the case\n' "$det" "$path"
    return
  }
  local out got ok=no
  out="$(cd "$repo" && GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL="$work/empty-config" \
    bash "$here/relearn-pre-commit.sh" "$det" 2>&1)"
  got=$?
  case "$expect:$action" in
    clean:*) [ "$got" -eq 0 ] && [ -z "$out" ] && ok=yes ;;
    hit:block) [ "$got" -eq 1 ] && case "$out" in *BLOCKED*) true ;; *) false ;; esac && ok=yes ;;
    hit:warn) [ "$got" -eq 0 ] && case "$out" in *WARNING*) true ;; *) false ;; esac && ok=yes ;;
    disarmed:*) [ "$got" -eq 2 ] && case "$out" in *DISARMED*) true ;; *) false ;; esac && ok=yes ;;
  esac
  if [ "$ok" = yes ]; then
    pass=$((pass + 1))
  else
    failed=$((failed + 1))
    printf 'FAIL  relearn-pre-commit.sh  %s  %s expected %s (%s): exit %s, output %s bytes\n' \
      "$det" "$path" "$expect" "$action" "$got" "${#out}"
  fi
}

"#,
    );
    for (name, (_, gate)) in gates {
        let script = sh_quote(&format!("{name}.sh"));
        let _ = writeln!(s, "# {name}");
        for fixture in gate.fixtures() {
            let expect = match fixture.expect() {
                Detection::Hit => "hit",
                Detection::Clean => "clean",
            };
            let _ = write!(s, "judge_case {script} {expect}");
            for line in fixture.lines() {
                let _ = write!(s, " {}", ansi_quote(&line));
            }
            s.push('\n');
        }
        let _ = writeln!(
            s,
            "if [ -n \"$trees\" ]; then\n  tree_case {script} {g} hit {a}\n  tree_case {script} {g} clean {a}\nfi",
            g = sh_quote(name),
            a = gate.action().as_str(),
        );
    }
    for (name, (_, detector, engine)) in committed {
        let det = sh_quote(name);
        let action = detector.action().as_str();
        let _ = writeln!(s, "# {name} (commit time)");
        for fixture in detector.fixtures() {
            let expect = fixture.expect().as_str();
            let path = sh_quote(fixture.path().as_str());
            match fixture.input() {
                FixtureInput::Text(text) => {
                    let _ = writeln!(
                        s,
                        "commit_case {det} {expect} {action} {path} {} none",
                        sh_quote(text)
                    );
                }
                FixtureInput::Listed(list) => {
                    let entries: Vec<String> = list.iter().map(|e| sh_quote(e)).collect();
                    let _ = writeln!(
                        s,
                        "commit_case {det} {expect} {action} {path} '' given {}",
                        entries.join(" ")
                    );
                }
                // Names in a directory belong to `sibling-case`, which never
                // runs at commit.
                FixtureInput::Siblings(_) => {}
            }
        }
        if let CommitEngine::ListedPath = engine {
            // Fail closed: no list, and a list holding a line that is not a
            // relative path, both come back as "could not check".
            if let Some(first) = detector.fixtures().first() {
                let path = sh_quote(first.path().as_str());
                let _ = writeln!(
                    s,
                    "commit_case {det} disarmed {action} {path} '' none\n\
                     commit_case {det} disarmed {action} {path} '' given '../outside'"
                );
            }
        }
    }
    s.push_str(
        r#"
if [ "$failed" -eq 0 ]; then
  echo "PASS: $pass gate case(s) agree with the generated scripts"
  exit 0
fi
echo "FAIL: $failed of $((pass + failed)) gate case(s) disagree with the generated scripts"
exit 1
"#,
    );
    s
}
