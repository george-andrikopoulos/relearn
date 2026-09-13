//! End-to-end tests for audience scoping — `applies_to` on a rule, and the
//! `--scope` narrowing behind `build --scope` / `verify --scope` / `list
//! --scope`.
//!
//! The four-row table these assert is the load-bearing part of the design:
//!
//! | build invocation           | rule with no `applies_to` | rule scoped `["rust"]` |
//! |----------------------------|---------------------------|------------------------|
//! | no `--scope`               | emitted                   | **emitted**            |
//! | `--scope rust`             | emitted                   | emitted                |
//! | `--scope java`             | emitted                   | **not emitted**        |
//! | `--scope rust --scope java`| emitted                   | emitted                |
//!
//! **An unscoped rule is emitted under every invocation.** That is the safety
//! decision, not a convenience: adding `applies_to` to one rule must never be
//! able to remove a *different* rule from an existing build, because a silently
//! dropped rule is a lost correction — the invariant at the top of `CLAUDE.md`.
//! `--scope` narrows scoped rules only, and with no `--scope` nothing narrows.
//!
//! Scope never touches the emitted path. `Home` alone decides where a rule
//! lands; scope decides only whether it is included. Nothing here may produce a
//! `rust-low-latency.md`, and `scope_never_changes_the_emitted_path` is the pin.

use relearn::emit;
use relearn::library::{Library, Validated};
use relearn::rule::{
    Authority, Body, Date, ErrorClass, Home, Incident, Origin, Rule, RuleTag, ScopeTag, Status,
    Title,
};

fn scopes(names: &[&str]) -> Vec<ScopeTag> {
    names
        .iter()
        .map(|n| ScopeTag::parse(*n).expect("valid scope"))
        .collect()
}

fn rule(tag: &str, home: Home, applies_to: &[&str]) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("Title").expect("non-empty title"),
        ErrorClass::parse("ec").expect("non-empty error class"),
        home,
        Date::parse("2026-09-13").expect("valid date"),
        Origin::Mined,
        Status::active(),
        Incident::parse("an incident").expect("non-empty incident"),
        Body::parse("The body.").expect("non-empty body"),
        Vec::new(),
        scopes(applies_to),
        Authority::Local,
    )
}

/// One unscoped rule and one scoped to `rust` — the two shapes the table
/// distinguishes, in one library so a filter cannot pass by emptiness.
fn mixed() -> Library<Validated> {
    Library::from_rules(vec![
        rule("R:unscoped", Home::global(), &[]),
        rule(
            "R:scoped",
            Home::domain("low-latency").expect("non-empty domain"),
            &["rust"],
        ),
    ])
    .validate()
    .expect("distinct tags validate")
}

fn served_tags(lib: &Library<Validated>, audience: &[&str]) -> Vec<String> {
    let audience = scopes(audience);
    lib.rules()
        .iter()
        .filter(|r| r.serves(&audience))
        .map(|r| r.tag().as_str().to_owned())
        .collect()
}

// ── the four rows ───────────────────────────────────────────────────────────

#[test]
fn with_no_scope_every_rule_is_served_including_the_scoped_one() {
    assert_eq!(served_tags(&mixed(), &[]), vec!["R:unscoped", "R:scoped"]);
}

#[test]
fn a_matching_scope_serves_both_the_scoped_and_the_unscoped_rule() {
    assert_eq!(
        served_tags(&mixed(), &["rust"]),
        vec!["R:unscoped", "R:scoped"]
    );
}

/// The only row that withholds anything — and it withholds the scoped rule
/// only. The unscoped rule survives a scope it does not mention, which is the
/// whole safety argument.
#[test]
fn a_non_matching_scope_withholds_the_scoped_rule_and_keeps_the_unscoped_one() {
    assert_eq!(served_tags(&mixed(), &["java"]), vec!["R:unscoped"]);
}

#[test]
fn several_scopes_serve_a_rule_matching_any_one_of_them() {
    assert_eq!(
        served_tags(&mixed(), &["rust", "java"]),
        vec!["R:unscoped", "R:scoped"]
    );
}

// ── scope is not a home ─────────────────────────────────────────────────────

/// Scope decides inclusion; `Home` alone decides the path. A rule homed in
/// `domain-low-latency` and scoped to `rust` emits at the low-latency path,
/// never at a fabricated `rust-low-latency` one.
#[test]
fn scope_never_changes_the_emitted_path() {
    let scoped_only = Library::from_rules(vec![rule(
        "R:scoped",
        Home::domain("low-latency").expect("non-empty domain"),
        &["rust", "java"],
    )])
    .validate()
    .expect("one tag validates");

    // The Claude **skill** emitter, not the rules layer: `claude_rules` emits
    // only domains with known file globs, and `low-latency` has none, so it
    // would report an empty set here for a reason that has nothing to do with
    // scope. (That gap is real and is carried in TODO.md — a rule homed in the
    // motivating domain reaches four of the five targets.)
    let paths: Vec<String> = emit::claude::emit(&scoped_only)
        .iter()
        .map(|f| f.path().as_str().to_owned())
        .collect();
    assert_eq!(
        paths,
        vec!["skills/domain-low-latency/SKILL.md".to_owned()],
        "the path follows `home` alone — a scope must never fabricate a path"
    );
}

/// The same rule, homed identically but unscoped, emits byte-identically:
/// `applies_to` changes *whether* a rule is included, never *what* is written.
#[test]
fn scoping_a_rule_does_not_change_a_single_emitted_byte() {
    let home = || Home::domain("low-latency").expect("non-empty domain");
    let with = Library::from_rules(vec![rule("R:x", home(), &["rust", "java"])])
        .validate()
        .expect("one tag validates");
    let without = Library::from_rules(vec![rule("R:x", home(), &[])])
        .validate()
        .expect("one tag validates");

    for (a, b) in emit::claude::emit(&with)
        .iter()
        .zip(emit::claude::emit(&without).iter())
    {
        assert_eq!(a.path().as_str(), b.path().as_str());
        assert_eq!(a.contents(), b.contents());
    }
}

// ── the type ────────────────────────────────────────────────────────────────

#[test]
fn a_rule_is_scoped_only_when_it_declares_an_audience() {
    assert!(!rule("R:a", Home::global(), &[]).is_scoped());
    assert!(rule("R:b", Home::global(), &["rust"]).is_scoped());
}

/// A filtered library is still a `Library<Validated>` for every emitter — the
/// same claim `tests/home_filter.rs` makes for the home filter, because the two
/// narrowings compose and either could be the one that broke it.
#[test]
fn a_scope_filtered_library_is_still_validated_for_every_emitter() {
    let audience = scopes(&["rust"]);
    let lib = mixed().filter(|r| r.serves(&audience));
    assert_eq!(lib.len(), 2);
    assert!(!emit::claude::emit(&lib).is_empty());
    assert!(!emit::cursor::emit(&lib).is_empty());
    assert!(!emit::copilot::emit(&lib).is_empty());
    assert!(!emit::agents::emit(&lib).is_empty());
    // `claude_rules` is deliberately absent: it emits project homes and known
    // language domains only, so this fixture's `global` + `low-latency` rules
    // reach it for no scope-related reason. `tests/home_filter.rs` covers that
    // emitter with a fixture built for it.
}

/// `--home` and `--scope` compose: each narrows independently, and neither
/// reaches into the other's question.
#[test]
fn home_and_scope_narrow_independently() {
    let audience = scopes(&["java"]);
    let lib = mixed()
        .filter(|r| emit::HomeSlug::of(r.home()).as_str() == "global")
        .filter(|r| r.serves(&audience));
    assert_eq!(lib.len(), 1);
    assert_eq!(lib.rules()[0].tag().as_str(), "R:unscoped");
}
