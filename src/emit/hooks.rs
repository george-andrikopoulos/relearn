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
    Action, Detection, Detector, DetectorName, Engine, Event, FixtureInput, HeadingLevel, Pattern,
    Rule, RuleTag, Subject, Token,
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

/// Which file of the pack this is. The self-exclusion sweep reads it to skip,
/// **by identity**, the two files that must contain what a detector looks for:
/// its own script, which quotes the pattern, and the runner, which pipes every
/// hit fixture through it. A path glob would skip them as well, and would also
/// stop the detector reading a real repository's `hooks/*.sh` at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookRole {
    /// `<name>.sh`, the script compiled from this detector.
    Script(DetectorName),
    /// `test-detectors.sh`, every fixture of every detector.
    Runner,
    /// `settings-snippet.md`, names and commands only.
    Snippet,
    /// The repo pack's `relearn-pre-commit.sh`, which quotes every
    /// commit-time detector and no other.
    Dispatcher,
}

impl HookRole {
    /// The role of the script compiled from the detector called `name`;
    /// `None` when `name` is not a detector name.
    #[must_use]
    pub fn script_of(name: &str) -> Option<Self> {
        DetectorName::new(name).ok().map(HookRole::Script)
    }

    /// Whether `detector` necessarily matches this file, so a hit here says
    /// nothing about its scope.
    fn holds_own_definition(&self, detector: &Detector) -> bool {
        match self {
            HookRole::Script(name) => name == detector.name(),
            HookRole::Runner => true,
            HookRole::Dispatcher => detector.runs_at(Event::GitPreCommit),
            HookRole::Snippet => false,
        }
    }
}

/// The Claude Code event a hook script is registered under. The derived
/// order is the explicit rank the settings snippet uses: the event a tool call
/// meets first comes first (`[R:order-by-explicit-rank]`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HookEvent {
    PreToolUse,
    PostToolUse,
}

impl HookEvent {
    fn as_str(self) -> &'static str {
        match self {
            HookEvent::PreToolUse => "PreToolUse",
            HookEvent::PostToolUse => "PostToolUse",
        }
    }
}

/// When a `sections` hook runs: before a `Write`, after an `Edit`, or both.
#[derive(Clone, Copy)]
enum SectionsAt {
    BeforeWrite,
    AfterEdit,
    Both,
}

/// What a hook script can run, for a detector that runs at a Claude Code
/// event. `listed-path` reads a staged path, so a hook script for one is
/// unconstructible rather than emitted empty.
enum WriteEngine<'a> {
    LineRegex {
        pattern: &'a Pattern,
        comment: Option<&'a Token>,
        allow: Option<&'a Token>,
    },
    SiblingCase,
    Sections {
        heading: HeadingLevel,
        require: &'a Pattern,
        at: SectionsAt,
    },
}

impl<'a> WriteEngine<'a> {
    fn of(d: &'a Detector) -> Option<Self> {
        let before = d.runs_at(Event::ClaudeCodePreToolUse);
        let after = d.runs_at(Event::ClaudeCodePostToolUse);
        match d.engine() {
            Engine::LineRegex {
                pattern,
                comment,
                allow,
            } if before => Some(WriteEngine::LineRegex {
                pattern,
                comment: comment.as_ref(),
                allow: allow.as_ref(),
            }),
            Engine::SiblingCase if before => Some(WriteEngine::SiblingCase),
            Engine::Sections { heading, require } => {
                let at = match (before, after) {
                    (true, true) => SectionsAt::Both,
                    (true, false) => SectionsAt::BeforeWrite,
                    (false, true) => SectionsAt::AfterEdit,
                    (false, false) => return None,
                };
                Some(WriteEngine::Sections {
                    heading: *heading,
                    require,
                    at,
                })
            }
            Engine::LineRegex { .. } | Engine::SiblingCase | Engine::ListedPath => None,
        }
    }

    /// The Claude Code tools this script reads, at any event. Creating a file
    /// is only ever a `Write`; changing text is any of the three.
    fn tools(&self) -> &'static str {
        match self {
            WriteEngine::LineRegex { .. }
            | WriteEngine::Sections {
                at: SectionsAt::Both,
                ..
            } => "Write|Edit|MultiEdit",
            WriteEngine::SiblingCase
            | WriteEngine::Sections {
                at: SectionsAt::BeforeWrite,
                ..
            } => "Write",
            WriteEngine::Sections {
                at: SectionsAt::AfterEdit,
                ..
            } => "Edit|MultiEdit",
        }
    }

    /// The registration: `(event, matcher)` for each event this script runs at.
    fn registrations(&self) -> Vec<(HookEvent, &'static str)> {
        match self {
            WriteEngine::LineRegex { .. } => vec![(HookEvent::PreToolUse, "Write|Edit|MultiEdit")],
            WriteEngine::SiblingCase => vec![(HookEvent::PreToolUse, "Write")],
            WriteEngine::Sections { at, .. } => match at {
                SectionsAt::BeforeWrite => vec![(HookEvent::PreToolUse, "Write")],
                SectionsAt::AfterEdit => vec![(HookEvent::PostToolUse, "Edit|MultiEdit")],
                SectionsAt::Both => vec![
                    (HookEvent::PreToolUse, "Write"),
                    (HookEvent::PostToolUse, "Edit|MultiEdit"),
                ],
            },
        }
    }

    /// Whether this script reads a `PreToolUse` payload at all.
    fn runs_before_a_write(&self) -> bool {
        self.registrations()
            .iter()
            .any(|(event, _)| *event == HookEvent::PreToolUse)
    }

    /// The header's event line.
    fn events_label(&self) -> String {
        match self {
            WriteEngine::LineRegex { .. } | WriteEngine::SiblingCase => {
                format!("Claude Code PreToolUse, on {}", self.tools())
            }
            WriteEngine::Sections { at, .. } => match at {
                SectionsAt::BeforeWrite => String::from("Claude Code PreToolUse, on Write"),
                SectionsAt::AfterEdit => String::from(
                    "Claude Code PostToolUse, on Edit|MultiEdit (warns only: the edit has happened)",
                ),
                SectionsAt::Both => String::from(
                    "Claude Code PreToolUse on Write, and PostToolUse on Edit|MultiEdit\n\
                     # (warns only after an edit: the edit has happened)",
                ),
            },
        }
    }
}

/// The `sections` judge, shared by the hook scripts and the commit
/// dispatcher: `sections_judge <file> <require ERE> <level>` prints every
/// section of that level with no line matching the pattern, as
/// `<line>: <heading>`, and returns 0 clean, 1 hit, 2 could not read. `grep`
/// is the only reader of the pattern, as for `line-regex`; `awk` reads only
/// headings and fences, mirroring `rule::detector`'s `section_lacks`.
pub(super) const SECTIONS_JUDGE: &str = r##"sections_judge() {
  grep -nE -- "$2" "$1" > "$work/req.raw"
  case $? in 0|1) ;; *) return 2 ;; esac
  cut -d: -f1 "$work/req.raw" > "$work/req" || return 2
  RL_LEVEL="$3" awk '
BEGIN { lvl = ENVIRON["RL_LEVEL"] + 0; hits = 0 }
FILENAME == ARGV[1] { ok[$1] = 1; next }
{
  line = $0; sub(/\r$/, "", line)
  fence = (substr(line, 1, 3) == "```" || substr(line, 1, 3) == "~~~")
  if (!infence && !fence) {
    n = 0; while (n < 7 && substr(line, n + 1, 1) == "#") n++
    c = substr(line, n + 1, 1)
    if (n >= 1 && n <= 6 && (c == " " || c == "\t" || c == "") && n <= lvl) {
      if (open && !sat) { print head; hits++ }
      open = (n == lvl); sat = 0; head = FNR ": " line
      next
    }
  }
  if (fence) infence = !infence
  if (open && (FNR in ok)) sat = 1
}
END { if (open && !sat) { print head; hits++ } exit (hits > 0) }
' "$work/req" "$1" || return $?
}
"##;

/// One hook script's inputs: the rule, the detector, and what it runs.
type Hooked<'a> = (&'a Rule, &'a Detector, WriteEngine<'a>);

/// Emit the hook pack. A library with no write-time detector produces no file.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    emit_with_roles(library)
        .into_iter()
        .map(|(file, _)| file)
        .collect()
}

/// [`emit`], with each file's [`HookRole`]: the identity the self-exclusion
/// sweep needs, from the only function that names these files.
#[must_use]
pub fn emit_with_roles(library: &Library<Validated>) -> Vec<(OutputFile, HookRole)> {
    // By name: the name is the file, so this is the order a reader lists them.
    let mut detectors: BTreeMap<&str, Hooked<'_>> = BTreeMap::new();
    for rule in emittable(library) {
        for detector in rule.detectors() {
            if let Some(engine) = WriteEngine::of(detector) {
                detectors.insert(detector.name().as_str(), (rule, detector, engine));
            }
        }
    }
    if detectors.is_empty() {
        return Vec::new();
    }

    let mut files = Vec::with_capacity(detectors.len() + 2);
    for (name, (rule, detector, engine)) in &detectors {
        let file = format!("{name}.sh");
        files.push((
            OutputFile::script(
                RelativePath::from_segments(&[HOOKS_DIR, &file]),
                script(rule.tag(), detector, engine),
                vec![rule.tag().clone()], // allow:clone: the OutputFile owns its provenance, outliving the &Library borrow
            ),
            HookRole::Script(detector.name().clone()), // allow:clone: the role names its detector beyond the &Library borrow
        ));
    }

    let mut sources: Vec<RuleTag> = detectors
        .values()
        .map(|(r, _, _)| r.tag().clone())
        .collect(); // allow:clone: same
    sources.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    sources.dedup();
    let runner = format!("{RUNNER}.sh");
    files.push((
        OutputFile::script(
            RelativePath::from_segments(&[HOOKS_DIR, &runner]),
            test_runner(&detectors),
            sources.clone(), // allow:clone: two generated files share one provenance list
        ),
        HookRole::Runner,
    ));
    files.push((
        OutputFile::new(
            RelativePath::from_segments(&[HOOKS_DIR, "settings-snippet.md"]),
            settings_snippet(&detectors),
            sources,
        ),
        HookRole::Snippet,
    ));
    files
}

/// One file the self-exclusion sweep runs every detector over, and whether
/// the hooks target made it.
#[derive(Debug, Clone, Copy)]
pub struct Swept<'a> {
    path: &'a str,
    text: &'a str,
    made_by_hooks: Option<&'a HookRole>,
}

impl<'a> Swept<'a> {
    /// A file on disk, such as a rule file, at the path given.
    #[must_use]
    pub fn source(path: &'a str, text: &'a str) -> Self {
        Swept {
            path,
            text,
            made_by_hooks: None,
        }
    }

    /// A file a target other than `hooks` emitted.
    #[must_use]
    pub fn emitted(file: &'a OutputFile) -> Self {
        Swept {
            path: file.path().as_str(),
            text: file.contents(),
            made_by_hooks: None,
        }
    }

    /// A file the hooks target emitted, with the role [`emit_with_roles`] gave it.
    #[must_use]
    pub fn hook(file: &'a OutputFile, role: &'a HookRole) -> Self {
        Swept {
            path: file.path().as_str(),
            text: file.contents(),
            made_by_hooks: Some(role),
        }
    }

    fn holds_own_definition(&self, detector: &Detector) -> bool {
        self.made_by_hooks
            .is_some_and(|role| role.holds_own_definition(detector))
    }
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
/// emits, at the path it is emitted to. A detector's own script and the fixture
/// runner are skipped for that detector, by their [`HookRole`]: they contain
/// the pattern and its hit fixtures by construction, so a hit there says
/// nothing about the detector's scope. A rule file holds a pattern and the
/// inputs that must hit it, and an emitted instruction layer quotes what a rule
/// forbids; a detector that fires on either has a scope wide enough to read its
/// own definitions, and would block the edit that maintains it. The implicit
/// excludes are meant to make this empty; this is what shows they do, for the
/// corpus as it is rather than as the excludes assumed it would be.
///
/// Siblings for the case engine are the other given files in the same
/// directory, so a case collision **among** relearn's own files is a hit too.
#[must_use]
pub fn self_matches(library: &Library<Validated>, files: &[Swept<'_>]) -> Vec<SelfMatch> {
    let dir_of = |p: &str| p.rsplit_once('/').map_or("", |(d, _)| d).to_owned();
    let mut found = Vec::new();
    for rule in library.rules() {
        for detector in rule.detectors() {
            for file in files {
                if file.holds_own_definition(detector) {
                    continue;
                }
                let path = file.path;
                let dir = dir_of(path);
                let siblings: Vec<String> = files
                    .iter()
                    .filter(|other| other.path != path && dir_of(other.path) == dir)
                    .map(|other| {
                        other
                            .path
                            .rsplit('/')
                            .next()
                            .unwrap_or(other.path)
                            .to_owned()
                    })
                    .collect();
                let subject = Subject::new(path, file.text, &siblings);
                match detector.evaluate(&subject) {
                    Detection::Hit => found.push(SelfMatch {
                        detector: detector.name().as_str().to_owned(),
                        file: path.to_owned(),
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
pub(super) fn ere_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if ".[]()*+?{}|^$\\".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// The prologue every script shares: identity, the dependency check, the tool
/// filter, the path, and the scope.
fn prologue(tag: &RuleTag, d: &Detector, engine: &WriteEngine<'_>) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "#!/usr/bin/env bash");
    let _ = writeln!(
        s,
        "# Detector `{name}` for [{tag}], generated by relearn from the rule file.\n\
         # Edit the rule, never this script: `relearn verify` reports a hand edit.\n\
         # Engine: {engine}. Event: {events}.\n\
         # Exit 2 blocks, and stderr is the reason Claude reads. Exit 0 allows.\n\
         # Any other exit is this script failing: reported, never a block.",
        name = d.name().as_str(),
        tag = tag.as_str(),
        engine = d.engine().as_str(),
        events = engine.events_label(),
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
        engine.tools()
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

/// The verdict: how a hit leaves the script.
fn verdict(d: &Detector, report: &str) -> String {
    match d.action() {
        Action::Block => format!("{{\n{report}}} >&2\nexit 2\n"),
        Action::Warn => warning("PreToolUse", report),
    }
}

/// A warning: the report as context Claude reads, and exit 0.
fn warning(event: &str, report: &str) -> String {
    format!(
        "context=\"$(\n{report})\"\n\
         jq -n --arg c \"$context\" \
         '{{hookSpecificOutput:{{hookEventName:\"{event}\",additionalContext:$c}}}}' \
         || fail \"jq could not build the warning\"\n\
         exit 0\n"
    )
}

/// The body of a `sections` script, after the prologue. Before a `Write` the
/// content is the payload's; after an `Edit` it is the file on disk, and the
/// verdict is a warning whatever the action, because the edit has happened.
fn sections_body(d: &Detector, heading: HeadingLevel, require: &Pattern, at: SectionsAt) -> String {
    let (before, after) = match at {
        SectionsAt::BeforeWrite => (true, false),
        SectionsAt::AfterEdit => (false, true),
        SectionsAt::Both => (true, true),
    };
    let mut s = String::new();
    let _ = writeln!(
        s,
        "\nrequire={}\nlevel={}",
        sh_quote(require.as_str()),
        heading.get()
    );
    s.push_str(
        "work=\"$(mktemp -d)\" || fail \"no scratch directory\"\n\
         trap 'rm -rf \"$work\"' EXIT\n\n",
    );
    s.push_str(SECTIONS_JUDGE);
    s.push_str(
        "\nevent=\"$(jq -r '.hook_event_name // \"\"' <<<\"$input\")\" \
         || fail \"the payload has no readable hook_event_name\"\n\
         case \"$event:$tool_name\" in\n",
    );
    if before {
        s.push_str(
            "  PreToolUse:Write)\n\
             \x20   jq -j '.tool_input.content // \"\"' <<<\"$input\" > \"$work/file\" \
             || fail \"the payload's content could not be read\"\n\
             \x20   when=before ;;\n",
        );
    }
    if after {
        s.push_str(
            "  PostToolUse:Edit | PostToolUse:MultiEdit)\n\
             \x20   [ -f \"$file_path\" ] || exit 0\n\
             \x20   cat -- \"$file_path\" > \"$work/file\" || fail \"the edited file could not be read\"\n\
             \x20   when=after ;;\n",
        );
    }
    s.push_str(
        "  *) exit 0 ;;\nesac\n\n\
         hits=\"$(sections_judge \"$work/file\" \"$require\" \"$level\")\"\n\
         case $? in 0) exit 0 ;; 1) ;; *) fail \"the sections judge could not read the file\" ;; esac\n\n",
    );
    let report = |word: &str| {
        format!(
            "  printf '[%s] %s: {word}. %s\\n' \"$tag\" \"$name\" \"$message\"\n\
             \x20 printf 'Section(s) in %s with no line matching %s:\\n%s\\n' \"$file_path\" \"$require\" \"$hits\"\n"
        )
    };
    if after {
        let post = warning("PostToolUse", &report("WARNING"));
        if before {
            let _ = write!(s, "if [ \"$when\" = after ]; then\n{post}fi\n");
        } else {
            s.push_str(&post);
            return s;
        }
    }
    s.push_str(&verdict(d, &report(verdict_word(d.action()))));
    s
}

fn verdict_word(action: Action) -> &'static str {
    match action {
        Action::Block => "BLOCKED",
        Action::Warn => "WARNING",
    }
}

/// One detector's script.
fn script(tag: &RuleTag, d: &Detector, engine: &WriteEngine<'_>) -> String {
    let mut s = prologue(tag, d, engine);
    let word = verdict_word(d.action());
    match engine {
        WriteEngine::LineRegex {
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
        WriteEngine::SiblingCase => {
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
        WriteEngine::Sections {
            heading,
            require,
            at,
        } => s.push_str(&sections_body(d, *heading, require, *at)),
    }
    s
}

/// The fixture runner: every fixture of every detector, through the scripts.
fn test_runner(detectors: &BTreeMap<&str, Hooked<'_>>) -> String {
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
    '{hook_event_name:"PreToolUse",tool_name:"Write",tool_input:{file_path:$p,content:$c}}' \
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
    let any_post = detectors
        .values()
        .any(|(_, d, _)| d.runs_at(Event::ClaudeCodePostToolUse));
    if any_post {
        s.push_str(
            r#"# run_post <script> <expect hit|clean> <path> <text>
# The file is written to disk first, then an Edit's PostToolUse payload names
# it. After an edit a hit only ever warns, whatever the detector's action.
run_post() {
  local script="$1" expect="$2" path="$3" text="$4"
  local case_dir="$work/$((pass + failed))"
  local target="$case_dir/$path"
  mkdir -p "$(dirname "$target")"
  printf '%s' "$text" > "$target"
  local out got
  out="$(jq -nc --arg p "$target" \
    '{hook_event_name:"PostToolUse",tool_name:"Edit",tool_input:{file_path:$p,old_string:"",new_string:""},tool_response:{}}' \
    | bash "$here/$script" 2>"$work/stderr")"
  got=$?
  local err
  err="$(cat "$work/stderr")"
  local ok=no
  case "$expect" in
    clean) [ "$got" -eq 0 ] && [ -z "$out" ] && [ -z "$err" ] && ok=yes ;;
    hit) [ "$got" -eq 0 ] && case "$out" in *'"PostToolUse"'*additionalContext*) true ;; *) false ;; esac && ok=yes ;;
  esac
  if [ "$ok" = yes ]; then
    pass=$((pass + 1))
  else
    failed=$((failed + 1))
    printf 'FAIL  %s  %s expected %s after an edit: exit %s, stdout %s bytes, stderr %s bytes\n' \
      "$script" "$path" "$expect" "$got" "${#out}" "${#err}"
  fi
}

"#,
        );
    }
    for (name, (_, detector, engine)) in detectors {
        let _ = writeln!(s, "# {name}");
        for fixture in detector.fixtures() {
            let expect = match fixture.expect() {
                Detection::Hit => "hit",
                Detection::Clean => "clean",
            };
            let (text, siblings) = match fixture.input() {
                FixtureInput::Text(text) => (text.as_str(), Vec::new()),
                FixtureInput::Siblings(names) => ("", names.iter().map(|n| sh_quote(n)).collect()),
                // A list belongs to `listed-path`, which has no hook script.
                FixtureInput::Listed(_) => continue,
            };
            if detector.runs_at(Event::ClaudeCodePostToolUse) {
                let _ = writeln!(
                    s,
                    "run_post {} {expect} {} {}",
                    sh_quote(&format!("{name}.sh")),
                    sh_quote(fixture.path().as_str()),
                    sh_quote(text),
                );
            }
            if !engine.runs_before_a_write() {
                continue;
            }
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
fn settings_snippet(detectors: &BTreeMap<&str, Hooked<'_>>) -> String {
    // Keyed by `HookEvent`, whose order is the explicit rank; a string sort
    // would put PostToolUse before PreToolUse.
    let mut by_event: BTreeMap<HookEvent, BTreeMap<&str, Vec<&str>>> = BTreeMap::new();
    for (name, (_, _, engine)) in detectors {
        for (event, matcher) in engine.registrations() {
            by_event
                .entry(event)
                .or_default()
                .entry(matcher)
                .or_default()
                .push(name);
        }
    }
    let mut events = Vec::new();
    for (event, by_matcher) in &by_event {
        let mut groups = Vec::new();
        for (matcher, names) in by_matcher {
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
        events.push(format!(
            "    \"{}\": [\n{}\n    ]",
            event.as_str(),
            groups.join(",\n")
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
         ```json\n{{\n  \"hooks\": {{\n{}\n  }}\n}}\n```\n",
        events.join(",\n")
    )
}
