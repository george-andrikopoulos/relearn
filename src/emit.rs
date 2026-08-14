//! `emit` — one submodule per target assistant. Each emitter is a pure
//! function `&Library<Validated> -> Vec<OutputFile>`: it is handed already
//! validated rules and returns values for the caller (`fsio`, via `cli`) to
//! write. Emitters never perform the write themselves, which is what makes
//! them trivially unit-testable without a filesystem.
//!
//! Each [`OutputFile`] carries the rules that produced it ([`OutputFile::sources`]);
//! the generated-by header and content hash are rendered and appended by `fsio`
//! at write time (decision 2026-08-13), so the marker the overwrite guard reads
//! back lives in exactly one place. Emitters stay pure body-producers.
//!
//! **Must NOT:** read the filesystem, compute the hash / render the header, or
//! read anything not carried by the rule. If a target needs data, the data
//! belongs in the rule, not in the emitter.
//!
//! This module root holds the types every emitter shares: [`OutputFile`] (what
//! an emitter produces), [`RelativePath`] (where, relative to the output root),
//! and [`HomeSlug`] (the filesystem-safe identity of a home layer, which is
//! also a valid Claude skill `name`).

pub mod agents;
pub mod claude;
pub mod claude_md;
pub mod copilot;
pub mod cursor;

use crate::library::{Library, Validated};
use crate::rule::{Emittability, Home, Rule, RuleTag, Status};

/// A path relative to the output root, in portable forward-slash form. Built
/// only from known-safe segments inside this crate (`pub(crate)` constructor),
/// so it never carries untrusted, absolute, or `..`-bearing input; `fsio` joins
/// it onto the chosen output directory at write time.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelativePath(String);

impl RelativePath {
    /// Join ordered path segments with `/`. Internal to `emit` — callers pass
    /// literal, already-safe segments (`"skills"`, a [`HomeSlug`], `"SKILL.md"`).
    pub(crate) fn from_segments(segments: &[&str]) -> Self {
        RelativePath(segments.join("/"))
    }

    /// A relative path already in portable forward-slash form. Used by `fsio`
    /// when it discovers a generated file on disk (orphan detection) and needs to
    /// compare its path against the emitters' output paths, which are also
    /// forward-slash. Internal to the crate; the caller normalizes separators.
    pub(crate) fn from_forward_slash(s: impl Into<String>) -> Self {
        RelativePath(s.into())
    }

    /// The path text, forward-slash separated.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A file an emitter wants written: where (relative to the output root), the
/// body content, and which rules produced it. An emitter returns these; it does
/// not write them — the write, the generated-by header, and the content hash
/// are all `fsio`'s single audited responsibility. `sources` is the provenance
/// `fsio` serializes into that header (it is emit's knowledge, not fsio's).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFile {
    path: RelativePath,
    contents: String,
    sources: Vec<RuleTag>,
}

impl OutputFile {
    /// Assemble an output file. Internal to `emit`. `sources` are the tags of
    /// the rules that produced `contents`, in a deterministic order.
    pub(crate) fn new(path: RelativePath, contents: String, sources: Vec<RuleTag>) -> Self {
        OutputFile {
            path,
            contents,
            sources,
        }
    }

    /// Where to write, relative to the output root.
    #[must_use]
    pub fn path(&self) -> &RelativePath {
        &self.path
    }

    /// The body content, before `fsio` appends the generated-by header.
    #[must_use]
    pub fn contents(&self) -> &str {
        &self.contents
    }

    /// The tags of the rules that produced this file — the provenance the
    /// overwrite guard's header records.
    #[must_use]
    pub fn sources(&self) -> &[RuleTag] {
        &self.sources
    }
}

/// A filesystem-safe identity for a home layer: `[a-z0-9-]+`, derived from a
/// [`Home`]. It names the skill directory (`skills/<slug>/`) and doubles as the
/// Claude skill `name` (which has the same shape), so one type serves both.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HomeSlug(String);

impl HomeSlug {
    /// Derive the slug for a home. `Global` → `global`; a domain or project
    /// carries its name/path slugified (lowercased, non-alphanumerics collapsed
    /// to single `-`), so `project = "C:/repo"` becomes `project-c-repo`.
    #[must_use]
    pub fn of(home: &Home) -> Self {
        let slug = match home {
            Home::Global => "global".to_owned(),
            Home::Domain { name } => format!("domain-{}", slugify(name.as_str())),
            Home::Project { path } => format!("project-{}", slugify(path.as_str())),
        };
        HomeSlug(slug)
    }

    /// The slug text, shape `[a-z0-9-]+`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Lowercase `s` and collapse every run of non-alphanumeric characters to a
/// single `-`, with no leading or trailing `-`. Turns free-form domain names
/// and project paths into a single filesystem- and skill-name-safe token.
fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending_dash = false;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    out
}

/// Quote `s` as a YAML double-quoted scalar, escaping the characters that would
/// otherwise break it. Shared by every emitter whose target parses its
/// front-matter as **strict** YAML (Claude skills) and so needs a rule title
/// or error class containing `:` or `"` kept safe. Targets with a lenient
/// line-based front-matter (Cursor `.mdc`) deliberately do not use this.
pub(crate) fn yaml_double_quote(s: &str) -> String {
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

/// How a rule's [`Home`] translates into a target's scope vocabulary. The
/// neutral concept is `Home`; turning it into Cursor's `globs`/`alwaysApply`
/// (and, later, Copilot's ordering) is the emitter's job, and that translation
/// lives here — in one place — rather than being re-derived per emitter or,
/// worse, carried as vendor-specific fields on the rule (decision 2026-08-13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    globs: Vec<String>,
    always_apply: bool,
}

impl Scope {
    /// Derive the scope for a home:
    /// - `Global` → always applies, no globs.
    /// - `Project` → always applies within its tree, no globs.
    /// - `Domain` with a known language → auto-attaches on that language's file
    ///   globs (`rust` → `**/*.rs`), not always-on.
    /// - `Domain` with an unknown language → no globs and not always-on, i.e.
    ///   agent-requested via its description (never silently everywhere).
    #[must_use]
    pub fn for_home(home: &Home) -> Self {
        match home {
            Home::Global | Home::Project { .. } => Scope {
                globs: Vec::new(),
                always_apply: true,
            },
            Home::Domain { name } => Scope {
                globs: domain_globs(name.as_str()),
                always_apply: false,
            },
        }
    }

    /// The file globs this scope auto-attaches to (may be empty).
    #[must_use]
    pub fn globs(&self) -> &[String] {
        &self.globs
    }

    /// Whether the rule applies to every file regardless of glob.
    #[must_use]
    pub fn always_apply(&self) -> bool {
        self.always_apply
    }
}

/// The file globs for a language domain. An unknown domain returns no globs —
/// the rule then reaches the assistant by description, never blanket-applied.
/// Language knowledge lives here so every target that scopes by language reads
/// one table.
fn domain_globs(name: &str) -> Vec<String> {
    let patterns: &[&str] = match name.to_ascii_lowercase().as_str() {
        "rust" => &["**/*.rs"],
        "python" | "py" => &["**/*.py"],
        "typescript" | "ts" => &["**/*.ts", "**/*.tsx"],
        "javascript" | "js" => &["**/*.js", "**/*.jsx"],
        "go" | "golang" => &["**/*.go"],
        "java" => &["**/*.java"],
        "c" => &["**/*.c", "**/*.h"],
        "cpp" | "c++" => &["**/*.cpp", "**/*.hpp", "**/*.cc", "**/*.hh"],
        "ruby" | "rb" => &["**/*.rb"],
        _ => &[],
    };
    patterns.iter().map(|p| (*p).to_owned()).collect()
}

/// A stable ordering rank for a home, general → specific: `Global` (0) before
/// `Domain` (1) before `Project` (2). Emitters that concatenate every home into
/// one file (Copilot, `AGENTS.md`) order by this rank first — so the broadest
/// rules lead — then by home slug (same-home rules stay grouped), then by tag.
#[must_use]
pub(crate) fn home_rank(home: &Home) -> u8 {
    match home {
        Home::Global => 0,
        Home::Domain { .. } => 1,
        Home::Project { .. } => 2,
    }
}

/// The rules a library permits into the instruction layer, in library order:
/// every rule whose [`Status::emittability`] is `Emit` (active + graduated),
/// with atticked rules filtered out. The single place the emit-status policy is
/// applied — every emitter starts here rather than from `library.rules()`, so
/// "withdrawn guidance never leaks" is enforced once, not re-derived five times.
/// The full library (including atticked rules) stays available to `lint`, which
/// needs retired rules present to flag references to them.
#[must_use]
pub(crate) fn emittable(library: &Library<Validated>) -> Vec<&Rule> {
    library
        .rules()
        .iter()
        .filter(|r| r.status().emittability() == Emittability::Emit)
        .collect()
}

/// The one-line annotation a graduated rule carries in every emitted format: a
/// markdown blockquote naming the stronger control that *also* holds the
/// guarantee, so a reader of the instruction layer knows the prose is
/// belt-and-suspenders rather than the sole enforcement (decision 2026-08-13).
/// `None` for any non-graduated rule — active rules need no note, and atticked
/// rules never reach an emitter. Returns the note plus its trailing blank line,
/// so a caller splices it between a rule's heading and body with one `push_str`.
#[must_use]
pub(crate) fn graduation_note(rule: &Rule) -> Option<String> {
    match rule.status() {
        Status::Graduated { to } => Some(format!("> Also enforced by {}.\n\n", to.as_str())),
        Status::Active | Status::Attic { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::Library;
    use crate::rule::{Body, Date, ErrorClass, Home, Incident, RuleTag, Status, Title};

    fn rule_with_status(tag: &str, status: Status) -> Rule {
        Rule::new(
            RuleTag::parse(tag).expect("valid tag"),
            Title::parse("A title").expect("non-empty title"),
            ErrorClass::parse("an error class").expect("non-empty error class"),
            Home::global(),
            Date::parse("2026-08-13").expect("valid date"),
            status,
            Incident::parse("an incident").expect("non-empty incident"),
            Body::parse("Do the thing.").expect("non-empty body"),
        )
    }

    #[test]
    fn emittable_keeps_active_and_graduated_and_drops_attic() {
        let date = Date::parse("2026-09-01").expect("valid date");
        let lib = Library::from_rules(vec![
            rule_with_status("R:active", Status::active()),
            rule_with_status(
                "R:grad",
                Status::graduated("hook:x").expect("non-empty destination"),
            ),
            rule_with_status(
                "R:attic",
                Status::attic("retired", date).expect("non-empty reason"),
            ),
        ])
        .validate()
        .expect("distinct tags validate");

        let kept: Vec<&str> = emittable(&lib).iter().map(|r| r.tag().as_str()).collect();
        assert_eq!(kept, vec!["R:active", "R:grad"]);
    }

    #[test]
    fn graduation_note_only_annotates_graduated_rules() {
        let date = Date::parse("2026-09-01").expect("valid date");
        assert_eq!(
            graduation_note(&rule_with_status("R:a", Status::active())),
            None
        );
        assert_eq!(
            graduation_note(&rule_with_status(
                "R:x",
                Status::attic("retired", date).expect("non-empty reason"),
            )),
            None
        );
        assert_eq!(
            graduation_note(&rule_with_status(
                "R:g",
                Status::graduated("hook:no-unwrap-in-src").expect("non-empty destination"),
            )),
            Some("> Also enforced by hook:no-unwrap-in-src.\n\n".to_owned())
        );
    }

    #[test]
    fn global_slug_is_global() {
        assert_eq!(HomeSlug::of(&Home::global()).as_str(), "global");
    }

    #[test]
    fn domain_slug_is_prefixed() {
        assert_eq!(
            HomeSlug::of(&Home::domain("rust").expect("domain")).as_str(),
            "domain-rust"
        );
    }

    #[test]
    fn project_path_is_made_filesystem_safe() {
        let home = Home::project("C:/repo/sub").expect("project");
        assert_eq!(HomeSlug::of(&home).as_str(), "project-c-repo-sub");
    }

    #[test]
    fn slugify_collapses_and_trims_separators() {
        assert_eq!(slugify("  Foo__Bar!! "), "foo-bar");
        assert_eq!(slugify("a"), "a");
    }

    #[test]
    fn relative_path_joins_with_forward_slash() {
        let p = RelativePath::from_segments(&["skills", "global", "SKILL.md"]);
        assert_eq!(p.as_str(), "skills/global/SKILL.md");
    }

    #[test]
    fn global_scope_always_applies_with_no_globs() {
        let scope = Scope::for_home(&Home::global());
        assert!(scope.always_apply());
        assert!(scope.globs().is_empty());
    }

    #[test]
    fn project_scope_always_applies_with_no_globs() {
        let scope = Scope::for_home(&Home::project("C:/repo").expect("non-empty project"));
        assert!(scope.always_apply());
        assert!(scope.globs().is_empty());
    }

    #[test]
    fn known_domain_scopes_to_language_globs_and_is_not_always() {
        let scope = Scope::for_home(&Home::domain("rust").expect("non-empty domain"));
        assert!(!scope.always_apply());
        assert_eq!(scope.globs(), &["**/*.rs".to_owned()]);
    }

    #[test]
    fn unknown_domain_has_no_globs_and_is_not_always() {
        // Agent-requested via description — never blanket-applied.
        let scope = Scope::for_home(&Home::domain("cobol").expect("non-empty domain"));
        assert!(!scope.always_apply());
        assert!(scope.globs().is_empty());
    }

    #[test]
    fn home_rank_orders_general_before_specific() {
        assert!(home_rank(&Home::global()) < home_rank(&Home::domain("rust").expect("domain")));
        assert!(
            home_rank(&Home::domain("rust").expect("domain"))
                < home_rank(&Home::project("C:/repo").expect("project"))
        );
    }

    #[test]
    fn domain_matching_is_case_insensitive_and_aliased() {
        assert_eq!(
            Scope::for_home(&Home::domain("TypeScript").expect("non-empty domain")).globs(),
            &["**/*.ts".to_owned(), "**/*.tsx".to_owned()]
        );
        assert_eq!(
            Scope::for_home(&Home::domain("ts").expect("non-empty domain")).globs(),
            &["**/*.ts".to_owned(), "**/*.tsx".to_owned()]
        );
    }
}
