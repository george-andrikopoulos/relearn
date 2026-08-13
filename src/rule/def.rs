//! `Rule` — a fully-parsed rule: an aggregate of witness newtypes. Because
//! every field is a distinct validated type, assembling one cannot fail (each
//! part already proved its own invariant) and the constructor cannot be called
//! with its arguments in the wrong order — a swap is a compile error.

use super::{Body, Date, ErrorClass, Home, Incident, RuleTag, Status, Title};

/// One rule, every field already validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    tag: RuleTag,
    title: Title,
    error_class: ErrorClass,
    home: Home,
    created: Date,
    status: Status,
    incident: Incident,
    body: Body,
}

impl Rule {
    /// Assemble a rule from its validated parts.
    ///
    /// Eight arguments, deliberately: these are the rule's essential fields and
    /// construction requires all of them (there is no incomplete-`Rule` state
    /// to guard against). Every parameter is a distinct newtype, so an
    /// argument-order mistake is a compile error rather than a runtime bug — the
    /// clippy `too_many_arguments` lint is suppressed for that reason.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        tag: RuleTag,
        title: Title,
        error_class: ErrorClass,
        home: Home,
        created: Date,
        status: Status,
        incident: Incident,
        body: Body,
    ) -> Self {
        Rule {
            tag,
            title,
            error_class,
            home,
            created,
            status,
            incident,
            body,
        }
    }

    /// The rule's identity.
    #[must_use]
    pub fn tag(&self) -> &RuleTag {
        &self.tag
    }

    /// The short human title.
    #[must_use]
    pub fn title(&self) -> &Title {
        &self.title
    }

    /// The error class the rule retires.
    #[must_use]
    pub fn error_class(&self) -> &ErrorClass {
        &self.error_class
    }

    /// Where the rule lives.
    #[must_use]
    pub fn home(&self) -> &Home {
        &self.home
    }

    /// The date the rule was created.
    #[must_use]
    pub fn created(&self) -> Date {
        self.created
    }

    /// The rule's lifecycle status.
    #[must_use]
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// The triggering incident.
    #[must_use]
    pub fn incident(&self) -> &Incident {
        &self.incident
    }

    /// The imperative body.
    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }
}
