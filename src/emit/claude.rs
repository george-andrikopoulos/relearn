//! `emit::claude` — one Claude skill **per home layer** (decision 2026-08-13),
//! not one per rule. Each home produces `skills/<home-slug>/SKILL.md` with YAML
//! front-matter (`name`, `description`); the description aggregates the home's
//! rule titles and error classes so the skill's trigger coverage matches the
//! rules it carries. Per-home keeps the always-resident skill metadata bounded
//! (P6), rather than growing it `O(rules)`.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.

use std::collections::BTreeMap;

use super::{HomeSlug, OutputFile, RelativePath};
use crate::library::{Library, Validated};
use crate::rule::{Home, Rule};

/// Emit one skill per home layer. Homes are ordered by slug and rules within a
/// home by tag, so the output is a deterministic function of the library — the
/// property `emit(lib) == emit(lib)` holds, which the overwrite guard relies on.
///
/// Accepts only `&Library<Validated>`: an unvalidated library does not have
/// this type, so emitting unchecked rules is a compile error, not a runtime
/// guard.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    // Group by home slug, keeping a reference to the home for its human label.
    // `&Rule`/`&Home` borrow the library for this call; nothing is cloned.
    let mut by_home: BTreeMap<HomeSlug, (&Home, Vec<&Rule>)> = BTreeMap::new();
    for rule in library.rules() {
        let slug = HomeSlug::of(rule.home());
        by_home
            .entry(slug)
            .or_insert_with(|| (rule.home(), Vec::new()))
            .1
            .push(rule);
    }

    let mut files = Vec::with_capacity(by_home.len());
    for (slug, (home, mut rules)) in by_home {
        rules.sort_by(|a, b| a.tag().as_str().cmp(b.tag().as_str()));
        let contents = render_skill(&slug, home, &rules);
        let path = RelativePath::from_segments(&["skills", slug.as_str(), "SKILL.md"]);
        files.push(OutputFile::new(path, contents));
    }
    files
}

/// Render one skill file: YAML front-matter then a markdown section per rule.
fn render_skill(slug: &HomeSlug, home: &Home, rules: &[&Rule]) -> String {
    let label = home_label(home);
    let covers = rules
        .iter()
        .map(|r| format!("{} ({})", r.title().as_str(), r.error_class().as_str()))
        .collect::<Vec<_>>()
        .join("; ");
    let description = format!("Rules for {label}. Covers: {covers}");

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("name: {}\n", slug.as_str()));
    out.push_str(&format!(
        "description: {}\n",
        yaml_double_quote(&description)
    ));
    out.push_str("---\n\n");
    out.push_str(&format!("# {label} rules\n"));
    for r in rules {
        out.push_str(&format!(
            "\n## {} [{}]\n\n",
            r.title().as_str(),
            r.tag().as_str()
        ));
        out.push_str(r.body().as_str());
        out.push('\n');
    }
    out
}

/// The human-readable label for a home, used in headings and the description.
fn home_label(home: &Home) -> String {
    match home {
        Home::Global => "global".to_owned(),
        Home::Domain { name } => format!("domain: {}", name.as_str()),
        Home::Project { path } => format!("project: {}", path.as_str()),
    }
}

/// Quote `s` as a YAML double-quoted scalar, escaping the characters that would
/// otherwise break it. Rule titles and error classes are free text and may
/// contain `:` or `"`, so the description value must be quoted to stay valid
/// YAML front-matter.
fn yaml_double_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::Library;
    use crate::rule::{Body, Date, ErrorClass, Home, Incident, RuleTag, Status, Title};

    fn rule(tag: &str, home: Home, title: &str, error_class: &str, body: &str) -> Rule {
        Rule::new(
            RuleTag::parse(tag).expect("valid tag"),
            Title::parse(title).expect("non-empty title"),
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

    fn mixed_library() -> Library<Validated> {
        validated(vec![
            rule("R:g", Home::global(), "Global rule", "gc", "Global body."),
            rule(
                "R:r2",
                Home::domain("rust").expect("non-empty domain"),
                "Rust two",
                "rc2",
                "Rust two body.",
            ),
            rule(
                "R:r1",
                Home::domain("rust").expect("non-empty domain"),
                "Rust one",
                "rc1",
                "Rust one body.",
            ),
            rule(
                "R:p",
                Home::project("C:/repo").expect("non-empty project"),
                "Project rule",
                "pc",
                "Project body.",
            ),
        ])
    }

    #[test]
    fn one_skill_per_home_layer() {
        let files = emit(&mixed_library());
        assert_eq!(files.len(), 3, "global, domain-rust, project-c-repo");
        let paths: Vec<&str> = files.iter().map(|f| f.path().as_str()).collect();
        assert!(paths.contains(&"skills/global/SKILL.md"));
        assert!(paths.contains(&"skills/domain-rust/SKILL.md"));
        assert!(paths.contains(&"skills/project-c-repo/SKILL.md"));
    }

    #[test]
    fn home_skill_aggregates_its_rules_sorted_by_tag() {
        let files = emit(&mixed_library());
        let rust = files
            .iter()
            .find(|f| f.path().as_str() == "skills/domain-rust/SKILL.md")
            .expect("the rust skill is present");
        let c = rust.contents();
        let i1 = c.find("[R:r1]").expect("R:r1 heading present");
        let i2 = c.find("[R:r2]").expect("R:r2 heading present");
        assert!(i1 < i2, "rules are ordered by tag within a home");
        assert!(c.contains("Rust one body."));
        assert!(c.contains("Rust two body."));
    }

    #[test]
    fn front_matter_names_the_home_slug() {
        let files = emit(&mixed_library());
        let rust = files
            .iter()
            .find(|f| f.path().as_str() == "skills/domain-rust/SKILL.md")
            .expect("the rust skill is present");
        assert!(
            rust.contents()
                .starts_with("---\nname: domain-rust\ndescription: ")
        );
    }

    #[test]
    fn description_covers_each_rule_in_the_home() {
        let files = emit(&mixed_library());
        let rust = files
            .iter()
            .find(|f| f.path().as_str() == "skills/domain-rust/SKILL.md")
            .expect("the rust skill is present");
        let c = rust.contents();
        assert!(c.contains("Rust one (rc1)"));
        assert!(c.contains("Rust two (rc2)"));
    }

    #[test]
    fn description_is_quoted_so_a_colon_in_a_title_stays_valid_yaml() {
        let lib = validated(vec![rule(
            "R:x",
            Home::global(),
            "Title: with colon",
            "err",
            "b",
        )]);
        let files = emit(&lib);
        let c = files[0].contents();
        assert!(
            c.contains("description: \"Rules for global. Covers: Title: with colon (err)\""),
            "the description value is double-quoted, so an inner colon is safe: {c}"
        );
    }

    #[test]
    fn emission_is_deterministic() {
        let lib = mixed_library();
        assert_eq!(emit(&lib), emit(&lib));
    }
}
