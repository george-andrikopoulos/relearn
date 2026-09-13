//! `aggregate` — many installs' reports, recomputed into one document.
//!
//! **A repository, not a service.** Reports arrive as pull requests into a
//! clone; a scheduled job runs `relearn aggregate` over them and commits the
//! result. That is the entire infrastructure, and it is the reason `relearn`
//! itself never speaks to anything: publication is deliberate by construction
//! because it is a pull request, every byte that ever crossed is in public
//! history, review is free because it is code review, and if it fails it fails
//! as an empty repository rather than as an outage.
//!
//! Three properties this module exists to hold, each one named in the
//! programme as what goes wrong:
//!
//! * **The k-floor is applied here**, and below it a rule does not appear at
//!   all — not its count, and not its tag. Naming a rule while withholding its
//!   number points at the same person.
//! * **The confounds print beside the numbers, unconditionally.** They are not
//!   a flag, because the way they get dropped is by being droppable.
//! * **The aggregate is never authoritative.** Nothing in the build path reads
//!   it; a build that behaved differently for having seen one would make the
//!   instruction layer depend on a downloaded file, which is a different
//!   product.
//!
//! Reports come from strangers, so parsing is strict: an unknown schema version
//! is refused rather than read optimistically, because silently ignoring fields
//! it cannot interpret is how an aggregate miscounts with nobody noticing.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::Deserialize;

use crate::report::{InstallId, Month, MonthError};

pub use crate::report::K_ANONYMITY_FLOOR;

/// The schema version this build reads and writes.
pub const SCHEMA: u32 = 1;

/// The two sentences that ship beside every aggregate, always.
///
/// **Not a flag and not a verbosity level.** Cross-install recurrence measures
/// frequency *and* diligence, inseparably: a class that appears rarely may be
/// rare, or may be one nobody admits to. And since the error-class catalogue was
/// dropped (§12.1), the aggregate can only ever count classes somebody has
/// published a rule for — a bias toward the cheap incidents, not mere
/// sparseness, because the authoring-and-scrubbing bar is highest exactly where
/// the incidents are most sensitive.
///
/// A headline figure without both is the overclaim this project exists to
/// prevent, so they are part of the document's construction rather than an
/// option its renderer takes.
pub const CONFOUNDS: [&str; 2] = [
    "Cross-install recurrence measures frequency AND diligence, inseparably: a class that \
     appears rarely may be rare, or may be one nobody admits to.",
    "This counts only classes somebody has published a rule for, never the universe of error \
     classes — a bias toward cheap incidents, since the authoring and scrubbing bar is highest \
     where incidents are most sensitive.",
];

/// Why an aggregate could not be recomputed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AggregateError {
    /// A report was not valid TOML.
    #[error("a report is not valid TOML: {0}")]
    Toml(String),
    /// A report declared a schema version this build does not understand.
    ///
    /// Refused rather than read as far as it goes: a future schema may mean
    /// something different by a field this one recognises, and an aggregate
    /// that guesses is an aggregate that miscounts.
    #[error("a report declares schema {found}, and this build reads schema {SCHEMA}")]
    UnknownSchema {
        /// The version the report declared.
        found: u32,
    },
    /// A report's install id was not a well-formed pseudonym.
    #[error("a report's install id is not a pseudonym: {0}")]
    InstallId(String),
    /// A month field was malformed.
    #[error("a report has a malformed month: {0}")]
    Month(MonthError),
    /// A report used a bucket spelling this build does not know.
    #[error("a report uses an unknown recurrence bucket `{0}`")]
    UnknownBucket(String),
}

/// One install's report, as it arrives from a stranger.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReport {
    schema: u32,
    install: String,
    #[allow(dead_code)]
    generated: String,
    #[serde(default)]
    observation: Vec<RawObservation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawObservation {
    rule: String,
    recurrences: String,
    latest: String,
    #[allow(dead_code)]
    status: String,
    #[serde(default)]
    control: Option<String>,
}

/// What one install said about one rule: which bucket it fell in, which control
/// kind holds it if any, and the month it last recurred.
type Said = (String, Option<String>, Month);

/// Every install's word on every rule: `tag -> install ->`[`Said`].
///
/// Keyed by install at the inner level **on purpose**: a repeated report from
/// one install replaces rather than accumulates, so the floor below counts
/// distinct installs and cannot be cleared by one person reporting five times.
type Heard = BTreeMap<String, BTreeMap<String, Said>>;

/// The published spellings of a recurrence bucket, in order.
///
/// The aggregate does not re-bucket: it counts how many installs fell in each,
/// so the distribution is readable without any install's exact number existing
/// anywhere.
const BUCKETS: [&str; 4] = ["1", "2-4", "5-9", "10+"];

/// One rule's cross-install signal, above the floor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    tag: String,
    installs: usize,
    distribution: BTreeMap<String, usize>,
    controls: Vec<String>,
    latest: Month,
}

impl Row {
    /// The rule this row is about.
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// How many **distinct** installs reported it. The headline number, and the
    /// one the floor is about.
    #[must_use]
    pub fn installs(&self) -> usize {
        self.installs
    }

    /// How many installs fell in a given bucket.
    #[must_use]
    pub fn in_bucket(&self, bucket: &str) -> usize {
        self.distribution.get(bucket).copied().unwrap_or(0)
    }

    /// The control **kinds** seen, sorted and deduplicated — never a control's
    /// name, which no report carries in the first place.
    #[must_use]
    pub fn controls(&self) -> &[String] {
        &self.controls
    }

    /// The most recent month any install reported.
    #[must_use]
    pub fn latest(&self) -> Month {
        self.latest
    }
}

/// Everything the aggregate publishes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aggregate {
    generated: Month,
    rows: Vec<Row>,
    suppressed: usize,
    installs: usize,
}

impl Aggregate {
    /// Recompute from report documents.
    ///
    /// Duplicate pseudonyms collapse: the floor counts **distinct** installs,
    /// because one install reporting the same rule ten times is the obvious way
    /// to fake a population, and a floor that counted reports would be no floor
    /// at all.
    pub fn of<'a>(
        reports: impl Iterator<Item = &'a str>,
        generated: Month,
    ) -> Result<Self, AggregateError> {
        let mut seen: Heard = BTreeMap::new();
        let mut installs: BTreeSet<String> = BTreeSet::new();

        for text in reports {
            let raw: RawReport =
                toml::from_str(text).map_err(|e| AggregateError::Toml(e.to_string()))?;
            if raw.schema != SCHEMA {
                return Err(AggregateError::UnknownSchema { found: raw.schema });
            }
            let install = InstallId::parse(&raw.install)
                .map_err(|_| AggregateError::InstallId(raw.install.clone()))? // allow:clone: the error owns the offending id on the failure path, where nothing else needs it
                .as_str()
                .to_owned();
            installs.insert(install.clone()); // allow:clone: the set and the per-rule maps both key on the id, and it is eight characters

            for observation in raw.observation {
                if !BUCKETS.contains(&observation.recurrences.as_str()) {
                    return Err(AggregateError::UnknownBucket(observation.recurrences));
                }
                let latest = Month::parse(&observation.latest).map_err(AggregateError::Month)?;
                seen.entry(observation.rule).or_default().insert(
                    install.clone(),
                    (observation.recurrences, observation.control, latest),
                ); // allow:clone: each rule's map owns its own copy of the id it is keyed by
            }
        }

        let mut rows = Vec::new();
        let mut suppressed = 0;
        for (tag, by_install) in seen {
            // The floor, applied here and nowhere else. Below it the rule does
            // not appear at all: publishing the tag while withholding the count
            // points at the same person.
            if by_install.len() < K_ANONYMITY_FLOOR {
                suppressed += 1;
                continue;
            }
            let mut distribution: BTreeMap<String, usize> = BTreeMap::new();
            let mut controls: BTreeSet<String> = BTreeSet::new();
            let mut latest: Option<Month> = None;
            for (bucket, control, month) in by_install.values() {
                *distribution.entry(bucket.clone()).or_insert(0) += 1; // allow:clone: the distribution owns its bucket keys, which outlive the borrowed report data
                if let Some(control) = control {
                    controls.insert(control.clone()); // allow:clone: the row owns the kinds it publishes
                }
                latest = Some(latest.map_or(*month, |held: Month| held.max(*month)));
            }
            let installs = by_install.len();
            rows.push(Row {
                tag,
                installs,
                distribution,
                controls: controls.into_iter().collect(),
                latest: latest.unwrap_or(generated),
            });
        }

        Ok(Aggregate {
            generated,
            rows,
            suppressed,
            installs: installs.len(),
        })
    }

    /// The published rows — every rule at or above the floor.
    #[must_use]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// How many rules were withheld for being below the floor.
    ///
    /// Published as a number so a reader can tell a small corpus from a heavily
    /// suppressed one. The tags are not published, which is the whole point of
    /// withholding them.
    #[must_use]
    pub fn suppressed(&self) -> usize {
        self.suppressed
    }

    /// How many distinct installs contributed reports.
    #[must_use]
    pub fn installs(&self) -> usize {
        self.installs
    }

    /// Render the document a scheduled job commits.
    ///
    /// The confounds are written by this function with no condition attached,
    /// so there is no invocation of it that omits them.
    #[must_use]
    pub fn to_toml(&self) -> String {
        let mut out = String::new();
        for line in CONFOUNDS {
            for wrapped in wrap(line, 76) {
                let _ = writeln!(out, "# {wrapped}");
            }
            out.push_str("#\n");
        }
        let _ = writeln!(out, "schema     = {SCHEMA}");
        let _ = writeln!(out, "generated  = \"{}\"", self.generated);
        let _ = writeln!(out, "k_floor    = {K_ANONYMITY_FLOOR}");
        let _ = writeln!(out, "installs   = {}", self.installs);
        let _ = writeln!(
            out,
            "suppressed = {}   # rules below the floor; their tags are not published",
            self.suppressed
        );

        for row in &self.rows {
            out.push_str("\n[[rule]]\n");
            let _ = writeln!(out, "tag      = \"{}\"", row.tag);
            let _ = writeln!(out, "installs = {}", row.installs);
            let _ = writeln!(out, "latest   = \"{}\"", row.latest);
            let counts: Vec<String> = BUCKETS
                .iter()
                .map(|b| format!("\"{b}\" = {}", row.in_bucket(b)))
                .collect();
            let _ = writeln!(out, "buckets  = {{ {} }}", counts.join(", "));
            let kinds: Vec<String> = row.controls.iter().map(|c| format!("\"{c}\"")).collect();
            let _ = writeln!(out, "controls = [{}]", kinds.join(", "));
        }
        out
    }
}

/// Wrap a sentence to `width` columns for the comment block.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapping_keeps_every_word() {
        let wrapped = wrap("one two three four five", 9);
        assert!(wrapped.len() > 1);
        assert_eq!(wrapped.join(" "), "one two three four five");
    }

    #[test]
    fn the_floor_is_the_reports_floor() {
        // One constant, read by the producer and the aggregate both: a floor the
        // two ends could disagree about is a floor either could lower.
        assert_eq!(K_ANONYMITY_FLOOR, crate::report::K_ANONYMITY_FLOOR);
    }

    #[test]
    fn an_unknown_bucket_is_refused() {
        let text = "schema = 1\ninstall = \"7f3c9a1e\"\ngenerated = \"2026-09\"\n\n\
                    [[observation]]\nrule = \"R:x\"\nrecurrences = \"7\"\n\
                    latest = \"2026-08\"\nstatus = \"active\"\n";
        let month = Month::parse("2026-09").expect("valid month");
        assert_eq!(
            Aggregate::of([text].into_iter(), month),
            Err(AggregateError::UnknownBucket("7".to_owned()))
        );
    }
}
