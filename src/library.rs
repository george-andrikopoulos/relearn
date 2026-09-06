//! `library` — collection semantics over rules: tag uniqueness across the
//! library, and the `Library<Unvalidated>` / `Library<Validated>` typestate
//! that gates emission. A validated library is the witness that every rule
//! parsed and every cross-rule invariant held; `emit` (elsewhere) accepts only
//! `Library<Validated>`, so emitting unvalidated rules does not compile.
//!
//! **Must NOT:** perform I/O. The library is a pure in-memory value; reading
//! rules from disk and writing emitted files belong to `fsio`.

use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::rule::{Rule, RuleTag};

/// Typestate marker: rules collected but cross-rule invariants not yet checked.
#[derive(Debug, Clone, Copy)]
pub struct Unvalidated;

/// Typestate marker: the witness that every cross-rule invariant holds. Only
/// [`Library::validate`] can produce a `Library<Validated>`.
#[derive(Debug, Clone, Copy)]
pub struct Validated;

/// A collection of rules in one of two states.
#[derive(Debug, Clone)]
pub struct Library<S> {
    rules: Vec<Rule>,
    _state: PhantomData<S>,
}

/// Why a library failed validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    /// Two rules share a tag. A duplicate tag is never allowed and never
    /// silently dropped — it stops validation.
    #[error("duplicate rule tag {0}")]
    DuplicateTag(RuleTag),
}

impl Library<Unvalidated> {
    /// An empty, unvalidated library.
    #[must_use]
    pub fn new() -> Self {
        Library {
            rules: Vec::new(),
            _state: PhantomData,
        }
    }

    /// An unvalidated library from already-parsed rules.
    #[must_use]
    pub fn from_rules(rules: Vec<Rule>) -> Self {
        Library {
            rules,
            _state: PhantomData,
        }
    }

    /// Add a parsed rule.
    pub fn push(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Check cross-rule invariants (tag uniqueness) and mint the validated
    /// witness. A duplicate tag stops here rather than being skipped — a
    /// dropped rule is a lost correction.
    pub fn validate(self) -> Result<Library<Validated>, ValidationError> {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for rule in &self.rules {
            if !seen.insert(rule.tag().as_str()) {
                return Err(ValidationError::DuplicateTag(rule.tag().clone())); // allow:clone: own the offending tag for the diagnostic on the failure path
            }
        }
        Ok(Library {
            rules: self.rules,
            _state: PhantomData,
        })
    }
}

impl Default for Library<Unvalidated> {
    fn default() -> Self {
        Self::new()
    }
}

impl Library<Validated> {
    /// The validated rules. Tags are unique across this slice.
    #[must_use]
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// The number of rules.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Whether the library holds no rules.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// A validated library holding only the rules `keep` accepts.
    ///
    /// **No re-validation, and that is a claim about the invariant, not a
    /// shortcut.** The only cross-rule invariant is tag uniqueness; a subset of
    /// a set with unique tags still has unique tags, so the witness carries over
    /// and the filtered value is a `Library<Validated>` by construction. Should
    /// a future invariant *not* survive subsetting, this method must go back
    /// through `validate` — the compiler will not catch that, so it is stated
    /// here beside the code it constrains.
    ///
    /// Used to emit one home layer on its own. Emitting the rules layer into a
    /// **user** scope must carry the domain layer and nothing else: project
    /// homes are always-resident, so a project layer written to a user scope
    /// would load unscoped in every session in every repository.
    #[must_use]
    pub fn filter(&self, keep: impl Fn(&Rule) -> bool) -> Library<Validated> {
        Library {
            // allow:clone: the filtered library owns its subset, outliving this
            // borrow; the alternative is threading a predicate through all five
            // emitter signatures for a path taken once per invocation.
            rules: self.rules.iter().filter(|r| keep(r)).cloned().collect(),
            _state: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Body, Date, ErrorClass, Home, Incident, Origin, RuleTag, Status, Title};

    fn rule_with_tag(tag: &str) -> Rule {
        Rule::new(
            RuleTag::parse(tag).expect("valid tag"),
            Title::parse("A title").expect("non-empty"),
            ErrorClass::parse("an error class").expect("non-empty"),
            Home::global(),
            Date::parse("2026-08-13").expect("valid date"),
            Origin::Mined,
            Status::active(),
            Incident::parse("an incident").expect("non-empty"),
            Body::parse("Do the thing.").expect("non-empty"),
            Vec::new(),
        )
    }

    #[test]
    fn distinct_tags_validate() {
        let lib = Library::from_rules(vec![
            rule_with_tag("R:alpha"),
            rule_with_tag("R:beta"),
            rule_with_tag("R:gamma"),
        ]);
        let validated = lib.validate().expect("distinct tags validate");
        assert_eq!(validated.len(), 3);
        assert!(!validated.is_empty());
    }

    #[test]
    fn empty_library_validates() {
        let validated = Library::new().validate().expect("empty validates");
        assert!(validated.is_empty());
    }

    #[test]
    fn duplicate_tag_is_rejected_with_the_offending_tag() {
        let mut lib = Library::new();
        lib.push(rule_with_tag("R:dup"));
        lib.push(rule_with_tag("R:other"));
        lib.push(rule_with_tag("R:dup"));
        assert_eq!(
            lib.validate().expect_err("duplicate must be rejected"),
            ValidationError::DuplicateTag(RuleTag::parse("R:dup").expect("valid tag"))
        );
    }
}
