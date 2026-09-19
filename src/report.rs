//! `report` — the anonymous recurrence report.
//!
//! Separate flow, separate file, separate privacy model from
//! [`contribute`](crate::contribute). A contribution is **attributed**, because
//! it is authorship and credit is what makes anyone contribute. A report is
//! **anonymous, always**, because recurrence is somebody writing down that a
//! rule of theirs failed, and nobody does that with their name on it.
//!
//! What leaves: an upstream tag, a bucketed count, a month, a status kind, a
//! control kind. The field list is defined as much by what is absent — no
//! title, no incident, no body, no path, no name, no repository, no language,
//! **no day-level date anywhere**. Every one of those absences is structural
//! here: [`Observation`] holds the five fields and nothing else, and [`Month`]
//! cannot represent a day.
//!
//! Two things this module deliberately does not do. It does not transmit —
//! `report` writes a file and a person opens the pull request, which is why
//! §9's aggregate is a repository rather than a service. And it does not
//! decide whether a count may be published: the k-floor is the aggregate's to
//! apply, and [`K_ANONYMITY_FLOOR`] is here so both sides read one constant
//! rather than two policies.

use std::fmt;
use std::fmt::Write as _;

use crate::library::{Library, Validated};
use crate::rule::{ControlKind, Date, Provenance, Rule, RuleTag, Status};

/// The number of distinct installs that must have reported a rule before the
/// aggregate publishes any count for it.
///
/// **A published constant a reader can check, not a policy they must trust.**
/// Five is the standard statistical-disclosure cell-suppression floor — national
/// statistics offices commonly use three or five — so the number can be looked
/// up rather than taken on the author's taste. Below it, a count of one is a
/// finger pointing at someone.
///
/// It lives beside the report rather than only in the aggregate so that both
/// ends read the same number: a floor the producer cannot see is a floor the
/// consumer can quietly lower.
pub const K_ANONYMITY_FLOOR: usize = 5;

/// A month. **There is no day here, and that is the point.**
///
/// Dates coarsen to the month as a privacy decision rather than a formatting
/// one: in a team of eight, *"someone hit this on 28 August"* identifies a
/// person to anyone who was in the room. Making the type incapable of holding a
/// day means no formatting mistake, no forgotten field and no later edit can
/// reintroduce one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Month {
    year: i32,
    month: u8,
}

/// A string was not a well-formed `YYYY-MM`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("expected a month as `YYYY-MM` (got {0:?})")]
pub struct MonthError(String);

impl Month {
    /// Parse `YYYY-MM`.
    pub fn parse(s: &str) -> Result<Self, MonthError> {
        let (year, month) = s.trim().split_once('-').ok_or(MonthError(s.to_owned()))?;
        let year: i32 = year.parse().map_err(|_| MonthError(s.to_owned()))?;
        let month: u8 = month.parse().map_err(|_| MonthError(s.to_owned()))?;
        if !(1..=12).contains(&month) || !(1000..=9999).contains(&year) {
            return Err(MonthError(s.to_owned()));
        }
        Ok(Month { year, month })
    }

    /// The month a date falls in — **the projection that drops the day**, and
    /// the only way a [`Date`] reaches a report.
    #[must_use]
    pub fn of(date: Date) -> Self {
        Month {
            year: date.year(),
            month: date.month(),
        }
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

/// A recurrence count, coarsened.
///
/// **Counts publish as buckets because an exact count plus a month is a
/// fingerprint**: reported month after month, exact counts stitch a rotating
/// pseudonym back into one install. The decay curve the aggregate is for needs
/// orders of magnitude, not integers, so the analytical cost is near zero.
///
/// The boundaries are published rather than tuned: `1-4`, `5-9`, `10+`.
///
/// **There is deliberately no bucket of one** (merged 2026-09-19). The design
/// specified `1 | 2-4 | 5-9 | 10+` and the federation programme warned, of that
/// same boundary, that "a bucket of 1–1 is not a bucket"; the design won where
/// the two disagreed, so a spelling that published an exact count shipped. The
/// k-anonymity floor does not reach it — that floor protects the **aggregate**,
/// while a raw report is a file sitting in a public git repository, where
/// `recurrences = "1"` is an exact count for one install, republished every
/// month it appears. That is the fingerprint the coarsening exists to prevent,
/// arriving at the only size where a fingerprint is worth having.
///
/// Merging costs almost nothing analytically: the decay curve the aggregate is
/// for needs orders of magnitude, and one-versus-four is not one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bucket {
    /// One to four.
    Few,
    /// Five to nine.
    Several,
    /// Ten or more.
    Many,
}

impl Bucket {
    /// The bucket a count falls in.
    #[must_use]
    pub fn of(count: usize) -> Self {
        // Zero is unreachable from `Report::of`, which drops a rule with no
        // recurrences before it gets here (`latest_recurrence()?`). It is
        // mapped rather than refused because this is a total function over
        // `usize` and an unreachable arm is cheaper than a panic nobody can
        // trigger — but nothing ever publishes it.
        match count {
            0..=4 => Bucket::Few,
            5..=9 => Bucket::Several,
            _ => Bucket::Many,
        }
    }

    /// The published spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Bucket::Few => "1-4",
            Bucket::Several => "5-9",
            Bucket::Many => "10+",
        }
    }
}

/// The kind of control a graduated rule moved to, if it names exactly one.
///
/// **Never the control's own name.** `gate:internal-payments-lint` publishes
/// `gate` and the rest of that string stays on the machine — which is now a
/// property of the types rather than of this function, because [`Controls`]
/// holds a control's kind and its name apart instead of in one string that had
/// to be re-split here.
///
/// Active and atticked rules have no control. A **partial** graduation does have
/// one and still reports none: this reads `Status::whole_class_claim`, which is
/// `None` for `Partial`, so a partly-held rule contributes its recurrences
/// without a control kind. That is an under-report rather than a false one, and
/// publishing less is the safe direction for something that leaves the machine.
/// `TODO.md` carries it.
///
/// A destination naming **two** kinds — this corpus has three such — reports
/// none: the field is single-valued, and choosing one of the two would be
/// inventing a fact. Losing the signal is the honest failure.
pub fn control_of(status: &Status) -> Option<ControlKind> {
    status
        .whole_class_claim()
        .and_then(|(to, _)| to.sole_kind())
}

/// This install's pseudonym.
///
/// **Eight lowercase hexadecimal characters, so a name-shaped id cannot be
/// typed.** `george-laptop` is not a pseudonym and the type refuses it; the
/// shape is the control, because nothing downstream can tell a chosen alias
/// from a random one.
///
/// Where it lives is §0.2's decision: inside the cloned aggregate repository,
/// never on the machine — `reports/<install-id>.toml` **is** the id, the clone
/// is a path the human passes, and deleting the clone deletes the pseudonym.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstallId(String);

/// A string was not a well-formed install pseudonym.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "an install id is eight lowercase hex characters (got {0:?}) — it is a pseudonym, not a name"
)]
pub struct InstallIdError(String);

impl InstallId {
    /// Parse an install pseudonym.
    pub fn parse(s: &str) -> Result<Self, InstallIdError> {
        let s = s.trim();
        let well_formed = s.len() == 8
            && s.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
        if well_formed {
            Ok(InstallId(s.to_owned()))
        } else {
            Err(InstallIdError(s.to_owned()))
        }
    }

    /// The pseudonym.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One rule's recurrence signal, as it leaves the machine.
///
/// Five fields and **no others**: the type is the field list, so a sixth cannot
/// be added by an edit to a renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    rule: RuleTag,
    recurrences: Bucket,
    latest: Month,
    status: &'static str,
    control: Option<ControlKind>,
}

impl Observation {
    /// The upstream tag this observation is about.
    #[must_use]
    pub fn rule(&self) -> &RuleTag {
        &self.rule
    }

    /// The bucketed count.
    #[must_use]
    pub fn recurrences(&self) -> Bucket {
        self.recurrences
    }

    /// The month of the most recent recurrence.
    #[must_use]
    pub fn latest(&self) -> Month {
        self.latest
    }

    /// The kind of control holding the rule, if it names exactly one.
    #[must_use]
    pub fn control(&self) -> Option<ControlKind> {
        self.control
    }
}

/// One install's report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    install: InstallId,
    generated: Month,
    observations: Vec<Observation>,
}

impl Report {
    /// The schema version the format is at.
    pub const SCHEMA: u32 = 1;

    /// Build a report from a library.
    ///
    /// **Four conditions, and a rule reports only if it meets all of them.**
    /// Each is a different privacy or honesty question, and none is a
    /// convenience:
    ///
    /// * Its home is publishable — A2's exclusion in its second flow, so an
    ///   org-homed rule never appears here, exactly as it can never be
    ///   contributed.
    /// * Its authority names an upstream. **Only upstream tags have a shared
    ///   identity**; a local-only tag is a private name, and publishing one is
    ///   precisely the leak this flow is shaped to avoid. B1's `Authority` is
    ///   what answers the question.
    /// * It is not mandated — a mandate was never mined, so it is not
    ///   recurrence evidence, here as in the local tally.
    /// * It has actually recurred. Reporting a zero for every cached rule would
    ///   publish the shape of the local corpus, which is a different secret.
    #[must_use]
    pub fn of(library: &Library<Validated>, install: InstallId, generated: Month) -> Self {
        let observations = library
            .rules()
            .iter()
            .filter(|rule| reportable(rule))
            .filter_map(|rule| {
                Some(Observation {
                    rule: rule.tag().clone(), // allow:clone: the report owns its observations and outlives the library borrow it was built from
                    recurrences: Bucket::of(rule.recurrences().len()),
                    latest: Month::of(rule.latest_recurrence()?),
                    status: status_kind(rule.status()),
                    control: control_of(rule.status()),
                })
            })
            .collect();

        Report {
            install,
            generated,
            observations,
        }
    }

    /// The observations, in library order.
    #[must_use]
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }

    /// The pseudonym this report is filed under — which is also the name of the
    /// file it belongs in, since `reports/<install-id>.toml` **is** the id.
    #[must_use]
    pub fn install(&self) -> &InstallId {
        &self.install
    }

    /// Render the report as the TOML document a human publishes.
    #[must_use]
    pub fn to_toml(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "schema    = {}", Self::SCHEMA);
        let _ = writeln!(out, "install   = \"{}\"", self.install.as_str());
        let _ = writeln!(out, "generated = \"{}\"", self.generated);
        for observation in &self.observations {
            out.push_str("\n[[observation]]\n");
            let _ = writeln!(out, "rule        = \"{}\"", observation.rule.as_str());
            let _ = writeln!(
                out,
                "recurrences = \"{}\"",
                observation.recurrences.as_str()
            );
            let _ = writeln!(out, "latest      = \"{}\"", observation.latest);
            let _ = writeln!(out, "status      = \"{}\"", observation.status);
            if let Some(control) = observation.control {
                let _ = writeln!(out, "control     = \"{}\"", control.published());
            }
        }
        out
    }
}

/// Whether a rule may appear in a report at all. See [`Report::of`] for why each
/// condition is there; it is written as one function so the four questions are
/// asked in one place and no caller can ask three of them.
fn reportable(rule: &Rule) -> bool {
    let upstream = rule.authority().provenance() == Provenance::FromUpstream;
    rule.is_publishable()
        && upstream
        && rule.counts_toward_recurrence_statistics()
        && rule.has_recurred()
}

/// The status kind, as the report spells it. A closed set: no payload travels,
/// so a graduation destination and an attic reason both stay on the machine.
fn status_kind(status: &Status) -> &'static str {
    match status {
        Status::Active => "active",
        Status::Partial { .. } => "partial",
        Status::Graduated { .. } => "graduated",
        Status::Attic { .. } => "attic",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_month_cannot_hold_a_day() {
        assert_eq!(
            Month::parse("2026-09").expect("valid").to_string(),
            "2026-09"
        );
        assert!(Month::parse("2026-09-13").is_err());
        assert!(Month::parse("2026-13").is_err());
        assert!(Month::parse("2026").is_err());
        assert!(Month::parse("").is_err());
    }

    #[test]
    fn a_month_projects_a_date_by_dropping_the_day() {
        let date = Date::parse("2026-08-30").expect("valid date");
        assert_eq!(Month::of(date).to_string(), "2026-08");
    }

    // A zero count cannot occur (a rule with no recurrences is not reported),
    // and if it somehow did it must not render as its own distinguishable
    // bucket — the smallest published count is the smallest bucket.
    /// Zero is unreachable from `Report::of`, which drops a rule with no
    /// recurrences before bucketing. Pinned anyway because `of` is total over
    /// `usize`, and the arm it lands in must be the one that publishes a range
    /// rather than a count.
    #[test]
    fn the_smallest_bucket_absorbs_zero() {
        assert_eq!(Bucket::of(0), Bucket::Few);
        assert_eq!(Bucket::of(0).as_str(), "1-4");
    }

    // Kept, and inverted. This test used to assert that an unknown prefix
    // *silently named no kind* — the status was constructible and the report
    // simply had nothing to say about it. Since `Controls` parses at the
    // perimeter that state is unreachable: the rule stops the build instead.
    // The weaker guarantee is not deleted, it is superseded, and the test now
    // pins which of the two holds.
    #[test]
    fn an_unknown_control_prefix_is_refused_rather_than_silently_unreported() {
        assert!(
            Status::graduated(
                "ritual:standing-up-slowly",
                Date::parse("2026-07-21").expect("valid date"),
            )
            .is_err(),
            "an unknown control kind must stop the build, not report as absent"
        );
    }

    // Two mentions of the *same* kind are still one kind: `hook:a + hook:b` is
    // held by hooks, and saying so loses nothing.
    #[test]
    fn one_kind_named_twice_is_still_that_kind() {
        let status = Status::graduated(
            "hook:gate-verdict-intact + hook:multiline-pattern-eol",
            Date::parse("2026-09-06").expect("valid date"),
        )
        .expect("non-empty destination");
        assert_eq!(control_of(&status), Some(ControlKind::Hook));
    }
}
