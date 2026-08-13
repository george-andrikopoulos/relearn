//! `RuleTag` — a rule's stable identity, constructible only by parsing.

use std::fmt;

/// A rule's stable identity, e.g. `R:parse-wide-then-range-check`.
///
/// Shape: the literal prefix `R:`, then a body whose first character is
/// `[a-z0-9]` and whose remaining characters are `[a-z0-9-]`. Constructible
/// only through [`RuleTag::parse`] — possessing a `RuleTag` is itself the proof
/// that the shape held, so nothing downstream re-validates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuleTag(String);

/// Why a string is not a valid [`RuleTag`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RuleTagError {
    /// The input was empty.
    #[error("rule tag is empty")]
    Empty,
    /// The input did not begin with the required `R:` prefix.
    #[error("rule tag must begin with `R:` (got {0:?})")]
    MissingPrefix(String),
    /// The `R:` prefix was present but nothing followed it.
    #[error("rule tag has an empty body after `R:`")]
    EmptyBody,
    /// A body character was outside `[a-z0-9-]`, or the first body character
    /// was not `[a-z0-9]`. Carries the offending body.
    #[error("rule tag body must be [a-z0-9-] with first char [a-z0-9] (got {0:?})")]
    BadShape(String),
}

impl RuleTag {
    /// Parse a string into a [`RuleTag`], enforcing the shape at this one
    /// perimeter.
    pub fn parse(s: impl Into<String>) -> Result<Self, RuleTagError> {
        let s = s.into();
        if s.is_empty() {
            return Err(RuleTagError::Empty);
        }
        // `body` borrows `s`; on the no-prefix path nothing borrows `s`, so it
        // is moved into the error rather than cloned.
        let Some(body) = s.strip_prefix("R:") else {
            return Err(RuleTagError::MissingPrefix(s));
        };
        let mut chars = body.chars();
        match chars.next() {
            None => return Err(RuleTagError::EmptyBody),
            Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
            Some(_) => return Err(RuleTagError::BadShape(body.to_owned())),
        }
        if chars.any(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')) {
            return Err(RuleTagError::BadShape(body.to_owned()));
        }
        Ok(RuleTag(s))
    }

    /// The full tag, including the `R:` prefix.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The tag body: everything after the `R:` prefix, e.g.
    /// `parse-wide-then-range-check`. Guaranteed non-empty and `[a-z0-9][a-z0-9-]*`
    /// by construction, so it is a safe filesystem stem (unlike the full tag,
    /// whose `:` is not a valid filename character on every platform).
    #[must_use]
    pub fn body(&self) -> &str {
        // The `R:` prefix is an invariant of every `RuleTag`, so this slice is
        // always in bounds — `parse` is the only constructor and it enforces it.
        &self.0["R:".len()..]
    }
}

impl fmt::Display for RuleTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn accepts_a_known_tag() {
        let t = RuleTag::parse("R:parse-wide-then-range-check").expect("known tag parses");
        assert_eq!(t.as_str(), "R:parse-wide-then-range-check");
    }

    #[test]
    fn body_drops_the_prefix_and_is_a_safe_stem() {
        let t = RuleTag::parse("R:parse-wide-then-range-check").expect("known tag parses");
        assert_eq!(t.body(), "parse-wide-then-range-check");
        assert!(!t.body().contains(':'), "body is a valid filename stem");
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(RuleTag::parse(""), Err(RuleTagError::Empty));
    }

    #[test]
    fn rejects_missing_prefix() {
        assert!(matches!(
            RuleTag::parse("parse-wide"),
            Err(RuleTagError::MissingPrefix(_))
        ));
    }

    #[test]
    fn rejects_empty_body() {
        assert_eq!(RuleTag::parse("R:"), Err(RuleTagError::EmptyBody));
    }

    #[test]
    fn rejects_leading_hyphen() {
        assert!(matches!(
            RuleTag::parse("R:-x"),
            Err(RuleTagError::BadShape(_))
        ));
    }

    #[test]
    fn rejects_uppercase_body() {
        assert!(matches!(
            RuleTag::parse("R:Parse"),
            Err(RuleTagError::BadShape(_))
        ));
    }

    proptest! {
        /// Any well-shaped tag parses and round-trips through `as_str`.
        #[test]
        fn wellformed_roundtrips(body in "[a-z0-9][a-z0-9-]{0,40}") {
            let s = format!("R:{body}");
            let t = RuleTag::parse(&s).expect("wellformed tag parses");
            prop_assert_eq!(t.as_str(), s);
        }

        /// A body whose second character is out of class is always rejected.
        #[test]
        fn rejects_out_of_class(head in "[a-z0-9]", bad in "[A-Z_.]") {
            let s = format!("R:{head}{bad}");
            prop_assert!(RuleTag::parse(&s).is_err());
        }
    }
}
