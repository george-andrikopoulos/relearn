//! `Status` — a rule's lifecycle state, each variant carrying exactly the data
//! that state requires. "Graduated" cannot exist without a destination and
//! "atticked" cannot exist without a reason and a date, because those live in
//! the variant and there is no other way to build it.

use super::control::{ControlError, Controls};
use super::date::Date;
use super::text::{EmptyText, nonempty};

/// Why a status could not be built.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StatusError {
    /// The controls could not be read.
    #[error("field `status.by`: {0}")]
    Control(#[from] ControlError),
    /// A required text field was empty.
    #[error("{0}")]
    Text(#[from] EmptyText),
}

/// Why a rule was atticked — non-empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason(String);

impl Reason {
    /// Parse a non-empty attic reason.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("attic reason", s)?))
    }

    /// The reason text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What a *partial* graduation does **not** cover -- non-empty.
///
/// Mandatory in [`Status::Partial`] rather than optional, and that is the whole
/// point of the variant. A partial graduation that does not say which part is
/// still uncovered reads exactly like a full one: it names a control, the reader
/// stops looking, and the uncovered half is protected by nothing while appearing
/// to be held -- `[R:guarantee-needs-a-reader]` wearing a status. Living in the
/// variant means "partly held, but I will not say what is missing" cannot be
/// constructed at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uncovered(String);

impl Uncovered {
    /// Parse a non-empty statement of what a partial graduation leaves uncovered.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("uncovered part", s)?))
    }

    /// The uncovered-part text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether prose is still holding any part of a rule.
///
/// The authority for two of [`crate::lint`]'s questions: whether a recurrence is
/// an *unheld* one, and whether a citation points at a rule whose prose has
/// stopped carrying it. `Active` and `Partial` answer `Holds` for both -- a
/// partly-held rule is live guidance and perfectly ordinary to cite.
///
/// Produced by an exhaustive match on [`Status`] for the same reason as
/// [`Emittability`]: a future status variant cannot compile until somebody
/// decides whether prose still carries it. The previous form was a
/// `matches!(status, Status::Active)` at the call site, which would have
/// answered "no" for a new variant *silently* -- the failure mode this codebase
/// pushes into the type system rather than leaving to review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProseCoverage {
    /// Prose holds this rule: the whole of it (`Active`), or the part no named
    /// control covers (`Partial`). A recurrence therefore says the prose did not
    /// hold, which is exactly the unheld-recurrence finding.
    Holds,
    /// Prose holds none of it -- a stronger control claims the whole class
    /// (`Graduated`), or the guidance has been withdrawn (`Attic`).
    None,
}

/// Whether a rule's lifecycle status permits writing it into an active
/// instruction layer (a skill, `AGENTS.md`, a project `CLAUDE.md`, …).
///
/// This is the type home of the emit-status policy (decision 2026-08-13): the
/// instruction layer states *currently-in-force* guidance, so a rule is emitted
/// iff its guidance still stands — regardless of which layer ultimately enforces
/// it. Producing this from an exhaustive match on [`Status`] means a future
/// status variant cannot compile until its emit policy is decided here, rather
/// than defaulting to "emitted" and silently leaking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emittability {
    /// Write the rule into the instruction layer. Both `Active` (enforced by the
    /// prose itself) and `Graduated` (enforced by a stronger layer *as well*)
    /// qualify: the instruction layer tunes generation *before* it happens — a
    /// control distinct from the layer that catches *after* — so a graduated
    /// rule keeps its tuning job. The emitter annotates it with its destination.
    Emit,
    /// Keep the rule out of the instruction layer: its guidance has been
    /// withdrawn (`Attic`), and writing it would instruct a retired rule.
    Suppress,
}

/// A rule's lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Live and enforced.
    Active,
    /// Partly held: the controls named in `by` cover only some of the class, and
    /// `uncovered` states what prose is still carrying alone, as of `date`.
    ///
    /// This variant exists because the two honest options without it were both
    /// wrong. Recording such a rule as `Graduated` emits *"Also enforced by X"*
    /// into five instruction layers for a class X only half covers, and arms
    /// `RecurrenceAfterGraduation` -- an `Error` that fails CI -- against a
    /// recurrence in the part X never claimed. Recording it as `Active` throws
    /// away the fact that real controls exist and were built.
    ///
    /// So `Partial` reports like `Active` and annotates like `Graduated`: prose
    /// still holds part of it, so a recurrence is still an unheld recurrence; a
    /// named control still holds part of it, so the reader is told which. What it
    /// deliberately does **not** do is claim the whole class -- see
    /// [`Status::whole_class_claim`].
    Partial {
        /// The controls that hold part of the class.
        by: Controls,
        /// The part they do not hold, which prose still carries.
        uncovered: Uncovered,
        /// When this became the state of affairs.
        date: Date,
    },
    /// Superseded by a stronger control at `to` (e.g. a hook), on `date`.
    ///
    /// The date is not decoration. Without it nothing can ask the question that
    /// matters about a graduated rule -- *did it recur since it graduated?* -- and
    /// that recurrence is the sharpest finding the library can produce: the named
    /// stronger control was claimed to hold this and demonstrably did not, which
    /// is a lying artefact rather than a rule merely wanting promotion.
    Graduated {
        /// The controls the guarantee moved to.
        to: Controls,
        /// When it moved there.
        date: Date,
    },
    /// Retired to the attic, with why and when — never deleted.
    Attic {
        /// Why it was retired.
        reason: Reason,
        /// When it was retired.
        date: Date,
    },
}

impl Status {
    /// The active state.
    #[must_use]
    pub fn active() -> Self {
        Status::Active
    }

    /// Graduated to `to` on `date`; the destination must be non-empty and the
    /// date is an already-validated [`Date`].
    pub fn graduated(to: impl AsRef<str>, date: Date) -> Result<Self, ControlError> {
        Ok(Status::Graduated {
            to: Controls::parse(to)?,
            date,
        })
    }

    /// Partly held as of `date`: `by` names the controls that cover part of the
    /// class and `uncovered` states the part they do not. Both must be non-empty,
    /// and both are required -- a partial graduation that will not say what is
    /// missing is the exact artefact this variant exists to prevent.
    pub fn partial(
        by: impl AsRef<str>,
        uncovered: impl Into<String>,
        date: Date,
    ) -> Result<Self, StatusError> {
        Ok(Status::Partial {
            by: Controls::parse(by)?,
            uncovered: Uncovered::parse(uncovered)?,
            date,
        })
    }

    /// Atticked with `reason` on `date`; the reason must be non-empty and the
    /// date is an already-validated [`Date`].
    pub fn attic(reason: impl Into<String>, date: Date) -> Result<Self, EmptyText> {
        Ok(Status::Attic {
            reason: Reason::parse(reason)?,
            date,
        })
    }

    /// Whether this status permits the rule to be written into an active
    /// instruction layer. The single authority for the emit-status policy; every
    /// emitter filters through it (via `emit::emittable`) rather than deciding
    /// per target. The match is exhaustive by design — see [`Emittability`].
    #[must_use]
    pub fn emittability(&self) -> Emittability {
        match self {
            Status::Active | Status::Partial { .. } | Status::Graduated { .. } => {
                Emittability::Emit
            }
            Status::Attic { .. } => Emittability::Suppress,
        }
    }

    /// Whether prose still holds any part of this rule, and therefore whether a
    /// recurrence is an *unheld* recurrence.
    ///
    /// The single authority for that policy, exhaustive by design -- see
    /// [`ProseCoverage`]. `Partial` answers `Holds` on purpose: naming a control
    /// for half a class does not relieve the prose of the other half, and a
    /// recurrence there is precisely the signal the finding exists to surface.
    /// Whether this rule's guidance has been withdrawn.
    ///
    /// The same partition as [`Emittability::Suppress`] today, and deliberately
    /// a separate question: that one asks "write this into an instruction
    /// layer?", this one asks "is this guidance still in force?" -- which is
    /// what `pull` needs before caching a rule and what a drop plan needs
    /// before calling a cache unwanted. Both are exhaustive, so a new status
    /// must answer both rather than inheriting one by accident.
    #[must_use]
    pub fn is_withdrawn(&self) -> bool {
        match self {
            Status::Attic { .. } => true,
            Status::Active | Status::Partial { .. } | Status::Graduated { .. } => false,
        }
    }

    #[must_use]
    pub fn prose_coverage(&self) -> ProseCoverage {
        match self {
            Status::Active | Status::Partial { .. } => ProseCoverage::Holds,
            Status::Graduated { .. } | Status::Attic { .. } => ProseCoverage::None,
        }
    }

    /// The claim, if any, that a named control holds the **whole** of this rule's
    /// class from a given date -- what `recurrence-after-graduation` tests.
    ///
    /// `None` for `Partial`, and that asymmetry with [`Self::prose_coverage`] is
    /// the point of the variant: a recurrence in a partly-held rule says the
    /// prose failed, never that the named control lied, because the control never
    /// claimed that ground. Reporting it as a lying artefact would fail CI for
    /// doing the honest thing.
    #[must_use]
    pub fn whole_class_claim(&self) -> Option<(&Controls, &Date)> {
        match self {
            Status::Graduated { to, date } => Some((to, date)),
            Status::Active | Status::Partial { .. } | Status::Attic { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_is_trivial() {
        assert_eq!(Status::active(), Status::Active);
    }

    #[test]
    fn graduated_requires_destination() {
        assert!(
            Status::graduated(
                "hook:no-narrow-parse",
                Date::parse("2026-07-21").expect("valid date")
            )
            .is_ok()
        );
        assert!(Status::graduated("", Date::parse("2026-07-21").expect("valid date")).is_err());
    }

    #[test]
    fn attic_requires_reason_and_carries_date() {
        let date = Date::parse("2026-09-01").expect("valid date");
        let s = Status::attic("cold surface, challenge-tested", date).expect("non-empty reason");
        assert!(matches!(s, Status::Attic { .. }));
        assert!(Status::attic("  ", date).is_err());
    }

    #[test]
    fn active_and_graduated_emit_but_attic_is_suppressed() {
        let date = Date::parse("2026-09-01").expect("valid date");
        assert_eq!(Status::active().emittability(), Emittability::Emit);
        assert_eq!(
            Status::graduated(
                "hook:no-unwrap-in-src",
                Date::parse("2026-07-21").expect("valid date")
            )
            .expect("non-empty destination")
            .emittability(),
            Emittability::Emit,
        );
        assert_eq!(
            Status::attic("retired", date)
                .expect("non-empty reason")
                .emittability(),
            Emittability::Suppress,
        );
    }

    // -- partial graduation ------------------------------------------------

    #[test]
    fn partial_requires_a_control() {
        assert!(
            Status::partial("", "the rest", date()).is_err(),
            "a partial graduation naming no control is not partial, it is active"
        );
    }

    #[test]
    fn partial_requires_the_uncovered_part() {
        // The whole argument for the variant: a partial graduation that will not
        // say what is missing reads exactly like a full one, and the reader stops
        // looking. Unconstructible rather than discouraged.
        assert!(
            Status::partial("test:a_test", "", date()).is_err(),
            "a partial graduation must state what it does not cover"
        );
    }

    #[test]
    fn partial_is_emitted() {
        assert_eq!(
            partial_status().emittability(),
            Emittability::Emit,
            "a partly-held rule is live guidance and belongs in the instruction layer"
        );
    }

    #[test]
    fn partial_prose_still_holds() {
        assert_eq!(partial_status().prose_coverage(), ProseCoverage::Holds);
    }

    #[test]
    fn partial_makes_no_whole_class_claim() {
        // The asymmetry with the test above IS the variant. Prose holds part, so
        // a recurrence is reported; no control claims the whole, so that report
        // is a warning about the prose and never an Error about a lying control.
        assert!(partial_status().whole_class_claim().is_none());
    }

    #[test]
    fn graduated_claims_the_whole_class() {
        let status = Status::graduated("hook:a-hook", date()).expect("non-empty destination");
        assert!(status.whole_class_claim().is_some());
    }

    #[test]
    fn active_and_atticked_claim_nothing() {
        assert!(Status::active().whole_class_claim().is_none());
        assert!(
            Status::attic("cold surface", date())
                .expect("non-empty reason")
                .whole_class_claim()
                .is_none()
        );
    }

    fn date() -> Date {
        Date::parse("2026-09-16").expect("valid date")
    }

    fn partial_status() -> Status {
        Status::partial("test:a_test", "everything else", date())
            .expect("non-empty control and uncovered part")
    }

    #[test]
    fn only_the_attic_is_withdrawn() {
        // `pull` refuses to cache withdrawn guidance and a drop plan calls a
        // withdrawn cache unwanted; both used to ask with a `matches!`, which a
        // new status would have passed through as "still in force".
        assert!(
            Status::attic("cold surface", date())
                .expect("non-empty reason")
                .is_withdrawn()
        );
        assert!(!Status::active().is_withdrawn());
        assert!(!partial_status().is_withdrawn());
        assert!(
            !Status::graduated("hook:a-hook", date())
                .expect("non-empty destination")
                .is_withdrawn()
        );
    }
}
