//! `emit::agents` — a single `<out>/AGENTS.md` at the repository root holding
//! **all** rules concatenated (decision 2026-08-13), not one file per rule and
//! not one per home. `AGENTS.md` is the cross-assistant convention: one plain
//! markdown file, no front-matter, read whole. Rules are ordered by home rank
//! (global → domain → project), then home slug, then tag, so the output is a
//! deterministic function of the library.
//!
//! An empty library emits no file — an `AGENTS.md` with a header and no rules
//! would be a misleading artifact, so absence is represented by absence.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.

use super::{
    HomeSlug, OutputFile, RelativePath, emittable, graduation_note, home_rank, recurrence_note,
};
use crate::library::{Library, Validated};
use crate::rule::{Rule, RuleTag};

/// Emit the single `AGENTS.md`, or nothing when the library is empty.
///
/// Rules are ordered by home rank (global → domain → project), then home slug,
/// then tag, so the output — both the file contents and its `sources`
/// provenance — is a deterministic function of the library; the property
/// `emit(lib) == emit(lib)` holds.
///
/// Accepts only `&Library<Validated>`: an unvalidated library does not have
/// this type, so emitting unchecked rules is a compile error, not a runtime
/// guard.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    // Only emittable rules (active + graduated); atticked guidance is suppressed.
    // `&Rule` borrows the library for this call; nothing is cloned to sort.
    let mut rules: Vec<&Rule> = emittable(library);
    if rules.is_empty() {
        return Vec::new();
    }
    rules.sort_by(|a, b| {
        let (ha, hb) = (HomeSlug::of(a.home()), HomeSlug::of(b.home()));
        home_rank(a.home())
            .cmp(&home_rank(b.home()))
            .then_with(|| ha.as_str().cmp(hb.as_str()))
            .then_with(|| a.tag().as_str().cmp(b.tag().as_str()))
    });

    let contents = render(&rules);
    let path = RelativePath::from_segments(&["AGENTS.md"]);
    // The OutputFile outlives the `&Library` borrow, so it owns its provenance
    // tags. In the same sorted order as the sections above, so the header's
    // recorded provenance is deterministic.
    let sources: Vec<RuleTag> = rules.iter().map(|r| r.tag().clone()).collect(); // allow:clone: the OutputFile owns its provenance, outliving the &Library borrow
    vec![OutputFile::new(path, contents, sources)]
}

/// Render the whole file: a top heading then a markdown section per rule, in
/// the caller's already-sorted order.
fn render(rules: &[&Rule]) -> String {
    let mut out = String::new();
    out.push_str("# Agent instructions\n");
    for r in rules {
        out.push_str(&format!(
            "\n## {} [{}]\n\n",
            r.title().as_str(),
            r.tag().as_str()
        ));
        if let Some(note) = graduation_note(r) {
            out.push_str(&note);
        }
        if let Some(note) = recurrence_note(r) {
            out.push_str(&note);
        }
        out.push_str(&format!("{}\n", r.body().as_str()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::Library;
    use crate::rule::{
        Authority, Body, Date, ErrorClass, Home, Incident, Origin, RuleTag, Status, Title,
    };

    fn rule(tag: &str, home: Home, title: &str, error_class: &str, body: &str) -> Rule {
        Rule::new(
            RuleTag::parse(tag).expect("valid tag"),
            Title::parse(title).expect("non-empty title"),
            ErrorClass::parse(error_class).expect("non-empty error class"),
            home,
            Date::parse("2026-08-13").expect("valid date"),
            Origin::Mined,
            Status::active(),
            Incident::parse("an incident").expect("non-empty incident"),
            Body::parse(body).expect("non-empty body"),
            Vec::new(),
            Vec::new(),
            Authority::Local,
            None,
        )
    }

    fn validated(rules: Vec<Rule>) -> Library<Validated> {
        Library::from_rules(rules)
            .validate()
            .expect("distinct tags validate")
    }

    #[test]
    fn single_file_at_the_repo_root() {
        let lib = validated(vec![rule(
            "R:g",
            Home::global(),
            "Global rule",
            "gc",
            "Global body.",
        )]);
        let files = emit(&lib);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path().as_str(), "AGENTS.md");
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
        let c = files[0].contents();
        let ig = c.find("[R:g]").expect("global heading present");
        let ir = c.find("[R:r]").expect("rust heading present");
        // Ordered by home rank: global (0) precedes domain (1), so the global
        // section comes before the rust-domain one.
        assert!(
            ig < ir,
            "global rank precedes domain rank, so [R:g] precedes [R:r]"
        );
    }

    #[test]
    fn empty_library_emits_nothing() {
        let lib = validated(vec![]);
        let files = emit(&lib);
        assert!(files.is_empty());
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
