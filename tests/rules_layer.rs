//! The rules layer is written for **one** destination at a time: a project's own
//! tree, or the user's configuration. Pinned because both ways of getting this
//! wrong shipped: relearn's own `.claude/rules/` carried two other projects'
//! homes, resident in every relearn session, and the rust domain layer loaded
//! from the user directory and the repository at once (2026-10-31).

use proptest::prelude::*;
use relearn::emit::claude_rules::{self, RulesLayer};
use relearn::library::{Library, Validated};
use relearn::rule::{
    Authority, Body, Date, ErrorClass, Home, Incident, Origin, Rule, RuleTag, Status, Title,
};

fn rule(tag: &str, home: Home) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("Title").expect("non-empty title"),
        ErrorClass::parse("ec").expect("non-empty error class"),
        home,
        Date::parse("2026-10-31").expect("valid date"),
        Origin::Mined,
        Status::active(),
        Incident::parse("an incident").expect("non-empty incident"),
        Body::parse("The body.").expect("non-empty body"),
        Vec::new(),
        Vec::new(),
        Authority::local(),
        None,
        Vec::new(),
    )
}

fn library() -> Library<Validated> {
    Library::from_rules(vec![
        rule("R:g", Home::global()),
        rule("R:d", Home::domain("rust").expect("non-empty domain")),
        rule("R:a", Home::project("alpha").expect("non-empty project")),
        rule("R:b", Home::project("beta").expect("non-empty project")),
    ])
    .validate()
    .expect("distinct tags validate")
}

fn paths(files: &[relearn::emit::OutputFile]) -> Vec<&str> {
    files.iter().map(|f| f.path().as_str()).collect()
}

#[test]
fn building_project_a_emits_no_file_for_project_b() {
    let layer = RulesLayer::project("alpha").expect("non-empty name");
    let files = claude_rules::emit(&library(), &layer);
    assert_eq!(paths(&files), [".claude/rules/project-alpha.md"]);
}

#[test]
fn a_project_layer_carries_no_domain_home() {
    let layer = RulesLayer::project("beta").expect("non-empty name");
    let files = claude_rules::emit(&library(), &layer);
    assert!(!paths(&files).iter().any(|p| p.contains("domain-")));
}

#[test]
fn the_user_layer_carries_domain_homes_and_nothing_else() {
    let files = claude_rules::emit(&library(), &RulesLayer::User);
    assert_eq!(paths(&files), [".claude/rules/domain-rust.md"]);
}

#[test]
fn a_project_with_no_rules_emits_nothing() {
    let layer = RulesLayer::project("gamma").expect("non-empty name");
    assert!(claude_rules::emit(&library(), &layer).is_empty());
}

#[test]
fn an_empty_project_name_is_not_a_layer() {
    assert!(RulesLayer::project("").is_err());
}

#[test]
fn a_project_name_is_matched_by_slug_as_the_home_spells_it() {
    // `Home::project("alpha")` and a destination named `Alpha` are one project:
    // the file name is the slug, so the match is too.
    let layer = RulesLayer::project("Alpha").expect("non-empty name");
    assert_eq!(
        paths(&claude_rules::emit(&library(), &layer)),
        [".claude/rules/project-alpha.md"]
    );
}

proptest! {
    /// Whatever projects a library holds, a project layer emits at most one
    /// file, and it is named for that project.
    #[test]
    fn a_project_layer_is_at_most_its_own_file(
        homes in prop::collection::vec("[a-d]", 1..8),
        target in "[a-e]",
    ) {
        let rules: Vec<Rule> = homes
            .iter()
            .enumerate()
            .map(|(i, h)| rule(&format!("R:r{i}"), Home::project(h.as_str()).expect("non-empty")))
            .collect();
        let lib = Library::from_rules(rules).validate().expect("distinct tags");
        let layer = RulesLayer::project(&target).expect("non-empty name");
        let files = claude_rules::emit(&lib, &layer);
        prop_assert!(files.len() <= 1);
        for f in &files {
            prop_assert_eq!(f.path().as_str(), format!(".claude/rules/project-{target}.md"));
        }
        prop_assert_eq!(files.is_empty(), !homes.contains(&target));
    }
}
