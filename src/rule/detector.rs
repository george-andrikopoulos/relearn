//! `[[detector]]` — the mechanical half of a rule, carried by the rule itself.
//!
//! A detector is what a hand-written hook used to be: a scope (which files), an
//! engine (what it looks for there), and an action (block or warn). It lives in
//! the rule so the rule is the **only** source of the hook that enforces it
//! (`[R:one-home-per-rule]`), and `emit::hooks` compiles it to a script.
//!
//! **Three guarantees are held at construction, not checked later:**
//!
//! * A detector with no `hit` fixture or no `clean` fixture does not parse
//!   (`[R:evidence-must-discriminate]`): one that cannot show a miss is evidence
//!   of nothing, and one that cannot show a hit may detect nothing.
//! * Every fixture is **run** while parsing, and one whose verdict differs from
//!   its `expect` stops the parse. A [`Detector`] therefore cannot exist with
//!   fixtures that disagree with its own pattern.
//! * A pattern is accepted only inside the subset of ERE that `grep -E` (which
//!   the generated hook runs) and the `regex` crate (which this module runs)
//!   read the same way, so a fixture passing here is a claim about the hook.
//!   The claim is still checked on the production path: the pack's generated
//!   `test-detectors.sh` pipes every fixture through the generated scripts.
//!
//! **Must NOT:** read the filesystem. Evaluation takes a [`Subject`] — a path,
//! the text being written, and the names already beside it — and decides.

use std::fmt;

use regex::bytes::{Regex, RegexBuilder};
use serde::Deserialize;

mod gate;

pub use gate::{Gate, GateEngine, GateFixture, LintDefault, LintLevel, LintName, RecordProblem};

/// What a detector concluded about one write.
///
/// The same type is a fixture's `expect`, so "the fixture agrees" is an
/// equality between two values of one type rather than a translation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detection {
    /// The write is in scope and the engine found what it looks for.
    Hit,
    /// Out of scope, or nothing found.
    Clean,
}

impl Detection {
    /// The spelling a rule file uses for this verdict.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Detection::Hit => "hit",
            Detection::Clean => "clean",
        }
    }

    fn parse(s: &str) -> Result<Self, DetectorError> {
        match s {
            "hit" => Ok(Detection::Hit),
            "clean" => Ok(Detection::Clean),
            other => Err(DetectorError::UnknownExpect(other.to_owned())),
        }
    }
}

impl fmt::Display for Detection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Every way a `[[detector]]` table can be refused. Each names what failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DetectorError {
    #[error(
        "`name` {0:?} must be lowercase letters, digits and hyphens, starting with a letter or digit"
    )]
    Name(String),
    #[error(
        "unknown `engine` {0:?} (expected line-regex | sibling-case | listed-path | sections | eol-attribute | root-documents | lint-enabled)"
    )]
    UnknownEngine(String),
    #[error("engine `{engine}` runs at {expected}")]
    EventNotForEngine {
        engine: &'static str,
        expected: &'static str,
    },
    #[error("`lint` {0:?} must be a lint name, `[a-z][a-z0-9_]*` with at most one `tool::` prefix")]
    LintName(String),
    #[error("unknown `default` {0:?} (expected allow | warn)")]
    LintDefault(String),
    #[error("fixture {index}, record {record}: {problem}")]
    Record {
        index: usize,
        record: usize,
        problem: Box<RecordProblem>,
    },
    #[error("engine `{engine}` requires field `{field}`")]
    FieldRequired {
        engine: &'static str,
        field: &'static str,
    },
    #[error("engine `{engine}` does not read field `{field}`; remove it")]
    FieldNotUsed {
        engine: &'static str,
        field: &'static str,
    },
    #[error("`pattern` {pattern:?}: {problem}")]
    Pattern {
        pattern: String,
        problem: PatternProblem,
    },
    #[error("glob {glob:?}: {problem}")]
    Glob { glob: String, problem: GlobProblem },
    #[error("`files` must name at least one glob")]
    NoFiles,
    #[error("`{field}` must not be empty or span more than one line")]
    Token { field: &'static str },
    #[error("`message` must not be empty")]
    EmptyMessage,
    #[error("`events` must name at least one event")]
    NoEvents,
    #[error("event {0:?} is listed twice")]
    DuplicateEvent(String),
    #[error(
        "unknown event {0:?} (expected claude-code-pre-tool-use | claude-code-post-tool-use | git-pre-commit)"
    )]
    UnknownEvent(String),
    #[error("`heading` {0} must be a Markdown heading level, 1 to 6")]
    HeadingLevel(i64),
    #[error(
        "fixture {index}: `list` entry {entry:?} must be a relative forward-slash path with no `..`"
    )]
    ListEntry { index: usize, entry: String },
    #[error("unknown `action` {0:?} (expected block | warn)")]
    UnknownAction(String),
    #[error("unknown fixture `expect` {0:?} (expected hit | clean)")]
    UnknownExpect(String),
    #[error("fixture {index}: `path` {path:?} must be a relative forward-slash path with no `..`")]
    FixturePath { index: usize, path: String },
    #[error(
        "fixture {index}: engine `{engine}` needs {} and nothing else",
        match *engine {
            "sibling-case" => "`path` and `siblings`",
            "line-regex" | "sections" => "`path` and `input`",
            "listed-path" => "`path` and `list`",
            _ => "`records`",
        }
    )]
    FixtureInputMismatch { index: usize, engine: &'static str },
    #[error("no fixture expects `hit`: a detector that cannot show a hit may detect nothing")]
    NoHitFixture,
    #[error(
        "no fixture expects `clean`: a detector that cannot show a miss is evidence of nothing"
    )]
    NoCleanFixture,
    #[error("fixture {index} ({path}) expects {expected} and the detector says {got}")]
    FixtureDisagrees {
        index: usize,
        path: String,
        expected: Detection,
        got: Detection,
    },
}

/// Why a pattern is outside the dialect both engines share.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PatternProblem {
    #[error("must not be empty")]
    Empty,
    #[error("must be ASCII, so grep and the regex crate read the same bytes")]
    NotAscii,
    #[error("escape `\\{0}` is read differently by `grep -E` and the regex crate")]
    Escape(char),
    #[error("a backslash inside `[...]` is literal to `grep -E` and an escape to the regex crate")]
    BackslashInBracket,
    #[error("inline flags `(?` are not ERE")]
    InlineFlags,
    #[error("lazy quantifier `{0}?` is not ERE")]
    Lazy(char),
    #[error("unterminated `[...]`")]
    UnterminatedBracket,
    #[error("matches the empty line, so every write would hit")]
    MatchesEmpty,
    #[error("does not compile: {0}")]
    Compile(String),
}

/// Why a glob is refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GlobProblem {
    #[error("must not be empty")]
    Empty,
    #[error("has an empty segment (a leading, trailing or doubled `/`)")]
    EmptySegment,
    #[error("`**` must be a whole segment")]
    PartialDoubleStar,
    #[error(
        "character {0:?} is outside the glob alphabet (letters, digits, `.`, `_`, `-`, `*`, `?`, `/`)"
    )]
    Character(char),
}

/// `[a-z0-9][a-z0-9-]*`: the hook's file stem, and the `X` of `hook:X`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DetectorName(String);

impl DetectorName {
    /// A name from text: the same check a rule file's `name` passes.
    ///
    /// # Errors
    /// [`DetectorError::Name`] when the text is not `[a-z0-9][a-z0-9-]*` or is
    /// a reserved stem.
    pub fn new(s: &str) -> Result<Self, DetectorError> {
        Self::parse(s.to_owned())
    }

    fn parse(s: String) -> Result<Self, DetectorError> {
        let ok = s
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !Self::is_reserved(&s);
        if ok {
            Ok(DetectorName(s))
        } else {
            Err(DetectorError::Name(s))
        }
    }

    /// A file stem another generated script already takes in the directory
    /// the hooks install into: hooks-pack's fixture runner, and the load-log
    /// pack's hook and runner, which install beside the detectors; and the
    /// repo pack's runner and commit dispatcher, which sit beside the gates.
    #[must_use]
    pub fn is_reserved(stem: &str) -> bool {
        matches!(
            stem,
            "test-detectors"
                | "relearn-load-log"
                | "test-load-log"
                | "test-gates"
                | "relearn-pre-commit"
        )
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A path glob over a closed alphabet, matched against a **component-aligned
/// suffix** of a path, so `src/**/*.rs` matches an absolute path from any
/// checkout.
///
/// The authoring form only. What is evaluated — here and in the generated hook —
/// is the one ERE [`Glob::to_ere`] produces, so path scope cannot mean two
/// things in two places.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glob(String);

impl Glob {
    /// Parse a glob, refusing anything outside the alphabet.
    ///
    /// # Errors
    /// [`GlobProblem`] naming the first defect.
    pub fn parse(s: &str) -> Result<Self, GlobProblem> {
        if s.is_empty() {
            return Err(GlobProblem::Empty);
        }
        for segment in s.split('/') {
            if segment.is_empty() {
                return Err(GlobProblem::EmptySegment);
            }
            if segment != "**" && segment.contains("**") {
                return Err(GlobProblem::PartialDoubleStar);
            }
        }
        if let Some(bad) = s.chars().find(|c| {
            !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '*' | '?' | '/'))
        }) {
            return Err(GlobProblem::Character(bad));
        }
        Ok(Glob(s.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The ERE this glob means: `(^|/)` + the translated segments + `$`.
    ///
    /// `**` before another segment is `([^/]+/)*` (zero or more whole
    /// components); `**` last is `.*`; `*` is `[^/]*`; `?` is `[^/]`; `.` is
    /// escaped. Nothing else needs escaping, because nothing else is admitted.
    #[must_use]
    pub fn to_ere(&self) -> String {
        let segments: Vec<&str> = self.0.split('/').collect();
        let mut out = String::from("(^|/)");
        for (i, segment) in segments.iter().enumerate() {
            let last = i + 1 == segments.len();
            if *segment == "**" {
                out.push_str(if last { ".*" } else { "([^/]+/)*" });
                continue;
            }
            for c in segment.chars() {
                match c {
                    '*' => out.push_str("[^/]*"),
                    '?' => out.push_str("[^/]"),
                    '.' => out.push_str("\\."),
                    c => out.push(c),
                }
            }
            if !last {
                out.push('/');
            }
        }
        out.push('$');
        out
    }

    /// Whether this glob matches `path` (already forward-slash).
    #[must_use]
    pub fn matches(&self, path: &str) -> bool {
        compile(&self.to_ere()).is_some_and(|re| re.is_match(path.as_bytes()))
    }
}

/// The globs every **content** detector excludes without being told:
/// rule files (which hold patterns and inputs that must hit them), the packs
/// relearn generates that quote patterns and fixtures, and the directory the
/// hook scripts are installed into (`[R:detector-excludes-own-definitions]`).
///
/// Not applied to `sibling-case`, which reads no text and so cannot match its
/// own definition — excluding `rules/` there would only stop it catching a real
/// collision among rule files.
pub const SELF_EXCLUDES: &[&str] = &[
    "**/rules/*.md",
    "**/hooks-pack/**",
    "**/repo-pack/**",
    "**/hooks/relearn/**",
];

/// Which files a detector reads: authored globs plus, for a content engine,
/// [`SELF_EXCLUDES`].
#[derive(Debug, Clone)]
pub struct PathScope {
    files: Vec<Glob>,
    exclude: Vec<Glob>,
    implicit: Vec<Glob>,
    files_re: Regex,
    exclude_re: Option<Regex>,
}

impl PartialEq for PathScope {
    fn eq(&self, other: &Self) -> bool {
        // The compiled forms are functions of the globs, so the globs decide.
        self.files == other.files
            && self.exclude == other.exclude
            && self.implicit == other.implicit
    }
}

impl Eq for PathScope {}

impl PathScope {
    fn new(
        files: Vec<Glob>,
        exclude: Vec<Glob>,
        implicit: Vec<Glob>,
    ) -> Result<Self, DetectorError> {
        if files.is_empty() {
            return Err(DetectorError::NoFiles);
        }
        let files_re = compile(&alternation(&files)).ok_or(DetectorError::NoFiles)?;
        let all_excludes: Vec<Glob> = exclude.iter().chain(&implicit).cloned().collect(); // allow:clone: two short glob lists joined once at parse, so the compiled ERE owns its inputs
        let exclude_re = if all_excludes.is_empty() {
            None
        } else {
            Some(compile(&alternation(&all_excludes)).ok_or(DetectorError::NoFiles)?)
        };
        Ok(PathScope {
            files,
            exclude,
            implicit,
            files_re,
            exclude_re,
        })
    }

    /// The authored `files` globs.
    #[must_use]
    pub fn files(&self) -> &[Glob] {
        &self.files
    }

    /// The authored `exclude` globs, without the implicit ones.
    #[must_use]
    pub fn exclude(&self) -> &[Glob] {
        &self.exclude
    }

    /// The one ERE for `files`.
    #[must_use]
    pub fn files_ere(&self) -> String {
        alternation(&self.files)
    }

    /// The one ERE for every exclusion, authored and implicit; `None` if there
    /// is none.
    #[must_use]
    pub fn exclude_ere(&self) -> Option<String> {
        let all: Vec<&Glob> = self.exclude.iter().chain(&self.implicit).collect();
        if all.is_empty() {
            None
        } else {
            Some(
                all.iter()
                    .map(|g| format!("({})", g.to_ere()))
                    .collect::<Vec<_>>()
                    .join("|"),
            )
        }
    }

    fn admits(&self, path: &str) -> bool {
        let path = path.as_bytes();
        self.files_re.is_match(path)
            && !self.exclude_re.as_ref().is_some_and(|re| re.is_match(path))
    }
}

fn alternation(globs: &[Glob]) -> String {
    globs
        .iter()
        .map(|g| format!("({})", g.to_ere()))
        .collect::<Vec<_>>()
        .join("|")
}

fn compile(ere: &str) -> Option<Regex> {
    RegexBuilder::new(ere).unicode(false).build().ok()
}

/// A line pattern in the dialect `grep -E` and the regex crate share.
#[derive(Debug, Clone)]
pub struct Pattern {
    source: String,
    compiled: Regex,
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}

impl Eq for Pattern {}

impl Pattern {
    fn parse(source: String) -> Result<Self, DetectorError> {
        let refuse = |problem| DetectorError::Pattern {
            pattern: source.clone(), // allow:clone: the refused pattern is owned by the error on the failure path only
            problem,
        };
        dialect(&source).map_err(&refuse)?;
        let compiled = RegexBuilder::new(&source)
            .unicode(false)
            .build()
            .map_err(|e| refuse(PatternProblem::Compile(e.to_string())))?;
        if compiled.is_match(b"") {
            return Err(refuse(PatternProblem::MatchesEmpty));
        }
        Ok(Pattern { source, compiled })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.source
    }
}

/// Refuse every construct outside the shared ERE subset.
///
/// An allow-list for escapes, and a bracket tracker so a backslash inside
/// `[...]` — literal to POSIX, an escape to Rust — is caught wherever it is.
fn dialect(p: &str) -> Result<(), PatternProblem> {
    if p.is_empty() {
        return Err(PatternProblem::Empty);
    }
    if !p.is_ascii() {
        return Err(PatternProblem::NotAscii);
    }
    let b = p.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' => {
                let next = *b.get(i + 1).ok_or(PatternProblem::Escape('\\'))?;
                if !b".[](){}*+?|^$\\b".contains(&next) {
                    return Err(PatternProblem::Escape(char::from(next)));
                }
                i += 2;
            }
            b'[' => i = bracket_end(b, i)?,
            b'(' if b.get(i + 1) == Some(&b'?') => return Err(PatternProblem::InlineFlags),
            q @ (b'*' | b'+' | b'?' | b'}') if b.get(i + 1) == Some(&b'?') => {
                return Err(PatternProblem::Lazy(char::from(q)));
            }
            _ => i += 1,
        }
    }
    Ok(())
}

/// The index just past the bracket expression opening at `open`.
fn bracket_end(b: &[u8], open: usize) -> Result<usize, PatternProblem> {
    let mut i = open + 1;
    if b.get(i) == Some(&b'^') {
        i += 1;
    }
    if b.get(i) == Some(&b']') {
        i += 1; // a leading `]` is a literal member
    }
    while i < b.len() {
        match b[i] {
            b'\\' => return Err(PatternProblem::BackslashInBracket),
            b'[' if b.get(i + 1) == Some(&b':') => {
                let close = b[i + 2..]
                    .windows(2)
                    .position(|w| w == b":]")
                    .ok_or(PatternProblem::UnterminatedBracket)?;
                i += 2 + close + 2;
            }
            b']' => return Ok(i + 1),
            _ => i += 1,
        }
    }
    Err(PatternProblem::UnterminatedBracket)
}

/// A single-line, non-empty token: a comment prefix or an override marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token(String);

impl Token {
    fn parse(s: String, field: &'static str) -> Result<Self, DetectorError> {
        if s.trim().is_empty() || s.contains('\n') || s.contains('\r') {
            Err(DetectorError::Token { field })
        } else {
            Ok(Token(s))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A Markdown ATX heading level, 1 to 6: the `#` count that opens a section.
/// Parsed from the file's integer wide, then range-checked, so `0` and `300`
/// are named as out of range rather than lost in a narrower type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeadingLevel(u8);

impl HeadingLevel {
    fn parse(raw: i64) -> Result<Self, DetectorError> {
        match u8::try_from(raw) {
            Ok(level @ 1..=6) => Ok(HeadingLevel(level)),
            _ => Err(DetectorError::HeadingLevel(raw)),
        }
    }

    /// The level, 1 to 6.
    #[must_use]
    pub fn get(self) -> u8 {
        self.0
    }
}

/// The level of an ATX heading line (CR already removed): one to six `#` and
/// then a space, a tab or the end of the line. `None` for any other line. The
/// generated `awk` judge reads the same shape.
fn heading_of(line: &str) -> Option<u8> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    let after = line.as_bytes().get(hashes);
    let level = u8::try_from(hashes).ok().filter(|n| (1..=6).contains(n))?;
    matches!(after, None | Some(b' ' | b'\t')).then_some(level)
}

/// Whether some section opened by a heading of `level` has no line matching
/// `require`. A section runs to the next heading of that level or shallower;
/// a deeper heading is part of it. A heading inside a fenced block (a line
/// starting with three backticks or three tildes) opens nothing. The heading
/// line itself never satisfies its section.
fn section_lacks(text: &str, level: HeadingLevel, require: &Regex) -> bool {
    let mut in_fence = false;
    let mut open: Option<bool> = None;
    let mut lacking = false;
    for line in text.split('\n') {
        let bare = line.strip_suffix('\r').unwrap_or(line);
        let fence = bare.starts_with("```") || bare.starts_with("~~~");
        if !in_fence
            && !fence
            && let Some(n) = heading_of(bare).filter(|n| *n <= level.get())
        {
            lacking |= open == Some(false);
            open = (n == level.get()).then_some(false);
            continue;
        }
        if fence {
            in_fence = !in_fence;
        }
        if let Some(satisfied) = open.as_mut() {
            *satisfied |= require.is_match(line.as_bytes());
        }
    }
    lacking || open == Some(false)
}

/// What the detector looks for, once the path is in scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Engine {
    /// A line of the written text matches `pattern`, is not a comment line
    /// (first non-blank characters are `comment`), and does not carry `allow`.
    LineRegex {
        pattern: Pattern,
        comment: Option<Token>,
        allow: Option<Token>,
    },
    /// A `Write` creates a file whose name differs from an existing sibling's
    /// only by ASCII case. Reads names, never text.
    SiblingCase,
    /// A staged path is **exactly** one of the entries on a machine-local
    /// list. The list is never in the rule: the generated dispatcher reads it
    /// from the path a clone's `git config --local relearn.<name>.list` names,
    /// and a fixture carries a synthetic one. Reads paths, never text.
    ListedPath,
    /// A **whole file**: every section opened by a heading of `heading` must
    /// contain a line matching `require` ([`section_lacks`]). It needs the
    /// whole text, so at write time it reads a `Write`'s content before, and
    /// an edited file from disk after (`claude-code-post-tool-use`).
    Sections {
        heading: HeadingLevel,
        require: Pattern,
    },
}

impl Engine {
    /// The spelling a rule file uses.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Engine::LineRegex { .. } => "line-regex",
            Engine::SiblingCase => "sibling-case",
            Engine::ListedPath => "listed-path",
            Engine::Sections { .. } => "sections",
        }
    }

    /// The globs this engine excludes without being told: a content engine
    /// excludes [`SELF_EXCLUDES`].
    fn implicit_excludes(&self) -> Vec<Glob> {
        match self {
            Engine::LineRegex { .. } | Engine::Sections { .. } => SELF_EXCLUDES
                .iter()
                .map(|g| Glob(String::from(*g)))
                .collect(),
            Engine::SiblingCase | Engine::ListedPath => Vec::new(),
        }
    }

    /// Whether this engine can read what `event` hands it. `sibling-case`
    /// lists a directory on disk, which a commit does not change; `listed-path`
    /// guards what enters history, which a write does not do; after a tool
    /// call only a whole-file engine has anything an earlier event lacked.
    fn runs_at(&self, event: Event) -> bool {
        match (self, event) {
            (Engine::Sections { .. }, _)
            | (Engine::LineRegex { .. }, Event::ClaudeCodePreToolUse | Event::GitPreCommit)
            | (Engine::SiblingCase, Event::ClaudeCodePreToolUse)
            | (Engine::ListedPath, Event::GitPreCommit) => true,
            (Engine::LineRegex { .. }, Event::ClaudeCodePostToolUse)
            | (Engine::SiblingCase, Event::GitPreCommit | Event::ClaudeCodePostToolUse)
            | (Engine::ListedPath, Event::ClaudeCodePreToolUse | Event::ClaudeCodePostToolUse) => {
                false
            }
        }
    }

    /// The events this engine accepts, for the refusal when one is not.
    fn accepted_events(&self) -> &'static str {
        match self {
            Engine::LineRegex { .. } => "claude-code-pre-tool-use or git-pre-commit",
            Engine::SiblingCase => "claude-code-pre-tool-use only",
            Engine::ListedPath => "git-pre-commit only",
            Engine::Sections { .. } => {
                "claude-code-pre-tool-use, claude-code-post-tool-use or git-pre-commit"
            }
        }
    }
}

/// When a detector runs. A closed set: an event that is not built is
/// unrepresentable rather than accepted and ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    /// Claude Code `PreToolUse`, on the file tools the engine reads.
    ClaudeCodePreToolUse,
    /// Claude Code `PostToolUse` on `Edit` and `MultiEdit`: the edited file,
    /// read back from disk. The edit has happened, so a hit here only ever
    /// warns, whatever the detector's action.
    ClaudeCodePostToolUse,
    /// A repository's own `pre-commit`, through the generated dispatcher: the
    /// staged paths, and for `line-regex` the staged content, read from the
    /// index rather than the working tree.
    GitPreCommit,
}

impl Event {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Event::ClaudeCodePreToolUse => "claude-code-pre-tool-use",
            Event::ClaudeCodePostToolUse => "claude-code-post-tool-use",
            Event::GitPreCommit => "git-pre-commit",
        }
    }

    fn parse(s: &str) -> Result<Self, DetectorError> {
        match s {
            "claude-code-pre-tool-use" => Ok(Event::ClaudeCodePreToolUse),
            "claude-code-post-tool-use" => Ok(Event::ClaudeCodePostToolUse),
            "git-pre-commit" => Ok(Event::GitPreCommit),
            other => Err(DetectorError::UnknownEvent(other.to_owned())),
        }
    }
}

/// What a hit does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Refuse the tool call; the message is the reason Claude reads.
    Block,
    /// Allow it, and put the message in front of the model beside the result.
    Warn,
}

impl Action {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Action::Block => "block",
            Action::Warn => "warn",
        }
    }

    fn parse(s: &str) -> Result<Self, DetectorError> {
        match s {
            "block" => Ok(Action::Block),
            "warn" => Ok(Action::Warn),
            other => Err(DetectorError::UnknownAction(other.to_owned())),
        }
    }
}

/// A relative, forward-slash fixture path with no `..`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixturePath(String);

impl FixturePath {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What a fixture feeds its engine. Which variant is legal is decided by the
/// engine, at parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureInput {
    /// The text written, for `line-regex`.
    Text(String),
    /// The names already in the target's directory, for `sibling-case`.
    Siblings(Vec<String>),
    /// A synthetic machine-local list, for `listed-path`: relative
    /// forward-slash paths, one entry each. Never the real list.
    Listed(Vec<String>),
}

/// One synthetic write and the verdict it must produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixture {
    path: FixturePath,
    input: FixtureInput,
    expect: Detection,
}

impl Fixture {
    #[must_use]
    pub fn path(&self) -> &FixturePath {
        &self.path
    }

    #[must_use]
    pub fn input(&self) -> &FixtureInput {
        &self.input
    }

    #[must_use]
    pub fn expect(&self) -> Detection {
        self.expect
    }

    fn subject(&self) -> Subject<'_> {
        match &self.input {
            FixtureInput::Text(text) => Subject::new(&self.path.0, text, &[]),
            FixtureInput::Siblings(names) => Subject::new(&self.path.0, "", names),
            FixtureInput::Listed(list) => Subject::new(&self.path.0, "", &[]).with_listed(list),
        }
    }
}

/// One write or one staged path, as a detector sees it.
#[derive(Debug, Clone, Copy)]
pub struct Subject<'a> {
    path: &'a str,
    text: &'a str,
    siblings: &'a [String],
    listed: &'a [String],
}

impl<'a> Subject<'a> {
    /// A write of `text` to `path`, beside `siblings` (names, not paths),
    /// with no machine-local list: `listed-path` is clean on it.
    #[must_use]
    pub fn new(path: &'a str, text: &'a str, siblings: &'a [String]) -> Self {
        Subject {
            path,
            text,
            siblings,
            listed: &[],
        }
    }

    /// The same subject, judged against `listed`, the entries of a
    /// machine-local list.
    #[must_use]
    pub fn with_listed(self, listed: &'a [String]) -> Self {
        Subject { listed, ..self }
    }
}

/// A detector whose fixtures have all been run and agree with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detector {
    name: DetectorName,
    engine: Engine,
    scope: PathScope,
    events: Vec<Event>,
    action: Action,
    message: String,
    fixtures: Vec<Fixture>,
}

impl Detector {
    #[must_use]
    pub fn name(&self) -> &DetectorName {
        &self.name
    }

    #[must_use]
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    #[must_use]
    pub fn scope(&self) -> &PathScope {
        &self.scope
    }

    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Whether this detector runs at `event`: which emitters compile it.
    #[must_use]
    pub fn runs_at(&self, event: Event) -> bool {
        self.events.contains(&event)
    }

    /// Whether the `hooks` target compiles this detector to a script: it runs
    /// at a Claude Code event, before a tool call or after one. The controls
    /// map, `lint` and the pack's README all ask this one question.
    #[must_use]
    pub fn has_hook_script(&self) -> bool {
        self.runs_at(Event::ClaudeCodePreToolUse) || self.runs_at(Event::ClaudeCodePostToolUse)
    }

    #[must_use]
    pub fn action(&self) -> Action {
        self.action
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Every fixture, in file order. At least one expects `hit` and one
    /// expects `clean`, and every one agrees with [`Detector::evaluate`].
    #[must_use]
    pub fn fixtures(&self) -> &[Fixture] {
        &self.fixtures
    }

    /// Decide one write.
    #[must_use]
    pub fn evaluate(&self, subject: &Subject<'_>) -> Detection {
        let path = subject.path.replace('\\', "/");
        if !self.scope.admits(&path) {
            return Detection::Clean;
        }
        let hit = match &self.engine {
            Engine::LineRegex {
                pattern,
                comment,
                allow,
            } => subject.text.split('\n').any(|line| {
                pattern.compiled.is_match(line.as_bytes())
                    && !comment.as_ref().is_some_and(|c| {
                        line.trim_start_matches([' ', '\t', '\x0b', '\x0c', '\r'])
                            .starts_with(c.as_str())
                    })
                    && !allow.as_ref().is_some_and(|a| line.contains(a.as_str()))
            }),
            Engine::SiblingCase => {
                let base = path.rsplit('/').next().unwrap_or(&path);
                !subject.siblings.iter().any(|s| s == base)
                    && subject
                        .siblings
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(base))
            }
            Engine::ListedPath => subject.listed.contains(&path),
            Engine::Sections { heading, require } => {
                section_lacks(subject.text, *heading, &require.compiled)
            }
        };
        if hit {
            Detection::Hit
        } else {
            Detection::Clean
        }
    }
}

/// The `[[detector]]` table as a file holds it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawDetector {
    name: String,
    engine: String,
    pattern: Option<String>,
    comment: Option<String>,
    allow: Option<String>,
    heading: Option<i64>,
    require: Option<String>,
    always: Option<Vec<String>>,
    lint: Option<String>,
    groups: Option<Vec<String>>,
    default: Option<String>,
    files: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
    events: Vec<String>,
    action: String,
    message: String,
    #[serde(default)]
    fixture: Vec<RawFixture>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFixture {
    path: Option<String>,
    input: Option<String>,
    siblings: Option<Vec<String>>,
    list: Option<Vec<String>>,
    records: Option<Vec<String>>,
    expect: String,
}

/// A `[[detector]]` table, parsed: a write detector, or a repository gate.
/// The engine decides which, so one table shape authors both.
#[derive(Debug)]
pub(crate) enum Parsed {
    Detector(Detector),
    Gate(Gate),
}

/// Every glob in a list, or the first refusal.
fn globs(list: Vec<String>) -> Result<Vec<Glob>, DetectorError> {
    list.into_iter()
        .map(|g| Glob::parse(&g).map_err(|problem| DetectorError::Glob { glob: g, problem }))
        .collect()
}

/// A relative, forward-slash path with no `..` and no drive: a fixture's
/// path, and a gate record's path field.
fn fixture_path_ok(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains(':')
        && path.split('/').all(|s| !s.is_empty() && s != "..")
}

impl RawDetector {
    /// The name as written, for an error raised before it is validated.
    pub(crate) fn raw_name(&self) -> &str {
        &self.name
    }

    /// Parse, then run every fixture. A detector or a gate exists only if
    /// they agree.
    pub(crate) fn into_parsed(self) -> Result<Parsed, DetectorError> {
        let name = DetectorName::parse(self.name.clone())?; // allow:clone: the gate path below consumes the raw table whole, so the name is taken by value first
        match self.engine.as_str() {
            "eol-attribute" | "root-documents" | "lint-enabled" => {
                gate::gate(self, name).map(Parsed::Gate)
            }
            _ => self.into_detector(name).map(Parsed::Detector),
        }
    }

    fn into_detector(self, name: DetectorName) -> Result<Detector, DetectorError> {
        for (field, present) in [
            ("always", self.always.is_some()),
            ("lint", self.lint.is_some()),
            ("groups", self.groups.is_some()),
            ("default", self.default.is_some()),
        ] {
            if present {
                return Err(DetectorError::FieldNotUsed {
                    engine: "a write engine",
                    field,
                });
            }
        }
        if self.engine != "sections" {
            for (field, present) in [
                ("heading", self.heading.is_some()),
                ("require", self.require.is_some()),
            ] {
                if present {
                    let engine = match self.engine.as_str() {
                        "line-regex" => "line-regex",
                        "sibling-case" => "sibling-case",
                        "listed-path" => "listed-path",
                        other => return Err(DetectorError::UnknownEngine(other.to_owned())),
                    };
                    return Err(DetectorError::FieldNotUsed { engine, field });
                }
            }
        }
        let engine = match self.engine.as_str() {
            "sections" => {
                for (field, present) in [
                    ("pattern", self.pattern.is_some()),
                    ("comment", self.comment.is_some()),
                    ("allow", self.allow.is_some()),
                ] {
                    if present {
                        return Err(DetectorError::FieldNotUsed {
                            engine: "sections",
                            field,
                        });
                    }
                }
                let heading = self.heading.ok_or(DetectorError::FieldRequired {
                    engine: "sections",
                    field: "heading",
                })?;
                let require = self.require.ok_or(DetectorError::FieldRequired {
                    engine: "sections",
                    field: "require",
                })?;
                Engine::Sections {
                    heading: HeadingLevel::parse(heading)?,
                    require: Pattern::parse(require)?,
                }
            }
            "line-regex" => {
                let pattern = self.pattern.ok_or(DetectorError::FieldRequired {
                    engine: "line-regex",
                    field: "pattern",
                })?;
                Engine::LineRegex {
                    pattern: Pattern::parse(pattern)?,
                    comment: self
                        .comment
                        .map(|c| Token::parse(c, "comment"))
                        .transpose()?,
                    allow: self.allow.map(|a| Token::parse(a, "allow")).transpose()?,
                }
            }
            "sibling-case" | "listed-path" => {
                let engine = if self.engine == "sibling-case" {
                    Engine::SiblingCase
                } else {
                    Engine::ListedPath
                };
                for (field, present) in [
                    ("pattern", self.pattern.is_some()),
                    ("comment", self.comment.is_some()),
                    ("allow", self.allow.is_some()),
                ] {
                    if present {
                        return Err(DetectorError::FieldNotUsed {
                            engine: engine.as_str(),
                            field,
                        });
                    }
                }
                engine
            }
            other => return Err(DetectorError::UnknownEngine(other.to_owned())),
        };
        let scope = PathScope::new(
            globs(self.files)?,
            globs(self.exclude)?,
            engine.implicit_excludes(),
        )?;

        let mut events: Vec<Event> = Vec::new();
        for raw in &self.events {
            if raw == "repository-gate" {
                return Err(DetectorError::EventNotForEngine {
                    engine: engine.as_str(),
                    expected: "a write event, never repository-gate",
                });
            }
            let event = Event::parse(raw)?;
            if !engine.runs_at(event) {
                return Err(DetectorError::EventNotForEngine {
                    engine: engine.as_str(),
                    expected: engine.accepted_events(),
                });
            }
            if events.contains(&event) {
                return Err(DetectorError::DuplicateEvent(raw.clone())); // allow:clone: the duplicate's spelling is owned by the error on the failure path only
            }
            events.push(event);
        }
        if events.is_empty() {
            return Err(DetectorError::NoEvents);
        }
        let action = Action::parse(&self.action)?;
        if self.message.trim().is_empty() {
            return Err(DetectorError::EmptyMessage);
        }

        let fixtures = self
            .fixture
            .into_iter()
            .enumerate()
            .map(|(i, raw)| raw.into_fixture(i + 1, &engine))
            .collect::<Result<Vec<_>, _>>()?;
        if !fixtures.iter().any(|f| f.expect == Detection::Hit) {
            return Err(DetectorError::NoHitFixture);
        }
        if !fixtures.iter().any(|f| f.expect == Detection::Clean) {
            return Err(DetectorError::NoCleanFixture);
        }

        let detector = Detector {
            name,
            engine,
            scope,
            events,
            action,
            message: self.message,
            fixtures,
        };
        for (i, fixture) in detector.fixtures.iter().enumerate() {
            let got = detector.evaluate(&fixture.subject());
            if got != fixture.expect {
                return Err(DetectorError::FixtureDisagrees {
                    index: i + 1,
                    path: fixture.path.0.clone(), // allow:clone: owned by the error on the failure path only
                    expected: fixture.expect,
                    got,
                });
            }
        }
        Ok(detector)
    }
}

impl RawFixture {
    fn into_fixture(self, index: usize, engine: &Engine) -> Result<Fixture, DetectorError> {
        let mismatch = || DetectorError::FixtureInputMismatch {
            index,
            engine: engine.as_str(),
        };
        if self.records.is_some() {
            return Err(mismatch());
        }
        let path = self.path.ok_or_else(mismatch)?;
        if !fixture_path_ok(&path) {
            return Err(DetectorError::FixturePath { index, path });
        }
        let input = match (engine, self.input, self.siblings, self.list) {
            (Engine::LineRegex { .. } | Engine::Sections { .. }, Some(text), None, None) => {
                FixtureInput::Text(text)
            }
            (Engine::SiblingCase, None, Some(names), None) => FixtureInput::Siblings(names),
            (Engine::ListedPath, None, None, Some(list)) => {
                if let Some(bad) = list.iter().find(|entry| !fixture_path_ok(entry)) {
                    return Err(DetectorError::ListEntry {
                        index,
                        entry: bad.clone(), // allow:clone: owned by the error on the failure path only
                    });
                }
                FixtureInput::Listed(list)
            }
            (
                Engine::LineRegex { .. }
                | Engine::SiblingCase
                | Engine::ListedPath
                | Engine::Sections { .. },
                _,
                _,
                _,
            ) => {
                return Err(mismatch());
            }
        };
        Ok(Fixture {
            path: FixturePath(path),
            input,
            expect: Detection::parse(&self.expect)?,
        })
    }
}
