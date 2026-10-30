//! The committed mapping from a control's name to the place it lives.
//!
//! A rule names its controls by kind and name — `hook:no-anyhow-in-lib`,
//! `test:the_catalogue_has_not_grown_without_a_migration_to_carry_it` — and
//! nothing in that text says *which repository* holds the artefact or *where in
//! it*. The rule format is deliberately left alone (ARCHITECTURE.md decisions
//! log, 2026-10-30): this file answers the question beside it, one entry per
//! control, so the answer is versioned, reviewable and checked against the
//! corpus in both directions.
//!
//! Parsed once, here, into types that cannot hold a malformed entry: a hook
//! with an invoker, a gate without one, a path that climbs out of its root.
//! Interior code never re-checks any of it (`[R:parse-dont-validate]`).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::Deserialize;

use crate::rule::{Control, ControlError, ControlKind, Controls};

/// Why a mapping file could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MapError {
    /// The file is not the TOML shape this module reads.
    #[error("the control map is not valid: {0}")]
    Toml(String),

    /// A root name outside `[a-z0-9][a-z0-9-]*`.
    #[error("root name `{0}` must match [a-z0-9][a-z0-9-]*")]
    RootName(String),

    /// A path that is empty, absolute, backslashed, or climbs with `..`.
    #[error("path `{value}` is not a relative path inside its root: {why}")]
    Path {
        /// The path as written.
        value: String,
        /// What is wrong with it.
        why: PathProblem,
    },

    /// A function name that is not an identifier.
    #[error("function `{0}` is not an identifier")]
    Function(String),

    /// The `control` field does not parse as a control.
    #[error("control `{value}`: {source}")]
    Control {
        /// The text as written.
        value: String,
        /// Why it does not parse.
        #[source]
        source: ControlError,
    },

    /// The `control` field names more than one control, or carries a coverage
    /// note. A mapping entry is about one artefact; a note is about a rule.
    #[error("control `{0}` must be exactly one `kind:name`, with no `(coverage note)`")]
    NotOneControl(String),

    /// Two entries for one control — two answers to "where does it live?".
    #[error("control `{0}` is mapped twice")]
    Duplicate(String),

    /// The fields given do not describe an artefact of this kind.
    #[error("control `{control}`: {why}")]
    Shape {
        /// The control as written.
        control: String,
        /// What the entry has that its kind does not take, or lacks.
        why: ShapeProblem,
    },
}

/// What is wrong with a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PathProblem {
    /// Nothing at all.
    #[error("it is empty")]
    Empty,
    /// Starts at a filesystem root or a drive.
    #[error("it is absolute")]
    Absolute,
    /// Uses `\`. The committed file is read on every OS, so it is written once,
    /// with `/`.
    #[error("it uses `\\`; write `/`")]
    Backslash,
    /// A `..` or empty segment — a path that can leave its root is a path into
    /// a repository nobody supplied.
    #[error("it has a `..` or empty segment")]
    Segment,
}

/// What an entry's fields fail to describe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ShapeProblem {
    /// A hook is a script, registered by a settings file; it has no function
    /// and no invoker.
    #[error("a hook takes `path` only — no `function`, no `invoked_by`")]
    HookTakesPathOnly,
    /// A test is run by the test harness, not by an invoker.
    #[error("a test takes no `invoked_by`; the test harness is what runs it")]
    TestTakesNoInvoker,
    /// A test named by file only must be a Rust file, the only kind whose
    /// tests this module can recognise.
    #[error("a test named by file must be a `.rs` file")]
    TestFileNotRust,
    /// A gate nothing invokes is a gate that never runs.
    #[error("a gate needs `invoked_by`: the file that runs it")]
    GateNeedsInvoker,
    /// A type control has no resolver yet, so mapping one would be a claim the
    /// tool could only ever answer `UNCHECKABLE`.
    #[error("a `type:` control has no resolver; leave it unmapped and it reads UNCHECKABLE")]
    NoResolverForType,
}

/// The name of a root: a repository or directory the caller supplies with
/// `--root <name>=<path>`. The map says which root a control lives in; only
/// the command line says where that root is on this machine.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RootName(String);

impl RootName {
    /// Parse a root name: `[a-z0-9][a-z0-9-]*`, the same shape as a tag body.
    pub fn parse(s: impl Into<String>) -> Result<Self, MapError> {
        let s = s.into();
        let mut chars = s.chars();
        let head_ok = chars
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
        let tail_ok = chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if head_ok && tail_ok {
            Ok(RootName(s))
        } else {
            Err(MapError::RootName(s))
        }
    }

    /// The name as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RootName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A path relative to a root, forward-slash separated, that cannot leave it.
/// `.` is the root itself.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelPath(String);

impl RelPath {
    /// Parse a relative path.
    pub fn parse(s: impl Into<String>) -> Result<Self, MapError> {
        let s = s.into();
        let problem = if s.is_empty() {
            Some(PathProblem::Empty)
        } else if s.contains('\\') {
            Some(PathProblem::Backslash)
        } else if s.starts_with('/') || s.as_bytes().get(1) == Some(&b':') {
            Some(PathProblem::Absolute)
        } else if s != "." && s.split('/').any(|seg| seg.is_empty() || seg == "..") {
            Some(PathProblem::Segment)
        } else {
            None
        };
        match problem {
            Some(why) => Err(MapError::Path { value: s, why }),
            None => Ok(RelPath(s)),
        }
    }

    /// The path as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The last segment — the file name a settings file or an invoker
    /// references.
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    /// Whether this names the root itself.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.0 == "."
    }
}

impl fmt::Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A function name: `[A-Za-z_][A-Za-z0-9_]*`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident(String);

impl Ident {
    /// Parse an identifier.
    pub fn parse(s: impl Into<String>) -> Result<Self, MapError> {
        let s = s.into();
        let mut chars = s.chars();
        let head_ok = chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
        if head_ok && chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
            Ok(Ident(s))
        } else {
            Err(MapError::Function(s))
        }
    }

    /// The name as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A control's identity: its kind and its name, without the coverage note. Two
/// rules naming one hook with different notes name one artefact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ControlId {
    kind: ControlKind,
    name: String,
}

impl ControlId {
    /// The identity of a control a rule names.
    #[must_use]
    pub fn of(control: &Control) -> Self {
        ControlId {
            kind: control.kind(),
            name: control.name().to_owned(),
        }
    }

    /// Parse a mapping entry's `control` field: exactly one control, no note.
    fn parse(s: &str) -> Result<Self, MapError> {
        let controls = Controls::parse(s).map_err(|source| MapError::Control {
            value: s.to_owned(),
            source,
        })?;
        match controls.as_slice() {
            [only] if only.covers().is_none() => Ok(ControlId::of(only)),
            _ => Err(MapError::NotOneControl(s.to_owned())),
        }
    }

    /// The kind of control.
    #[must_use]
    pub fn kind(&self) -> ControlKind {
        self.kind
    }
}

impl fmt::Display for ControlId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind.prefix(), self.name)
    }
}

/// What a control is, as a thing on disk — and therefore what "exists" and
/// "wired" mean for it. One variant per shape, so an entry that does not
/// describe one of them cannot be held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Artefact {
    /// A hook script. Exists: the file. Wired: a settings file registers it.
    HookScript {
        /// The script.
        file: RelPath,
    },
    /// A test function, searched for under a file or a directory. Exists:
    /// `fn <name>` is defined there. Wired: it carries `#[test]` and no
    /// `#[ignore]`, so the test harness runs it.
    TestFunction {
        /// A `.rs` file, a directory, or `.` for the whole root.
        within: RelPath,
        /// The function.
        function: Ident,
    },
    /// A Rust test file named as a whole. Exists: the file. Wired: it holds at
    /// least one `#[test]`.
    TestFile {
        /// The file.
        file: RelPath,
    },
    /// A gate script. Exists: the file. Wired: the invoker references it.
    GateScript {
        /// The script.
        file: RelPath,
        /// The file that runs it.
        invoked_by: RelPath,
    },
    /// A function inside a gate script. Exists: the shell function is defined
    /// in the file. Wired: the invoker calls it on a line that is not its
    /// definition.
    GateFunction {
        /// The script defining the function.
        file: RelPath,
        /// The function.
        function: Ident,
        /// The file that calls it.
        invoked_by: RelPath,
    },
}

impl Artefact {
    /// The path that *is* the artefact — the script, the test file, or where
    /// a test function is searched for.
    #[must_use]
    pub fn primary(&self) -> &RelPath {
        match self {
            Artefact::HookScript { file }
            | Artefact::TestFile { file }
            | Artefact::GateScript { file, .. }
            | Artefact::GateFunction { file, .. } => file,
            Artefact::TestFunction { within, .. } => within,
        }
    }

    /// Every path resolution needs to read, under this artefact's root.
    #[must_use]
    pub fn reads(&self) -> Vec<&RelPath> {
        match self {
            Artefact::HookScript { file } | Artefact::TestFile { file } => vec![file],
            Artefact::TestFunction { within, .. } => vec![within],
            Artefact::GateScript { file, invoked_by }
            | Artefact::GateFunction {
                file, invoked_by, ..
            } => vec![file, invoked_by],
        }
    }

    /// Build the artefact a kind and a set of fields describe — an exhaustive
    /// decision over the kind, so a new kind cannot compile until it says what
    /// it looks like on disk.
    fn build(
        kind: ControlKind,
        path: RelPath,
        function: Option<Ident>,
        invoked_by: Option<RelPath>,
    ) -> Result<Self, ShapeProblem> {
        match kind {
            ControlKind::Hook => match (function, invoked_by) {
                (None, None) => Ok(Artefact::HookScript { file: path }),
                _ => Err(ShapeProblem::HookTakesPathOnly),
            },
            ControlKind::UnitTest | ControlKind::PropertyTest => match (function, invoked_by) {
                (_, Some(_)) => Err(ShapeProblem::TestTakesNoInvoker),
                (Some(function), None) => Ok(Artefact::TestFunction {
                    within: path,
                    function,
                }),
                (None, None) if path.as_str().ends_with(".rs") => {
                    Ok(Artefact::TestFile { file: path })
                }
                (None, None) => Err(ShapeProblem::TestFileNotRust),
            },
            ControlKind::Gate => match (function, invoked_by) {
                (_, None) => Err(ShapeProblem::GateNeedsInvoker),
                (None, Some(invoked_by)) => Ok(Artefact::GateScript {
                    file: path,
                    invoked_by,
                }),
                (Some(function), Some(invoked_by)) => Ok(Artefact::GateFunction {
                    file: path,
                    function,
                    invoked_by,
                }),
            },
            ControlKind::Type => Err(ShapeProblem::NoResolverForType),
        }
    }
}

/// Where one control lives: a root, and the artefact under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    root: RootName,
    artefact: Artefact,
}

impl Location {
    /// The root the artefact lives under.
    #[must_use]
    pub fn root(&self) -> &RootName {
        &self.root
    }

    /// The artefact.
    #[must_use]
    pub fn artefact(&self) -> &Artefact {
        &self.artefact
    }
}

/// The committed answer to "where does each control live?".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlMap {
    here: RootName,
    entries: BTreeMap<ControlId, Location>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMap {
    here: String,
    #[serde(default)]
    control: Vec<RawEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    control: String,
    root: String,
    path: String,
    function: Option<String>,
    invoked_by: Option<String>,
}

impl ControlMap {
    /// Parse a mapping file. All-or-nothing: one bad entry refuses the file,
    /// because a half-read map reports the controls it dropped as `UNMAPPED`
    /// and the reader would go and map them a second time.
    pub fn parse(text: &str) -> Result<Self, MapError> {
        let raw: RawMap = toml::from_str(text).map_err(|e| MapError::Toml(e.to_string()))?;
        let here = RootName::parse(raw.here)?;
        let mut entries = BTreeMap::new();
        for entry in raw.control {
            let id = ControlId::parse(&entry.control)?;
            let location = Location {
                root: RootName::parse(entry.root)?,
                artefact: Artefact::build(
                    id.kind(),
                    RelPath::parse(entry.path)?,
                    entry.function.map(Ident::parse).transpose()?,
                    entry.invoked_by.map(RelPath::parse).transpose()?,
                )
                .map_err(|why| MapError::Shape {
                    control: entry.control.clone(), // allow:clone: the error outlives the entry it names, which is consumed field by field above
                    why,
                })?,
            };
            if entries.contains_key(&id) {
                return Err(MapError::Duplicate(entry.control));
            }
            entries.insert(id, location);
        }
        Ok(ControlMap { here, entries })
    }

    /// The root this map file sits in. The caller locates it from the map
    /// file's own path, so it never has to be supplied.
    #[must_use]
    pub fn here(&self) -> &RootName {
        &self.here
    }

    /// Where a control lives, if the map says.
    #[must_use]
    pub fn locate(&self, id: &ControlId) -> Option<&Location> {
        self.entries.get(id)
    }

    /// Every mapped control, in a stable order.
    pub fn entries(&self) -> impl Iterator<Item = (&ControlId, &Location)> {
        self.entries.iter()
    }

    /// Every root any entry names, plus `here`.
    #[must_use]
    pub fn roots(&self) -> BTreeSet<&RootName> {
        let mut roots: BTreeSet<&RootName> = self.entries.values().map(Location::root).collect();
        roots.insert(&self.here);
        roots
    }

    /// Every `(root, path)` resolution reads, deduplicated.
    #[must_use]
    pub fn reads(&self) -> BTreeSet<(&RootName, &RelPath)> {
        self.entries
            .values()
            .flat_map(|loc| loc.artefact.reads().into_iter().map(|p| (&loc.root, p)))
            .collect()
    }
}
