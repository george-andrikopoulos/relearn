//! Non-empty, trimmed text newtypes. `ErrorClass`, `Incident`, `Title` and
//! `Body` are distinct types though they share a representation (Pattern 1:
//! newtype liberally) — a `Title` is not a `Body`, and the compiler should
//! refuse to swap them. All non-empty text in the crate passes through
//! [`nonempty`], the one perimeter.

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

    /// The key two rules are considered to describe the *same* class by.
    ///
    /// One definition, because there are now two readers — the linter's
    /// overlapping-scope check and the poke's reactive trigger — and two
    /// spellings of "the same class" would eventually disagree about a rule
    /// that one of them flags and the other does not.
    ///
    /// **It is a literal, case-folded comparison, and that is weak on
    /// purpose.** Two rules can describe one class in different words and
    /// never meet here; nothing short of judgement closes that, and a fuzzy
    /// key dressed up as certainty would be worse than an honest miss. The
    /// failure is therefore under-matching — a poke that does not fire, never
    /// one that fires about an unrelated rule.
    #[must_use]
    pub fn match_key(&self) -> String {
        self.0.to_lowercase()
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

/// The account of an incident that may be **published**: what class of thing
/// went wrong and what it cost, written for a stranger.
///
/// **A separate field, never a transformation of [`Incident`].** The raw
/// incident is a verbatim quotation from a private working session; this is a
/// rewritten account with no quotation, no names, no paths and no repository
/// identifiers. Nothing derives one from the other — a scrubber would leak what
/// it did not recognise and destroy context it did not understand, and worse, it
/// would stop people reading the output because something appeared to be
/// handling it.
///
/// A distinct type rather than a second `Incident` so the two cannot be swapped
/// at a call site: the compiler refuses to put a raw incident where a published
/// one belongs, which is the only place that mistake could ever be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedIncident(String);

impl PublishedIncident {
    /// Parse a non-empty published incident.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("published_incident", s)?))
    }

    /// The account text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A rule's short human title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title(String);

impl Title {
    /// Parse a non-empty title.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("title", s)?))
    }

    /// The title text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The imperative rule body — the markdown after the front-matter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Body(String);

impl Body {
    /// Parse a non-empty body.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("body", s)?))
    }

    /// The body text.
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

    #[test]
    fn title_and_body_reject_blank() {
        assert!(Title::parse("  ").is_err());
        assert!(Body::parse("").is_err());
        assert_eq!(Body::parse(" do it ").expect("non-empty").as_str(), "do it");
    }
}
