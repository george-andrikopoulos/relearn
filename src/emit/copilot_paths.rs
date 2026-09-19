//! `emit::copilot_paths` — one `<out>/.github/instructions/<home-slug>.instructions.md`
//! per home layer, each carrying an `applyTo:` glob so Copilot attaches it only
//! to the files it is about.
//!
//! **The second Copilot shape, not a replacement for the first.**
//! [`emit::copilot`](super::copilot) writes one `.github/copilot-instructions.md`
//! holding the whole library, which Copilot applies to every file in the
//! repository. That is right for a small corpus and wrong at eighty-three rules:
//! the eighteen Rust rules occupy context in a repository with no Rust in it,
//! which is the P6 argument that motivated `--home`. Copilot also reads
//! `.github/instructions/*.instructions.md`, each with an `applyTo:` glob — the
//! same load model [`LoadSemantics`] already derives for Cursor's `globs` and the
//! Claude rules layer's `paths:`. This emitter is reuse: the per-`Home` glob
//! table already exists and nothing new is modelled here.
//!
//! **Every home gets a file, and that is where this differs from
//! [`emit::claude_rules`](super::claude_rules).** That emitter skips a home it
//! cannot express: an unknown-language domain is [`LoadSemantics::OnRequest`], a
//! `.claude/rules/` file has no way to say "on request", and skipping is safe
//! there because the Claude **skill** target carries those rules by description
//! instead. Copilot has no second channel in this layer. Skipping
//! `domain-low-latency` would leave fifteen rules reachable through no Copilot
//! file at all — a correction lost, which is the failure at the top of
//! `CLAUDE.md` and the one thing this tool exists to prevent.
//!
//! So an unscopable home declares `applyTo: "**"`, the narrowest thing the
//! format can say, **and says in its own header that it is unscoped and why**.
//! That puts the choice where it can actually be made: these files are copied
//! into a repository one at a time, so a reader installs the domains they work
//! in. A file nobody copies costs nothing; a rule that reaches no file cannot be
//! copied at all.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.

use std::collections::BTreeMap;

use super::{
    HomeSlug, LoadSemantics, OutputFile, RelativePath, audience_note, emittable, enforcement_note,
    home_label, recurrence_note, source_note,
};
use crate::library::{Library, Validated};
use crate::rule::{Home, Rule, RuleTag};

/// The directory this emitter owns. Copilot reads `*.instructions.md` from here;
/// the repository-wide file `emit::copilot` writes is its sibling, one level up.
pub(crate) const INSTRUCTIONS_DIR: &[&str] = &[".github", "instructions"];

/// Emit one path-scoped instructions file per home.
///
/// Homes are ordered by slug and rules within a home by tag, so the output is a
/// deterministic function of the library — `emit(lib) == emit(lib)`, which the
/// overwrite guard relies on.
///
/// Accepts only `&Library<Validated>`: an unvalidated library does not have this
/// type, so emitting unchecked rules is a compile error rather than a runtime
/// guard. A library with no emittable rules produces no file.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    // Only emittable rules (active + graduated); atticked guidance is suppressed,
    // so a home whose rules are all atticked produces no file.
    let mut by_home: BTreeMap<HomeSlug, (&Home, Vec<&Rule>)> = BTreeMap::new();
    for rule in emittable(library) {
        by_home
            .entry(HomeSlug::of(rule.home()))
            .or_insert_with(|| (rule.home(), Vec::new()))
            .1
            .push(rule);
    }

    let mut files = Vec::with_capacity(by_home.len());
    for (slug, (home, mut rules)) in by_home {
        rules.sort_by(|a, b| a.tag().as_str().cmp(b.tag().as_str()));
        let filename = format!("{}.instructions.md", slug.as_str());
        let mut segments: Vec<&str> = INSTRUCTIONS_DIR.to_vec();
        segments.push(filename.as_str());
        let path = RelativePath::from_segments(&segments);
        // The OutputFile owns its provenance tags, outliving the `&Library`
        // borrow; in tag order, so the header's `rules=` list is deterministic.
        let sources: Vec<RuleTag> = rules
            .iter()
            .map(|r| r.tag().clone()) // allow:clone: the returned OutputFile owns its provenance, outliving the library borrow
            .collect();
        files.push(OutputFile::new(path, render(home, &rules), sources));
    }
    files
}

/// The `applyTo` value for a home, and whether it is a real narrowing.
///
/// Exhaustive over [`LoadSemantics`] with no catch-all, so a fourth load model
/// cannot acquire a glob by default — the same discipline the other emitters
/// apply to the same enum.
fn apply_to(home: &Home) -> (String, Scoped) {
    match LoadSemantics::for_home(home) {
        // Copilot has no "attach by description" mode, so the honest rendering
        // of both of these is "everywhere" — and `OnRequest` is the one that has
        // to say so, because for it "everywhere" is a widening rather than the
        // intent.
        LoadSemantics::Always => ("**".to_owned(), Scoped::Everywhere),
        LoadSemantics::OnRequest => ("**".to_owned(), Scoped::Unscopable),
        LoadSemantics::WhenReading(globs) => (globs.as_slice().join(","), Scoped::ToGlobs),
    }
}

/// Whether a file's `applyTo` narrows anything, and if not, why not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scoped {
    /// A language domain: the globs are the point.
    ToGlobs,
    /// Global, org or project — always-resident by design, and `**` states that.
    Everywhere,
    /// A domain with no glob table. `**` is a widening the reader must be told
    /// about, because the alternative was dropping the rules entirely.
    Unscopable,
}

/// Render one home's instructions file: Copilot front-matter, then the rules as
/// markdown sections in the same shape every other target uses.
fn render(home: &Home, rules: &[&Rule]) -> String {
    let (glob, scoped) = apply_to(home);
    let label = home_label(home);

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("applyTo: \"{glob}\"\n"));
    out.push_str("---\n\n");
    out.push_str(&format!("# {label}\n\n"));

    match scoped {
        Scoped::ToGlobs => out.push_str(&format!(
            "Attached when Copilot is working on `{glob}`.\n\n"
        )),
        Scoped::Everywhere => {
            out.push_str(
                "Attached to every file: this layer applies whatever you are working on.\n\n",
            );
        }
        // The one case a reader has to decide about, so it is stated rather than
        // implied by an absence.
        Scoped::Unscopable => out.push_str(
            "**This layer is not scoped by path.** It is a discipline rather than a language, \
             so there is no file extension that identifies the code it applies to, and Copilot \
             instructions can only attach by glob. `applyTo: \"**\"` is the narrowest thing the \
             format can express here.\n\nInstall this file when you are working in this \
             discipline and leave it out when you are not — that choice is yours to make and \
             cannot be made for you by a glob.\n\n",
        ),
    }

    for r in rules {
        out.push_str(&format!(
            "## {} [{}]\n\n",
            r.title().as_str(),
            r.tag().as_str()
        ));
        if let Some(note) = audience_note(r) {
            out.push_str(&note);
        }
        if let Some(note) = enforcement_note(r) {
            out.push_str(&note);
        }
        if let Some(note) = recurrence_note(r) {
            out.push_str(&note);
        }
        if let Some(note) = source_note(r) {
            out.push_str(&note);
        }
        out.push_str(&format!("{}\n\n", r.body().as_str()));
    }
    out
}
