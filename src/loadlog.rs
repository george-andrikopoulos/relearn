//! `loadlog` — what the live load log says about each home: COLD, STALE,
//! SHADOWED, and LOADED.
//!
//! The log is what `load-log-pack`'s hook appends, one JSON object per line:
//! a `skill` event for every `Skill` call (PostToolUse), and an `instructions`
//! event for every `CLAUDE.md` or `.claude/rules/` file the loader reads
//! (InstructionsLoaded). The hook records where it found the file and the hash
//! in that file's `relearn:generated` marker; this module decides what they
//! mean.
//!
//! **Home-level, not rule-level.** A load is a home's whole layer arriving in a
//! session. Whether one rule inside it was read is not something any event
//! carries; the probes are the candidate for that.
//!
//! **Must NOT:** perform I/O, or be reachable from `build`.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::loadset::{LayerSlug, Outcome};
use crate::rule::Date;

/// The form a home's layer arrives in. The two are emitted by different
/// targets with different bodies, so each has its own expected hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Form {
    /// \`skills/<slug>/SKILL.md\`, opened with the \`Skill\` tool.
    Skill,
    /// \`.claude/rules/<slug>.md\`, read by the loader with no tool call.
    RulesLayer,
}

impl Form {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Form::Skill => "skill",
            Form::RulesLayer => "rules layer",
        }
    }
}

/// Where the copy that loaded lives. The hook records it: for a skill, by
/// which of its lookups found the file; for a rules file, from the loader's own
/// \`memory_type\`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Location {
    /// A project's \`.claude/\` — its skills or its rules.
    Project,
    /// The user's \`~/.claude/\`, outside \`skills/synced/\`.
    User,
    /// \`~/.claude/skills/synced/\`: the copy Claude Code downloads from the
    /// claude.ai account, loaded as \`<plugin>:<skill>\`.
    AccountSynced,
    /// A \`CLAUDE.local.md\` the loader classed as Local.
    Local,
    /// A managed-policy file.
    Managed,
    /// The hook found no file for the name it was given.
    Unresolved,
}

impl Location {
    fn parse(s: Option<&str>) -> Option<Self> {
        Some(match s {
            None => Location::Unresolved,
            Some("project") => Location::Project,
            Some("user") => Location::User,
            Some("account-synced") => Location::AccountSynced,
            Some("local") => Location::Local,
            Some("managed") => Location::Managed,
            Some(_) => return None,
        })
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Location::Project => "project",
            Location::User => "user",
            Location::AccountSynced => "account-synced",
            Location::Local => "local",
            Location::Managed => "managed",
            Location::Unresolved => "unresolved",
        }
    }
}

/// A \`relearn:generated\` marker's sha256: 64 lowercase hex digits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContentHash(String);

impl ContentHash {
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        (s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
        .then(|| ContentHash(s.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Why a log could not be read. Each names the line.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LoadLogError {
    #[error("line {line}: not a JSON object: {reason}")]
    NotJson { line: usize, reason: String },
    #[error("line {line}: `ts` {ts:?} does not start with a YYYY-MM-DD date")]
    Timestamp { line: usize, ts: String },
    #[error("line {line}: unknown event {event:?} (expected skill | instructions)")]
    Event { line: usize, event: String },
    #[error(
        "line {line}: unknown location {location:?} \
         (expected project | user | account-synced | local | managed, or null)"
    )]
    Location { line: usize, location: String },
    #[error("line {line}: `hash` {hash:?} is not 64 lowercase hex digits")]
    Hash { line: usize, hash: String },
}

/// One load of a relearn home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadEvent {
    date: Date,
    session: String,
    home: LayerSlug,
    form: Form,
    location: Location,
    hash: Option<ContentHash>,
}

impl LoadEvent {
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    #[must_use]
    pub fn session(&self) -> &str {
        &self.session
    }

    #[must_use]
    pub fn home(&self) -> &LayerSlug {
        &self.home
    }

    #[must_use]
    pub fn form(&self) -> Form {
        self.form
    }

    #[must_use]
    pub fn location(&self) -> Location {
        self.location
    }

    /// The marker's hash; \`None\` when the copy carried no marker.
    #[must_use]
    pub fn hash(&self) -> Option<&ContentHash> {
        self.hash.as_ref()
    }
}

/// A parsed log: the loads of relearn homes, and how many other loads (a
/// \`CLAUDE.md\`, a skill relearn did not write) were skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedLog {
    events: Vec<LoadEvent>,
    ignored: usize,
}

impl ParsedLog {
    #[must_use]
    pub fn events(&self) -> &[LoadEvent] {
        &self.events
    }

    #[must_use]
    pub fn ignored(&self) -> usize {
        self.ignored
    }

    /// Append another log's loads; several logs read as one.
    pub fn extend(&mut self, other: ParsedLog) {
        self.events.extend(other.events);
        self.ignored += other.ignored;
    }
}

/// The home a load belongs to, if it is a relearn home: a skill by its name
/// (the part after the last \`:\` for an account-synced copy), a rules file by
/// its stem.
fn home_of(event: &str, name: Option<&str>, path: Option<&str>) -> Option<(LayerSlug, Form)> {
    match event {
        "skill" => {
            let name = name?;
            let bare = name.rsplit_once(':').map_or(name, |(_, s)| s);
            LayerSlug::parse(bare).map(|s| (s, Form::Skill))
        }
        _ => {
            let path = path?.replace('\\', "/");
            let (dir, file) = path.rsplit_once('/')?;
            let stem = file.strip_suffix(".md")?;
            // Only a file in a `rules` directory is a rules layer; a CLAUDE.md
            // or an imported file is not a home's layer.
            dir.ends_with("/rules")
                .then(|| LayerSlug::parse(stem).map(|s| (s, Form::RulesLayer)))
                .flatten()
        }
    }
}

/// Parse a log. Blank lines are skipped; anything else that is not a JSON
/// object stops the read, because a truncated log must not read as a quiet one.
///
/// # Errors
/// [\`LoadLogError\`] naming the first line that could not be read.
pub fn parse_log(text: &str) -> Result<ParsedLog, LoadLogError> {
    let mut out = ParsedLog::default();
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        let raw = raw.trim_end_matches('\r');
        if raw.trim().is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(raw).map_err(|e| LoadLogError::NotJson {
            line,
            reason: e.to_string(),
        })?;
        if !v.is_object() {
            return Err(LoadLogError::NotJson {
                line,
                reason: "not an object".into(),
            });
        }
        let event = v["event"].as_str().unwrap_or_default();
        if event != "skill" && event != "instructions" {
            return Err(LoadLogError::Event {
                line,
                event: event.to_owned(),
            });
        }
        let ts = v["ts"].as_str().unwrap_or_default();
        let date = ts
            .get(..10)
            .and_then(|d| Date::parse(d).ok())
            .ok_or_else(|| LoadLogError::Timestamp {
                line,
                ts: ts.to_owned(),
            })?;
        let location_text = v["location"].as_str();
        let location = Location::parse(location_text).ok_or_else(|| LoadLogError::Location {
            line,
            location: location_text.unwrap_or_default().to_owned(),
        })?;
        let hash = match v["hash"].as_str() {
            None => None,
            Some(h) => Some(ContentHash::parse(h).ok_or_else(|| LoadLogError::Hash {
                line,
                hash: h.to_owned(),
            })?),
        };
        match home_of(event, v["name"].as_str(), v["path"].as_str()) {
            Some((home, form)) => out.events.push(LoadEvent {
                date,
                session: v["session"].as_str().unwrap_or_default().to_owned(),
                home,
                form,
                location,
                hash,
            }),
            None => out.ignored += 1,
        }
    }
    Ok(out)
}

/// The days a report covers, inclusive at both ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    since: Date,
    until: Date,
}

impl Window {
    /// \`None\` when the window ends before it starts.
    #[must_use]
    pub fn new(since: Date, until: Date) -> Option<Self> {
        (since <= until).then_some(Window { since, until })
    }

    #[must_use]
    pub fn contains(self, day: Date) -> bool {
        self.since <= day && day <= self.until
    }

    #[must_use]
    pub fn since(self) -> Date {
        self.since
    }

    #[must_use]
    pub fn until(self) -> Date {
        self.until
    }
}

/// What \`relearn build\` would emit now: the content hash of each home's
/// layer, per form.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Expected(BTreeMap<(LayerSlug, Form), ContentHash>);

impl Expected {
    #[must_use]
    pub fn new(entries: impl IntoIterator<Item = ((LayerSlug, Form), ContentHash)>) -> Self {
        Expected(entries.into_iter().collect())
    }

    fn get(&self, home: &LayerSlug, form: Form) -> Option<&ContentHash> {
        self.0.get(&(home.clone(), form)) // allow:clone: a tuple key is looked up by value; the map owns its keys
    }
}

/// One finding of the reader. Closed: a new kind cannot be reported until
/// \`render\` and the exit status answer it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadFinding {
    /// The home did not load in any form, from anywhere, in the window.
    Cold { home: LayerSlug },
    /// The home loaded with a hash \`relearn build\` would not emit now — or
    /// with no marker at all, which \`build\` always writes — from one location
    /// class, in this many sessions. \`found\` is the hash that loaded.
    Stale {
        home: LayerSlug,
        form: Form,
        location: Location,
        found: Option<ContentHash>,
        sessions: u32,
    },
    /// The home loaded in one form from two or more location classes in the
    /// window: the skill-side twin of \`loadset\`'s \`DuplicateLayer\`, over a
    /// window rather than inside one session, and so carrying classes rather
    /// than paths.
    Shadowed {
        home: LayerSlug,
        form: Form,
        locations: Vec<Location>,
    },
}

impl LoadFinding {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            LoadFinding::Cold { .. } => "COLD",
            LoadFinding::Stale { .. } => "STALE",
            LoadFinding::Shadowed { .. } => "SHADOWED",
        }
    }
}

impl std::fmt::Display for LoadFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadFinding::Cold { home } => {
                write!(
                    f,
                    "COLD {}: no load in any form in the window",
                    home.as_str()
                )
            }
            LoadFinding::Stale {
                home,
                form,
                location,
                found,
                sessions,
            } => write!(
                f,
                "STALE {} ({}, {} copy): {} — not what `relearn build` emits now; {} session(s)",
                home.as_str(),
                form.as_str(),
                location.as_str(),
                found
                    .as_ref()
                    .map_or("no relearn:generated marker".to_owned(), |h| format!(
                        "sha256 {}…",
                        &h.as_str()[..12]
                    )),
                sessions
            ),
            LoadFinding::Shadowed {
                home,
                form,
                locations,
            } => write!(
                f,
                "SHADOWED {} ({}): loaded from {}",
                home.as_str(),
                form.as_str(),
                locations
                    .iter()
                    .map(|l| l.as_str())
                    .collect::<Vec<_>>()
                    .join(" and ")
            ),
        }
    }
}

/// The reader's report over one window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadReport {
    in_window: usize,
    findings: Vec<LoadFinding>,
    loaded: BTreeMap<(LayerSlug, Location), u32>,
}

impl LoadReport {
    /// COLD first, then STALE, then SHADOWED; each by home.
    #[must_use]
    pub fn findings(&self) -> &[LoadFinding] {
        &self.findings
    }

    /// Loads of \`home\` from \`location\` in the window, every form.
    #[must_use]
    pub fn loaded(&self, home: &LayerSlug, location: Location) -> u32 {
        self.loaded
            .get(&(home.clone(), location)) // allow:clone: lookup by owned tuple key
            .copied()
            .unwrap_or(0)
    }

    /// Every (home, location) count, in order.
    pub fn loaded_rows(&self) -> impl Iterator<Item = (&LayerSlug, Location, u32)> {
        self.loaded.iter().map(|((h, l), n)| (h, *l, *n))
    }

    /// All loads in the window.
    #[must_use]
    pub fn loaded_total(&self) -> u32 {
        self.loaded.values().sum()
    }

    /// Home loads that fell in the window.
    #[must_use]
    pub fn in_window(&self) -> usize {
        self.in_window
    }

    /// No load in the window at all is \`Unchecked\`, not every home cold: the
    /// likelier reading is that the hook is not installed, and a report that
    /// cannot tell the two apart must not pass either.
    #[must_use]
    pub fn outcome(&self) -> Outcome {
        if self.in_window == 0 {
            Outcome::Unchecked
        } else if self.findings.is_empty() {
            Outcome::Clean
        } else {
            Outcome::Findings
        }
    }
}

/// Read the loads in \`window\` against what \`build\` emits now. \`homes\` is
/// every home the library has, so a home that never loaded can be named.
#[must_use]
pub fn read(
    events: &[LoadEvent],
    window: Window,
    expected: &Expected,
    homes: &[LayerSlug],
) -> LoadReport {
    let seen: Vec<&LoadEvent> = events.iter().filter(|e| window.contains(e.date)).collect();

    let mut loaded: BTreeMap<(LayerSlug, Location), u32> = BTreeMap::new();
    let mut classes: BTreeMap<(&LayerSlug, Form), BTreeSet<Location>> = BTreeMap::new();
    let mut stale: BTreeMap<(&LayerSlug, Form, Location, Option<&ContentHash>), BTreeSet<&str>> =
        BTreeMap::new();
    for e in &seen {
        *loaded.entry((e.home.clone(), e.location)).or_insert(0) += 1; // allow:clone: the report owns its keys, outliving the events
        classes
            .entry((&e.home, e.form))
            .or_default()
            .insert(e.location);
        let current = match (e.hash.as_ref(), expected.get(&e.home, e.form)) {
            (Some(found), Some(want)) => found == want,
            // No marker, or a form `build` no longer emits for this home.
            (None, _) | (Some(_), None) => false,
        };
        if !current {
            stale
                .entry((&e.home, e.form, e.location, e.hash.as_ref()))
                .or_default()
                .insert(&e.session);
        }
    }

    let loaded_homes: BTreeSet<&LayerSlug> = seen.iter().map(|e| &e.home).collect();
    let mut findings: Vec<LoadFinding> = homes
        .iter()
        .filter(|h| !loaded_homes.contains(h))
        .map(|h| LoadFinding::Cold { home: h.clone() }) // allow:clone: the finding owns the home it names
        .collect();
    findings.extend(
        stale.into_iter().map(
            |((home, form, location, found), sessions)| LoadFinding::Stale {
                home: home.clone(), // allow:clone: same
                form,
                location,
                found: found.cloned(),
                sessions: u32::try_from(sessions.len()).unwrap_or(u32::MAX),
            },
        ),
    );
    findings.extend(classes.into_iter().filter_map(|((home, form), locs)| {
        // An unresolved load says nothing about where a copy lives, so it
        // cannot be one of two places.
        let placed: Vec<Location> = locs
            .into_iter()
            .filter(|l| *l != Location::Unresolved)
            .collect();
        (placed.len() > 1).then(|| LoadFinding::Shadowed {
            home: home.clone(), // allow:clone: same
            form,
            locations: placed,
        })
    }));

    LoadReport {
        in_window: seen.len(),
        findings,
        loaded,
    }
}
