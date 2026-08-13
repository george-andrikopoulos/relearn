//! `emit::claude_md` — emits `<out>/CLAUDE.md`, project-layer rules only
//! (`Home::Project`), so a project's `CLAUDE.md` never absorbs global or
//! domain rules that belong in their own home.
//!
//! When the library carries no project-layer rules the emitter returns no file
//! at all, rather than an empty `CLAUDE.md` that would clobber a hand-authored
//! one.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.

use super::{OutputFile, RelativePath};
use crate::library::{Library, Validated};
use crate::rule::{Home, Rule, RuleTag};

/// Emit the project-layer `CLAUDE.md`, containing only `Home::Project` rules,
/// ordered by tag so the output is a deterministic function of the library.
///
/// Returns an empty `Vec` (no file) when there are no project-layer rules, so a
/// real hand-authored `CLAUDE.md` is never overwritten with an empty one.
///
/// Accepts only `&Library<Validated>` — an unvalidated library does not have
/// this type, so emitting unchecked rules is a compile error.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    let mut project_rules: Vec<&Rule> = library
        .rules()
        .iter()
        .filter(|r| matches!(r.home(), Home::Project { .. }))
        .collect();
    if project_rules.is_empty() {
        return Vec::new();
    }
    project_rules.sort_by(|a, b| a.tag().as_str().cmp(b.tag().as_str()));

    let contents = render_claude_md(&project_rules);
    let path = RelativePath::from_segments(&["CLAUDE.md"]);
    let sources: Vec<RuleTag> = project_rules.iter().map(|r| r.tag().clone()).collect(); // allow:clone: the OutputFile owns its provenance, outliving the &Library borrow
    vec![OutputFile::new(path, contents, sources)]
}

/// Render the `CLAUDE.md` body: a heading then one markdown section per rule,
/// in the tag order the caller sorted them into.
fn render_claude_md(rules: &[&Rule]) -> String {
    let mut out = String::new();
    out.push_str("# Project rules\n");
    for r in rules {
        out.push_str(&format!(
            "\n## {} [{}]\n\n{}\n",
            r.title().as_str(),
            r.tag().as_str(),
            r.body().as_str()
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn only_project_rules_are_included() {
        let lib = validated(vec![
            rule("R:g", Home::global(), "Global rule", "gc", "Global body."),
            rule(
                "R:d",
                Home::domain("rust").expect("non-empty domain"),
                "Domain rule",
                "dc",
                "Domain body.",
            ),
            rule(
                "R:p",
                Home::project("C:/repo").expect("non-empty project"),
                "Project rule",
                "pc",
                "Project body.",
            ),
        ]);
        let files = emit(&lib);
        assert_eq!(files.len(), 1, "one project CLAUDE.md");
        let c = files[0].contents();
        assert!(c.contains("[R:p]"), "project rule is present");
        assert!(!c.contains("[R:g]"), "global rule is excluded");
        assert!(!c.contains("[R:d]"), "domain rule is excluded");
    }

    #[test]
    fn file_is_at_claude_md_path() {
        let lib = validated(vec![rule(
            "R:p",
            Home::project("C:/repo").expect("non-empty project"),
            "Project rule",
            "pc",
            "Project body.",
        )]);
        let files = emit(&lib);
        assert_eq!(files.len(), 1, "one project CLAUDE.md");
        assert_eq!(files[0].path().as_str(), "CLAUDE.md");
    }

    #[test]
    fn no_project_rules_emits_nothing() {
        let lib = validated(vec![
            rule("R:g", Home::global(), "Global rule", "gc", "Global body."),
            rule(
                "R:d",
                Home::domain("rust").expect("non-empty domain"),
                "Domain rule",
                "dc",
                "Domain body.",
            ),
        ]);
        let files = emit(&lib);
        assert!(files.is_empty(), "no project rules means no file");
    }

    #[test]
    fn emission_is_deterministic() {
        let lib = validated(vec![
            rule(
                "R:p2",
                Home::project("C:/repo").expect("non-empty project"),
                "Project two",
                "pc2",
                "Project two body.",
            ),
            rule(
                "R:p1",
                Home::project("C:/repo").expect("non-empty project"),
                "Project one",
                "pc1",
                "Project one body.",
            ),
        ]);
        assert_eq!(emit(&lib), emit(&lib));
    }
}
