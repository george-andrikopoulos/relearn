//! Integration tests for the advisory linter (`relearn::lint`). Written before
//! the module (red-green): they exercise the public API only — `lint` over a
//! validated library, and the `Finding`/`Severity` types it returns.

use relearn::library::{Library, Validated};
use relearn::lint::{Finding, Severity, lint};
use relearn::rule::{
    Body, Date, ErrorClass, Home, Incident, Origin, Recurrence, Rule, RuleTag, Status, Title,
};

fn rule(tag: &str, home: Home, error_class: &str, body: &str) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("A title").expect("non-empty title"),
        ErrorClass::parse(error_class).expect("non-empty error class"),
        home,
        Date::parse("2026-08-13").expect("valid date"),
        Origin::Mined,
        Status::active(),
        Incident::parse("an incident").expect("non-empty incident"),
        Body::parse(body).expect("non-empty body"),
        Vec::new(),
        Vec::new(),
    )
}

fn validated(rules: Vec<Rule>) -> Library<Validated> {
    Library::from_rules(rules)
        .validate()
        .expect("distinct tags validate")
}

/// An atticked (retired) rule with the given tag.
fn atticked(tag: &str) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("A title").expect("non-empty title"),
        ErrorClass::parse("some class").expect("non-empty error class"),
        Home::global(),
        Date::parse("2026-08-13").expect("valid date"),
        Origin::Mined,
        Status::attic(
            "cold surface, challenge-tested",
            Date::parse("2026-09-01").expect("valid date"),
        )
        .expect("non-empty reason"),
        Incident::parse("an incident").expect("non-empty incident"),
        Body::parse("Body.").expect("non-empty body"),
        Vec::new(),
        Vec::new(),
    )
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
fn retired_reference_flags_a_citation_of_a_retired_rule() {
    let lib = validated(vec![
        rule(
            "R:a",
            Home::global(),
            "class a",
            "Builds on R:old, now retired.",
        ),
        atticked("R:old"),
    ]);
    let findings = lint(&lib);
    assert!(
        findings
            .iter()
            .any(|f| matches!(f, Finding::RetiredReference { .. })),
        "citing an atticked rule is flagged"
    );
    // It exists, so it is retired — not dangling.
    assert!(
        !findings
            .iter()
            .any(|f| matches!(f, Finding::DanglingReference { .. }))
    );
}

#[test]
fn a_retired_reference_is_info_severity() {
    let lib = validated(vec![
        rule("R:a", Home::global(), "class a", "See R:old for history."),
        atticked("R:old"),
    ]);
    let retired = lint(&lib)
        .into_iter()
        .find(|f| matches!(f, Finding::RetiredReference { .. }))
        .expect("a retired-reference finding");
    assert_eq!(retired.severity(), Severity::Info);
}

#[test]
fn a_tag_cited_twice_in_the_body_yields_one_finding() {
    // Regression pin: cited tags are deduplicated per rule, so a body that
    // mentions R:ghost twice is one dangling reference, not two.
    //
    // Renamed 2026-08-16. It previously read "..._in_both_body_and_incident",
    // pinning dedup ACROSS those two fields — but `cited_tags` now reads the
    // body only (provenance is not a citation surface), so that framing
    // described behaviour the code no longer has. The test kept passing, which
    // made it read stronger than it was: a green test with a false comment is
    // the test-level form of [R:guarantee-needs-a-reader].
    let lib = validated(vec![Rule::new(
        RuleTag::parse("R:a").expect("valid tag"),
        Title::parse("A title").expect("non-empty title"),
        ErrorClass::parse("class a").expect("non-empty error class"),
        Home::global(),
        Date::parse("2026-08-13").expect("valid date"),
        Origin::Mined,
        Status::active(),
        Incident::parse("first seen alongside no tags at all").expect("non-empty incident"),
        Body::parse("This builds on R:ghost, and again on R:ghost.").expect("non-empty body"),
        Vec::new(),
        Vec::new(),
    )]);
    let dangling: Vec<_> = lint(&lib)
        .into_iter()
        .filter(|f| matches!(f, Finding::DanglingReference { .. }))
        .collect();
    assert_eq!(
        dangling.len(),
        1,
        "one finding per (from, to), not per mention"
    );
}

#[test]
fn citing_an_active_rule_is_not_a_retired_reference() {
    let lib = validated(vec![
        rule("R:a", Home::global(), "class a", "See R:b."),
        rule("R:b", Home::global(), "class b", "Body b."),
    ]);
    assert!(
        !lint(&lib)
            .iter()
            .any(|f| matches!(f, Finding::RetiredReference { .. }))
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

// ── Provenance is not a citation surface ──────────────────────────────────
//
// `incident` records why a rule exists. It legitimately names tags that are
// retired, renamed, or owned by another library — "retagged from R:x",
// "supersedes R:y". Those are historical mentions, not live citations, and
// reading them as citations makes the linter flag its own provenance.
//
// Same class as [R:detector-excludes-own-definitions]: documentation *about* a
// pattern must never be read as an instance of it. An always-warning linter
// gets muted, and a muted check is worse than none.
//
// Incident 2026-08-16: splitting R:revision-integrity into R:doc-currency, the
// new rule recorded "retagged from <old tag>" in its incident. `relearn lint`
// reported a dangling reference to a tag that only appeared in provenance, and
// the wording had to be contorted to silence a false positive.

fn rule_with_incident(tag: &str, incident: &str, body: &str) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("A title").expect("non-empty title"),
        ErrorClass::parse("some class").expect("non-empty error class"),
        Home::global(),
        Date::parse("2026-08-16").expect("valid date"),
        Origin::Mined,
        Status::active(),
        Incident::parse(incident).expect("non-empty incident"),
        Body::parse(body).expect("non-empty body"),
        Vec::new(),
        Vec::new(),
    )
}

#[test]
fn a_tag_named_only_in_provenance_is_not_a_dangling_reference() {
    let lib = validated(vec![rule_with_incident(
        "R:beta",
        "beta (2026-08-16): retagged from R:gamma because that tag was taken.",
        "The body cites nobody.",
    )]);
    let findings = lint(&lib);
    assert!(
        !findings
            .iter()
            .any(|f| matches!(f, Finding::DanglingReference { .. })),
        "a tag mentioned only in `incident` is provenance, not a citation: {findings:?}"
    );
}

#[test]
fn a_tag_cited_in_the_body_is_still_flagged() {
    // The regression guard for the fix above: narrowing the scan to `body`
    // must not blind the detector to real dangling citations.
    let lib = validated(vec![rule_with_incident(
        "R:beta",
        "beta (2026-08-16): an ordinary incident naming no tags.",
        "This rule builds on R:ghost, which was never written.",
    )]);
    let findings = lint(&lib);
    assert!(
        findings.iter().any(|f| matches!(
            f,
            Finding::DanglingReference { to, .. } if to.as_str() == "R:ghost"
        )),
        "a dangling citation in the BODY must still be reported: {findings:?}"
    );
}

/// A rule with the given status carrying the given recurrence dates.
fn recurred(tag: &str, status: Status, dates: &[&str]) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("A title").expect("non-empty title"),
        ErrorClass::parse("some class").expect("non-empty error class"),
        Home::global(),
        Date::parse("2026-08-13").expect("valid date"),
        Origin::Mined,
        status,
        Incident::parse("the triggering incident").expect("non-empty incident"),
        Body::parse("Body.").expect("non-empty body"),
        dates
            .iter()
            .map(|d| {
                Recurrence::new(
                    Date::parse(d).expect("valid date"),
                    Incident::parse("it happened again").expect("non-empty incident"),
                )
            })
            .collect(),
        Vec::new(),
    )
}

#[test]
fn an_active_rule_that_has_recurred_is_flagged() {
    let lib = validated(vec![recurred(
        "R:bitten",
        Status::active(),
        &["2026-08-24"],
    )]);
    let findings = lint(&lib);
    assert!(
        findings.iter().any(|f| matches!(
            f,
            Finding::UnheldRecurrence { tag, times, .. }
                if tag.as_str() == "R:bitten" && *times == 1
        )),
        "an active rule that has fired again must be reported: {findings:?}"
    );
}

/// The first recurrence is the whole finding — there is no count threshold to
/// cross, because one recurrence already proves the prose failed.
#[test]
fn one_recurrence_is_enough_and_the_latest_date_is_the_maximum() {
    let lib = validated(vec![recurred(
        "R:bitten",
        Status::active(),
        &["2026-08-30", "2026-08-16"],
    )]);
    let findings = lint(&lib);
    let latest = findings
        .iter()
        .find_map(|f| match f {
            Finding::UnheldRecurrence { times, latest, .. } => Some((*times, *latest)),
            _ => None,
        })
        .expect("the finding is raised");
    assert_eq!(latest.0, 2);
    assert_eq!(latest.1.to_string(), "2026-08-30");
}

#[test]
fn an_unheld_recurrence_is_a_warning_not_an_error() {
    // Deliberately not `Error`: `Error` means the emitted tree would be wrong.
    // This is a fault in the library, and a build that emits correctly must not
    // fail on it.
    let lib = validated(vec![recurred(
        "R:bitten",
        Status::active(),
        &["2026-08-24"],
    )]);
    let finding = lint(&lib)
        .into_iter()
        .find(|f| matches!(f, Finding::UnheldRecurrence { .. }))
        .expect("the finding is raised");
    assert_eq!(finding.severity(), Severity::Warning);
}

#[test]
fn a_rule_that_has_never_recurred_is_not_flagged() {
    let lib = validated(vec![recurred("R:quiet", Status::active(), &[])]);
    assert!(
        !lint(&lib)
            .iter()
            .any(|f| matches!(f, Finding::UnheldRecurrence { .. })),
        "a rule with no recurrences must not be flagged"
    );
}

/// A **graduated** rule that has recurred is deliberately NOT this finding. It
/// is the sharper one — a named stronger control that demonstrably did not hold
/// — and it needs a graduation date `Status::Graduated` does not carry. Flagging
/// it here as an ordinary `UnheldRecurrence` would say the wrong thing (the rule
/// is not held by prose alone) and would pre-empt the finding it deserves.
#[test]
fn a_graduated_rule_that_has_recurred_is_not_this_finding() {
    let lib = validated(vec![recurred(
        "R:grad",
        Status::graduated("hook:x", Date::parse("2026-07-21").expect("valid date"))
            .expect("non-empty destination"),
        &["2026-08-24"],
    )]);
    assert!(
        !lint(&lib)
            .iter()
            .any(|f| matches!(f, Finding::UnheldRecurrence { .. })),
        "a graduated rule is not held by prose alone; this finding does not apply to it"
    );
}

/// A rule with the given status, origin and recurrence dates.
fn with_origin(tag: &str, origin: Origin, status: Status, dates: &[&str]) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("A title").expect("non-empty title"),
        ErrorClass::parse("some class").expect("non-empty error class"),
        Home::global(),
        Date::parse("2026-08-13").expect("valid date"),
        origin,
        status,
        Incident::parse("the triggering incident").expect("non-empty incident"),
        Body::parse("Body.").expect("non-empty body"),
        dates
            .iter()
            .map(|d| {
                Recurrence::new(
                    Date::parse(d).expect("valid date"),
                    Incident::parse("it happened again").expect("non-empty incident"),
                )
            })
            .collect(),
        Vec::new(),
    )
}

fn graduated_on(date: &str) -> Status {
    Status::graduated("hook:x", Date::parse(date).expect("valid date"))
        .expect("non-empty destination")
}

/// The sharpest finding: the named stronger control was claimed to hold this and
/// demonstrably did not. An `Error`, because every emitted layer is carrying an
/// "Also enforced by" line that the recurrence disproves.
#[test]
fn a_recurrence_after_graduation_is_an_error() {
    let lib = validated(vec![with_origin(
        "R:broke",
        Origin::Mined,
        graduated_on("2026-07-21"),
        &["2026-08-24"],
    )]);
    let finding = lint(&lib)
        .into_iter()
        .find(|f| matches!(f, Finding::RecurrenceAfterGraduation { .. }))
        .expect("the finding is raised");
    assert_eq!(finding.severity(), Severity::Error);
    match finding {
        Finding::RecurrenceAfterGraduation {
            graduated,
            recurred,
            ..
        } => {
            assert_eq!(graduated.to_string(), "2026-07-21");
            assert_eq!(recurred.to_string(), "2026-08-24");
        }
        other => panic!("wrong finding: {other:?}"),
    }
}

/// A recurrence *before* the graduation is very often the incident that
/// prompted it. Flagging that would punish exactly the response the library
/// wants, so only a recurrence strictly after the date counts.
#[test]
fn a_recurrence_before_graduation_is_not_a_finding() {
    let lib = validated(vec![with_origin(
        "R:fixed",
        Origin::Mined,
        graduated_on("2026-08-24"),
        &["2026-07-01", "2026-08-16"],
    )]);
    assert!(
        !lint(&lib)
            .iter()
            .any(|f| matches!(f, Finding::RecurrenceAfterGraduation { .. })),
        "a recurrence that predates the graduation is not evidence against it"
    );
}

/// A recurrence exactly *on* the graduation date is not after it — the boundary
/// is pinned because an off-by-one here manufactures an `Error` out of the day
/// the control landed.
#[test]
fn a_recurrence_on_the_graduation_date_is_not_after_it() {
    let lib = validated(vec![with_origin(
        "R:sameday",
        Origin::Mined,
        graduated_on("2026-08-24"),
        &["2026-08-24"],
    )]);
    assert!(
        !lint(&lib)
            .iter()
            .any(|f| matches!(f, Finding::RecurrenceAfterGraduation { .. }))
    );
}

/// The counter-metric. Inert is *codified and never fired* — a mined rule that
/// has not recurred is not inert, it is a rule that may well be working, and
/// conflating the two is the whole error this number exists to prevent.
#[test]
fn the_tally_counts_recurred_and_inert_separately() {
    let lib = validated(vec![
        with_origin("R:a", Origin::Mined, Status::active(), &["2026-08-24"]),
        with_origin("R:b", Origin::Mined, Status::active(), &[]),
        with_origin("R:c", Origin::Codified, Status::active(), &[]),
        with_origin("R:d", Origin::Codified, Status::active(), &["2026-08-24"]),
    ]);
    let t = relearn::lint::tally(&lib);
    assert_eq!(t.total(), 4);
    assert_eq!(t.recurred(), 2, "R:a and R:d have recurred");
    assert_eq!(t.inert(), 1, "only R:c is codified AND never fired");
}
