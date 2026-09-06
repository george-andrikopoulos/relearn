//! `Origin` — where a rule came from: a real failure, or existing practice
//! written down.
//!
//! This is the **counter-metric** to recurrence, and it exists because a metric
//! without one is a number waiting to be gamed. Recurrence counts only what
//! someone was willing to record about their own rule failing, so it falls
//! through under-reporting exactly as easily as through prevention. The counter
//! is the corpus's *inert* fraction — rules that have never recurred **and**
//! were never mined from a real failure — because the null result says those are
//! the ones that change nothing: you cannot author your way to a delta. A
//! library that looks healthy because it is full of them is the failure the
//! recurrence count would otherwise hide.

/// Where a rule came from.
///
/// **Two variants, no payload.** `Codified` deliberately does not carry its
/// source: a practice written down from standing doctrine is meaningful without
/// naming a document, so "codified without a source" is not an illegal state and
/// there is nothing for a payload to make unrepresentable. Where the source
/// matters it is already in the rule's `incident` prose. A `PortedFrom(source)`
/// variant was considered and deferred — it is a third modelled fact with its
/// own argument, and adding it later is additive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    /// Written because something actually went wrong: there is a specific,
    /// dated failure behind it. These are the rules the error loop produced.
    Mined,
    /// Written down from existing practice or doctrine rather than from a
    /// single incident — the `incident` field records when and from where it
    /// was codified, not a failure it retired.
    Codified,
}

/// The `origin` field held a value that is not a known origin.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`origin` has unknown value `{0}` (expected mined | codified)")]
pub struct UnknownOrigin(String);

impl Origin {
    /// Parse the neutral format's `origin` value.
    pub fn parse(s: &str) -> Result<Self, UnknownOrigin> {
        match s.trim() {
            "mined" => Ok(Origin::Mined),
            "codified" => Ok(Origin::Codified),
            other => Err(UnknownOrigin(other.to_owned())),
        }
    }

    /// The neutral format's spelling, and the inverse of [`Origin::parse`].
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Origin::Mined => "mined",
            Origin::Codified => "codified",
        }
    }

    /// Whether this rule was produced by the error loop rather than authored.
    ///
    /// The distinction the counter-metric turns on: a `Codified` rule that has
    /// never recurred is evidence of nothing, whereas a `Mined` rule that has
    /// never recurred is a rule that may well be working.
    #[must_use]
    pub fn is_mined(self) -> bool {
        matches!(self, Origin::Mined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_spellings() {
        assert_eq!(Origin::parse("mined"), Ok(Origin::Mined));
        assert_eq!(Origin::parse("codified"), Ok(Origin::Codified));
    }

    #[test]
    fn trims_before_matching() {
        assert_eq!(Origin::parse("  codified  "), Ok(Origin::Codified));
    }

    // The error names the offending value and the alternatives, so a typo in a
    // rule file reports what to write rather than only that it was wrong.
    #[test]
    fn an_unknown_value_is_named() {
        let err = Origin::parse("invented").expect_err("not an origin");
        let text = err.to_string();
        assert!(text.contains("invented"), "{text}");
        assert!(text.contains("mined | codified"), "{text}");
    }

    #[test]
    fn as_str_round_trips_through_parse() {
        for origin in [Origin::Mined, Origin::Codified] {
            assert_eq!(Origin::parse(origin.as_str()), Ok(origin));
        }
    }

    #[test]
    fn only_mined_is_mined() {
        assert!(Origin::Mined.is_mined());
        assert!(!Origin::Codified.is_mined());
    }
}
