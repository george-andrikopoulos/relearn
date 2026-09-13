//! `ScopeTag` — one audience a rule serves.
//!
//! **Not a second home.** [`Home`](super::Home) answers *who owns and maintains
//! this rule*, and answers it exactly once; that is the P2 invariant the whole
//! tool exists to enforce. A scope answers a different question with different
//! cardinality — *who should load it* — and the two were conflated only because
//! `Home` was the sole field available.
//!
//! The case that forced the split: a rule about NUMA residence, cache-line
//! behaviour or why a busy-spin core must be exclusive is a **low-latency**
//! rule, and Rust and Java engineers both need it. Homed in `domain-rust` it is
//! noise for one language and invisible to the other; homed in
//! `domain-low-latency` it is invisible to both, because nobody's Rust build
//! asks for that home; written twice it is two homes, drifting, in the tool
//! built to prevent two homes.
//!
//! Nesting (`rust/low-latency`, `java/low-latency`) is the instinct and it is
//! wrong: a tree expresses one axis and forks the other. What varies by
//! language is the implementation; what varies by discipline is the principle.
//! The ARCHITECTURE decisions log carries the rejection in full.

/// One audience a rule serves — a language, a platform, a discipline: `rust`,
/// `java`, `low-latency`.
///
/// Shape: lowercase kebab, first character `[a-z0-9]`, the rest `[a-z0-9-]`,
/// trimmed, non-empty, and no longer than [`ScopeTag::MAX_CHARS`]. The same
/// discipline as [`RuleTag`](super::RuleTag) minus the `R:` prefix, and for the
/// same reason: constructible only through [`ScopeTag::parse`], so possessing
/// one *is* the proof and nothing downstream re-checks.
///
/// A scope is an **audience**, not a topic. `rust`, `java`, `embedded` — things
/// an install can declare it *is*. Not `performance` or `security`, which turn
/// the field into tags and make it a second home for what `error_class` already
/// carries. That distinction cannot be typed, so it is stated here and in
/// `CLAUDE.md`; the shape rule below is the half a compiler can hold.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeTag(String);

/// Why a string is not a valid [`ScopeTag`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScopeTagError {
    /// The input was empty, or whitespace only.
    #[error("scope must not be empty")]
    Empty,
    /// The input was longer than [`ScopeTag::MAX_CHARS`].
    #[error("scope is {got} characters, over the {max}-character limit")]
    TooLong {
        /// The length that was given.
        got: usize,
        /// The limit.
        max: usize,
    },
    /// A character was outside `[a-z0-9-]`, or the first character was not
    /// `[a-z0-9]`. Carries the offending value.
    #[error("scope must be [a-z0-9-] with first char [a-z0-9] (got {0:?})")]
    BadShape(String),
}

impl ScopeTag {
    /// The longest a scope may be.
    ///
    /// Bounded because a scope is an identifier a person types on a command
    /// line and reads in a rule's front matter, not prose. The specific number
    /// is a judgement: long enough for `low-latency-networking`, short enough
    /// that a sentence pasted into the field is rejected where it was written
    /// rather than carried into every instruction layer.
    pub const MAX_CHARS: usize = 40;

    /// Parse a string into a [`ScopeTag`], enforcing the shape at this one
    /// perimeter.
    ///
    /// Trims first, so a stray space in a TOML array is a formatting accident
    /// rather than a parse failure — the same courtesy every other text field
    /// in the format extends.
    pub fn parse(s: impl Into<String>) -> Result<Self, ScopeTagError> {
        let s = s.into();
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ScopeTagError::Empty);
        }
        let length = trimmed.chars().count();
        if length > Self::MAX_CHARS {
            return Err(ScopeTagError::TooLong {
                got: length,
                max: Self::MAX_CHARS,
            });
        }
        let mut chars = trimmed.chars();
        match chars.next() {
            // `trimmed` is non-empty, so this arm is unreachable; it is written
            // for exhaustiveness rather than for a case that can occur.
            None => return Err(ScopeTagError::Empty),
            Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
            Some(_) => return Err(ScopeTagError::BadShape(trimmed.to_owned())),
        }
        if chars.any(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')) {
            return Err(ScopeTagError::BadShape(trimmed.to_owned()));
        }
        Ok(ScopeTag(trimmed.to_owned()))
    }

    /// The scope text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ScopeTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wellformed_scopes_round_trip() {
        for s in ["rust", "java", "low-latency", "c99", "x"] {
            assert_eq!(
                ScopeTag::parse(s).expect("valid scope").as_str(),
                s,
                "{s} must parse unchanged"
            );
        }
    }

    #[test]
    fn trims_before_checking() {
        assert_eq!(
            ScopeTag::parse("  low-latency \t")
                .expect("valid scope")
                .as_str(),
            "low-latency"
        );
    }

    #[test]
    fn rejects_empty_and_whitespace_only() {
        assert_eq!(ScopeTag::parse(""), Err(ScopeTagError::Empty));
        assert_eq!(ScopeTag::parse("   "), Err(ScopeTagError::Empty));
    }

    #[test]
    fn rejects_uppercase_and_spaces_and_underscores() {
        for s in ["Rust", "low latency", "low_latency", "rust!"] {
            assert!(
                matches!(ScopeTag::parse(s), Err(ScopeTagError::BadShape(_))),
                "{s} must be rejected"
            );
        }
    }

    #[test]
    fn rejects_a_leading_hyphen() {
        assert!(matches!(
            ScopeTag::parse("-rust"),
            Err(ScopeTagError::BadShape(_))
        ));
    }

    // The bound is checked on the trimmed value, so padding cannot push a legal
    // scope over the limit.
    #[test]
    fn rejects_an_over_long_scope_and_names_the_length() {
        let long = "a".repeat(ScopeTag::MAX_CHARS + 1);
        assert_eq!(
            ScopeTag::parse(&long),
            Err(ScopeTagError::TooLong {
                got: ScopeTag::MAX_CHARS + 1,
                max: ScopeTag::MAX_CHARS,
            })
        );
        let at_limit = "a".repeat(ScopeTag::MAX_CHARS);
        assert!(ScopeTag::parse(&at_limit).is_ok());
        assert!(ScopeTag::parse(format!("  {at_limit}  ")).is_ok());
    }
}
