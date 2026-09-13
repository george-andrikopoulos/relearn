//! `Authority`, and the refusal that makes a silent fork impossible.
//!
//! A local install holds two species of rule once anything is cached: rules this
//! install is the home of, and copies of rules whose home is elsewhere. Editing
//! the second kind is a **silent fork** — P2 gone with no error to read, because
//! the edit succeeds and nothing anywhere records that the copy and its source
//! have diverged.
//!
//! **The refusal is structural, not a check in the write function.** A check
//! lives in one code path; the second write path, added next year by someone who
//! never read this file, does not have it and nothing objects. Instead
//! `fsio::write_rule` takes an [`EditableRule`] — a witness whose only
//! constructor refuses `Cached` — so a future write path cannot even be *called*
//! without minting the witness first. Same shape as `Library<Validated>` gating
//! emission, and the same shape as `[R:generate-guards-unversioned]`: a marker
//! the code recognises, never discipline at each call site.
//!
//! Forking is allowed. Forking by accident is not: `adopt` converts a cached
//! rule to `Adopted`, which is editable and **remembers what it was forked
//! from**.

use relearn::fsio::{self, RuleWriteError};
use relearn::rule::{
    Authority, Body, Date, EditableRule, ErrorClass, Home, Incident, Origin, Rule, RuleTag,
    SourceId, Status, Title, Version, parse_document,
};

fn cached_at(version: u32) -> Authority {
    Authority::cached(
        SourceId::parse("relearn-upstream").expect("valid source"),
        Version::new(version),
        Date::parse("2026-09-13").expect("valid date"),
    )
}

fn rule(tag: &str, authority: Authority) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("Title").expect("non-empty title"),
        ErrorClass::parse("ec").expect("non-empty error class"),
        Home::global(),
        Date::parse("2026-09-13").expect("valid date"),
        Origin::Mined,
        Status::active(),
        Incident::parse("an incident").expect("non-empty incident"),
        Body::parse("The body.").expect("non-empty body"),
        Vec::new(),
        Vec::new(),
        authority,
    )
}

// ── the refusal ─────────────────────────────────────────────────────────────

/// The load-bearing assertion: a cached rule cannot be minted as editable, so no
/// write path can take one.
#[test]
fn a_cached_rule_cannot_be_minted_editable() {
    let cached = rule("R:c", cached_at(3));
    assert_eq!(
        EditableRule::of(&cached).expect_err("a cached rule is not editable"),
        relearn::rule::CachedIsNotEditable
    );
    assert!(!cached.is_editable());
}

#[test]
fn a_local_rule_and_an_adopted_one_are_both_editable() {
    let local = rule("R:l", Authority::Local);
    let adopted = rule(
        "R:a",
        Authority::adopted(
            SourceId::parse("relearn-upstream").expect("valid source"),
            Version::new(3),
            Date::parse("2026-09-13").expect("valid date"),
            Date::parse("2026-09-14").expect("valid date"),
        ),
    );
    assert!(local.is_editable());
    assert!(adopted.is_editable());
    assert!(EditableRule::of(&local).is_ok());
    assert!(EditableRule::of(&adopted).is_ok());
}

/// The refusal reaches the filesystem: `write_rule` is the only path that writes
/// a rule file, and it cannot be called at all without the witness. Here the
/// witness is refused, so nothing is written.
#[test]
fn writing_a_cached_rule_is_impossible_and_nothing_reaches_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cached = rule("R:c", cached_at(1));

    assert!(EditableRule::of(&cached).is_err());
    // And the writer, reached with a *local* rule, does write — so the test
    // above is measuring the refusal rather than a writer that never works.
    let local = rule("R:l", Authority::Local);
    let editable = EditableRule::of(&local).expect("a local rule is editable");
    let written = fsio::write_rule(dir.path(), &editable).expect("the write succeeds");
    assert!(written.exists());
    assert!(!dir.path().join("c.md").exists());
}

/// A rule file written by `write_rule` parses back to the same rule — the
/// property that makes `adopt` safe to run twice and makes a hand-edit of the
/// result a normal authored change rather than a corruption.
#[test]
fn a_written_rule_round_trips() {
    let dir = tempfile::tempdir().expect("tempdir");
    let local = rule("R:l", Authority::Local);
    let editable = EditableRule::of(&local).expect("a local rule is editable");
    let path = fsio::write_rule(dir.path(), &editable).expect("the write succeeds");

    let text = std::fs::read_to_string(&path).expect("the file is readable");
    assert_eq!(parse_document(&text).expect("it parses"), local);
}

/// `write_rule` refuses to overwrite a file it cannot account for: a target that
/// is not a parseable rule for the same tag. A rule file is hand-authored
/// source, so clobbering an unrelated one is destroying work — the same argument
/// as the emitted tree's clobber guard, one layer over.
#[test]
fn writing_refuses_to_clobber_a_file_that_is_not_the_same_rule() {
    let dir = tempfile::tempdir().expect("tempdir");
    let local = rule("R:l", Authority::Local);
    let editable = EditableRule::of(&local).expect("a local rule is editable");
    std::fs::write(dir.path().join("l.md"), "not a rule at all\n").expect("write the decoy");

    match fsio::write_rule(dir.path(), &editable) {
        Err(RuleWriteError::WouldClobberUnrelated { .. }) => {}
        other => panic!("expected a clobber refusal, got {other:?}"),
    }
    assert_eq!(
        std::fs::read_to_string(dir.path().join("l.md")).expect("still readable"),
        "not a rule at all\n",
        "the decoy must be untouched"
    );
}

// ── the total order ─────────────────────────────────────────────────────────

/// Two caches of one rule must be comparable, **always**. Without a total order
/// "is this cache stale?" has no answer for some pairs, and staleness is the
/// whole reason the version is recorded.
#[test]
fn versions_are_totally_ordered() {
    let mut versions: Vec<Version> = vec![Version::new(7), Version::new(2), Version::new(30)];
    versions.sort();
    assert_eq!(
        versions,
        vec![Version::new(2), Version::new(7), Version::new(30)]
    );
    assert!(Version::new(2) < Version::new(30));
    assert_eq!(Version::new(4), Version::new(4));
    // Trichotomy, spelled out: for any pair exactly one of <, ==, > holds.
    for (a, b) in [(1u32, 1u32), (1, 2), (2, 1)] {
        let (a, b) = (Version::new(a), Version::new(b));
        assert_eq!([a < b, a == b, a > b].iter().filter(|x| **x).count(), 1);
    }
}

/// A cache knows whether it is behind an upstream version, and the answer is
/// never "cannot tell".
#[test]
fn a_cache_can_always_say_whether_it_is_behind() {
    let cached = cached_at(3);
    assert!(cached.is_behind(Version::new(4)));
    assert!(!cached.is_behind(Version::new(3)));
    assert!(!cached.is_behind(Version::new(2)));
    // A local rule is behind nothing: there is no upstream to be behind.
    assert!(!Authority::Local.is_behind(Version::new(99)));
}

// ── adopt keeps the provenance ──────────────────────────────────────────────

/// Adoption is the deliberate fork, and it **remembers**. A cached rule that
/// became plain `Local` would be indistinguishable from one authored here, which
/// is the silent fork wearing a different hat.
#[test]
fn adopting_records_what_it_was_forked_from_and_when() {
    let cached = cached_at(3);
    let adopted = cached
        .adopt(Date::parse("2026-09-14").expect("valid date"))
        .expect("a cached rule can be adopted");

    match &adopted {
        Authority::Adopted {
            from,
            version,
            pulled,
            adopted,
        } => {
            assert_eq!(from.as_str(), "relearn-upstream");
            assert_eq!(*version, Version::new(3));
            assert_eq!(pulled.to_string(), "2026-09-13");
            assert_eq!(adopted.to_string(), "2026-09-14");
        }
        other => panic!("expected Adopted, got {other:?}"),
    }
    assert!(adopted.is_editable());
}

/// Adopting anything that is not a cache is refused rather than silently
/// accepted: it would write a fork provenance naming a source the rule never
/// came from.
#[test]
fn only_a_cached_rule_can_be_adopted() {
    assert!(
        Authority::Local
            .adopt(Date::parse("2026-09-14").expect("valid date"))
            .is_err()
    );
    let already = cached_at(1)
        .adopt(Date::parse("2026-09-14").expect("valid date"))
        .expect("first adoption works");
    assert!(
        already
            .adopt(Date::parse("2026-09-15").expect("valid date"))
            .is_err(),
        "adopting twice would overwrite the first fork's provenance"
    );
}
