//! The path-scoped Copilot layer: one file per home, each declaring the
//! narrowest `applyTo` it can express, and **no rule left behind**.
//!
//! **Why a second Copilot target.** `emit::copilot` writes one
//! `.github/copilot-instructions.md` holding the whole library, which Copilot
//! applies to every file in the repository. That is the right shape for a small
//! corpus and the wrong one at 83 rules: eighteen Rust rules occupy context in a
//! repository with no Rust in it, which is the P6 argument that motivated
//! `--home` in the first place. Copilot also reads
//! `.github/instructions/*.instructions.md`, each carrying an `applyTo:` glob —
//! the same load-semantics idea `LoadSemantics` already models for Cursor and
//! the Claude rules layer. This target is reuse, not new modelling.
//!
//! **Where it deliberately differs from `emit::claude_rules`.** That emitter
//! *skips* a home it cannot express — an unknown-language domain is
//! `LoadSemantics::OnRequest`, a `.claude/rules/` file cannot say "on request",
//! and skipping is safe there because the Claude **skill** target carries those
//! rules by description instead. Copilot has no such second channel in this
//! layer: skipping `domain-low-latency` would mean fifteen rules reachable
//! through no Copilot file at all, which is a correction lost — the failure at
//! the top of `CLAUDE.md`.
//!
//! So every home gets a file. A home whose load model cannot be expressed as a
//! glob declares `applyTo: "**"`, the narrowest thing the format can say, and
//! the file states in its own header that it is unscoped and why. The choice
//! then sits where it can actually be made: these files are copied into a
//! repository by hand, one at a time, so a reader installs the domains they
//! work in. A file nobody copies costs nothing; a rule that reaches no file
//! cannot be copied at all.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use relearn::emit;
use relearn::fsio;
use relearn::library::{Library, Validated};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn corpus() -> Library<Validated> {
    fsio::load_rules(&repo_root().join("rules"))
        .expect("the committed rules load")
        .validate()
        .expect("and validate")
}

/// **The load-bearing assertion.** Every rule the library would emit reaches
/// exactly one file in this layer — none dropped, none duplicated.
#[test]
fn every_emittable_rule_reaches_exactly_one_file() {
    let library = corpus();
    let files = emit::copilot_paths::emit(&library);

    let mut seen: Vec<String> = Vec::new();
    for file in &files {
        for tag in file.sources() {
            seen.push(tag.as_str().to_owned());
        }
    }
    let unique: BTreeSet<&String> = seen.iter().collect();
    assert_eq!(
        seen.len(),
        unique.len(),
        "a rule reaches this layer twice, so one correction is stated in two files"
    );

    let expected: BTreeSet<String> = emit::copilot::emit(&library)
        .iter()
        .flat_map(|f| f.sources().iter().map(|t| t.as_str().to_owned()))
        .collect();
    let got: BTreeSet<String> = seen.into_iter().collect();
    assert_eq!(
        got, expected,
        "the path-scoped layer must carry exactly what the single-file Copilot \
         target carries — a rule in one and not the other is a correction that \
         reaches Copilot only if you happened to install the right shape"
    );
}

/// One file per home, named by the home slug, under `.github/instructions/`.
#[test]
fn one_file_per_home_named_by_its_slug() {
    let library = corpus();
    let files = emit::copilot_paths::emit(&library);

    let homes: BTreeSet<String> = library
        .rules()
        .iter()
        .map(|r| emit::HomeSlug::of(r.home()).as_str().to_owned())
        .collect();

    assert_eq!(files.len(), homes.len());
    for file in &files {
        let path = file.path().as_str();
        assert!(
            path.starts_with(".github/instructions/"),
            "unexpected path {path}"
        );
        assert!(
            path.ends_with(".instructions.md"),
            "Copilot only reads *.instructions.md: {path}"
        );
        let slug = path
            .trim_start_matches(".github/instructions/")
            .trim_end_matches(".instructions.md");
        assert!(homes.contains(slug), "{slug} is not a home in the library");
    }
}

/// `applyTo` is derived from `LoadSemantics`, never invented — so a language
/// domain narrows and everything else says so out loud.
#[test]
fn apply_to_is_the_homes_load_semantics() {
    let library = corpus();
    for file in emit::copilot_paths::emit(&library) {
        let contents = file.contents();
        let apply_to = contents
            .lines()
            .find(|l| l.starts_with("applyTo:"))
            .expect("every instructions file declares applyTo");

        // The rule that produced the file tells us what to expect.
        let tag = &file.sources()[0];
        let rule = library
            .rules()
            .iter()
            .find(|r| r.tag() == tag)
            .expect("a file's source is a rule of the library");

        match emit::LoadSemantics::for_home(rule.home()) {
            emit::LoadSemantics::WhenReading(globs) => {
                for glob in globs.as_slice() {
                    assert!(
                        apply_to.contains(glob.as_str()),
                        "{} declares {apply_to}, missing {glob}",
                        file.path().as_str()
                    );
                }
                assert!(
                    !apply_to.contains("\"**\""),
                    "{} widened a scoped home to everything: {apply_to}",
                    file.path().as_str()
                );
            }
            emit::LoadSemantics::Always | emit::LoadSemantics::OnRequest => {
                assert!(
                    apply_to.contains("**"),
                    "{} must apply everywhere or say why: {apply_to}",
                    file.path().as_str()
                );
            }
        }
    }
}

/// A home that cannot be expressed as a glob says so in the file, rather than
/// being quietly unscoped — the reader is the one choosing whether to install
/// it, and they cannot choose without being told.
#[test]
fn an_unscopable_home_declares_that_it_is_unscoped() {
    let library = corpus();
    for file in emit::copilot_paths::emit(&library) {
        let tag = &file.sources()[0];
        let rule = library
            .rules()
            .iter()
            .find(|r| r.tag() == tag)
            .expect("a file's source is a rule of the library");

        if matches!(
            emit::LoadSemantics::for_home(rule.home()),
            emit::LoadSemantics::OnRequest
        ) {
            assert!(
                file.contents().contains("not scoped"),
                "{} is unscopable and does not say so",
                file.path().as_str()
            );
        }
    }
}

/// An empty library emits nothing — never an empty instructions file.
#[test]
fn an_empty_library_emits_no_file() {
    let empty = Library::from_rules(Vec::new())
        .validate()
        .expect("an empty library validates");
    assert!(emit::copilot_paths::emit(&empty).is_empty());
}
