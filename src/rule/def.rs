//! `Rule` — a fully-parsed rule: an aggregate of witness newtypes. Because
//! every field is a distinct validated type, assembling one cannot fail (each
//! part already proved its own invariant) and the constructor cannot be called
//! with its arguments in the wrong order — a swap is a compile error.

use super::{Body, Date, ErrorClass, Home, Incident, Origin, RuleTag, ScopeTag, Status, Title};

/// One later occurrence of the error class a rule already covers — evidence
/// that the rule was written down and the error happened anyway.
///
/// Deliberately **not** an [`Incident`] on its own, and deliberately not a
/// second element of one `incidents` list. The triggering incident's date is
/// the rule's [`Rule::created`]; a recurrence has no such field, so it must
/// carry its own. Collapsing the two into a single list would make
/// `incidents[0]` mean something different from every other element — a shape
/// that lies about itself. A recurrence is therefore the pair: when it
/// happened, and what happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recurrence {
    date: Date,
    incident: Incident,
}

impl Recurrence {
    /// Record a later occurrence: the date it happened, and what happened.
    #[must_use]
    pub fn new(date: Date, incident: Incident) -> Self {
        Recurrence { date, incident }
    }

    /// When the error class recurred.
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    /// What happened when it recurred.
    #[must_use]
    pub fn incident(&self) -> &Incident {
        &self.incident
    }
}

/// One rule, every field already validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    tag: RuleTag,
    title: Title,
    error_class: ErrorClass,
    home: Home,
    created: Date,
    origin: Origin,
    status: Status,
    incident: Incident,
    body: Body,
    recurrences: Vec<Recurrence>,
    applies_to: Vec<ScopeTag>,
}

impl Rule {
    /// Assemble a rule from its validated parts.
    ///
    /// Eleven arguments, deliberately: these are the rule's essential fields and
    /// construction requires all of them (there is no incomplete-`Rule` state
    /// to guard against). Every parameter is a distinct newtype, so an
    /// argument-order mistake is a compile error rather than a runtime bug — the
    /// clippy `too_many_arguments` lint is suppressed for that reason.
    ///
    /// `recurrences` is a required argument rather than a defaulted field or a
    /// post-construction setter, even though the overwhelming majority of rules
    /// pass an empty vector. A default would make *dropping* a rule's
    /// recurrence history — in the parser, in a transform, in a future
    /// constructor site — silent; as an argument it is a compile error at the
    /// site that drops it. Same reasoning as "a parse failure stops the build":
    /// the fact that is easiest to lose is the one that must be impossible to
    /// lose quietly. `applies_to` is required for the same reason, though its
    /// failure runs the other way: a dropped audience makes a rule *more*
    /// visible, not less, and a rule silently re-universalised is still a rule
    /// nobody decided to broadcast.
    ///
    /// **The two collection arguments come last, in the order the format grew
    /// them** (`recurrences` 2026-09-06, `applies_to` 2026-09-13), so adding a
    /// field never renumbers an existing argument. Position carries no risk
    /// here — every parameter is a distinct type, `Vec<Recurrence>` and
    /// `Vec<ScopeTag>` included, so a swap is a compile error. The *file*
    /// format orders itself for a human reader instead: `applies_to` is
    /// written beside `home`, because that is where it is read.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        tag: RuleTag,
        title: Title,
        error_class: ErrorClass,
        home: Home,
        created: Date,
        origin: Origin,
        status: Status,
        incident: Incident,
        body: Body,
        recurrences: Vec<Recurrence>,
        applies_to: Vec<ScopeTag>,
    ) -> Self {
        Rule {
            tag,
            title,
            error_class,
            home,
            created,
            origin,
            status,
            incident,
            body,
            recurrences,
            applies_to,
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

    /// Where the rule lives. One answer, always.
    #[must_use]
    pub fn home(&self) -> &Home {
        &self.home
    }

    /// Which audiences the rule serves, or empty for "every audience".
    ///
    /// Deliberately not a second home: [`Rule::home`] says who owns the rule
    /// and says it once; this says who should load it, which has no such
    /// cardinality. An emitter must never read this to decide a *path* —
    /// `Home` alone decides where a rule lands, and scope decides only whether
    /// it is included.
    #[must_use]
    pub fn applies_to(&self) -> &[ScopeTag] {
        &self.applies_to
    }

    /// Whether this rule is narrowed to particular audiences.
    #[must_use]
    pub fn is_scoped(&self) -> bool {
        !self.applies_to.is_empty()
    }

    /// Whether this rule should be emitted for `audience`.
    ///
    /// Two defaults, and both are the safety decision rather than a
    /// convenience:
    ///
    /// * **An empty `audience` serves everything.** No `--scope` means no
    ///   narrowing, so an invocation that asks for nothing in particular gets
    ///   the whole library, exactly as it did before this field existed.
    /// * **An unscoped rule serves every audience.** This is the load-bearing
    ///   one. Adding `applies_to` to one rule must never be able to remove a
    ///   *different* rule from an existing build, and a rule that has declared
    ///   no audience has not opted out of anyone's. A silently dropped rule is
    ///   a lost correction — the invariant at the top of `CLAUDE.md`.
    ///
    /// Only a rule that *has* declared an audience can be withheld, and only
    /// from an audience it does not name.
    #[must_use]
    pub fn serves(&self, audience: &[ScopeTag]) -> bool {
        audience.is_empty()
            || !self.is_scoped()
            || self.applies_to.iter().any(|s| audience.contains(s))
    }

    /// The date the rule was created.
    #[must_use]
    pub fn created(&self) -> Date {
        self.created
    }

    /// Where the rule came from: a real failure, or existing practice written
    /// down. The counter-metric to recurrence — see [`Origin`].
    #[must_use]
    pub fn origin(&self) -> Origin {
        self.origin
    }

    /// Whether this rule is inert evidence: authored rather than mined, and
    /// never seen to fire. The null result says these are the rules that change
    /// nothing — you cannot author your way to a delta — so a library that
    /// looks healthy because it is full of them is the failure a bare
    /// recurrence count would hide.
    #[must_use]
    pub fn is_inert(&self) -> bool {
        !self.origin.is_mined() && !self.has_recurred()
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

    /// Every recorded later occurrence, in the order the rule file states them.
    #[must_use]
    pub fn recurrences(&self) -> &[Recurrence] {
        &self.recurrences
    }

    /// Whether the error class has happened again since the rule was written.
    ///
    /// Recurrence is the one observation that says whether a rule is working:
    /// a rule that has bitten twice is not the same artifact as one written
    /// once and never seen again, and until this field existed the two were
    /// indistinguishable.
    #[must_use]
    pub fn has_recurred(&self) -> bool {
        !self.recurrences.is_empty()
    }

    /// The date of the most recent recurrence, or `None` if there are none.
    ///
    /// The **maximum** date, not the last element: recurrences round-trip in
    /// the order the file states them, which is not required to be
    /// chronological.
    #[must_use]
    pub fn latest_recurrence(&self) -> Option<Date> {
        self.recurrences.iter().map(Recurrence::date).max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(recurrences: Vec<Recurrence>) -> Rule {
        Rule::new(
            RuleTag::parse("R:x").expect("valid tag"),
            Title::parse("A title").expect("non-empty title"),
            ErrorClass::parse("an error class").expect("non-empty error class"),
            Home::global(),
            Date::parse("2026-08-13").expect("valid date"),
            Origin::Mined,
            Status::active(),
            Incident::parse("the triggering incident").expect("non-empty incident"),
            Body::parse("Do the thing.").expect("non-empty body"),
            recurrences,
            Vec::new(),
        )
    }

    fn recurrence(date: &str) -> Recurrence {
        Recurrence::new(
            Date::parse(date).expect("valid date"),
            Incident::parse("it happened again").expect("non-empty incident"),
        )
    }

    fn scoped(names: &[&str]) -> Rule {
        Rule::new(
            RuleTag::parse("R:x").expect("valid tag"),
            Title::parse("A title").expect("non-empty title"),
            ErrorClass::parse("an error class").expect("non-empty error class"),
            Home::domain("low-latency").expect("non-empty domain"),
            Date::parse("2026-09-13").expect("valid date"),
            Origin::Mined,
            Status::active(),
            Incident::parse("the triggering incident").expect("non-empty incident"),
            Body::parse("Do the thing.").expect("non-empty body"),
            Vec::new(),
            names
                .iter()
                .map(|n| ScopeTag::parse(*n).expect("valid scope"))
                .collect(),
        )
    }

    fn audience(names: &[&str]) -> Vec<ScopeTag> {
        names
            .iter()
            .map(|n| ScopeTag::parse(*n).expect("valid scope"))
            .collect()
    }

    // The two safety defaults, pinned where they are defined. Either one
    // inverted would let `--scope` withhold a rule nobody scoped, which is a
    // correction lost.
    #[test]
    fn an_unscoped_rule_serves_every_audience() {
        let r = scoped(&[]);
        assert!(!r.is_scoped());
        assert!(r.serves(&[]));
        assert!(r.serves(&audience(&["rust"])));
        assert!(r.serves(&audience(&["java", "cobol"])));
    }

    #[test]
    fn an_empty_audience_is_served_by_every_rule() {
        assert!(scoped(&["rust"]).serves(&[]));
    }

    #[test]
    fn a_scoped_rule_serves_only_an_audience_it_names() {
        let r = scoped(&["rust", "java"]);
        assert!(r.is_scoped());
        assert!(r.serves(&audience(&["java"])));
        assert!(r.serves(&audience(&["cobol", "rust"])));
        assert!(!r.serves(&audience(&["cobol"])));
    }

    #[test]
    fn a_rule_with_no_recurrences_has_not_recurred() {
        let r = rule(Vec::new());
        assert!(!r.has_recurred());
        assert!(r.recurrences().is_empty());
        assert_eq!(r.latest_recurrence(), None);
    }

    #[test]
    fn recurrences_are_kept_in_file_order() {
        // Order is preserved because the serializer must reproduce the file it
        // parsed; it is deliberately not sorted on the way in.
        let r = rule(vec![recurrence("2026-08-30"), recurrence("2026-08-24")]);
        let dates: Vec<String> = r
            .recurrences()
            .iter()
            .map(|x| x.date().to_string())
            .collect();
        assert_eq!(dates, vec!["2026-08-30", "2026-08-24"]);
    }

    // The pin for the reason `latest_recurrence` is a max and not a `.last()`:
    // file order is not chronological order, and "most recently" must mean the
    // latest date whichever way round the file lists them.
    #[test]
    fn latest_recurrence_is_the_maximum_date_not_the_last_entry() {
        let r = rule(vec![recurrence("2026-08-30"), recurrence("2026-08-24")]);
        assert!(r.has_recurred());
        assert_eq!(
            r.latest_recurrence().map(|d| d.to_string()),
            Some("2026-08-30".to_owned())
        );
    }
}
