//! `emit::copilot` — emits a single `<out>/.github/copilot-instructions.md`
//! holding **every** rule concatenated, ordered by home rank (global → domain →
//! project), then home slug, then tag — so the broadest rules lead, same-home
//! rules stay grouped, and the output is a deterministic function of the
//! library. Unlike Cursor (one file per rule) and Claude (one skill per home),
//! Copilot reads a single plain-markdown instructions file — no front-matter,
//! just prose sections it treats as guidance.
//!
//! A library with zero rules emits **no** file (an empty instructions file
//! would be noise), so the returned `Vec` is empty in that case.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.

use super::{HomeSlug, OutputFile, RelativePath, home_rank};
use crate::library::{Library, Validated};
use crate::rule::{Rule, RuleTag};

/// Emit the single Copilot instructions file. Rules are ordered by home rank
/// (global → domain → project), then home slug, then tag, so the output is a
/// deterministic function of the library — the property `emit(lib) == emit(lib)`
/// holds, which the overwrite guard relies on.
///
/// Accepts only `&Library<Validated>` — an unvalidated library does not have
/// this type, so emitting unchecked rules is a compile error, not a runtime
/// guard.
///
/// A library with no rules produces no file (an empty `Vec`), never an empty
/// instructions file.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    if library.rules().is_empty() {
        return Vec::new();
    }

    // `&Rule`/`&Home` borrow the library for this call; nothing is cloned.
    let mut rules: Vec<&Rule> = library.rules().iter().collect();
    rules.sort_by(|a, b| {
        let (ha, hb) = (HomeSlug::of(a.home()), HomeSlug::of(b.home()));
        home_rank(a.home())
            .cmp(&home_rank(b.home()))
            .then_with(|| ha.as_str().cmp(hb.as_str()))
            .then_with(|| a.tag().as_str().cmp(b.tag().as_str()))
    });

    let path = RelativePath::from_segments(&[".github", "copilot-instructions.md"]);
    let contents = render(&rules);
    // The OutputFile outlives the `&Library` borrow, so it owns its provenance
    // tags. In sorted (home, tag) order, matching the sections above.
    let sources: Vec<RuleTag> = rules.iter().map(|r| r.tag().clone()).collect(); // allow:clone: the OutputFile owns its provenance, outliving the &Library borrow
    vec![OutputFile::new(path, contents, sources)]
}

/// Render all rules into one plain-markdown document: a top heading, then one
/// section per rule (`## {title} [{tag}]` then the body).
fn render(rules: &[&Rule]) -> String {
    let mut out = String::new();
    out.push_str("# Copilot instructions\n");
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

    #[test]
    fn single_file_at_the_github_path() {
        let lib = validated(vec![rule(
            "R:g",
            Home::global(),
            "Global rule",
            "gc",
            "Global body.",
        )]);
        let files = emit(&lib);
        assert_eq!(files.len(), 1, "exactly one concatenated file");
        assert_eq!(files[0].path().as_str(), ".github/copilot-instructions.md");
    }

    #[test]
    fn all_rules_present_ordered_by_home_then_tag() {
        let lib = validated(vec![
            rule(
                "R:r",
                Home::domain("rust").expect("non-empty domain"),
                "Rust rule",
                "rc",
                "Rust body.",
            ),
            rule("R:g", Home::global(), "Global rule", "gc", "Global body."),
        ]);
        let files = emit(&lib);
        assert_eq!(files.len(), 1, "one file holds every rule");
        let c = files[0].contents();
        let ig = c.find("[R:g]").expect("global section present");
        let ir = c.find("[R:r]").expect("rust section present");
        // home rank orders global (0) before domain (1), so the global rule leads.
        assert!(ig < ir, "global rule appears before the rust-domain rule");
        assert!(c.contains("Global body."));
        assert!(c.contains("Rust body."));
    }

    #[test]
    fn empty_library_emits_nothing() {
        let lib = Library::new().validate().expect("empty library validates");
        let files = emit(&lib);
        assert!(files.is_empty(), "a library with no rules produces no file");
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
