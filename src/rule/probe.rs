//! `[probe]` — the replayed half of a rule: a task that invites the rule's error
//! class, and a checker that says whether one session committed it.
//!
//! A probe measures **prose**. `scripts/probe-run.sh` runs the task with the
//! rule's home layer and without it, and the difference in violation rates is
//! the only direct evidence of whether the instruction does anything. This
//! module owns what a probe *is* and how one session is judged; it never starts
//! a model, reads a file, or runs a process (`tests/solo_mode.rs`).
//!
//! **Held at construction, not checked later:**
//!
//! * The checker is one of a closed set, and the set mirrors what the triage
//!   showed a checker must read: the session's events in order ([`Checker::Trace`]),
//!   the final workspace against the fixture ([`Checker::Diff`]), or the result of
//!   running what was written ([`Checker::Run`]). A model-judged rubric is named
//!   and refused as not built, the way `git-pre-commit` was for detectors
//!   until 2026-11-01.
//! * Every pattern compiles, so a probe that could never match is a parse error.
//! * A `diff` clause naming a detector names one of **this rule's** detectors.
//! * The task is authored text. Nothing here reads `incident`, and no emitter
//!   reads a probe (`no_emitter_reads_a_probe`), so the task cannot reach the
//!   instruction layer the WITH arm loads.
//!
//! A probe's checker also carries a held case and a violated case on disk, and
//! `probe-run.sh --self-test` runs both through it (`[R:evidence-must-discriminate]`):
//! a checker that cannot say `violated` about a known-bad session measures nothing.

use std::fmt;

use regex::bytes::{Regex, RegexBuilder};
use serde::Deserialize;

use super::{Detection, Detector, DetectorName, Subject};

/// What one replayed session is judged to have done.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verdict {
    /// The session did the task and did not commit the error class.
    Held,
    /// The session did the task and committed the error class.
    Violated,
    /// The session never reached the point the checker reads — it did not do
    /// the task. Reported, and never counted as held: a session that wrote
    /// nothing has not shown that the rule holds.
    Inconclusive,
}

impl Verdict {
    /// The spelling the runner and the results file use.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Held => "held",
            Verdict::Violated => "violated",
            Verdict::Inconclusive => "inconclusive",
        }
    }

    /// Parse a recorded verdict. The inverse of [`Verdict::as_str`].
    ///
    /// # Errors
    /// [`ProbeError::UnknownVerdict`] for any other spelling.
    pub fn parse(s: &str) -> Result<Self, ProbeError> {
        match s {
            "held" => Ok(Verdict::Held),
            "violated" => Ok(Verdict::Violated),
            "inconclusive" => Ok(Verdict::Inconclusive),
            other => Err(ProbeError::UnknownVerdict(other.to_owned())),
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Every way a `[probe]` table can be refused. Each names what failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProbeError {
    #[error("`task` must not be empty")]
    EmptyTask,
    #[error("unknown `workspace` {0:?} (expected fixture | empty)")]
    UnknownWorkspace(String),
    #[error("unknown `checker.kind` {0:?} (expected trace | diff | run)")]
    UnknownChecker(String),
    #[error(
        "checker `rubric` is named and not built: a model-judged checker must name the \
         strongest model and carry pass and fail fixtures before it can be used"
    )]
    RubricNotBuilt,
    #[error("checker `{kind}` requires field `{field}`")]
    FieldRequired {
        kind: &'static str,
        field: &'static str,
    },
    #[error("checker `{kind}` has no field `{field}`")]
    FieldNotAllowed {
        kind: &'static str,
        field: &'static str,
    },
    #[error("unknown event {0:?} (expected shell | write | read | text)")]
    UnknownEvent(String),
    #[error("unknown `window` {0:?} (expected before-first | after-last)")]
    UnknownWindow(String),
    #[error("pattern {pattern:?} does not compile: {reason}")]
    Pattern { pattern: String, reason: String },
    #[error("checker `diff` needs at least one `[[probe.checker.require]]` clause")]
    NoClause,
    #[error("a `require` clause must set exactly one of changed | contains | lacks | detector")]
    ClauseShape,
    #[error("a `contains` or `lacks` clause requires field `file`, and no other clause takes one")]
    ClauseFile,
    #[error("`file` {0:?} must be a relative path with `/` separators and no `..`")]
    ClausePath(String),
    #[error("clause names detector `{0}`, which this rule does not carry")]
    UnknownDetector(String),
    #[error("unknown verdict {0:?} (expected held | violated | inconclusive)")]
    UnknownVerdict(String),
}

/// The instruction given to the model, verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeTask(String);

impl ProbeTask {
    fn parse(raw: String) -> Result<Self, ProbeError> {
        if raw.trim().is_empty() {
            return Err(ProbeError::EmptyTask);
        }
        Ok(ProbeTask(raw))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What the session's directory starts with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Workspace {
    /// `tests/probes/<tag-body>/workspace/`, copied in.
    Fixture,
    /// Nothing but what the arm adds.
    Empty,
}

impl Workspace {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Workspace::Fixture => "fixture",
            Workspace::Empty => "empty",
        }
    }

    fn parse(s: &str) -> Result<Self, ProbeError> {
        match s {
            "fixture" => Ok(Workspace::Fixture),
            "empty" => Ok(Workspace::Empty),
            other => Err(ProbeError::UnknownWorkspace(other.to_owned())),
        }
    }
}

/// A compiled pattern that remembers its spelling, so it round-trips and
/// compares by what the rule file says.
#[derive(Debug, Clone)]
pub struct ProbePattern {
    source: String,
    compiled: Regex,
}

impl PartialEq for ProbePattern {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}

impl Eq for ProbePattern {}

impl ProbePattern {
    /// Compile `source` as a `regex` pattern over bytes, Unicode off — the
    /// same build the detectors use, so `(?i)` and `(?s)` mean ASCII folding
    /// and dot-matches-newline. Unlike a detector's, a probe's pattern never
    /// runs in `grep`, so it is not held to the shared ERE subset.
    ///
    /// # Errors
    /// [`ProbeError::Pattern`] when it does not compile.
    pub fn parse(source: impl Into<String>) -> Result<Self, ProbeError> {
        let source = source.into();
        match RegexBuilder::new(&source).unicode(false).build() {
            Ok(compiled) => Ok(ProbePattern { source, compiled }),
            Err(e) => Err(ProbeError::Pattern {
                reason: e.to_string(),
                pattern: source,
            }),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.source
    }

    #[must_use]
    pub fn is_match(&self, text: &str) -> bool {
        self.compiled.is_match(text.as_bytes())
    }
}

/// What kind of thing in a session an event pattern is matched against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// A shell command the model ran (Bash, or PowerShell where offered).
    Shell,
    /// A file the model wrote or edited, by path.
    Write,
    /// A file the model read, by path.
    Read,
    /// Text the model said.
    Text,
}

impl EventKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            EventKind::Shell => "shell",
            EventKind::Write => "write",
            EventKind::Read => "read",
            EventKind::Text => "text",
        }
    }

    /// # Errors
    /// [`ProbeError::UnknownEvent`] for any other spelling.
    pub fn parse(s: &str) -> Result<Self, ProbeError> {
        match s {
            "shell" => Ok(EventKind::Shell),
            "write" => Ok(EventKind::Write),
            "read" => Ok(EventKind::Read),
            "text" => Ok(EventKind::Text),
            other => Err(ProbeError::UnknownEvent(other.to_owned())),
        }
    }
}

/// One thing that happened in a session, in order. A write's or read's
/// subject is its path with `/` separators, a shell event's is the command,
/// and a text event's is what was said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEvent {
    kind: EventKind,
    subject: String,
}

impl SessionEvent {
    #[must_use]
    pub fn new(kind: EventKind, subject: impl Into<String>) -> Self {
        SessionEvent {
            kind,
            subject: subject.into(),
        }
    }

    #[must_use]
    pub fn kind(&self) -> EventKind {
        self.kind
    }

    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

/// A kind of event and the pattern its subject must match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventPattern {
    kind: EventKind,
    pattern: ProbePattern,
}

impl EventPattern {
    #[must_use]
    pub fn kind(&self) -> EventKind {
        self.kind
    }

    #[must_use]
    pub fn pattern(&self) -> &ProbePattern {
        &self.pattern
    }

    fn matches(&self, event: &SessionEvent) -> bool {
        event.kind == self.kind && self.pattern.is_match(&event.subject)
    }
}

/// Where, relative to the anchor, the expected event must fall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    /// Before the first anchor event: a query before the claim.
    BeforeFirst,
    /// After the last anchor event: a verification after the final edit.
    AfterLast,
}

impl Window {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Window::BeforeFirst => "before-first",
            Window::AfterLast => "after-last",
        }
    }

    fn parse(s: &str) -> Result<Self, ProbeError> {
        match s {
            "before-first" => Ok(Window::BeforeFirst),
            "after-last" => Ok(Window::AfterLast),
            other => Err(ProbeError::UnknownWindow(other.to_owned())),
        }
    }
}

/// Reads the session's events in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceChecker {
    expect: EventPattern,
    window: Window,
    anchor: EventPattern,
}

impl TraceChecker {
    #[must_use]
    pub fn expect(&self) -> &EventPattern {
        &self.expect
    }

    #[must_use]
    pub fn window(&self) -> Window {
        self.window
    }

    #[must_use]
    pub fn anchor(&self) -> &EventPattern {
        &self.anchor
    }

    /// Judge one session. No anchor event means the task was not done.
    #[must_use]
    pub fn judge(&self, events: &[SessionEvent]) -> Verdict {
        let mut anchors = events
            .iter()
            .enumerate()
            .filter(|(_, e)| self.anchor.matches(e))
            .map(|(i, _)| i);
        let Some(first) = anchors.next() else {
            return Verdict::Inconclusive;
        };
        let last = anchors.next_back().unwrap_or(first);
        let held = events
            .iter()
            .enumerate()
            .filter(|(_, e)| self.expect.matches(e))
            .any(|(i, _)| match self.window {
                Window::BeforeFirst => i < first,
                Window::AfterLast => i > last,
            });
        if held {
            Verdict::Held
        } else {
            Verdict::Violated
        }
    }
}

/// A relative path inside a workspace, `/`-separated, with no `..`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePath(String);

impl WorkspacePath {
    fn parse(raw: String) -> Result<Self, ProbeError> {
        let bad = raw.is_empty()
            || raw.starts_with('/')
            || raw.contains('\\')
            || raw.contains(':')
            || raw.split('/').any(|c| c == ".." || c.is_empty());
        if bad {
            return Err(ProbeError::ClausePath(raw));
        }
        Ok(WorkspacePath(raw))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One condition a `diff` checker requires of the final workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffClause {
    /// Some changed file's path matches.
    Changed(ProbePattern),
    /// The file exists at the end and its text matches.
    Contains {
        file: WorkspacePath,
        pattern: ProbePattern,
    },
    /// The file, if it exists at the end, does not match.
    Lacks {
        file: WorkspacePath,
        pattern: ProbePattern,
    },
    /// None of the changed files is a hit for this rule's detector.
    Detector(DetectorName),
}

/// What a `diff` checker reads: the paths that differ from the fixture, and
/// the final text of every file in the workspace. Filled by `fsio`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceChange {
    changed: Vec<String>,
    files: Vec<(String, String)>,
}

impl WorkspaceChange {
    /// `changed` are relative `/` paths that were added, removed or edited;
    /// `files` is every file present at the end, with its text.
    #[must_use]
    pub fn new(changed: Vec<String>, files: Vec<(String, String)>) -> Self {
        WorkspaceChange { changed, files }
    }

    #[must_use]
    pub fn changed(&self) -> &[String] {
        &self.changed
    }

    fn text(&self, path: &str) -> Option<&str> {
        self.files
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, t)| t.as_str())
    }
}

/// Reads the final workspace against the fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffChecker {
    anchor: ProbePattern,
    require: Vec<DiffClause>,
}

impl DiffChecker {
    #[must_use]
    pub fn anchor(&self) -> &ProbePattern {
        &self.anchor
    }

    #[must_use]
    pub fn require(&self) -> &[DiffClause] {
        &self.require
    }

    /// Judge one session's final workspace. `detectors` is the rule's own;
    /// parsing guaranteed every clause's detector is among them.
    #[must_use]
    pub fn judge(&self, change: &WorkspaceChange, detectors: &[Detector]) -> Verdict {
        if !change.changed.iter().any(|p| self.anchor.is_match(p)) {
            return Verdict::Inconclusive;
        }
        let held = self
            .require
            .iter()
            .all(|clause| clause_holds(clause, change, detectors));
        if held {
            Verdict::Held
        } else {
            Verdict::Violated
        }
    }
}

fn clause_holds(clause: &DiffClause, change: &WorkspaceChange, detectors: &[Detector]) -> bool {
    match clause {
        DiffClause::Changed(p) => change.changed.iter().any(|c| p.is_match(c)),
        DiffClause::Contains { file, pattern } => change
            .text(file.as_str())
            .is_some_and(|t| pattern.is_match(t)),
        DiffClause::Lacks { file, pattern } => !change
            .text(file.as_str())
            .is_some_and(|t| pattern.is_match(t)),
        DiffClause::Detector(name) => {
            let no_siblings: [String; 0] = [];
            detectors.iter().filter(|d| d.name() == name).all(|d| {
                change.changed.iter().all(|path| {
                    let text = change.text(path).unwrap_or("");
                    d.evaluate(&Subject::new(path, text, &no_siblings)) == Detection::Clean
                })
            })
        }
    }
}

/// Runs `tests/probes/<tag-body>/check.sh` against the final workspace. The
/// script holds the adversarial input; this says, in prose, what it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunChecker {
    adversary: String,
}

impl RunChecker {
    /// What the script feeds the written artefact.
    #[must_use]
    pub fn adversary(&self) -> &str {
        &self.adversary
    }
}

/// The closed set of checkers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Checker {
    Trace(TraceChecker),
    Diff(DiffChecker),
    Run(RunChecker),
}

impl Checker {
    /// The spelling of the kind.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Checker::Trace(_) => "trace",
            Checker::Diff(_) => "diff",
            Checker::Run(_) => "run",
        }
    }
}

/// A rule's probe: what the model is asked, what it starts with, and how a
/// session is judged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    task: ProbeTask,
    workspace: Workspace,
    checker: Checker,
}

impl Probe {
    #[must_use]
    pub fn task(&self) -> &ProbeTask {
        &self.task
    }

    #[must_use]
    pub fn workspace(&self) -> Workspace {
        self.workspace
    }

    #[must_use]
    pub fn checker(&self) -> &Checker {
        &self.checker
    }
}

/// The `[probe]` table as a file holds it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawProbe {
    task: String,
    workspace: String,
    checker: RawChecker,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawChecker {
    kind: String,
    expect: Option<RawEvent>,
    window: Option<String>,
    /// A table (`trace`: an event) or a string (`diff`: a path pattern), so it
    /// is read raw and shaped by the kind.
    anchor: Option<toml::Value>,
    #[serde(default)]
    require: Vec<RawClause>,
    adversary: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEvent {
    event: String,
    pattern: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawClause {
    changed: Option<String>,
    file: Option<String>,
    contains: Option<String>,
    lacks: Option<String>,
    detector: Option<String>,
}

impl RawProbe {
    /// Parse against the rule's own detectors, which a `detector` clause must
    /// name.
    pub(crate) fn into_probe(self, detectors: &[Detector]) -> Result<Probe, ProbeError> {
        Ok(Probe {
            task: ProbeTask::parse(self.task)?,
            workspace: Workspace::parse(&self.workspace)?,
            checker: self.checker.into_checker(detectors)?,
        })
    }
}

impl RawEvent {
    fn into_event(self) -> Result<EventPattern, ProbeError> {
        Ok(EventPattern {
            kind: EventKind::parse(&self.event)?,
            pattern: ProbePattern::parse(self.pattern)?,
        })
    }
}

impl RawChecker {
    fn into_checker(self, detectors: &[Detector]) -> Result<Checker, ProbeError> {
        match self.kind.as_str() {
            "trace" => self.into_trace(),
            "diff" => self.into_diff(detectors),
            "run" => self.into_run(),
            "rubric" => Err(ProbeError::RubricNotBuilt),
            other => Err(ProbeError::UnknownChecker(other.to_owned())),
        }
    }

    fn into_trace(self) -> Result<Checker, ProbeError> {
        const KIND: &str = "trace";
        refuse(
            KIND,
            &[
                (!self.require.is_empty(), "require"),
                (self.adversary.is_some(), "adversary"),
            ],
        )?;
        let expect = self.expect.ok_or(required(KIND, "expect"))?.into_event()?;
        let window = Window::parse(&self.window.ok_or(required(KIND, "window"))?)?;
        let anchor: RawEvent = self
            .anchor
            .ok_or(required(KIND, "anchor"))?
            .try_into()
            .map_err(|_| required(KIND, "anchor.event"))?;
        Ok(Checker::Trace(TraceChecker {
            expect,
            window,
            anchor: anchor.into_event()?,
        }))
    }

    fn into_diff(self, detectors: &[Detector]) -> Result<Checker, ProbeError> {
        const KIND: &str = "diff";
        refuse(
            KIND,
            &[
                (self.expect.is_some(), "expect"),
                (self.window.is_some(), "window"),
                (self.adversary.is_some(), "adversary"),
            ],
        )?;
        let anchor = match self.anchor.ok_or(required(KIND, "anchor"))? {
            toml::Value::String(s) => ProbePattern::parse(s)?,
            _ => return Err(required(KIND, "anchor")),
        };
        if self.require.is_empty() {
            return Err(ProbeError::NoClause);
        }
        let require = self
            .require
            .into_iter()
            .map(|c| c.into_clause(detectors))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Checker::Diff(DiffChecker { anchor, require }))
    }

    fn into_run(self) -> Result<Checker, ProbeError> {
        const KIND: &str = "run";
        refuse(
            KIND,
            &[
                (self.expect.is_some(), "expect"),
                (self.window.is_some(), "window"),
                (self.anchor.is_some(), "anchor"),
                (!self.require.is_empty(), "require"),
            ],
        )?;
        match self.adversary {
            Some(adversary) if !adversary.trim().is_empty() => {
                Ok(Checker::Run(RunChecker { adversary }))
            }
            _ => Err(required(KIND, "adversary")),
        }
    }
}

impl RawClause {
    fn into_clause(self, detectors: &[Detector]) -> Result<DiffClause, ProbeError> {
        let set = [
            self.changed.is_some(),
            self.contains.is_some(),
            self.lacks.is_some(),
            self.detector.is_some(),
        ];
        if set.iter().filter(|s| **s).count() != 1 {
            return Err(ProbeError::ClauseShape);
        }
        let file = self.file.map(WorkspacePath::parse).transpose()?;
        let needs_file = self.contains.is_some() || self.lacks.is_some();
        if needs_file != file.is_some() {
            return Err(ProbeError::ClauseFile);
        }
        match (self.changed, self.contains, self.lacks, self.detector, file) {
            (Some(p), None, None, None, None) => Ok(DiffClause::Changed(ProbePattern::parse(p)?)),
            (None, Some(p), None, None, Some(file)) => Ok(DiffClause::Contains {
                file,
                pattern: ProbePattern::parse(p)?,
            }),
            (None, None, Some(p), None, Some(file)) => Ok(DiffClause::Lacks {
                file,
                pattern: ProbePattern::parse(p)?,
            }),
            (None, None, None, Some(name), None) => own_detector(detectors, name),
            _ => Err(ProbeError::ClauseShape),
        }
    }
}

/// A `detector` clause, refused unless the rule carries a detector so named.
fn own_detector(detectors: &[Detector], name: String) -> Result<DiffClause, ProbeError> {
    let found = detectors.iter().find(|d| d.name().as_str() == name);
    match found {
        Some(d) => Ok(DiffClause::Detector(d.name().clone())), // allow:clone: the clause owns the name it refers to, outliving the borrow of the rule's detectors
        None => Err(ProbeError::UnknownDetector(name)),
    }
}

/// Refuse the first field that is present but means nothing for `kind`.
/// Refused rather than ignored: a field the parser drops reads back, to the
/// next person, as a condition the checker applies.
fn refuse(kind: &'static str, fields: &[(bool, &'static str)]) -> Result<(), ProbeError> {
    match fields.iter().find(|(present, _)| *present) {
        Some((_, field)) => Err(ProbeError::FieldNotAllowed { kind, field }),
        None => Ok(()),
    }
}

fn required(kind: &'static str, field: &'static str) -> ProbeError {
    ProbeError::FieldRequired { kind, field }
}
