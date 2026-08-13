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

/// A rule's lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Live and enforced.
    Active,
    /// Superseded by a stronger control at `to` (e.g. a hook).
    Graduated {
        /// Where the guarantee moved to.
        to: Destination,
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

    /// Graduated to `to`; the destination must be non-empty.
    pub fn graduated(to: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Status::Graduated {
            to: Destination::parse(to)?,
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
        assert!(Status::graduated("hook:no-narrow-parse").is_ok());
        assert!(Status::graduated("").is_err());
    }

    #[test]
    fn attic_requires_reason_and_carries_date() {
        let date = Date::parse("2026-09-01").expect("valid date");
        let s = Status::attic("cold surface, challenge-tested", date).expect("non-empty reason");
        assert!(matches!(s, Status::Attic { .. }));
        assert!(Status::attic("  ", date).is_err());
    }
}
