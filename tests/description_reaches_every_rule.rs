//! Every rule in the committed corpus is named in its skill's `description`.
//!
//! **Why this exists.** `description` is the only string a matcher reads when
//! deciding whether to load a skill, and it is capped at 1024 characters. The
//! field listed rule **titles**, which average sixty-odd characters, so a home
//! ran out of room at about a dozen rules and the rest were replaced by
//! `+N more`. A rule whose subject is absent from that field cannot be matched
//! on — and for a home `LoadSemantics` makes `OnRequest` (every domain that is
//! not a language, `low-latency` among them) the description is the *only* path
//! into a session, so the rule may never load at all.
//!
//! Measured on the corpus this test was written against: `global` listed 15 of
//! its 30 rules and `domain-low-latency` 13 of 15. The titles of those thirty
//! global rules total 1824 characters against a 1024 budget — the field could
//! not hold them at any ordering, so the rank that decides *which* are dropped
//! (`emit::claude::description_order`) was necessary and was never going to be
//! sufficient.
//!
//! **What changed.** The description now lists each rule's **tag body** with its
//! hyphens turned into spaces — `no coordinated omission`, `a view is not a
//! copy` — rather than its title. The same thirty global rules cost 749
//! characters that way, so every home fits with room. Tags are authored, unique
//! and stable, so this is not a heuristic compression of prose: it is the field
//! carrying the identifier the author already wrote as a keyword phrase.
//!
//! **What this test is, precisely.** It is a **budget** check, not a formatting
//! check. It asserts the outcome that matters — no rule is unmatchable — over
//! the real corpus, so it fails on the day a home grows past what the field can
//! hold, which is the warning that was missing. `emit::claude`'s unit tests hold
//! the formatting; this holds the guarantee.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use relearn::emit;
use relearn::fsio;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// The `description:` value of an emitted `SKILL.md`, unquoted.
fn description_of(contents: &str) -> String {
    let line = contents
        .lines()
        .find(|l| l.starts_with("description:"))
        .expect("every skill carries a description");
    line.trim_start_matches("description:")
        .trim()
        .trim_matches('"')
        .to_owned()
}

#[test]
fn every_rule_is_named_in_its_skills_description() {
    let library = fsio::load_rules(&repo_root().join("rules"))
        .expect("the committed rules load")
        .validate()
        .expect("and validate");

    // Which rules each emitted skill claims as sources, by file path.
    let mut expected: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let files = emit::claude::emit(&library);
    for file in &files {
        expected.insert(
            file.path().as_str().to_owned(),
            file.sources()
                .iter()
                .map(|t| t.body().replace('-', " "))
                .collect(),
        );
    }

    let mut unmatchable = Vec::new();
    for file in &files {
        let description = description_of(file.contents());
        for subject in &expected[file.path().as_str()] {
            if !description.contains(subject.as_str()) {
                unmatchable.push(format!("{}: {subject}", file.path().as_str()));
            }
        }
    }

    assert!(
        unmatchable.is_empty(),
        "{} rule(s) are absent from the description of the skill that carries them, so a \
         matcher cannot fire on their subject. For a home that is not a language this means \
         the rule may never load at all. Either shorten what the description lists, or split \
         the home.\n  {}",
        unmatchable.len(),
        unmatchable.join("\n  ")
    );
}

/// The remainder counter must therefore never appear. Stated separately because
/// it is the *symptom* the test above is the cause of: if `+N more` is present,
/// some home outgrew the field, and the message should say which.
///
/// **The match is on `+<digit>`, not on the word "more", and that is a
/// correction rather than a nicety.** Written the obvious way it looked for
/// `" more"` and reported `global` as truncated when every one of its thirty
/// rules was present — because `five-files-no-more` renders as `five files no
/// more`, and the detector was matching the content it was meant to be counting.
/// A detector whose pattern occurs in legitimate data reports a failure that
/// cannot be fixed, which is how a check gets muted.
/// `[R:detector-excludes-own-definitions]`
fn truncation_marker(description: &str) -> bool {
    description
        .as_bytes()
        .windows(2)
        .any(|w| w[0] == b'+' && w[1].is_ascii_digit())
}

#[test]
fn no_skill_description_has_had_to_truncate() {
    let library = fsio::load_rules(&repo_root().join("rules"))
        .expect("the committed rules load")
        .validate()
        .expect("and validate");

    let truncated: Vec<String> = emit::claude::emit(&library)
        .iter()
        .filter(|f| truncation_marker(&description_of(f.contents())))
        .map(|f| format!("{} ({} rules)", f.path().as_str(), f.sources().len()))
        .collect();

    assert!(
        truncated.is_empty(),
        "a description ran out of room and dropped rules:\n  {}",
        truncated.join("\n  ")
    );
}
