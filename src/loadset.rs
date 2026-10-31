//! `loadset` — what a Claude Code session loads at start, and how much of the
//! loader's character budget it spends.
//!
//! **Why this exists** (2026-10-31). An instruction layer grows past the
//! loader's budget, or loads a layer twice, or loads another project's layer,
//! and nothing notices until the tool warns. This module is the measure;
//! `relearn budget` is its command.
//!
//! **Pure.** Everything here is a function of text and of the list
//! [`crate::fsio::discover_load_set`] hands it; the walk that finds the files
//! is in `fsio`, which moves bytes and decides nothing.
//!
//! **What counts is the loader's definition, read from the loader** as well as
//! from its documentation; the decisions log (2026-10-31) quotes both and says
//! where the documentation is silent. In short:
//!
//! - a file's size is its length **in UTF-16 code units** after YAML front
//!   matter and HTML comments are removed — the loader is JavaScript and
//!   measures `content.length`;
//! - only files resident at session start count toward the combined total:
//!   every `CLAUDE.md`, `CLAUDE.local.md`, `@import` and rules file **without**
//!   a `paths:` list;
//! - each file, path-scoped or not, is separately subject to the per-file limit;
//! - a file reached twice is loaded once, so it is counted once.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

/// The loader's per-file limit on a 1M-context session, which is also its
/// combined limit there: `max(40_000, round(window * 0.05 * k))` per file and
/// `max(120_000, per-file)` combined, read from Claude Code 2.1.316 on
/// 2026-10-31. The default for `--limit`.
pub const DEFAULT_LIMIT: usize = 150_000;

/// The combined limit on the **smallest** window the loader supports
/// (`max(120_000, 40_000)` on a 200k-context session). A load set under it
/// warns in no session, so it is the default budget: growth is caught before
/// any session warns, not merely before the largest one does.
pub const DEFAULT_BUDGET: usize = 120_000;

/// A length as the loader measures it: UTF-16 code units of the counted text.
///
/// Not bytes, and not Unicode scalar values. A scalar outside the Basic
/// Multilingual Plane is two units to the loader, so counting scalars would
/// under-report exactly the files that carry them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct LoaderChars(usize);

impl LoaderChars {
    /// Wrap a count already in loader units — a limit or a budget typed on the
    /// command line.
    #[must_use]
    pub const fn new(units: usize) -> Self {
        Self(units)
    }

    /// The count.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }

    fn of_text(text: &str) -> Self {
        Self(text.encode_utf16().count())
    }
}

impl std::ops::Add for LoaderChars {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}

impl std::iter::Sum for LoaderChars {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |a, b| a + b)
    }
}

impl fmt::Display for LoaderChars {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Whose configuration a loaded file belongs to, in the loader's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Share {
    /// Under the user's configuration directory (`~/.claude`).
    User,
    /// A `CLAUDE.md`, `.claude/CLAUDE.md`, `.claude/rules/` or
    /// `CLAUDE.local.md` in a directory **above** the project.
    Ancestor,
    /// In the project's own tree.
    Project,
}

impl Share {
    /// The lowercase label a report prints.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Share::User => "user",
            Share::Ancestor => "ancestor",
            Share::Project => "project",
        }
    }
}

/// When a file is in context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Residency {
    /// Loaded at session start; counts toward the combined total.
    Always,
    /// Attaches when a file matching one of these globs is read. Never empty:
    /// an empty `paths:` list was measured on 2026-08-22 to load at session
    /// start, so it parses as [`Residency::Always`].
    PathScoped(Vec<String>),
}

/// A relearn layer's name as it appears in a generated rules-layer file's
/// stem: `global`, `org-*`, `domain-*` or `project-*`, shape `[a-z0-9-]+`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerSlug(String);

impl LayerSlug {
    /// Parse a file stem as a layer slug; `None` when it is not one — a
    /// generated file relearn cannot attribute to a home, never an error.
    #[must_use]
    pub fn parse(stem: &str) -> Option<Self> {
        let shaped = !stem.is_empty()
            && stem
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        let named = stem == "global"
            || ["org-", "domain-", "project-"]
                .iter()
                .any(|p| stem.strip_prefix(p).is_some_and(|rest| !rest.is_empty()));
        (shaped && named).then(|| Self(stem.to_owned()))
    }

    /// The slug of a project home, from the project's name, slugified exactly
    /// as [`crate::emit::HomeSlug`] slugifies it.
    #[must_use]
    pub fn project(name: &str) -> Self {
        Self(format!("project-{}", crate::emit::slugify(name)))
    }

    /// Whether this slug names a project home.
    #[must_use]
    pub fn is_project(&self) -> bool {
        self.0.starts_with("project-")
    }

    /// The slug text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LayerSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Who wrote a loaded file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Authorship {
    /// No `relearn:generated` marker.
    HandAuthored,
    /// Carries the marker. The layer is read from the file stem, because the
    /// marker names rules rather than a home; `None` when the stem is not a
    /// layer slug.
    Generated(Option<LayerSlug>),
}

impl Authorship {
    /// The relearn layer this file is a copy of, if any. Only a generated file
    /// names one: a hand-authored `domain-rust.md` is somebody's own file, and
    /// two of them are not a duplicated layer.
    #[must_use]
    pub fn layer(&self) -> Option<&LayerSlug> {
        match self {
            Authorship::Generated(layer) => layer.as_ref(),
            Authorship::HandAuthored => None,
        }
    }
}

/// What [`read_layer_text`] makes of one file's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerText {
    residency: Residency,
    chars: LoaderChars,
    generated: bool,
    imports: Vec<String>,
}

impl LayerText {
    /// When it loads.
    #[must_use]
    pub fn residency(&self) -> &Residency {
        &self.residency
    }
    /// What it costs.
    #[must_use]
    pub fn chars(&self) -> LoaderChars {
        self.chars
    }
    /// Whether it carries the `relearn:generated` marker.
    #[must_use]
    pub fn generated(&self) -> bool {
        self.generated
    }
    /// The `@path` imports it names, in order, unresolved.
    #[must_use]
    pub fn imports(&self) -> &[String] {
        &self.imports
    }
    /// Consume into its parts: residency, size, marker, imports.
    #[must_use]
    pub fn into_parts(self) -> (Residency, LoaderChars, bool, Vec<String>) {
        (self.residency, self.chars, self.generated, self.imports)
    }
}

/// The marker relearn's writer appends to every generated file.
const GENERATED_MARKER: &str = "<!-- relearn:generated";

/// Read one instruction file's text the way the loader does: strip YAML front
/// matter (reading `paths:` from it), strip HTML comments, count what is left
/// in UTF-16 units, and list its `@imports`.
#[must_use]
pub fn read_layer_text(raw: &str) -> LayerText {
    let (front, body) = split_front_matter(raw);
    let globs = front.map(paths_of).unwrap_or_default();
    let residency = if globs.is_empty() {
        Residency::Always
    } else {
        Residency::PathScoped(globs)
    };
    let counted = strip_html_comments(body);
    LayerText {
        residency,
        chars: LoaderChars::of_text(&counted),
        generated: raw.contains(GENERATED_MARKER),
        imports: imports_in(&counted),
    }
}

/// Split leading `---` YAML front matter from the body. A block that is never
/// closed is not front matter, and the whole text is body.
fn split_front_matter(raw: &str) -> (Option<&str>, &str) {
    let Some(rest) = raw
        .strip_prefix("---\r\n")
        .or_else(|| raw.strip_prefix("---\n"))
    else {
        return (None, raw);
    };
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return (Some(&rest[..offset]), &rest[offset + line.len()..]);
        }
        offset += line.len();
    }
    (None, raw)
}

/// The globs a front-matter block's `paths:` key lists: a block list, a flow
/// list, or one scalar. Anything else yields none — which is the loader's own
/// fallback for front matter it cannot read: load as if there were no `paths`.
fn paths_of(front: &str) -> Vec<String> {
    let mut lines = front.lines();
    while let Some(line) = lines.next() {
        let Some(value) = line.strip_prefix("paths:") else {
            continue;
        };
        let value = value.trim();
        if let Some(flow) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
            return flow.split(',').filter_map(unquote).collect();
        }
        if !value.is_empty() {
            return unquote(value).into_iter().collect();
        }
        return lines
            .map(str::trim)
            .take_while(|l| l.starts_with('-'))
            .filter_map(|l| unquote(l.trim_start_matches('-')))
            .collect();
    }
    Vec::new()
}

fn unquote(s: &str) -> Option<String> {
    let s = s.trim();
    let s = s
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .or_else(|| s.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
        .unwrap_or(s);
    (!s.is_empty()).then(|| s.to_owned())
}

/// Remove every `<!-- … -->` span; an unterminated one runs to the end.
fn strip_html_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        rest = match rest[start + 4..].find("-->") {
            Some(end) => &rest[start + 4 + end + 3..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// The `@path` imports in a text, skipping fenced code blocks and inline code
/// spans. An import is `@` at the start of a line or after whitespace, running
/// to the next unescaped whitespace; `\ ` is an escaped space.
fn imports_in(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        // Even-indexed pieces of a backtick split lie outside code spans.
        for (i, piece) in line.split('`').enumerate() {
            if i % 2 == 0 {
                imports_in_plain(piece, i == 0, &mut found);
            }
        }
    }
    found
}

fn imports_in_plain(piece: &str, at_line_start: bool, found: &mut Vec<String>) {
    let mut prev_space = at_line_start;
    let mut chars = piece.chars();
    while let Some(c) = chars.next() {
        if c != '@' || !prev_space {
            prev_space = c.is_whitespace();
            continue;
        }
        let mut path = String::new();
        let mut escaped = false;
        for d in chars.by_ref() {
            if escaped {
                path.push(d);
                escaped = false;
            } else if d == '\\' {
                escaped = true;
            } else if d.is_whitespace() {
                break;
            } else {
                path.push(d);
            }
        }
        if !path.is_empty() {
            found.push(path);
        }
        prev_space = true;
    }
}

/// One file in a session's load set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedFile {
    key: PathBuf,
    shown: String,
    share: Share,
    residency: Residency,
    chars: LoaderChars,
    origin: Authorship,
}

impl LoadedFile {
    /// A discovered file. `key` is its identity for de-duplication (the
    /// canonical path); `shown` is how a report names it.
    #[must_use]
    pub fn new(
        key: PathBuf,
        shown: String,
        share: Share,
        residency: Residency,
        chars: LoaderChars,
        origin: Authorship,
    ) -> Self {
        Self {
            key,
            shown,
            share,
            residency,
            chars,
            origin,
        }
    }
    /// Identity.
    #[must_use]
    pub fn key(&self) -> &Path {
        &self.key
    }
    /// Display name.
    #[must_use]
    pub fn shown(&self) -> &str {
        &self.shown
    }
    /// Whose it is.
    #[must_use]
    pub fn share(&self) -> Share {
        self.share
    }
    /// When it loads.
    #[must_use]
    pub fn residency(&self) -> &Residency {
        &self.residency
    }
    /// What it costs.
    #[must_use]
    pub fn chars(&self) -> LoaderChars {
        self.chars
    }
    /// Who wrote it.
    #[must_use]
    pub fn origin(&self) -> &Authorship {
        &self.origin
    }
    fn is_always(&self) -> bool {
        matches!(self.residency, Residency::Always)
    }
}

/// A session's load set: every file once, in load order.
///
/// The only constructor de-duplicates by [`LoadedFile::key`], keeping the
/// first, because the loader loads a file reached twice — by `@import` and by
/// `.claude/rules/`, say — once. A total over this type therefore cannot count
/// a file twice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadSet {
    files: Vec<LoadedFile>,
}

impl LoadSet {
    /// Build from files in discovery order; a repeated key is dropped.
    #[must_use]
    pub fn new(discovered: Vec<LoadedFile>) -> Self {
        let mut seen = BTreeSet::new();
        let files = discovered
            .into_iter()
            .filter(|f| seen.insert(f.key.clone())) // allow:clone: the set owns its keys while the files move on
            .collect();
        Self { files }
    }

    /// The files, in load order.
    #[must_use]
    pub fn files(&self) -> &[LoadedFile] {
        &self.files
    }

    /// The size of everything resident at session start.
    #[must_use]
    pub fn always_total(&self) -> LoaderChars {
        self.files
            .iter()
            .filter(|f| f.is_always())
            .map(LoadedFile::chars)
            .sum()
    }

    /// The size of everything path-scoped: what attaches when every scope's
    /// files are read.
    #[must_use]
    pub fn scoped_total(&self) -> LoaderChars {
        self.files
            .iter()
            .filter(|f| !f.is_always())
            .map(LoadedFile::chars)
            .sum()
    }

    /// The size of one share's always-resident files.
    #[must_use]
    pub fn share_total(&self, share: Share) -> LoaderChars {
        self.files
            .iter()
            .filter(|f| f.is_always() && f.share == share)
            .map(LoadedFile::chars)
            .sum()
    }
}

/// How much of the session's configuration the measurement saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    /// `--user` was given: user, ancestors and project were all walked, so the
    /// combined total is a verdict.
    Whole,
    /// `--project-only`: the caller asked about the project's share and
    /// nothing else. The combined total is not a question, so it is not a gap.
    ProjectOnly,
    /// Neither: the user share is **unchecked**, never zero. The combined
    /// total is not assessed and the outcome says so.
    UserUnchecked,
}

/// The loader's limit and the budget beneath it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    limit: LoaderChars,
    budget: LoaderChars,
}

/// Why a [`Limits`] could not be built.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LimitsError {
    /// A zero limit makes every load set over it.
    #[error("the limit must be positive")]
    ZeroLimit,
    /// A budget above the limit is never reached before the limit is.
    #[error("the budget ({budget}) is above the limit ({limit}); it would never fire first")]
    BudgetAboveLimit {
        /// The budget given.
        budget: LoaderChars,
        /// The limit given.
        limit: LoaderChars,
    },
}

impl Limits {
    /// A positive limit and a budget no greater than it.
    pub fn new(limit: LoaderChars, budget: LoaderChars) -> Result<Self, LimitsError> {
        if limit.get() == 0 {
            return Err(LimitsError::ZeroLimit);
        }
        if budget > limit {
            return Err(LimitsError::BudgetAboveLimit { budget, limit });
        }
        Ok(Self { limit, budget })
    }

    /// The loader's limit.
    #[must_use]
    pub fn limit(&self) -> LoaderChars {
        self.limit
    }

    /// The budget.
    #[must_use]
    pub fn budget(&self) -> LoaderChars {
        self.budget
    }
}

/// What a size finding measured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Measured {
    /// Everything resident at session start.
    Total,
    /// The project's own always-resident share.
    ProjectShare,
    /// One file, named as the report shows it.
    File(String),
}

impl fmt::Display for Measured {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Measured::Total => f.write_str("always-resident total"),
            Measured::ProjectShare => f.write_str("project share"),
            Measured::File(path) => f.write_str(path),
        }
    }
}

/// A finding. A closed set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    /// Over the loader's limit: the tool warns, and an over-limit file is
    /// truncated (if it loads on demand) or left out of the combined total.
    OverLimit {
        /// What was measured.
        measured: Measured,
        /// Its size.
        chars: LoaderChars,
        /// The limit.
        limit: LoaderChars,
    },
    /// Over a budget below the limit, so growth is caught before any session
    /// warns.
    OverBudget {
        /// What was measured.
        measured: Measured,
        /// Its size.
        chars: LoaderChars,
        /// The budget.
        budget: LoaderChars,
    },
    /// One relearn layer loads from two paths in one session.
    DuplicateLayer {
        /// The layer.
        layer: LayerSlug,
        /// Every path it loads from, in load order.
        paths: Vec<String>,
    },
    /// A project layer loads in a session for a different project.
    ForeignProjectLayer {
        /// The foreign layer.
        layer: LayerSlug,
        /// Where it loads from.
        path: String,
        /// The session's own project layer.
        session: LayerSlug,
    },
}

impl Finding {
    /// The finding's name as a report prints it.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Finding::OverLimit { .. } => "OVER_LIMIT",
            Finding::OverBudget { .. } => "OVER_BUDGET",
            Finding::DuplicateLayer { .. } => "DUPLICATE_LAYER",
            Finding::ForeignProjectLayer { .. } => "FOREIGN_PROJECT_LAYER",
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Finding::OverLimit {
                measured,
                chars,
                limit,
            } => write!(f, "{code} {measured}: {chars} chars, limit {limit}"),
            Finding::OverBudget {
                measured,
                chars,
                budget,
            } => write!(f, "{code} {measured}: {chars} chars, budget {budget}"),
            Finding::DuplicateLayer { layer, paths } => {
                write!(f, "{code} {layer}: {}", paths.join(", "))
            }
            Finding::ForeignProjectLayer {
                layer,
                path,
                session,
            } => write!(f, "{code} {layer}: {path} loads in a {session} session"),
        }
    }
}

/// The result of [`assess`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assessment {
    coverage: Coverage,
    findings: Vec<Finding>,
}

/// What the command's exit status reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Everything asked was measured and nothing was found.
    Clean,
    /// At least one finding.
    Findings,
    /// Nothing was found in what was measured, but the user share was not
    /// measured. Not a pass.
    Unchecked,
}

impl Assessment {
    /// The findings, in a fixed order: per-file size, total size, project
    /// share, duplicates, foreign layers.
    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// What was measured.
    #[must_use]
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }

    /// The verdict. Exhaustive over [`Coverage`], so a new coverage cannot
    /// compile until it says whether it is a pass.
    #[must_use]
    pub fn outcome(&self) -> Outcome {
        if !self.findings.is_empty() {
            return Outcome::Findings;
        }
        match self.coverage {
            Coverage::Whole | Coverage::ProjectOnly => Outcome::Clean,
            Coverage::UserUnchecked => Outcome::Unchecked,
        }
    }
}

/// Assess a load set for a session whose own project layer is `session`.
#[must_use]
pub fn assess(
    set: &LoadSet,
    session: &LayerSlug,
    coverage: Coverage,
    limits: Limits,
    project_budget: Option<LoaderChars>,
) -> Assessment {
    let mut findings: Vec<Finding> = set
        .files()
        .iter()
        .filter(|f| f.chars > limits.limit)
        .map(|f| Finding::OverLimit {
            measured: Measured::File(f.shown.clone()), // allow:clone: the finding outlives the borrowed set
            chars: f.chars,
            limit: limits.limit,
        })
        .collect();

    match coverage {
        Coverage::Whole => {
            let total = set.always_total();
            if total > limits.limit {
                findings.push(Finding::OverLimit {
                    measured: Measured::Total,
                    chars: total,
                    limit: limits.limit,
                });
            } else if total > limits.budget {
                findings.push(Finding::OverBudget {
                    measured: Measured::Total,
                    chars: total,
                    budget: limits.budget,
                });
            }
        }
        Coverage::ProjectOnly | Coverage::UserUnchecked => {}
    }
    if let Some(budget) = project_budget {
        let share = set.share_total(Share::Project);
        if share > budget {
            findings.push(Finding::OverBudget {
                measured: Measured::ProjectShare,
                chars: share,
                budget,
            });
        }
    }

    let mut by_layer: BTreeMap<&LayerSlug, Vec<&LoadedFile>> = BTreeMap::new();
    for file in set.files() {
        if let Some(layer) = file.origin.layer() {
            by_layer.entry(layer).or_default().push(file);
        }
    }
    for (layer, files) in &by_layer {
        if files.len() > 1 {
            findings.push(Finding::DuplicateLayer {
                layer: (*layer).clone(), // allow:clone: the finding outlives the borrowed set
                paths: files.iter().map(|f| f.shown.clone()).collect(), // allow:clone: the finding outlives the borrowed set
            });
        }
    }
    for (layer, files) in &by_layer {
        if layer.is_project() && *layer != session {
            for file in files {
                findings.push(Finding::ForeignProjectLayer {
                    layer: (*layer).clone(), // allow:clone: the finding outlives the borrowed set
                    path: file.shown.clone(), // allow:clone: the finding outlives the borrowed set
                    session: session.clone(), // allow:clone: the finding outlives the borrowed slug
                });
            }
        }
    }

    Assessment { coverage, findings }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_matter_paths_make_a_file_path_scoped_and_are_not_counted() {
        let text = read_layer_text("---\npaths:\n  - \"**/*.rs\"\n---\n\nabc");
        assert_eq!(
            text.residency(),
            &Residency::PathScoped(vec!["**/*.rs".into()])
        );
        assert_eq!(text.chars(), LoaderChars::new(4));
    }

    #[test]
    fn flow_and_scalar_paths_parse() {
        let flow = read_layer_text("---\npaths: [\"a/**\", b]\n---\nx");
        assert_eq!(
            flow.residency(),
            &Residency::PathScoped(vec!["a/**".into(), "b".into()])
        );
        let scalar = read_layer_text("---\npaths: '*.java'\n---\nx");
        assert_eq!(
            scalar.residency(),
            &Residency::PathScoped(vec!["*.java".into()])
        );
    }

    #[test]
    fn an_empty_paths_list_is_always_resident() {
        // Measured 2026-08-22: `paths: []` loads at session start.
        let text = read_layer_text("---\npaths: []\n---\nx");
        assert_eq!(text.residency(), &Residency::Always);
    }

    #[test]
    fn unclosed_front_matter_is_body() {
        let text = read_layer_text("---\npaths: [a]\nbody");
        assert_eq!(text.residency(), &Residency::Always);
        assert_eq!(text.chars(), LoaderChars::new(19));
    }

    #[test]
    fn html_comments_including_the_generated_marker_are_not_counted() {
        let text = read_layer_text("ab<!-- relearn:generated v0 -->c");
        assert!(text.generated());
        assert_eq!(text.chars(), LoaderChars::new(3));
    }

    #[test]
    fn size_is_utf16_units_not_bytes_or_scalars() {
        // U+00E9: 2 bytes, 1 unit. U+1F600: 4 bytes, 1 scalar, 2 units.
        let text = read_layer_text("\u{e9}\u{1F600}");
        assert_eq!(text.chars(), LoaderChars::new(3));
    }

    #[test]
    fn imports_skip_code_spans_fences_and_mid_word_at_signs() {
        let raw = "@a.md and `@b.md` x@c.md\n```\n@d.md\n```\n see @e\\ f.md";
        assert_eq!(read_layer_text(raw).imports(), ["a.md", "e f.md"]);
    }

    #[test]
    fn layer_slugs_parse_only_relearn_layer_names() {
        assert!(LayerSlug::parse("domain-rust").is_some());
        assert!(LayerSlug::parse("project-relearn").is_some());
        assert!(LayerSlug::parse("global").is_some());
        assert!(LayerSlug::parse("project-").is_none());
        assert!(LayerSlug::parse("Domain-Rust").is_none());
        assert!(LayerSlug::parse("coding-style").is_none());
        assert_eq!(LayerSlug::project("relearn").as_str(), "project-relearn");
    }

    #[test]
    fn a_budget_above_the_limit_is_refused() {
        assert!(Limits::new(LoaderChars::new(10), LoaderChars::new(11)).is_err());
        assert!(Limits::new(LoaderChars::new(0), LoaderChars::new(0)).is_err());
    }
}
