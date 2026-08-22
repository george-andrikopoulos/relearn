//! `emit::claude_md` — the Claude **project layer**: one file per project home
//! (`Home::Project`), at `<out>/.claude/rules/<home-slug>.md`.
//!
//! **Why not `<out>/CLAUDE.md`** (the original path; changed 2026-08-22). A
//! repository-root `CLAUDE.md` is the *hand-authored* project charter by Claude
//! Code convention, so emitting a generated file there put two owners on one
//! path. The write-side marker guard absorbed the collision correctly for
//! months, which is precisely why nobody looked: bare `relearn verify` exited
//! non-zero on a clean tree, and the green invocation named four of the five
//! targets explicitly — teaching every reader that the default command is the
//! wrong one. A guard that never fails visibly is indistinguishable from a
//! guard that is not needed. `[R:prefer-by-construction]`
//!
//! The fix is not a different constant. The output path is now **derived from
//! the rule's own `Home`** via [`HomeSlug`] — whose shape is `[a-z0-9-]+` by
//! construction — so this emitter has no free-form path with which to name a
//! conventionally hand-authored file, and the collision is unrepresentable
//! rather than avoided. The hand-authored `CLAUDE.md` pulls the generated layer
//! in with an `@.claude/rules/<home-slug>.md` import, which keeps the human's
//! charter and the machine's rule digest as two artefacts with one owner each.
//!
//! Deriving the path from the home also closed a second defect that the
//! collision had been hiding: this emitter previously wrote **every** project
//! home's rules into one file, so a repository's project layer carried other
//! projects' rules — a one-home-per-rule violation in the emission layer (P2).
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.

use std::collections::BTreeMap;

use super::{HomeSlug, OutputFile, RelativePath, emittable, graduation_note};
use crate::library::{Library, Validated};
use crate::rule::{Home, Rule, RuleTag};

/// The directory this emitter owns. Every file it writes lives here, named by
/// the home slug — see the module docs for why this is not the repository root.
pub(crate) const PROJECT_LAYER_DIR: &[&str] = &[".claude", "rules"];

/// Emit one project-layer file per project home, each containing only that
/// home's rules, ordered by tag so the output is a deterministic function of
/// the library.
///
/// Returns an empty `Vec` (no file) when the library carries no project-layer
/// rules — this emitter never writes an empty file.
///
/// Accepts only `&Library<Validated>` — an unvalidated library does not have
/// this type, so emitting unchecked rules is a compile error.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    // Group the emittable (active + graduated) project rules by home, so each
    // project's file carries its own rules and no others. Atticked guidance is
    // suppressed here too: a retired project rule never reaches the layer.
    let mut by_home: BTreeMap<HomeSlug, Vec<&Rule>> = BTreeMap::new();
    for rule in emittable(library) {
        if matches!(rule.home(), Home::Project { .. }) {
            by_home
                .entry(HomeSlug::of(rule.home()))
                .or_default()
                .push(rule);
        }
    }

    let mut files = Vec::with_capacity(by_home.len());
    for (slug, mut rules) in by_home {
        rules.sort_by(|a, b| a.tag().as_str().cmp(b.tag().as_str()));
        let contents = render_project_layer(&rules);
        // The path is a function of the home, not a constant: `HomeSlug` is
        // `[a-z0-9-]+` by construction, so no rule's project path can steer
        // this emitter at a hand-authored file.
        let file_name = format!("{}.md", slug.as_str());
        let mut segments: Vec<&str> = PROJECT_LAYER_DIR.to_vec();
        segments.push(&file_name);
        let path = RelativePath::from_segments(&segments);
        let sources: Vec<RuleTag> = rules
            .iter()
            .map(|r| r.tag().clone()) // allow:clone: the OutputFile owns its provenance, outliving the &Library borrow
            .collect();
        files.push(OutputFile::new(path, contents, sources));
    }
    files
}

/// Render one project layer: a heading then one markdown section per rule, in
/// the tag order the caller sorted them into.
fn render_project_layer(rules: &[&Rule]) -> String {
    let mut out = String::new();
    out.push_str("# Project rules\n");
    for r in rules {
        out.push_str(&format!(
            "\n## {} [{}]\n\n",
            r.title().as_str(),
            r.tag().as_str()
        ));
        if let Some(note) = graduation_note(r) {
            out.push_str(&note);
        }
        out.push_str(&format!("{}\n", r.body().as_str()));
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
                Home::project("repo").expect("non-empty project"),
                "Project rule",
                "pc",
                "Project body.",
            ),
        ]);
        let files = emit(&lib);
        assert_eq!(files.len(), 1, "one file, for the single project home");
        let c = files[0].contents();
        assert!(c.contains("[R:p]"), "project rule is present");
        assert!(!c.contains("[R:g]"), "global rule is excluded");
        assert!(!c.contains("[R:d]"), "domain rule is excluded");
    }

    #[test]
    fn file_is_under_the_owned_dir_named_by_home_slug() {
        let lib = validated(vec![rule(
            "R:p",
            Home::project("relearn").expect("non-empty project"),
            "Project rule",
            "pc",
            "Project body.",
        )]);
        let files = emit(&lib);
        assert_eq!(files.len(), 1, "one project layer file");
        assert_eq!(files[0].path().as_str(), ".claude/rules/project-relearn.md");
    }

    /// The regression pin for the collision this emitter used to have: the
    /// repository-root `CLAUDE.md` is the hand-authored charter and must never
    /// be a target path again, whatever a rule's project path says.
    #[test]
    fn never_targets_the_repo_root_claude_md() {
        let lib = validated(vec![
            rule(
                "R:p",
                Home::project("CLAUDE.md").expect("non-empty project"),
                "Adversarial project path",
                "pc",
                "Project body.",
            ),
            rule(
                "R:q",
                Home::project("../..").expect("non-empty project"),
                "Traversal-shaped project path",
                "qc",
                "Project body.",
            ),
        ]);
        for f in emit(&lib) {
            assert_ne!(
                f.path().as_str(),
                "CLAUDE.md",
                "the hand-authored charter is never a target"
            );
            assert!(
                f.path().as_str().starts_with(".claude/rules/"),
                "every project-layer file stays inside the owned dir: {}",
                f.path().as_str()
            );
        }
    }

    /// The second defect the collision was hiding: one file per project home,
    /// each carrying only its own rules (P2, one home per rule).
    #[test]
    fn each_project_home_gets_its_own_file_carrying_only_its_rules() {
        let lib = validated(vec![
            rule(
                "R:a",
                Home::project("relearn").expect("non-empty project"),
                "Relearn rule",
                "ac",
                "Relearn body.",
            ),
            rule(
                "R:b",
                Home::project("stochos-lab").expect("non-empty project"),
                "Stochos rule",
                "bc",
                "Stochos body.",
            ),
        ]);
        let files = emit(&lib);
        assert_eq!(files.len(), 2, "one file per project home");
        let relearn = files
            .iter()
            .find(|f| f.path().as_str() == ".claude/rules/project-relearn.md")
            .expect("relearn layer emitted");
        assert!(relearn.contents().contains("[R:a]"), "own rule present");
        assert!(
            !relearn.contents().contains("[R:b]"),
            "another project's rule must not leak into this project's layer"
        );
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
                Home::project("repo").expect("non-empty project"),
                "Project two",
                "pc2",
                "Project two body.",
            ),
            rule(
                "R:p1",
                Home::project("repo").expect("non-empty project"),
                "Project one",
                "pc1",
                "Project one body.",
            ),
        ]);
        assert_eq!(emit(&lib), emit(&lib));
    }
}
