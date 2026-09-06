//! End-to-end tests for restricting a validated library to one home layer —
//! the filter behind `build --home` / `verify --home`.
//!
//! The filter exists because emitting the rules layer to a **user** scope
//! (`~/.claude/rules/`) must carry the domain layer and nothing else. Project
//! homes are `LoadSemantics::Always`, so a project-layer file placed in a user
//! scope would load unscoped in every session in every repository — promoting
//! project rules to machine-wide. That is the failure this filter prevents, so
//! it is tested through the public API rather than trusted to a CLI flag.

use relearn::emit;
use relearn::library::{Library, Validated};
use relearn::rule::{Body, Date, ErrorClass, Home, Incident, Origin, Rule, RuleTag, Status, Title};

fn rule(tag: &str, home: Home) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("Title").expect("non-empty title"),
        ErrorClass::parse("ec").expect("non-empty error class"),
        home,
        Date::parse("2026-08-13").expect("valid date"),
        Origin::Mined,
        Status::active(),
        Incident::parse("an incident").expect("non-empty incident"),
        Body::parse("The body.").expect("non-empty body"),
        Vec::new(),
    )
}

/// A library with one rule in each of the three home kinds.
fn mixed() -> Library<Validated> {
    Library::from_rules(vec![
        rule("R:g", Home::global()),
        rule("R:d", Home::domain("rust").expect("non-empty domain")),
        rule("R:p", Home::project("relearn").expect("non-empty project")),
    ])
    .validate()
    .expect("distinct tags validate")
}

fn slug_of(rule: &Rule) -> String {
    emit::HomeSlug::of(rule.home()).as_str().to_owned()
}

#[test]
fn restricting_keeps_only_the_named_home() {
    let lib = mixed();
    let only_rust = lib.filter(|r| slug_of(r) == "domain-rust");
    assert_eq!(only_rust.len(), 1);
    assert_eq!(only_rust.rules()[0].tag().as_str(), "R:d");
}

/// The point of the filter, stated as the thing that must not happen: emitting
/// the rules layer for the domain home must not carry a project layer with it.
#[test]
fn the_rules_layer_for_one_domain_emits_exactly_one_file() {
    let lib = mixed();
    let files = emit::claude_rules::emit(&lib.filter(|r| slug_of(r) == "domain-rust"));
    assert_eq!(files.len(), 1, "only the domain layer");
    assert_eq!(files[0].path().as_str(), ".claude/rules/domain-rust.md");
    assert!(files[0].contents().contains("paths:"), "still scoped");
}

/// Unfiltered, the same library emits the project layer too — so the test above
/// is measuring the filter, not an accident of the fixture.
#[test]
fn without_the_filter_the_project_layer_is_emitted_as_well() {
    let paths: Vec<String> = emit::claude_rules::emit(&mixed())
        .iter()
        .map(|f| f.path().as_str().to_owned())
        .collect();
    assert_eq!(paths.len(), 2);
    assert!(paths.contains(&".claude/rules/domain-rust.md".to_owned()));
    assert!(paths.contains(&".claude/rules/project-relearn.md".to_owned()));
}

/// A filter matching nothing yields an empty library. The CLI turns this into a
/// loud error rather than emitting nothing: a silent empty emission would make
/// `verify` pass trivially, which is the "gate that reports success while doing
/// nothing" failure this repository exists to catch.
#[test]
fn a_filter_matching_nothing_yields_an_empty_library() {
    let lib = mixed();
    let none = lib.filter(|r| slug_of(r) == "domain-cobol");
    assert!(none.is_empty());
    assert!(emit::claude_rules::emit(&none).is_empty());
}

/// Filtering a validated library needs no re-validation: tag uniqueness over a
/// subset of a set with unique tags still holds, so the witness carries over.
/// This pins that the filtered value really is usable as `Library<Validated>`
/// by every emitter, which is what makes the no-revalidation claim safe.
#[test]
fn a_filtered_library_is_still_a_validated_library_for_every_emitter() {
    let lib = mixed().filter(|r| slug_of(r) != "project-relearn");
    assert_eq!(lib.len(), 2);
    assert!(!emit::claude::emit(&lib).is_empty());
    assert!(!emit::cursor::emit(&lib).is_empty());
    assert!(!emit::copilot::emit(&lib).is_empty());
    assert!(!emit::agents::emit(&lib).is_empty());
    assert!(!emit::claude_rules::emit(&lib).is_empty());
}
