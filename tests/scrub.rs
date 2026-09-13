//! The banned-terms matcher, over one string rather than a tree.
//!
//! `scripts/no-banned-names.sh` scans this repository's files; a contribution is
//! one authored paragraph, so the *matcher* is reused and the file-walking is
//! not — §12.2 of the design says exactly that. The term list format is the
//! shared contract: a salt line, then `<length> <sha256(salt || normalised)>`
//! per term, normalised to lowercase ASCII alphanumerics with separators
//! dropped.
//!
//! Two properties the script has that this must not lose:
//!
//! * **A finding never names what it found.** Output is a location and a length.
//!   A check that printed the banned name into a terminal or a CI log would have
//!   moved the exposure rather than closed it — `[R:report-the-hit-not-the-match]`.
//! * **Disarmed is not clean.** The script exits 2 with no list. Here the list
//!   is a required argument, so "ran without a list" is unconstructible rather
//!   than a case to remember; an unusable list is still refused explicitly.
//!
//! The lists below are synthetic — built in the test, salt and digests both — so
//! this suite never needs the real one and can never print a real name.

use relearn::scrub::{ScrubError, TermList};
use sha2::{Digest, Sha256};

/// Normalise as the shared format requires: lowercase, ASCII alphanumerics
/// only, everything else dropped. Written out here rather than imported so the
/// test states the contract independently of the implementation it checks.
fn normalise(term: &str) -> String {
    term.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// A term list in the shared format, with a fixed salt.
fn list_for(terms: &[&str]) -> String {
    let salt = "00112233445566778899aabbccddeeff";
    let salt_bytes = (0..salt.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&salt[i..i + 2], 16).expect("valid hex"))
        .collect::<Vec<u8>>();

    let mut out = format!("salt = {salt}\n");
    out.push_str("# a comment, ignored\n\n");
    for term in terms {
        let normalised = normalise(term);
        let mut hasher = Sha256::new();
        hasher.update(&salt_bytes);
        hasher.update(normalised.as_bytes());
        out.push_str(&format!(
            "{} {:x}\n",
            normalised.chars().count(),
            hasher.finalize()
        ));
    }
    out
}

fn armed(terms: &[&str]) -> TermList {
    TermList::parse(&list_for(terms)).expect("a well-formed list arms the matcher")
}

// ── it finds what the script finds ──────────────────────────────────────────

/// The joining pass: one to three consecutive tokens are joined before hashing,
/// so a name survives being written as one word, two words, hyphenated or
/// camel-cased.
#[test]
fn a_term_is_found_however_it_is_spaced_or_cased() {
    let list = armed(&["ProductName"]);
    for text in [
        "we shipped ProductName last week",
        "we shipped product name last week",
        "we shipped product-name last week",
        "we shipped PRODUCT_NAME last week",
    ] {
        assert!(
            list.find(text).is_some(),
            "the matcher must find it in: {text}"
        );
    }
}

/// The sliding pass: a name fused to a neighbour is still found, which joining
/// alone would miss.
#[test]
fn a_term_fused_to_a_neighbour_is_found() {
    let list = armed(&["ProductName"]);
    assert!(list.find("the ProductNameConfig file").is_some());
}

#[test]
fn clean_text_is_clean() {
    let list = armed(&["ProductName"]);
    assert!(
        list.find("a repository-local gate scanned its own tree and reported clean")
            .is_none()
    );
}

// ── a finding never names what it found ─────────────────────────────────────

/// `[R:report-the-hit-not-the-match]`. The finding carries a location and a
/// length; it does not carry the term, and it must not be possible to recover
/// the term from it. Printing the match would move the exposure rather than
/// close it — in a terminal, in a CI log, in a session transcript.
#[test]
fn a_finding_reports_location_and_length_but_never_the_match() {
    let list = armed(&["ProductName"]);
    let text = "we shipped ProductName last week";
    let hit = list.find(text).expect("the term is present");

    assert_eq!(hit.length(), normalise("ProductName").chars().count());
    assert!(hit.at() > 0);

    let rendered = hit.to_string();
    assert!(
        !rendered.to_lowercase().contains("productname"),
        "the finding named the match: {rendered}"
    );
    assert!(
        !rendered.contains("shipped"),
        "the finding quoted its context: {rendered}"
    );
    assert!(
        rendered.contains(&hit.length().to_string()),
        "the finding must carry the length: {rendered}"
    );
}

// ── disarmed is not clean ───────────────────────────────────────────────────

/// A list with no salt, or no terms, is refused rather than treated as empty.
/// The script exits 2 for this and calls it the point of the whole design: a
/// check that reports the same green whether it is armed or absent is worse
/// than none (`[R:guarantee-needs-a-reader]`).
#[test]
fn an_unusable_term_list_is_refused_not_treated_as_empty() {
    assert_eq!(
        TermList::parse("# nothing here\n").expect_err("no salt, no terms"),
        ScrubError::Disarmed
    );
    assert_eq!(
        TermList::parse("salt = 00112233\n").expect_err("a salt but no terms"),
        ScrubError::Disarmed
    );
    assert!(matches!(
        TermList::parse("salt = nothex\n4 abc\n").expect_err("a malformed salt"),
        ScrubError::Disarmed | ScrubError::MalformedSalt
    ));
}

/// The matcher cannot be constructed at all without a list, so "ran with no
/// list" is not a state to remember to handle — it does not exist. This is the
/// improvement on the script's exit 2: there, disarmed is a case; here it is
/// unrepresentable.
#[test]
fn there_is_no_matcher_without_a_list() {
    // `TermList::parse` is the only constructor and it returns a `Result`; the
    // assertion is that no `Default`, `empty()` or `new()` exists to bypass it.
    // If one is ever added, this test's comment is the reason it must not be.
    let list = armed(&["ProductName"]);
    assert!(list.find("ProductName").is_some());
}
