//! `Status` — a rule's lifecycle state, each variant carrying exactly the data
//! that state requires. "Graduated" cannot exist without a destination and
//! "atticked" cannot exist without a reason and a date, because those live in
//! the variant and there is no other way to build it.

use super::date::Date;
use super::text::{EmptyText, nonempty};

/// Where a rule graduated to, e.g. `hook:no-narrow-parse` — non-empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destination(String);

impl Destination {
    /// Parse a non-empty graduation destination.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("graduation destination", s)?))
    }

    /// The destination text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
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
    /// Superseded by a stronger control at `to` (e.g. a hook), on `date`.
    ///
    /// The date is not decoration. Without it nothing can ask the question that
    /// matters about a graduated rule -- *did it recur since it graduated?* -- and
    /// that recurrence is the sharpest finding the library can produce: the named
    /// stronger control was claimed to hold this and demonstrably did not, which
    /// is a lying artefact rather than a rule merely wanting promotion.
    Graduated {
        /// Where the guarantee moved to.
        to: Destination,
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
    pub fn graduated(to: impl Into<String>, date: Date) -> Result<Self, EmptyText> {
        Ok(Status::Graduated {
            to: Destination::parse(to)?,
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
            Status::Active | Status::Graduated { .. } => Emittability::Emit,
            Status::Attic { .. } => Emittability::Suppress,
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
}
