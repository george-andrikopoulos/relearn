//! Contribution: what leaves the machine, and what structurally cannot.
//!
//! A rule's `incident` is a verbatim quotation from a private working session.
//! It cannot travel, and "cannot" here is not a policy — [`Contribution`] is a
//! **projection** that never borrows the field at all, so no contribution
//! renderer can emit it, forget to strip it, or strip it incorrectly. The same
//! goes for `[[recurrence]]` incidents, of which there are more.
//!
//! The published incident is a **separate authored field**: a rewritten account
//! with no quotation, no names, no paths, no repository identifiers. Nothing
//! derives it from the original — a scrubber would leak what it did not
//! recognise and destroy context it did not understand, and worse, it would stop
//! people reading the output because something appeared to be handling it.
//!
//! What the code *can* do is refuse to publish what it can recognise, and force
//! a human to read the exact text before it is written. That is the banned-terms
//! matcher and the two-step confirm, and neither is a substitute for the
//! contributor's judgement — §12.2 of the design puts the scrub on the person
//! for a reason no gate changes.

use relearn::contribute::{Contribution, NotContributable, SupersedingVersion};
use relearn::rule::{
    Approval, Approver, Authority, Body, ControlRef, Date, ErrorClass, Home, Incident, Origin,
    PublishedIncident, Recurrence, Rule, RuleTag, SourceId, Status, Title, Version,
};

const RAW: &str =
    "George said in the 2026-09-06 session: the gate in C:/Users/ganak/relearn missed it";
const PUBLISHED: &str = "A repository-local gate scanned its own tree and reported clean while the quotation \
     travelled to another repository, which had no such gate.";

fn rule_with(
    home: Home,
    origin: Origin,
    authority: Authority,
    published: Option<&str>,
    recurrences: Vec<Recurrence>,
) -> Rule {
    Rule::new(
        RuleTag::parse("R:x").expect("valid tag"),
        Title::parse("A title").expect("non-empty title"),
        ErrorClass::parse("an error class").expect("non-empty error class"),
        home,
        Date::parse("2026-09-13").expect("valid date"),
        origin,
        Status::active(),
        Incident::parse(RAW).expect("non-empty incident"),
        Body::parse("Do the thing.").expect("non-empty body"),
        recurrences,
        Vec::new(),
        authority,
        published.map(|p| PublishedIncident::parse(p).expect("non-empty published incident")),
    )
}

fn contributable() -> Rule {
    rule_with(
        Home::global(),
        Origin::Mined,
        Authority::local(),
        Some(PUBLISHED),
        Vec::new(),
    )
}

// ── what cannot leave ───────────────────────────────────────────────────────

/// **The load-bearing assertion of this phase.** The raw incident is absent from
/// the rendered contribution — and absent because the projection never held it,
/// not because a renderer removed it.
#[test]
fn the_raw_incident_is_nowhere_in_what_would_leave() {
    let rule = contributable();
    let contribution = Contribution::of(&rule).expect("a global mined rule is contributable");
    let document = contribution.to_document(first(1));

    assert!(
        !document.contains(RAW),
        "the raw incident reached the output:\n{document}"
    );
    for fragment in ["George", "ganak", "C:/Users", "2026-09-06 session"] {
        assert!(
            !document.contains(fragment),
            "a fragment of the raw incident reached the output: {fragment}\n{document}"
        );
    }
    assert!(
        document.contains(PUBLISHED),
        "the published incident is what travels:\n{document}"
    );
}

/// Recurrence incidents are quotations too, and there are more of them. The
/// projection carries no recurrences at all: a contribution is the rule, not
/// this install's history of it — that history is the report flow's business,
/// and anonymous.
#[test]
fn recurrence_incidents_never_travel() {
    let rule = rule_with(
        Home::global(),
        Origin::Mined,
        Authority::local(),
        Some(PUBLISHED),
        vec![Recurrence::new(
            Date::parse("2026-09-05").expect("valid date"),
            Incident::parse("It happened again in George's other repository").expect("non-empty"),
        )],
    );
    let document = Contribution::of(&rule)
        .expect("contributable")
        .to_document(first(1));
    assert!(!document.contains("George"), "{document}");
    assert!(!document.contains("recurrence"), "{document}");
}

// ── who may be contributed ──────────────────────────────────────────────────

/// A2's federation exclusion, finally consumed. An org-homed rule cannot be
/// contributed — and neither can a project-homed one, whose home carries a
/// filesystem path.
#[test]
fn a_withheld_home_cannot_be_contributed() {
    for home in [
        Home::org("acme").expect("non-empty org"),
        Home::project("C:/Users/ganak/relearn").expect("non-empty project"),
    ] {
        let rule = rule_with(
            home,
            Origin::Mined,
            Authority::local(),
            Some(PUBLISHED),
            Vec::new(),
        );
        assert!(
            matches!(
                Contribution::of(&rule),
                Err(NotContributable::HomeIsWithheld)
            ),
            "a withheld home must not be contributable"
        );
    }
}

#[test]
fn a_rule_with_no_published_incident_cannot_be_contributed() {
    let rule = rule_with(
        Home::global(),
        Origin::Mined,
        Authority::local(),
        None,
        Vec::new(),
    );
    assert!(matches!(
        Contribution::of(&rule),
        Err(NotContributable::NoPublishedIncident)
    ));
}

/// A cache is already upstream's: contributing it back is either a no-op or an
/// attempt to publish someone else's rule as your own. A **fork** is a different
/// matter — that is a change you made, and contributing it is exactly what the
/// design tells you to do instead of editing the cache.
#[test]
fn a_cache_cannot_be_contributed_but_a_fork_can() {
    let cached = rule_with(
        Home::global(),
        Origin::Mined,
        Authority::cached(
            SourceId::parse("relearn-upstream").expect("valid source"),
            Version::new(3),
            Date::parse("2026-09-10").expect("valid date"),
        ),
        Some(PUBLISHED),
        Vec::new(),
    );
    assert!(matches!(
        Contribution::of(&cached),
        Err(NotContributable::IsACache)
    ));

    let adopted = rule_with(
        Home::global(),
        Origin::Mined,
        Authority::adopted(
            SourceId::parse("relearn-upstream").expect("valid source"),
            Version::new(3),
            Date::parse("2026-09-10").expect("valid date"),
            Date::parse("2026-09-13").expect("valid date"),
        ),
        Some(PUBLISHED),
        Vec::new(),
    );
    assert!(Contribution::of(&adopted).is_ok());
}

/// A mandate's provenance is an approver — a person or a board, by name. That is
/// the one thing a published rule may not carry, and a mandate stripped of its
/// approval would be a rule claiming a sign-off it no longer records. Refused
/// rather than transformed.
#[test]
fn a_mandated_rule_cannot_be_contributed() {
    let rule = rule_with(
        Home::global(),
        Origin::Mandated(Approval::new(
            Approver::parse("the security working group").expect("non-empty approver"),
            Date::parse("2026-07-11").expect("valid date"),
            ControlRef::parse("AC-6(9)").expect("non-empty control"),
        )),
        Authority::local(),
        Some(PUBLISHED),
        Vec::new(),
    );
    assert!(matches!(
        Contribution::of(&rule),
        Err(NotContributable::IsMandated)
    ));
}

/// The contribution arrives upstream as a rule in its own right: no authority
/// line (it is not a cache of anything, and the local fork's provenance is this
/// install's business), and it parses as a rule so a reviewer can run the same
/// tools on it.
#[test]
fn a_contribution_parses_as_a_rule_whose_incident_is_the_published_one() {
    let rule = contributable();
    let document = Contribution::of(&rule)
        .expect("contributable")
        .to_document(first(1));

    let reparsed = relearn::rule::parse_document(&document).expect("a contribution is a rule");
    assert_eq!(reparsed.incident().as_str(), PUBLISHED);
    assert_eq!(reparsed.tag(), rule.tag());
    // **Local at the stated revision**, not unnumbered. Upstream is the home of
    // what it publishes, and the revision is what every cache of this rule will
    // be compared against — without it, `pull` refuses the rule outright rather
    // than accept a copy whose staleness signal is dead on arrival.
    assert_eq!(reparsed.authority(), &Authority::local_at(Version::new(1)));
    assert!(reparsed.recurrences().is_empty());
}

// ── a republication must supersede what is already there ────────────────────

/// **A first publication supersedes nothing**, so any revision will do. The
/// destination is empty, and there is no number to be greater than.
#[test]
fn a_first_publication_supersedes_nothing() {
    assert_eq!(
        SupersedingVersion::of(Version::new(1), None)
            .expect("nothing is published yet")
            .get(),
        Version::new(1)
    );
    assert_eq!(
        SupersedingVersion::of(Version::new(97), None)
            .expect("a first publication may start anywhere")
            .get(),
        Version::new(97)
    );
}

/// **Republishing must go forwards.** A revision that does not supersede what is
/// already on the drive makes every cache of that rule read as newer than
/// upstream — and `cache-behind` compares numbers, so nothing would ever
/// notice. The type refuses it rather than a call site checking for it.
#[test]
fn a_republication_below_or_equal_to_what_is_published_is_refused() {
    for proposed in [1_u32, 2, 3] {
        let refused = SupersedingVersion::of(Version::new(proposed), Some(Version::new(3)))
            .expect_err("a revision at or below the published one does not supersede it");
        // The error carries **both** numbers, because a contributor told only
        // "refused" has to go and look up what is already published — and the
        // number they need is the one the tool just read.
        assert_eq!(refused.proposed(), Version::new(proposed));
        assert_eq!(refused.published(), Version::new(3));
        assert!(
            refused.to_string().contains(&format!("{proposed}"))
                && refused.to_string().contains('3'),
            "the message names both revisions: {refused}"
        );
    }
    assert_eq!(
        SupersedingVersion::of(Version::new(4), Some(Version::new(3)))
            .expect("4 supersedes 3")
            .get(),
        Version::new(4)
    );
}

/// The only way to render a contribution is with a version that has been through
/// that check — so "published a revision that goes backwards" is not a mistake
/// the renderer can make, it is a document that cannot be built.
#[test]
fn a_contribution_can_only_be_rendered_with_a_superseding_version() {
    let rule = contributable();
    let contribution = Contribution::of(&rule).expect("contributable");
    let version =
        SupersedingVersion::of(Version::new(5), Some(Version::new(4))).expect("5 supersedes 4");
    let document = contribution.to_document(version);
    assert!(document.contains("version = 5"), "{document}");
}

/// A first publication of revision `n`: nothing is published yet, so it
/// supersedes nothing and any number is legal.
fn first(n: u32) -> SupersedingVersion {
    SupersedingVersion::of(Version::new(n), None).expect("a first publication supersedes nothing")
}
