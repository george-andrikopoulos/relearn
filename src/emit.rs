//! `emit` — one submodule per target assistant. Each emitter is a pure
//! function `&Library<Validated> -> Vec<OutputFile>`: it is handed already
//! validated rules and returns values for the caller (`fsio`, via `cli`) to
//! write. Emitters never perform the write themselves, which is what makes
//! them trivially unit-testable without a filesystem.
//!
//! Every emitted file carries a generated-by header naming its source rule(s)
//! and a content hash — the string the overwrite guard checks.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the
//! rule. If a target needs data, the data belongs in the rule, not in the
//! emitter.
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

use crate::rule::Home;

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

    /// The path text, forward-slash separated.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A file an emitter wants written: where (relative to the output root) and
/// what. An emitter returns these; it does not write them — the write is
/// `fsio`'s single audited responsibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFile {
    path: RelativePath,
    contents: String,
}

impl OutputFile {
    /// Assemble an output file. Internal to `emit`.
    pub(crate) fn new(path: RelativePath, contents: String) -> Self {
        OutputFile { path, contents }
    }

    /// Where to write, relative to the output root.
    #[must_use]
    pub fn path(&self) -> &RelativePath {
        &self.path
    }

    /// The full file contents.
    #[must_use]
    pub fn contents(&self) -> &str {
        &self.contents
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::Home;

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
}
