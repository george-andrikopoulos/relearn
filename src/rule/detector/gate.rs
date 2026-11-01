//! Repository gates: a `[[detector]]` whose engine reads a whole checkout
//! rather than one write.
//!
//! A gate runs where a write hook cannot see: a consuming repository's own
//! `check.sh`-style gate calls the generated script with the repository root
//! (ARCHITECTURE.md, decision 3, approved 2026-11-01). The script has two
//! stages. **Collect** asks `git` and the files, and prints one tab-separated
//! **record** per fact. **Judge** reads records and decides. This module holds
//! the same judge, so a gate's fixtures are records and they run **while the
//! rule parses**, exactly as a write detector's do; the pack's runner then
//! feeds the same records to the generated judge, and builds a synthetic tree
//! per gate to exercise the collect stage on the production path.
//!
//! **Must NOT:** read the filesystem or run `git`. `tests/solo_mode.rs` forbids
//! a spawned process anywhere in the crate, which is why the collect stage is
//! bash in the pack and never Rust.

use std::fmt;

use super::{
    Action, Detection, DetectorError, DetectorName, Glob, PathScope, RawDetector, RawFixture,
    fixture_path_ok,
};

/// A lint's name as rustc and clippy spell it: `[a-z][a-z0-9_]*`, optionally
/// behind one tool prefix (`clippy::needless_pass_by_value`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LintName(String);

impl LintName {
    fn parse(s: &str) -> Result<Self, DetectorError> {
        let segment_ok = |seg: &str| {
            seg.chars().next().is_some_and(|c| c.is_ascii_lowercase())
                && seg
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        };
        let ok = match s.split_once("::") {
            Some((tool, lint)) => segment_ok(tool) && segment_ok(lint),
            None => segment_ok(s),
        };
        if ok {
            Ok(LintName(s.to_owned()))
        } else {
            Err(DetectorError::LintName(s.to_owned()))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What a lint is when nothing configures it: rustc's and clippy's own
/// default for that lint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintDefault {
    /// Off unless something switches it on (`unreachable_pub`, every pedantic lint).
    Allow,
    /// On unless something switches it off (`ptr_arg`, a style lint).
    Warn,
}

impl LintDefault {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            LintDefault::Allow => "allow",
            LintDefault::Warn => "warn",
        }
    }

    fn parse(s: &str) -> Result<Self, DetectorError> {
        match s {
            "allow" => Ok(LintDefault::Allow),
            "warn" => Ok(LintDefault::Warn),
            other => Err(DetectorError::LintDefault(other.to_owned())),
        }
    }
}

/// A level a lint is configured to, from a `[lints]` table or a crate-root
/// attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintLevel {
    Allow,
    Expect,
    Warn,
    Deny,
    Forbid,
}

/// Whether a level leaves the lint reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reports {
    Yes,
    No,
}

impl LintLevel {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            LintLevel::Allow => "allow",
            LintLevel::Expect => "expect",
            LintLevel::Warn => "warn",
            LintLevel::Deny => "deny",
            LintLevel::Forbid => "forbid",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "allow" => Some(LintLevel::Allow),
            "expect" => Some(LintLevel::Expect),
            "warn" => Some(LintLevel::Warn),
            "deny" => Some(LintLevel::Deny),
            "forbid" => Some(LintLevel::Forbid),
            _ => None,
        }
    }

    fn reports(self) -> Reports {
        match self {
            LintLevel::Warn | LintLevel::Deny | LintLevel::Forbid => Reports::Yes,
            LintLevel::Allow | LintLevel::Expect => Reports::No,
        }
    }
}

/// What a gate looks for in a checkout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateEngine {
    /// A tracked path whose bytes are load-bearing, with no pinned line
    /// ending. Load-bearing: its first line is a shebang, or it matches
    /// `always` (generated text whose hash is compared). Pinned: `eol=lf`,
    /// `-text` (never converted), or, for a non-shebang path, `eol=crlf`.
    /// Records: `path`, `first line`, `text` and `eol` as `git check-attr`
    /// prints them.
    EolAttribute { always: Vec<Glob> },
    /// A tracked file at the repository root that the scope admits. The
    /// allowed documents are the scope's `exclude`. Records: `path`.
    RootDocuments,
    /// `lint` does not report: a source in scope allows it, or nothing turns
    /// it or one of its `groups` on and its `default` is `allow`, or a group
    /// is allowed and nothing names it. Records: `source`, `lint`, `level`,
    /// from `[lints]` tables in `Cargo.toml` and crate-root `#![level(..)]`.
    LintEnabled {
        lint: LintName,
        groups: Vec<LintName>,
        default: LintDefault,
    },
}

impl GateEngine {
    /// The spelling a rule file uses.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            GateEngine::EolAttribute { .. } => "eol-attribute",
            GateEngine::RootDocuments => "root-documents",
            GateEngine::LintEnabled { .. } => "lint-enabled",
        }
    }

    /// The tab-separated fields of one record, in order.
    #[must_use]
    pub fn fields(&self) -> &'static [&'static str] {
        match self {
            GateEngine::EolAttribute { .. } => &["path", "first_line", "text", "eol"],
            GateEngine::RootDocuments => &["path"],
            GateEngine::LintEnabled { .. } => &["source", "lint", "level"],
        }
    }
}

/// `git check-attr`'s answer for one attribute on one path.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Attr {
    Set,
    Unset,
    Unspecified,
    Value(String),
}

impl Attr {
    fn parse(s: &str) -> Self {
        match s {
            "set" => Attr::Set,
            "unset" => Attr::Unset,
            "unspecified" => Attr::Unspecified,
            other => Attr::Value(other.to_owned()),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Attr::Set => "set",
            Attr::Unset => "unset",
            Attr::Unspecified => "unspecified",
            Attr::Value(v) => v,
        }
    }

    fn is_value(&self, v: &str) -> bool {
        match self {
            Attr::Value(value) => value == v,
            Attr::Set | Attr::Unset | Attr::Unspecified => false,
        }
    }
}

/// One record, parsed for its engine. Which variant a gate holds is decided
/// by its engine at parse.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Record {
    Eol {
        path: String,
        first_line: String,
        text: Attr,
        eol: Attr,
    },
    Path(String),
    Lint {
        source: String,
        lint: LintName,
        level: LintLevel,
    },
}

impl Record {
    fn path(&self) -> &str {
        match self {
            Record::Eol { path, .. } | Record::Path(path) | Record::Lint { source: path, .. } => {
                path
            }
        }
    }

    /// Every field, borrowed, in record order.
    fn fields(&self) -> Vec<&str> {
        match self {
            Record::Eol {
                path,
                first_line,
                text,
                eol,
            } => vec![path, first_line, text.as_str(), eol.as_str()],
            Record::Path(path) => vec![path],
            Record::Lint {
                source,
                lint,
                level,
            } => vec![source, lint.as_str(), level.as_str()],
        }
    }

    /// The line the collect stage prints and the judge reads.
    fn line(&self) -> String {
        match self {
            Record::Eol {
                path,
                first_line,
                text,
                eol,
            } => format!("{path}\t{first_line}\t{}\t{}", text.as_str(), eol.as_str()),
            Record::Path(path) => path.clone(), // allow:clone: the rendered line is a new owned string
            Record::Lint {
                source,
                lint,
                level,
            } => format!("{source}\t{}\t{}", lint.as_str(), level.as_str()),
        }
    }
}

/// Why a record line was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RecordProblem {
    #[error("has {got} tab-separated field(s); engine `{engine}` reads {want} ({fields})")]
    FieldCount {
        engine: &'static str,
        want: usize,
        got: usize,
        fields: String,
    },
    #[error("contains a line break")]
    LineBreak,
    #[error("path {0:?} must be a relative forward-slash path with no `..`")]
    Path(String),
    #[error("lint {0:?} is not a lint name")]
    Lint(String),
    #[error("level {0:?} is not allow | expect | warn | deny | forbid")]
    Level(String),
}

impl GateEngine {
    fn record(&self, line: &str) -> Result<Record, RecordProblem> {
        if line.contains('\n') || line.contains('\r') {
            return Err(RecordProblem::LineBreak);
        }
        let parts: Vec<&str> = line.split('\t').collect();
        let fields = self.fields();
        if parts.len() != fields.len() {
            return Err(RecordProblem::FieldCount {
                engine: self.as_str(),
                want: fields.len(),
                got: parts.len(),
                fields: fields.join(", "),
            });
        }
        if !fixture_path_ok(parts[0]) {
            return Err(RecordProblem::Path(parts[0].to_owned()));
        }
        Ok(match self {
            GateEngine::EolAttribute { .. } => Record::Eol {
                path: parts[0].to_owned(),
                first_line: parts[1].to_owned(),
                text: Attr::parse(parts[2]),
                eol: Attr::parse(parts[3]),
            },
            GateEngine::RootDocuments => Record::Path(parts[0].to_owned()),
            GateEngine::LintEnabled { .. } => Record::Lint {
                source: parts[0].to_owned(),
                lint: LintName::parse(parts[1])
                    .map_err(|_| RecordProblem::Lint(parts[1].to_owned()))?,
                level: LintLevel::parse(parts[2])
                    .ok_or_else(|| RecordProblem::Level(parts[2].to_owned()))?,
            },
        })
    }
}

/// One synthetic checkout, as records, and the verdict it must produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateFixture {
    records: Vec<Record>,
    expect: Detection,
}

impl GateFixture {
    /// The record lines, exactly as the collect stage would print them.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        self.records.iter().map(Record::line).collect()
    }

    #[must_use]
    pub fn expect(&self) -> Detection {
        self.expect
    }

    /// Every field of every record, borrowed: the authored text a contribution
    /// scan reads.
    #[must_use]
    pub fn texts(&self) -> Vec<&str> {
        self.records.iter().flat_map(Record::fields).collect()
    }
}

/// A repository gate whose fixtures have all been run and agree with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    name: DetectorName,
    engine: GateEngine,
    scope: PathScope,
    action: Action,
    message: String,
    fixtures: Vec<GateFixture>,
}

impl Gate {
    #[must_use]
    pub fn name(&self) -> &DetectorName {
        &self.name
    }

    #[must_use]
    pub fn engine(&self) -> &GateEngine {
        &self.engine
    }

    #[must_use]
    pub fn scope(&self) -> &PathScope {
        &self.scope
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
    /// `clean`, and every one agrees with the judge.
    #[must_use]
    pub fn fixtures(&self) -> &[GateFixture] {
        &self.fixtures
    }

    /// Judge record lines, parsing them first.
    ///
    /// # Errors
    /// [`RecordProblem`] for the first line that is not a record of this
    /// gate's engine.
    pub fn judge_lines(&self, lines: &[&str]) -> Result<Detection, RecordProblem> {
        let records = lines
            .iter()
            .map(|l| self.engine.record(l))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self.judge(&records))
    }

    /// Parse record lines and render them back, so a caller can see that the
    /// line a fixture holds is the line the runner feeds the script.
    ///
    /// # Errors
    /// [`RecordProblem`] as for [`Gate::judge_lines`].
    pub fn render_lines(&self, lines: &[&str]) -> Result<Vec<String>, RecordProblem> {
        lines
            .iter()
            .map(|l| self.engine.record(l).map(|r| r.line()))
            .collect()
    }

    fn judge(&self, records: &[Record]) -> Detection {
        let in_scope = records.iter().filter(|r| self.scope.admits(r.path()));
        let hit = match &self.engine {
            GateEngine::EolAttribute { always } => in_scope.into_iter().any(|r| match r {
                Record::Eol {
                    path,
                    first_line,
                    text,
                    eol,
                } => {
                    let shebang = first_line.starts_with("#!");
                    let load_bearing = shebang || always.iter().any(|g| g.matches(path));
                    let pinned = eol.is_value("lf")
                        || *text == Attr::Unset
                        || (eol.is_value("crlf") && !shebang);
                    load_bearing && !pinned
                }
                Record::Path(_) | Record::Lint { .. } => false,
            }),
            GateEngine::RootDocuments => in_scope.into_iter().any(|r| !r.path().contains('/')),
            GateEngine::LintEnabled {
                lint,
                groups,
                default,
            } => {
                let mut named = (false, false); // (reports, silenced)
                let mut grouped = (false, false);
                for r in in_scope {
                    let Record::Lint { lint: l, level, .. } = r else {
                        continue;
                    };
                    let slot = if l == lint {
                        &mut named
                    } else if groups.contains(l) {
                        &mut grouped
                    } else {
                        continue;
                    };
                    match level.reports() {
                        Reports::Yes => slot.0 = true,
                        Reports::No => slot.1 = true,
                    }
                }
                let by_default = match default {
                    LintDefault::Warn => !grouped.1,
                    LintDefault::Allow => false,
                };
                let enabled = !named.1 && (named.0 || grouped.0 || by_default);
                !enabled
            }
        };
        if hit {
            Detection::Hit
        } else {
            Detection::Clean
        }
    }
}

impl fmt::Display for GateEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Build a gate from a raw table whose engine is a gate engine. Called by
/// [`RawDetector::into_parsed`] only.
pub(super) fn gate(raw: RawDetector, name: DetectorName) -> Result<Gate, DetectorError> {
    let engine_name = raw.engine.as_str();
    let unused = |field: &'static str, present: bool, engine: &'static str| {
        if present {
            Err(DetectorError::FieldNotUsed { engine, field })
        } else {
            Ok(())
        }
    };
    let engine = match engine_name {
        "eol-attribute" => {
            let e = "eol-attribute";
            unused("pattern", raw.pattern.is_some(), e)?;
            unused("comment", raw.comment.is_some(), e)?;
            unused("allow", raw.allow.is_some(), e)?;
            unused("lint", raw.lint.is_some(), e)?;
            unused("groups", raw.groups.is_some(), e)?;
            unused("default", raw.default.is_some(), e)?;
            GateEngine::EolAttribute {
                always: super::globs(raw.always.unwrap_or_default())?,
            }
        }
        "root-documents" => {
            let e = "root-documents";
            unused("pattern", raw.pattern.is_some(), e)?;
            unused("comment", raw.comment.is_some(), e)?;
            unused("allow", raw.allow.is_some(), e)?;
            unused("always", raw.always.is_some(), e)?;
            unused("lint", raw.lint.is_some(), e)?;
            unused("groups", raw.groups.is_some(), e)?;
            unused("default", raw.default.is_some(), e)?;
            GateEngine::RootDocuments
        }
        "lint-enabled" => {
            let e = "lint-enabled";
            unused("pattern", raw.pattern.is_some(), e)?;
            unused("comment", raw.comment.is_some(), e)?;
            unused("allow", raw.allow.is_some(), e)?;
            unused("always", raw.always.is_some(), e)?;
            let lint = raw.lint.as_deref().ok_or(DetectorError::FieldRequired {
                engine: e,
                field: "lint",
            })?;
            let default = raw.default.as_deref().ok_or(DetectorError::FieldRequired {
                engine: e,
                field: "default",
            })?;
            GateEngine::LintEnabled {
                lint: LintName::parse(lint)?,
                groups: raw
                    .groups
                    .unwrap_or_default()
                    .iter()
                    .map(|g| LintName::parse(g))
                    .collect::<Result<_, _>>()?,
                default: LintDefault::parse(default)?,
            }
        }
        other => return Err(DetectorError::UnknownEngine(other.to_owned())),
    };

    // A gate runs at exactly one event, so the table names it rather than
    // leaving a reader to infer it from the engine.
    match raw.events.as_slice() {
        [only] if only == "repository-gate" => {}
        _ => {
            return Err(DetectorError::EventNotForEngine {
                engine: engine.as_str(),
                expected: "exactly [\"repository-gate\"]",
            });
        }
    }
    let scope = PathScope::new(
        super::globs(raw.files)?,
        super::globs(raw.exclude)?,
        Vec::new(),
    )?;
    let action = super::Action::parse(&raw.action)?;
    if raw.message.trim().is_empty() {
        return Err(DetectorError::EmptyMessage);
    }
    let fixtures = raw
        .fixture
        .into_iter()
        .enumerate()
        .map(|(i, f)| gate_fixture(f, i + 1, &engine))
        .collect::<Result<Vec<_>, _>>()?;
    if !fixtures.iter().any(|f| f.expect == Detection::Hit) {
        return Err(DetectorError::NoHitFixture);
    }
    if !fixtures.iter().any(|f| f.expect == Detection::Clean) {
        return Err(DetectorError::NoCleanFixture);
    }
    let gate = Gate {
        name,
        engine,
        scope,
        action,
        message: raw.message,
        fixtures,
    };
    for (i, fixture) in gate.fixtures.iter().enumerate() {
        let got = gate.judge(&fixture.records);
        if got != fixture.expect {
            return Err(DetectorError::FixtureDisagrees {
                index: i + 1,
                path: "records".to_owned(),
                expected: fixture.expect,
                got,
            });
        }
    }
    Ok(gate)
}

fn gate_fixture(
    raw: RawFixture,
    index: usize,
    engine: &GateEngine,
) -> Result<GateFixture, DetectorError> {
    let records = match (raw.path, raw.input, raw.siblings, raw.records) {
        (None, None, None, Some(records)) => records,
        _ => {
            return Err(DetectorError::FixtureInputMismatch {
                index,
                engine: engine.as_str(),
            });
        }
    };
    let records = records
        .iter()
        .enumerate()
        .map(|(r, line)| {
            engine
                .record(line)
                .map_err(|problem| DetectorError::Record {
                    index,
                    record: r + 1,
                    problem: Box::new(problem),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(GateFixture {
        records,
        expect: super::Detection::parse(&raw.expect)?,
    })
}
