//! Integration tests for the advisory linter (`relearn::lint`). Written before
//! the module (red-green): they exercise the public API only — `lint` over a
//! validated library, and the `Finding`/`Severity` types it returns.

use relearn::library::{Library, Validated};
use relearn::lint::{Finding, Severity, lint};
use relearn::rule::{Body, Date, ErrorClass, Home, Incident, Rule, RuleTag, Status, Title};

fn rule(tag: &str, home: Home, error_class: &str, body: &str) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("A title").expect("non-empty title"),
        ErrorClass::parse(error_class).expect("non-empty error class"),
        home,
        Date::parse("2026-08-13").expect("valid date"),
        Status::active(),
        Incident::parse("an incident").expect("non-empty incident"),
        Body::parse(body).expect("non-empty body"),
    )
}

fn validated(rules: Vec<Rule>) -> Library<Validated> {
    Library::from_rules(rules)
        .validate()
        .expect("distinct tags validate")
}

#[test]
fn a_clean_library_has_no_findings() {
    let lib = validated(vec![
        rule("R:a", Home::global(), "class one", "Body a."),
        rule(
            "R:b",
            Home::domain("rust").expect("non-empty domain"),
            "class two",
            "Body b.",
        ),
    ]);
    assert!(lint(&lib).is_empty());
}

#[test]
fn overlapping_scope_flags_a_shared_error_class() {
    let lib = validated(vec![
        rule("R:a", Home::global(), "Same Class", "Body a."),
        rule("R:b", Home::global(), "same class", "Body b."),
    ]);
    let findings = lint(&lib);
    assert_eq!(findings.len(), 1);
    match &findings[0] {
        Finding::OverlappingScope { tags, .. } => {
            assert_eq!(tags.len(), 2, "both rules are named");
        }
        other => panic!("expected OverlappingScope, got {other:?}"),
    }
}

#[test]
fn home_slug_collision_flags_distinct_homes_with_the_same_slug() {
    // "rust!" and "rust" are distinct domains that both slugify to `domain-rust`.
    let lib = validated(vec![
        rule(
            "R:a",
            Home::domain("rust!").expect("non-empty domain"),
            "class a",
            "Body a.",
        ),
        rule(
            "R:b",
            Home::domain("rust").expect("non-empty domain"),
            "class b",
            "Body b.",
        ),
    ]);
    assert!(
        lint(&lib)
            .iter()
            .any(|f| matches!(f, Finding::HomeSlugCollision { .. })),
        "distinct homes sharing a slug must be flagged"
    );
}

#[test]
fn same_home_is_not_a_collision() {
    // Two rules in the *same* home share a slug legitimately (that is how
    // per-home skills work) — not a collision.
    let lib = validated(vec![
        rule(
            "R:a",
            Home::domain("rust").expect("non-empty domain"),
            "class a",
            "Body a.",
        ),
        rule(
            "R:b",
            Home::domain("rust").expect("non-empty domain"),
            "class b",
            "Body b.",
        ),
    ]);
    assert!(
        !lint(&lib)
            .iter()
            .any(|f| matches!(f, Finding::HomeSlugCollision { .. }))
    );
}

#[test]
fn dangling_reference_flags_an_unknown_cited_tag() {
    let lib = validated(vec![rule(
        "R:a",
        Home::global(),
        "class a",
        "This builds on R:ghost which was never written.",
    )]);
    let findings = lint(&lib);
    assert_eq!(findings.len(), 1);
    match &findings[0] {
        Finding::DanglingReference { from, to } => {
            assert_eq!(from.as_str(), "R:a");
            assert_eq!(to.as_str(), "R:ghost");
        }
        other => panic!("expected DanglingReference, got {other:?}"),
    }
}

#[test]
fn a_resolved_reference_is_not_flagged() {
    let lib = validated(vec![
        rule(
            "R:a",
            Home::global(),
            "class a",
            "See R:b for the paired rule.",
        ),
        rule("R:b", Home::global(), "class b", "Body b."),
    ]);
    assert!(
        !lint(&lib)
            .iter()
            .any(|f| matches!(f, Finding::DanglingReference { .. }))
    );
}

#[test]
fn or_in_prose_is_not_read_as_a_tag() {
    // "OR:" must not be mistaken for a tag citation.
    let lib = validated(vec![rule(
        "R:a",
        Home::global(),
        "class a",
        "Do this OR:that, your choice.",
    )]);
    assert!(lint(&lib).is_empty());
}

#[test]
fn findings_are_sorted_most_severe_first() {
    let lib = validated(vec![
        // overlap (Warning) + a dangling ref (Warning)
        rule("R:a", Home::global(), "shared", "Body a."),
        rule("R:b", Home::global(), "shared", "Cites R:ghost."),
        // collision (Error): distinct domains, same slug
        rule(
            "R:c",
            Home::domain("rust!").expect("non-empty domain"),
            "c",
            "Body c.",
        ),
        rule(
            "R:d",
            Home::domain("rust").expect("non-empty domain"),
            "d",
            "Body d.",
        ),
    ]);
    let findings = lint(&lib);
    assert!(findings.len() >= 2);
    assert_eq!(
        findings[0].severity(),
        Severity::Error,
        "the collision (Error) sorts before the warnings"
    );
}
