//! The org layer, mandated content, and the exclusion that must hold before
//! anything can federate.
//!
//! **An `Org`-homed rule can never leave the machine.** Not by a filter in the
//! contribution path, not by a policy in a document: by an exhaustive match that
//! will not compile if a home variant is added without deciding its federation
//! behaviour. A filter lives in one code path and the second code path forgets
//! it; a match cannot be forgotten, because the compiler asks.
//!
//! Established **now, before `contribute` or `report` exist**, which is the
//! right order: the exclusion that arrives after the publishing code is the
//! exclusion somebody has to remember to apply to it.
//!
//! What these tests cannot prove is that a *future* variant is decided
//! correctly — only that it must be decided at all, which is
//! `tests/compile_fail/home_match_must_be_exhaustive.rs` and the wildcard check
//! below. The compiler holds the "must", a human holds the "correctly".

use relearn::rule::{
    Approval, Authority, Body, ControlRef, Date, ErrorClass, Federation, Home, Incident, Origin,
    Rule, RuleTag, Status, Title,
};

fn approval() -> Approval {
    Approval::new(
        relearn::rule::Approver::parse("the security working group").expect("non-empty approver"),
        Date::parse("2026-07-11").expect("valid date"),
        ControlRef::parse("AC-6(9)").expect("non-empty control"),
    )
}

fn rule(tag: &str, home: Home, origin: Origin) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("Title").expect("non-empty title"),
        ErrorClass::parse("ec").expect("non-empty error class"),
        home,
        Date::parse("2026-09-13").expect("valid date"),
        origin,
        Status::active(),
        Incident::parse("an incident").expect("non-empty incident"),
        Body::parse("The body.").expect("non-empty body"),
        Vec::new(),
        Vec::new(),
        Authority::Local,
    )
}

// ── the exclusion ───────────────────────────────────────────────────────────

/// The load-bearing assertion of this phase, stated over **every** home kind so
/// it measures the whole enum rather than the one variant it was written for.
#[test]
fn an_org_home_is_withheld_and_the_other_homes_are_not() {
    assert_eq!(
        Home::org("acme").expect("non-empty org").federation(),
        Federation::Withheld,
        "an Org-homed rule must never be publishable"
    );
    assert_eq!(Home::global().federation(), Federation::Publishable);
    assert_eq!(
        Home::domain("rust").expect("non-empty domain").federation(),
        Federation::Publishable
    );
    assert_eq!(
        Home::project("relearn")
            .expect("non-empty project")
            .federation(),
        Federation::Withheld,
        "a project home names a private path and is meaningless upstream"
    );
}

/// A rule inherits its home's federation answer, so the question is asked in one
/// place — there is no second predicate on `Rule` that could drift from it.
#[test]
fn a_rule_is_publishable_exactly_when_its_home_is() {
    let org = rule(
        "R:o",
        Home::org("acme").expect("non-empty org"),
        Origin::Mined,
    );
    let global = rule("R:g", Home::global(), Origin::Mined);
    assert!(!org.is_publishable());
    assert!(global.is_publishable());
}

/// The wildcard check. An exhaustive match is only load-bearing if it has no
/// catch-all arm: `_ => Publishable` compiles forever and silently publishes
/// every home variant added after it. Nothing but reading the source can see
/// this, so the source is read.
///
/// Comments are stripped first — this file's subject is the wildcard, and the
/// prose in `home.rs` explaining why there is no wildcard would otherwise be the
/// thing that trips it (`[R:detector-excludes-own-definitions]`).
#[test]
fn the_federation_match_has_no_catch_all_arm() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rule/home.rs"),
    )
    .expect("home.rs is readable");

    let code: String = source
        .lines()
        .map(|line| match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");

    let body = code
        .split_once("fn federation(")
        .expect("the federation function exists")
        .1;
    let body = body
        .split_once("\n    }")
        .expect("the federation function is closed")
        .0;

    assert!(
        body.contains("Home::Org"),
        "the federation match must name Org explicitly:\n{body}"
    );
    for wildcard in ["_ =>", "_ |", "| _"] {
        assert!(
            !body.contains(wildcard),
            "a catch-all arm ({wildcard}) silently classifies every future home \
             variant, which is the whole failure this match exists to prevent:\n{body}"
        );
    }
}

// ── mandated content ────────────────────────────────────────────────────────

/// `approval` is mandatory for `Mandated` and unrepresentable otherwise,
/// because it is the variant's payload rather than a field beside it. The
/// assertion here is that the *type* carries it — a `Mandated` with no approval
/// cannot be written, which is why there is no test for that case.
#[test]
fn a_mandate_carries_its_approval() {
    let mandated = Origin::Mandated(approval());
    match &mandated {
        Origin::Mandated(a) => {
            assert_eq!(a.by().as_str(), "the security working group");
            assert_eq!(a.date().to_string(), "2026-07-11");
            assert_eq!(a.control().as_str(), "AC-6(9)");
        }
        other => panic!("expected Mandated, got {other:?}"),
    }
    assert!(mandated.is_mandated());
    assert!(!Origin::Mined.is_mandated());
    assert!(!Origin::Codified.is_mandated());
}

/// A mandate is not evidence. It was never mined from a failure, so counting it
/// in the recurrence statistics would swamp the only number that says whether
/// prose is holding — and it would count as *inert* under the old definition,
/// inflating the counter-metric with rules that were never supposed to be
/// evidence of anything.
#[test]
fn a_mandated_rule_is_neither_recurrence_evidence_nor_inert() {
    let mandated = rule(
        "R:m",
        Home::org("acme").expect("non-empty org"),
        Origin::Mandated(approval()),
    );
    assert!(!mandated.is_inert(), "a mandate is not the counter-metric");
    assert!(
        !mandated.counts_toward_recurrence_statistics(),
        "a mandate is not recurrence evidence"
    );

    let codified = rule("R:c", Home::global(), Origin::Codified);
    assert!(codified.is_inert());
    assert!(codified.counts_toward_recurrence_statistics());
}
