//! `emit::cursor` — one Cursor rule file per rule: `<out>/.cursor/rules/<tag-
//! body>.mdc` with front-matter (`description`, `globs`, `alwaysApply`). The
//! `globs`/`alwaysApply` are **derived from the rule's `Home`** via the shared
//! [`Scope`](super::Scope) helper, never carried on the rule (decision
//! 2026-08-13) — vendor scope vocabulary is the emitter's job.
//!
//! The filename is the tag **body** (`R:foo` → `foo.mdc`): the full tag's `:`
//! is not a valid filename character on every platform, and the body is already
//! `[a-z0-9][a-z0-9-]*` and unique (every tag shares the `R:` prefix).
//!
//! Cursor parses `.mdc` front-matter leniently (line-based, and `globs` holds a
//! raw comma-separated glob list that is not valid strict YAML), so unlike the
//! Claude skill emitter this one does **not** YAML-quote its values.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.

use super::{OutputFile, RelativePath, Scope, emittable, graduation_note};
use crate::library::{Library, Validated};
use crate::rule::Rule;

/// Emit one `.mdc` per rule, ordered by tag so the output is a deterministic
/// function of the library.
///
/// Accepts only `&Library<Validated>` — an unvalidated library does not have
/// this type, so emitting unchecked rules is a compile error.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    // Only emittable rules (active + graduated); atticked guidance is suppressed.
    let mut rules: Vec<&Rule> = emittable(library);
    rules.sort_by(|a, b| a.tag().as_str().cmp(b.tag().as_str()));
    rules.into_iter().map(render_rule).collect()
}

/// Render one rule into its `.mdc` output file.
fn render_rule(rule: &Rule) -> OutputFile {
    let scope = Scope::for_home(rule.home());
    let filename = format!("{}.mdc", rule.tag().body());
    let path = RelativePath::from_segments(&[".cursor", "rules", filename.as_str()]);
    // The OutputFile owns its provenance tag, outliving the `&Library` borrow.
    let sources = vec![rule.tag().clone()]; // allow:clone: provenance for the returned OutputFile
    OutputFile::new(path, render_mdc(rule, &scope), sources)
}

/// Render the `.mdc` body: lenient front-matter (Cursor's own format) then the
/// rule as a markdown section.
fn render_mdc(rule: &Rule, scope: &Scope) -> String {
    // Description is a single line; strip any embedded newline defensively so it
    // cannot break the line-based front-matter.
    let description = format!(
        "{} ({})",
        rule.title().as_str(),
        rule.error_class().as_str()
    )
    .replace('\n', " ");
    let globs = scope.globs().join(",");

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("description: {description}\n"));
    out.push_str(&format!("globs: {globs}\n"));
    out.push_str(&format!("alwaysApply: {}\n", scope.always_apply()));
    out.push_str("---\n\n");
    out.push_str(&format!(
        "# {} [{}]\n\n",
        rule.title().as_str(),
        rule.tag().as_str()
    ));
    if let Some(note) = graduation_note(rule) {
        out.push_str(&note);
    }
    out.push_str(rule.body().as_str());
    out.push('\n');
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
    fn one_mdc_per_rule_named_by_tag_body() {
        let lib = validated(vec![
            rule("R:beta", Home::global(), "Beta", "bc", "Beta body."),
            rule("R:alpha", Home::global(), "Alpha", "ac", "Alpha body."),
        ]);
        let files = emit(&lib);
        assert_eq!(files.len(), 2);
        // Ordered by tag; filename is the body (no colon).
        assert_eq!(files[0].path().as_str(), ".cursor/rules/alpha.mdc");
        assert_eq!(files[1].path().as_str(), ".cursor/rules/beta.mdc");
    }

    #[test]
    fn rust_domain_rule_auto_attaches_on_rs_globs() {
        let lib = validated(vec![rule(
            "R:x",
            Home::domain("rust").expect("non-empty domain"),
            "Rust rule",
            "rc",
            "Body.",
        )]);
        let files = emit(&lib);
        let c = files[0].contents();
        assert!(c.contains("globs: **/*.rs"));
        assert!(c.contains("alwaysApply: false"));
        assert!(c.contains("description: Rust rule (rc)"));
        assert!(c.contains("[R:x]"));
    }

    #[test]
    fn global_rule_always_applies_with_no_globs() {
        let lib = validated(vec![rule(
            "R:g",
            Home::global(),
            "Global rule",
            "gc",
            "Body.",
        )]);
        let files = emit(&lib);
        let c = files[0].contents();
        assert!(c.contains("alwaysApply: true"));
        assert!(
            c.contains("globs: \n"),
            "empty globs line for a global rule"
        );
    }

    #[test]
    fn front_matter_is_delimited_and_ordered() {
        let lib = validated(vec![rule(
            "R:g",
            Home::global(),
            "Global rule",
            "gc",
            "Body.",
        )]);
        let files = emit(&lib);
        let c = files[0].contents();
        assert!(c.starts_with("---\ndescription: "));
        let front_matter_end = c.find("\n---\n\n").expect("front-matter is closed");
        assert!(c[..front_matter_end].contains("\nglobs: "));
        assert!(c[..front_matter_end].contains("\nalwaysApply: "));
    }

    #[test]
    fn emission_is_deterministic() {
        let lib = validated(vec![
            rule("R:a", Home::global(), "A", "ac", "A body."),
            rule(
                "R:b",
                Home::domain("rust").expect("non-empty domain"),
                "B",
                "bc",
                "B body.",
            ),
        ]);
        assert_eq!(emit(&lib), emit(&lib));
    }
}
