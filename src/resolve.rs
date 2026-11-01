//! `resolve` — does each control a rule names exist, and does anything run it?
//!
//! A `partial` or `graduated` rule names the artefacts that hold it —
//! `hook:no-anyhow-in-lib`, `test:…`, `gate:…` — and those names are the
//! deterministic half of the whole harness. `relearn verify` proves the
//! instruction layer matches its source and says nothing about them, so a hook
//! deleted from disk, or present and registered for no event, left its rule
//! announcing *"Also enforced by"* into five instruction layers with nothing
//! behind it: `[R:wired-artifact]`, unchecked across the corpus.
//!
//! Resolution is pure. The caller reads what the [`ControlMap`] says to read
//! into a [`Snapshot`] (`fsio::snapshot`) and a settings file into a
//! [`HookRegistry`]; this module decides. Every control occurrence gets exactly
//! one [`Verdict`] from a closed set, and the run's [`Outcome`] is an
//! exhaustive fold over them in which `UNCHECKABLE` can never become a pass —
//! a disarmed check exits 2, the repository's existing convention
//! (`[R:verdict-survives-the-channel]`).
//!
//! **Must NOT:** perform I/O, or be reachable from `build`/`verify`. Bare
//! `verify` runs on every push, and a check that needs other repositories
//! would make that gate depend on what happens to be cloned beside it.

mod map;
mod scan;
mod settings;

use std::collections::BTreeMap;
use std::fmt;

pub use map::{
    Artefact, ControlId, ControlMap, Ident, Location, MapError, PathProblem, Proposal, RelPath,
    RootName, ShapeProblem,
};
pub use settings::{HookRegistry, SettingsError};

use crate::library::{Library, Validated};
use crate::rule::{Control, ControlKind, RuleTag};
use scan::TestMark;

/// What was on disk at one `(root, path)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    /// Nothing there.
    Absent,
    /// A file, as text.
    File(String),
    /// A directory: every `.rs` file under it, as `(path below the root, text)`.
    Tree(Vec<(String, String)>),
}

/// What the caller supplied and read: which roots exist on this machine, and
/// what was at each path the map names under them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Snapshot {
    roots: BTreeMap<RootName, BTreeMap<RelPath, Found>>,
}

impl Snapshot {
    /// A snapshot with no roots supplied.
    #[must_use]
    pub fn new() -> Self {
        Snapshot::default()
    }

    /// Record that a root was supplied.
    pub fn supply(&mut self, root: RootName) {
        self.roots.entry(root).or_default();
    }

    /// Record what was found at a path under a root. Ignored for a root that
    /// was not supplied: what is "found" in a repository nobody pointed at is
    /// not evidence.
    pub fn record(&mut self, root: &RootName, path: RelPath, found: Found) {
        if let Some(paths) = self.roots.get_mut(root) {
            paths.insert(path, found);
        }
    }

    /// What was at a path, or `None` when its root was not supplied. A
    /// supplied root that holds no record for the path reads as
    /// [`Found::Absent`]: a snapshot that did not look cannot claim the
    /// artefact is there, so the error runs toward `MISSING`, never toward a
    /// pass.
    fn at(&self, root: &RootName, path: &RelPath) -> Option<&Found> {
        const ABSENT: &Found = &Found::Absent;
        self.roots
            .get(root)
            .map(|paths| paths.get(path).unwrap_or(ABSENT))
    }
}

/// The evidence a wired control was accepted on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wired {
    /// The settings file registers the hook script.
    Registered,
    /// The test function carries `#[test]` and no `#[ignore]`, in this file.
    RunsAsTest(String),
    /// The test file holds a test the harness runs.
    HoldsTests,
    /// The invoker references the gate.
    InvokedBy(RelPath),
}

/// Why an artefact that exists is not wired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotWired {
    /// No registered hook command runs the script.
    NotRegistered,
    /// The function has no test attribute, so the harness never runs it.
    NotATest(String),
    /// The test is `#[ignore]`d, so the default run skips it.
    Ignored(String),
    /// The file holds no test the harness runs by default.
    NoTests,
    /// The invoker exists and does not reference the gate.
    NotInvoked(RelPath),
    /// The invoker the map names is not there.
    InvokerAbsent(RelPath),
}

/// Why a control could not be checked at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Uncheckable {
    /// The control lives under a root nobody supplied.
    RootNotSupplied(RootName),
    /// A hook script exists, and no settings file was given to say whether it
    /// is registered.
    SettingsNotSupplied,
    /// No resolver exists for this kind of control.
    NoResolver(ControlKind),
}

/// The verdict on one control — a closed set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The artefact exists and something runs it.
    ResolvedAndWired(Wired),
    /// The artefact exists and nothing runs it.
    ResolvedNotWired(NotWired),
    /// The root was supplied and the artefact is not in it.
    Missing,
    /// It could not be checked. Never a pass.
    Uncheckable(Uncheckable),
    /// The map does not say where it lives.
    Unmapped,
}

/// What a run of verdicts amounts to, for a gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    /// Every control resolved and is wired.
    Pass,
    /// Something could not be checked, and nothing definitely failed. Exit 2.
    Disarmed,
    /// Something is missing, unwired, unmapped, or mapped to nothing. Exit 1.
    Fail,
}

impl Verdict {
    /// The label a table prints and a script greps.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Verdict::ResolvedAndWired(_) => "RESOLVED_AND_WIRED",
            Verdict::ResolvedNotWired(_) => "RESOLVED_NOT_WIRED",
            Verdict::Missing => "MISSING",
            Verdict::Uncheckable(_) => "UNCHECKABLE",
            Verdict::Unmapped => "UNMAPPED",
        }
    }

    /// What this verdict means to a gate. Exhaustive, so a new verdict cannot
    /// compile until somebody decides whether it passes.
    #[must_use]
    pub fn outcome(&self) -> Outcome {
        match self {
            Verdict::ResolvedAndWired(_) => Outcome::Pass,
            Verdict::Uncheckable(_) => Outcome::Disarmed,
            Verdict::ResolvedNotWired(_) | Verdict::Missing | Verdict::Unmapped => Outcome::Fail,
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::ResolvedAndWired(Wired::Registered) => {
                f.write_str("registered as a hook in the settings file")
            }
            Verdict::ResolvedAndWired(Wired::RunsAsTest(file)) => {
                write!(f, "a test the harness runs, in {file}")
            }
            Verdict::ResolvedAndWired(Wired::HoldsTests) => f.write_str("holds a running test"),
            Verdict::ResolvedAndWired(Wired::InvokedBy(by)) => write!(f, "invoked by {by}"),
            Verdict::ResolvedNotWired(NotWired::NotRegistered) => {
                f.write_str("the script exists and no hook in the settings file runs it")
            }
            Verdict::ResolvedNotWired(NotWired::NotATest(file)) => {
                write!(f, "defined in {file} with no test attribute")
            }
            Verdict::ResolvedNotWired(NotWired::Ignored(file)) => {
                write!(f, "#[ignore]d in {file}, so the default run skips it")
            }
            Verdict::ResolvedNotWired(NotWired::NoTests) => {
                f.write_str("the file holds no test the harness runs")
            }
            Verdict::ResolvedNotWired(NotWired::NotInvoked(by)) => {
                write!(f, "{by} does not reference it")
            }
            Verdict::ResolvedNotWired(NotWired::InvokerAbsent(by)) => {
                write!(f, "the invoker {by} is not there")
            }
            Verdict::Missing => f.write_str("not found under the supplied root"),
            Verdict::Uncheckable(Uncheckable::RootNotSupplied(root)) => {
                write!(f, "no --root {root}=<path> was supplied")
            }
            Verdict::Uncheckable(Uncheckable::SettingsNotSupplied) => {
                f.write_str("the script exists; no --settings was supplied to say if it is wired")
            }
            Verdict::Uncheckable(Uncheckable::NoResolver(kind)) => {
                write!(f, "no resolver exists for `{}:` controls", kind.prefix())
            }
            Verdict::Unmapped => f.write_str("controls map does not say where it lives"),
        }
    }
}

/// One control occurrence and its verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row<'a> {
    tag: &'a RuleTag,
    control: &'a Control,
    location: Option<&'a Location>,
    verdict: Verdict,
}

impl<'a> Row<'a> {
    /// The rule naming the control.
    #[must_use]
    pub fn tag(&self) -> &'a RuleTag {
        self.tag
    }

    /// The control, as the rule names it.
    #[must_use]
    pub fn control(&self) -> &'a Control {
        self.control
    }

    /// Where the map places it, if it does.
    #[must_use]
    pub fn location(&self) -> Option<&'a Location> {
        self.location
    }

    /// The verdict.
    #[must_use]
    pub fn verdict(&self) -> &Verdict {
        &self.verdict
    }
}

/// Whether a `[[proposed]]` entry still describes a proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// The rule exists and does not name the control yet.
    Open,
    /// No rule in the corpus has this tag.
    NoSuchRule,
    /// The rule already names the control: the proposal was taken, and the
    /// entry belongs in `[[control]]`.
    Accepted,
}

impl fmt::Display for Standing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Standing::Open => "open",
            Standing::NoSuchRule => "no rule in the corpus has this tag",
            Standing::Accepted => {
                "the rule already names this control; move the entry to [[control]]"
            }
        })
    }
}

/// One `[[proposed]]` entry: the verdict its control would get, and whether the
/// proposal still stands. The verdict is reported and never graded; only a
/// proposal that no longer stands fails the run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalRow<'a> {
    control: &'a ControlId,
    proposal: &'a Proposal,
    verdict: Verdict,
    standing: Standing,
}

impl<'a> ProposalRow<'a> {
    /// The rule the control is proposed for.
    #[must_use]
    pub fn tag(&self) -> &'a RuleTag {
        self.proposal.rule()
    }

    /// The control.
    #[must_use]
    pub fn control(&self) -> &'a ControlId {
        self.control
    }

    /// Where the map places it.
    #[must_use]
    pub fn location(&self) -> &'a Location {
        self.proposal.location()
    }

    /// The verdict it would get if the rule named it.
    #[must_use]
    pub fn verdict(&self) -> &Verdict {
        &self.verdict
    }

    /// Whether the proposal still stands.
    #[must_use]
    pub fn standing(&self) -> Standing {
        self.standing
    }

    /// Whether the proposal still stands. Exhaustive over [`Standing`].
    #[must_use]
    pub fn is_open(&self) -> bool {
        match self.standing {
            Standing::Open => true,
            Standing::NoSuchRule | Standing::Accepted => false,
        }
    }
}

/// Every verdict, every map entry no rule claims, and every proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution<'a> {
    rows: Vec<Row<'a>>,
    unclaimed: Vec<&'a ControlId>,
    proposals: Vec<ProposalRow<'a>>,
}

impl<'a> Resolution<'a> {
    /// One row per control occurrence, rules in tag order, controls in the
    /// order each rule writes them.
    #[must_use]
    pub fn rows(&self) -> &[Row<'a>] {
        &self.rows
    }

    /// Map entries no rule names — a placement for a control that no longer
    /// exists, which is a lying artefact in the map itself.
    #[must_use]
    pub fn unclaimed(&self) -> &[&'a ControlId] {
        &self.unclaimed
    }

    /// Every `[[proposed]]` entry, in map order.
    #[must_use]
    pub fn proposals(&self) -> &[ProposalRow<'a>] {
        &self.proposals
    }

    /// The run's outcome: `Fail` if anything definitely failed or the map is
    /// stale, else `Disarmed` if anything could not be checked, else `Pass`. A
    /// definite failure outranks "could not check", because reporting a
    /// missing hook as merely disarmed would hide the one fact that is known.
    #[must_use]
    pub fn outcome(&self) -> Outcome {
        let rows = self.rows.iter().map(|row| row.verdict.outcome());
        let stale = (!self.unclaimed.is_empty()).then_some(Outcome::Fail);
        // A proposal's verdict is never graded: the rule does not claim the
        // control, so nothing it says is false. One that no longer stands is.
        let lapsed = self
            .proposals
            .iter()
            .any(|p| !p.is_open())
            .then_some(Outcome::Fail);
        rows.chain(stale)
            .chain(lapsed)
            .max()
            .unwrap_or(Outcome::Pass)
    }
}

/// Resolve every control the library's rules name.
#[must_use]
pub fn resolve<'a>(
    library: &'a Library<Validated>,
    map: &'a ControlMap,
    snapshot: &Snapshot,
    settings: Option<&HookRegistry>,
) -> Resolution<'a> {
    let mut rules: Vec<_> = library.rules().iter().collect();
    rules.sort_by(|a, b| a.tag().as_str().cmp(b.tag().as_str()));

    let mut claimed = std::collections::BTreeSet::new();
    let mut rows = Vec::new();
    for rule in &rules {
        let Some(controls) = rule.status().named_controls() else {
            continue;
        };
        for control in controls.as_slice() {
            let id = ControlId::of(control);
            let location = map.locate(&id);
            let verdict = verdict(&id, location, snapshot, settings);
            claimed.insert(id);
            rows.push(Row {
                tag: rule.tag(),
                control,
                location,
                verdict,
            });
        }
    }
    let unclaimed = map
        .entries()
        // A generated entry exists because a detector does; only a hand-written
        // one can go stale.
        .filter(|(_, location)| location.artefact().is_authored())
        .map(|(id, _)| id)
        .filter(|id| !claimed.contains(*id))
        .collect();
    let proposals = map
        .proposals()
        .map(|(id, proposal)| {
            let standing = match rules.iter().find(|r| r.tag() == proposal.rule()) {
                None => Standing::NoSuchRule,
                Some(rule) => match rule.status().named_controls() {
                    Some(named) if named.as_slice().iter().any(|c| ControlId::of(c) == *id) => {
                        Standing::Accepted
                    }
                    Some(_) | None => Standing::Open,
                },
            };
            ProposalRow {
                control: id,
                proposal,
                verdict: verdict(id, Some(proposal.location()), snapshot, settings),
                standing,
            }
        })
        .collect();
    Resolution {
        rows,
        unclaimed,
        proposals,
    }
}

fn verdict(
    id: &ControlId,
    location: Option<&Location>,
    snapshot: &Snapshot,
    settings: Option<&HookRegistry>,
) -> Verdict {
    match id.kind() {
        ControlKind::Type => return Verdict::Uncheckable(Uncheckable::NoResolver(id.kind())),
        ControlKind::Hook
        | ControlKind::UnitTest
        | ControlKind::PropertyTest
        | ControlKind::Gate => {}
    }
    let Some(location) = location else {
        return Verdict::Unmapped;
    };
    let root = location.root();
    let look = |path: &RelPath| snapshot.at(root, path);
    match location.artefact() {
        Artefact::HookScript { file } => match look(file) {
            None => not_supplied(root),
            Some(Found::File(_)) => match settings {
                None => Verdict::Uncheckable(Uncheckable::SettingsNotSupplied),
                Some(registry) if registry.registers(file.file_name()) => {
                    Verdict::ResolvedAndWired(Wired::Registered)
                }
                Some(_) => Verdict::ResolvedNotWired(NotWired::NotRegistered),
            },
            Some(Found::Absent | Found::Tree(_)) => Verdict::Missing,
        },
        // Wired only at the path the pack installs to, so a hand-written
        // original of the same name, still registered mid-migration, is not
        // mistaken for the generated script it is waiting to be replaced by.
        Artefact::GeneratedHook { file, installed } => match look(file) {
            None => not_supplied(root),
            Some(Found::File(_)) => match settings {
                None => Verdict::Uncheckable(Uncheckable::SettingsNotSupplied),
                Some(registry) if registry.registers(installed) => {
                    Verdict::ResolvedAndWired(Wired::Registered)
                }
                Some(_) => Verdict::ResolvedNotWired(NotWired::NotRegistered),
            },
            Some(Found::Absent | Found::Tree(_)) => Verdict::Missing,
        },
        Artefact::TestFunction { within, function } => match look(within) {
            None => not_supplied(root),
            Some(Found::Absent) => Verdict::Missing,
            Some(Found::File(text)) => {
                test_function([(within.as_str(), text.as_str())], function.as_str())
            }
            Some(Found::Tree(files)) => test_function(
                files.iter().map(|(p, t)| (p.as_str(), t.as_str())),
                function.as_str(),
            ),
        },
        Artefact::TestFile { file } => match look(file) {
            None => not_supplied(root),
            Some(Found::File(text)) => {
                if scan::has_running_test(&scan::rust_lines(text)) {
                    Verdict::ResolvedAndWired(Wired::HoldsTests)
                } else {
                    Verdict::ResolvedNotWired(NotWired::NoTests)
                }
            }
            Some(Found::Absent | Found::Tree(_)) => Verdict::Missing,
        },
        Artefact::GateScript { file, invoked_by } => match look(file) {
            None => not_supplied(root),
            Some(Found::File(_)) => invoked(look(invoked_by), invoked_by, file.file_name()),
            Some(Found::Absent | Found::Tree(_)) => Verdict::Missing,
        },
        Artefact::GateFunction {
            file,
            function,
            invoked_by,
        } => match look(file) {
            None => not_supplied(root),
            Some(Found::File(text))
                if scan::shell_fn(&scan::shell_lines(text), function.as_str()) =>
            {
                invoked(look(invoked_by), invoked_by, function.as_str())
            }
            Some(Found::File(_) | Found::Absent | Found::Tree(_)) => Verdict::Missing,
        },
    }
}

fn not_supplied(root: &RootName) -> Verdict {
    Verdict::Uncheckable(Uncheckable::RootNotSupplied(root.clone())) // allow:clone: the verdict outlives the borrow of the map it names
}

/// The verdict on a test function searched for across one or more files: the
/// strongest definition found wins, so a test duplicated as a helper elsewhere
/// does not hide the one the harness runs.
fn test_function<'t>(files: impl IntoIterator<Item = (&'t str, &'t str)>, name: &str) -> Verdict {
    let mut best: Option<(TestMark, &str)> = None;
    for (path, text) in files {
        let Some(mark) = scan::rust_fn(&scan::rust_lines(text), name) else {
            continue;
        };
        let rank = |m: TestMark| match m {
            TestMark::Runs => 2,
            TestMark::Ignored => 1,
            TestMark::NotATest => 0,
        };
        if best.is_none_or(|(b, _)| rank(mark) > rank(b)) {
            best = Some((mark, path));
        }
    }
    match best {
        None => Verdict::Missing,
        Some((TestMark::Runs, path)) => {
            Verdict::ResolvedAndWired(Wired::RunsAsTest(path.to_owned()))
        }
        Some((TestMark::Ignored, path)) => {
            Verdict::ResolvedNotWired(NotWired::Ignored(path.to_owned()))
        }
        Some((TestMark::NotATest, path)) => {
            Verdict::ResolvedNotWired(NotWired::NotATest(path.to_owned()))
        }
    }
}

/// The verdict on a gate whose artefact exists: does its invoker reference it?
fn invoked(invoker: Option<&Found>, invoked_by: &RelPath, token: &str) -> Verdict {
    let by = || invoked_by.clone(); // allow:clone: the verdict owns the path it reports, outliving the map borrow
    match invoker {
        Some(Found::File(text)) if scan::shell_invokes(&scan::shell_lines(text), token) => {
            Verdict::ResolvedAndWired(Wired::InvokedBy(by()))
        }
        Some(Found::File(_)) => Verdict::ResolvedNotWired(NotWired::NotInvoked(by())),
        Some(Found::Absent | Found::Tree(_)) | None => {
            Verdict::ResolvedNotWired(NotWired::InvokerAbsent(by()))
        }
    }
}
