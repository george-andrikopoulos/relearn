//! The committed rule library is the regression fixture nothing else can
//! substitute for: it is the real corpus, on disk, in every shape the format
//! has accumulated. This test parses every `rules/*.md` and serializes it back,
//! asserting the result is **byte-identical** to the file it came from.
//!
//! It is what makes "this format change needs no migration" a fact rather than
//! a hope. A field added to the neutral format that changed the rendering of a
//! rule not using it — an unconditional `[[recurrence]]` block, a reordered
//! key, a lost escape — would rewrite every file in the library on the next
//! `relearn build`, and nothing else in the suite would notice: the property
//! tests generate rules rather than reading these, and `verify` checks the
//! *emitted* tree, not the sources it was emitted from.
//!
//! Line endings are pinned to LF for `rules/**` in `.gitattributes`
//! (`[R:pin-eol-for-executable-text]`): the comparison here is byte-exact, and
//! a checkout that rewrote LF to CRLF would fail it on Windows only.

use std::fs;
use std::path::{Path, PathBuf};

use relearn::rule::{parse_document, to_document};

/// The repository's own `rules/` directory, resolved from the manifest dir so
/// the test does not depend on the working directory it is run from.
fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("rules")
}

/// Every `*.md` in the corpus, in sorted order.
fn corpus_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(corpus_dir())
        .expect("the rules/ directory exists")
        .map(|entry| entry.expect("a readable directory entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    files.sort();
    files
}

#[test]
fn every_committed_rule_round_trips_byte_identically() {
    let files = corpus_files();
    assert!(
        !files.is_empty(),
        "the corpus is empty — this test would pass vacuously"
    );
    for path in files {
        let original = fs::read_to_string(&path).expect("a readable rule file");
        let rule = parse_document(&original)
            .unwrap_or_else(|e| panic!("{} must parse: {e}", path.display()));
        assert_eq!(
            to_document(&rule),
            original,
            "{} did not survive a parse/serialize round trip unchanged",
            path.display()
        );
    }
}

#[test]
fn every_committed_rule_parses() {
    // Separate from the round-trip assertion on purpose: a parse regression and
    // a serializer regression are different faults, and a single test that
    // fails for either makes the reader guess which.
    for path in corpus_files() {
        let doc = fs::read_to_string(&path).expect("a readable rule file");
        assert!(
            parse_document(&doc).is_ok(),
            "{} failed to parse",
            path.display()
        );
    }
}

/// Claude's skill front-matter caps `description`. An over-long one is not a
/// cosmetic problem: the skill is rejected on upload to claude.ai, and
/// `description` is the string the model reads to decide whether to load the
/// skill at all, so a 7000-character inventory is both invalid and useless as a
/// matcher.
///
/// Asserted over the **real** corpus rather than a generated one, because the
/// bound only binds at the size this library actually reaches: a property test
/// over two-rule libraries would pass forever while `skills/global/SKILL.md`
/// shipped seven times the cap, which is exactly what happened until
/// 2026-09-06.
#[test]
fn every_emitted_skill_description_fits_the_frontmatter_cap() {
    let lib = relearn::fsio::load_rules(&corpus_dir())
        .expect("the corpus loads")
        .validate()
        .expect("the corpus validates");
    let files = relearn::emit::claude::emit(&lib);
    assert!(!files.is_empty(), "the corpus must emit at least one skill");
    for file in &files {
        let description = file
            .contents()
            .lines()
            .find_map(|l| l.strip_prefix("description: "))
            .unwrap_or_else(|| panic!("{} has no description", file.path().as_str()));
        let chars = description.chars().count();
        assert!(
            chars <= relearn::emit::claude::DESCRIPTION_MAX_CHARS,
            "{} has a {chars}-character description, over the {}-character cap",
            file.path().as_str(),
            relearn::emit::claude::DESCRIPTION_MAX_CHARS
        );
    }
}
