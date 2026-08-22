//! `emit::claude_rules` — the Claude **rules layer**: one file per home, at
//! `<out>/.claude/rules/<home-slug>.md`.
//!
//! **Which homes reach it is a question about the target, not the home**
//! (2026-08-22). A `.claude/rules/` file has a **two-state** load model: no
//! front-matter means resident in every session, and a `paths:` list means it
//! attaches when the assistant reads a matching file. [`LoadSemantics`] has
//! three states, so one of them — `OnRequest`, reachable by description only —
//! has no spelling here at all. It is skipped rather than written unscoped,
//! because an unscoped file is not a narrower rule, it is one loaded *always*:
//! `paths: []` was measured on 2026-08-22 to load at `session_start`, so the
//! empty list says the opposite of what it reads as. `Global` is likewise kept
//! out: it is `Always`, but it already has an always-resident home, and a second
//! copy here would be the duplication this tool exists to prevent.
//!
//! Named `claude_md` until 2026-08-22, when it stopped writing a `CLAUDE.md`
//! (below) and the name outlived what it describes. The CLI target it backs was
//! renamed with it: `--targets claude-md` is now `--targets claude-rules`.
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

use super::{
    HomeSlug, LoadSemantics, OutputFile, RelativePath, emittable, graduation_note,
    yaml_double_quote,
};
use crate::library::{Library, Validated};
use crate::rule::{Home, Rule, RuleTag};

/// The directory this emitter owns. Every file it writes lives here, named by
/// the home slug — see the module docs for why this is not the repository root.
pub(crate) const RULES_LAYER_DIR: &[&str] = &[".claude", "rules"];

/// Emit one rules-layer file per home that this target can scope, each holding only that
/// home's rules, ordered by tag so the output is a deterministic function of
/// the library.
///
/// Returns an empty `Vec` (no file) when the library carries no rules-layer
/// rules — this emitter never writes an empty file.
///
/// Accepts only `&Library<Validated>` — an unvalidated library does not have
/// this type, so emitting unchecked rules is a compile error.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    // Group the emittable (active + graduated) rules by home, so each home's file
    // carries its own rules and no others. Atticked guidance is suppressed here
    // too: a retired rule never reaches the layer.
    //
    // Which homes reach this layer is decided by what the target can *say*, not
    // by the kind of home: a `.claude/rules/` file has a two-state load model
    // (front-matter absent → always; `paths:` present → on reading a match), so
    // `Always` and `WhenReading` are representable and `OnRequest` is not. An
    // unknown-language domain is therefore skipped rather than written with no
    // `paths:` — which would not be a narrower rule, it would be one resident in
    // every session. Those domains keep reaching the assistant through the skill
    // target, which is the mechanism that actually implements "by description".
    let mut by_home: BTreeMap<HomeSlug, Vec<&Rule>> = BTreeMap::new();
    for rule in emittable(library) {
        let representable = match LoadSemantics::for_home(rule.home()) {
            LoadSemantics::Always | LoadSemantics::WhenReading(_) => true,
            LoadSemantics::OnRequest => false,
        };
        // `Global` is `Always` too, but it is not a *rules-layer* concern: it has
        // an always-resident home already, and emitting it here as well would put
        // one rule in two Claude files — the duplication this tool exists to stop.
        let in_layer = matches!(rule.home(), Home::Project { .. } | Home::Domain { .. });
        if representable && in_layer {
            by_home
                .entry(HomeSlug::of(rule.home()))
                .or_default()
                .push(rule);
        }
    }

    let mut files = Vec::with_capacity(by_home.len());
    for (slug, mut rules) in by_home {
        rules.sort_by(|a, b| a.tag().as_str().cmp(b.tag().as_str()));
        // Every rule in a group shares a home (they were grouped by its slug), so
        // the first rule's home speaks for the group.
        let home = rules[0].home();
        let contents = render_layer(home, &rules);
        // The path is a function of the home, not a constant: `HomeSlug` is
        // `[a-z0-9-]+` by construction, so no rule's project path can steer
        // this emitter at a hand-authored file.
        let file_name = format!("{}.md", slug.as_str());
        let mut segments: Vec<&str> = RULES_LAYER_DIR.to_vec();
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

/// Render one home's layer: the load-model front-matter (when the target needs
/// one), a heading, then one markdown section per rule, in the tag order the
/// caller sorted them into.
///
/// **Front-matter is emitted only for [`LoadSemantics::WhenReading`].** `Always`
/// is spelled by its *absence*: a rule file with no front-matter loads at session
/// start. It must not be spelled `paths: []`, which was measured on 2026-08-22 to
/// load at session start as well — the same meaning by accident, and the opposite
/// of what an empty glob list reads as anywhere else. `OnRequest` never reaches
/// this function; `emit` filters it out, because this target has no spelling for
/// it at all.
fn render_layer(home: &Home, rules: &[&Rule]) -> String {
    let mut out = String::new();
    if let LoadSemantics::WhenReading(globs) = LoadSemantics::for_home(home) {
        out.push_str("---\npaths:\n");
        for glob in globs.as_slice() {
            out.push_str(&format!("  - {}\n", yaml_double_quote(glob)));
        }
        out.push_str("---\n\n");
    }
    out.push_str(&format!("# {}\n", layer_heading(home)));
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

/// The layer's heading. Project layers keep the wording they have always had;
/// a domain layer names its language so a reader of the file knows what it scopes
/// to without parsing the front-matter.
fn layer_heading(home: &Home) -> String {
    match home {
        Home::Project { .. } => "Project rules".to_owned(),
        Home::Domain { name } => format!("Rules for domain: {}", name.as_str()),
        // Not reachable: `emit` keeps `Global` out of this layer. Rendering a
        // truthful heading is cheaper than a panic and keeps the function total.
        Home::Global => "Global rules".to_owned(),
    }
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

    /// Project and known-domain homes each get their own file; `Global` does not
    /// reach this layer at all (it already has an always-resident home, and a
    /// second copy here would be the duplication the tool exists to prevent).
    #[test]
    fn project_and_domain_homes_each_get_a_file_but_global_does_not() {
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
        assert_eq!(
            files.len(),
            2,
            "one file each for the domain and project homes"
        );
        let all: String = files.iter().map(|f| f.contents()).collect();
        assert!(all.contains("[R:p]"), "project rule is present");
        assert!(all.contains("[R:d]"), "known-domain rule is present");
        assert!(!all.contains("[R:g]"), "global rule is excluded");
    }

    /// A known-language domain is scoped with `paths:` — it loads when a matching
    /// file is read, not in every session.
    #[test]
    fn a_known_domain_layer_carries_its_language_globs() {
        let lib = validated(vec![rule(
            "R:d",
            Home::domain("rust").expect("non-empty domain"),
            "Rust rule",
            "dc",
            "Rust body.",
        )]);
        let files = emit(&lib);
        assert_eq!(files[0].path().as_str(), ".claude/rules/domain-rust.md");
        let c = files[0].contents();
        assert!(
            c.starts_with("---\npaths:\n  - \"**/*.rs\"\n---\n\n"),
            "got: {c}"
        );
        assert!(c.contains("# Rules for domain: rust"));
    }

    /// A project layer carries **no** front-matter: absence is how this target
    /// spells "always resident". It must not be spelled `paths: []`, which loads
    /// always as well and so says the same thing by accident.
    #[test]
    fn a_project_layer_carries_no_front_matter() {
        let lib = validated(vec![rule(
            "R:p",
            Home::project("repo").expect("non-empty project"),
            "Project rule",
            "pc",
            "Project body.",
        )]);
        let files = emit(&lib);
        let c = files[0].contents();
        assert!(c.starts_with("# Project rules\n"), "got: {c}");
        assert!(!c.contains("paths:"));
    }

    /// An unknown-language domain has no glob to scope by, and this target cannot
    /// say "reach this by description". It is skipped rather than written
    /// unscoped, which would make it resident in every session.
    #[test]
    fn an_unknown_domain_is_not_emitted_to_this_layer() {
        let lib = validated(vec![rule(
            "R:d",
            Home::domain("cobol").expect("non-empty domain"),
            "Cobol rule",
            "dc",
            "Cobol body.",
        )]);
        assert!(
            emit(&lib).is_empty(),
            "on-request domains do not reach this layer"
        );
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
    fn no_rules_for_this_layer_emits_nothing() {
        let lib = validated(vec![
            rule("R:g", Home::global(), "Global rule", "gc", "Global body."),
            rule(
                "R:d",
                Home::domain("cobol").expect("non-empty domain"),
                "Cobol rule",
                "dc",
                "Cobol body.",
            ),
        ]);
        let files = emit(&lib);
        assert!(
            files.is_empty(),
            "a global home and an on-request domain both stay out of this layer"
        );
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
