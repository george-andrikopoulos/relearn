//! Non-empty, trimmed text newtypes. `ErrorClass` and `Incident` are distinct
//! types though they share a representation (Pattern 1: newtype liberally) — an
//! `Incident` is not an `ErrorClass`, and the compiler should refuse to swap
//! them. All non-empty text in the crate passes through [`nonempty`], the one
//! perimeter.

/// A trimmed, non-empty string failed to validate: the named field was empty
/// (or whitespace only).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{field} must not be empty")]
pub struct EmptyText {
    field: &'static str,
}

/// Trim `s` and reject it if nothing remains. Shared by every non-empty text
/// newtype in the crate (`pub(crate)`, not public API).
pub(crate) fn nonempty(field: &'static str, s: impl Into<String>) -> Result<String, EmptyText> {
    let s = s.into();
    let trimmed = s.trim();
    if trimmed.is_empty() {
        Err(EmptyText { field })
    } else {
        Ok(trimmed.to_owned())
    }
}

/// The class of error a rule retires — the "what went wrong" in one phrase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorClass(String);

impl ErrorClass {
    /// Parse a non-empty error-class description.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("error_class", s)?))
    }

    /// The description text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The triggering incident: what happened, and when, that produced the rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incident(String);

impl Incident {
    /// Parse a non-empty incident description.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("incident", s)?))
    }

    /// The description text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_and_trims() {
        let e = ErrorClass::parse("  narrowing at parse  ").expect("non-empty parses");
        assert_eq!(e.as_str(), "narrowing at parse");
    }

    #[test]
    fn rejects_blank_error_class() {
        assert_eq!(
            ErrorClass::parse("   "),
            Err(EmptyText {
                field: "error_class"
            })
        );
    }

    #[test]
    fn rejects_empty_incident() {
        assert!(Incident::parse("").is_err());
    }
}
