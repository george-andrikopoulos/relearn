//! End-to-end tests for generated-file drift detection (`fsio::verify_all`),
//! exercised entirely through the public API: build a validated library, emit
//! it, `write_all` it, then `verify_all` and assert the reported status. Each
//! case perturbs the on-disk tree in one way and pins the classification.

use std::fs;

use relearn::emit;
use relearn::fsio::{self, VerifyStatus};
use relearn::library::{Library, Validated};
use relearn::rule::{
    Authority, Body, Date, ErrorClass, Home, Incident, Origin, Rule, RuleTag, Status, Title,
};

fn rule(tag: &str, body: &str) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("Title").expect("non-empty title"),
        ErrorClass::parse("ec").expect("non-empty error class"),
        Home::global(),
        Date::parse("2026-08-13").expect("valid date"),
        Origin::Mined,
        Status::active(),
        Incident::parse("an incident").expect("non-empty incident"),
        Body::parse(body).expect("non-empty body"),
        Vec::new(),
        Vec::new(),
        Authority::local(),
        None,
    )
}

fn library(body: &str) -> Library<Validated> {
    Library::from_rules(vec![rule("R:one", body)])
        .validate()
        .expect("distinct tags validate")
}

/// The single Copilot file the one-rule library emits — a concatenated target,
/// so the whole library lands in one predictable path.
fn files(lib: &Library<Validated>) -> Vec<emit::OutputFile> {
    emit::copilot::emit(lib)
}

const TARGET: &str = ".github/copilot-instructions.md";

#[test]
fn freshly_built_files_verify_clean() {
    let dir = tempfile::tempdir().expect("tempdir");
    let lib = library("The body.");
    let f = files(&lib);
    fsio::write_all(dir.path(), &f).expect("write succeeds");

    let reports = fsio::verify_all(dir.path(), &f).expect("verify reads succeed");
    assert_eq!(reports.len(), 1);
    assert_eq!(*reports[0].status(), VerifyStatus::Ok);
    assert!(reports.iter().all(|r| r.status().is_clean()));
}

#[test]
fn a_missing_file_is_reported() {
    let dir = tempfile::tempdir().expect("tempdir");
    let lib = library("The body.");
    let f = files(&lib);
    fsio::write_all(dir.path(), &f).expect("write succeeds");
    fs::remove_file(dir.path().join(TARGET)).expect("delete the generated file");

    let reports = fsio::verify_all(dir.path(), &f).expect("verify reads succeed");
    assert_eq!(*reports[0].status(), VerifyStatus::Missing);
}

#[test]
fn a_hand_edited_body_is_detected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let lib = library("The body.");
    let f = files(&lib);
    fsio::write_all(dir.path(), &f).expect("write succeeds");

    // Tamper with the body while leaving the generated-by header in place.
    let path = dir.path().join(TARGET);
    let on_disk = fs::read_to_string(&path).expect("read back");
    fs::write(&path, format!("tampered {on_disk}")).expect("rewrite with an edited body");

    let reports = fsio::verify_all(dir.path(), &f).expect("verify reads succeed");
    assert_eq!(
        *reports[0].status(),
        VerifyStatus::HandEdited,
        "the body no longer hashes to the value its own header records"
    );
}

#[test]
fn an_unversioned_file_at_a_target_path_is_reported() {
    let dir = tempfile::tempdir().expect("tempdir");
    let lib = library("The body.");
    let f = files(&lib);

    // A hand-authored file (no marker) sitting exactly where a generated file goes.
    let path = dir.path().join(TARGET);
    fs::create_dir_all(path.parent().expect("has parent")).expect("mkdir");
    fs::write(&path, "hand authored, no marker\n").expect("seed a human file");

    let reports = fsio::verify_all(dir.path(), &f).expect("verify reads succeed");
    assert_eq!(*reports[0].status(), VerifyStatus::Unversioned);
}

#[test]
fn a_file_that_no_longer_matches_the_rules_is_stale() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Build from one library, then verify against a DIFFERENT one at the same path.
    let old = library("Old body.");
    fsio::write_all(dir.path(), &files(&old)).expect("write the old version");

    let new = library("New body.");
    let reports = fsio::verify_all(dir.path(), &files(&new)).expect("verify reads succeed");
    assert_eq!(
        *reports[0].status(),
        VerifyStatus::Stale,
        "the on-disk file is self-consistent but not what the current rules emit"
    );
}

fn validated(rules: Vec<Rule>) -> Library<Validated> {
    Library::from_rules(rules)
        .validate()
        .expect("distinct tags validate")
}

#[test]
fn an_orphan_generated_file_is_reported() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Two rules → two Cursor `.mdc` files (one per rule).
    let two = validated(vec![rule("R:alpha", "A body."), rule("R:beta", "B body.")]);
    fsio::write_all(dir.path(), &emit::cursor::emit(&two)).expect("write both");

    // The corpus shrinks to one rule: R:beta's `.mdc` now lingers on disk with
    // nothing to regenerate it — an orphan.
    let one = validated(vec![rule("R:alpha", "A body.")]);
    let reports =
        fsio::verify_all(dir.path(), &emit::cursor::emit(&one)).expect("verify reads succeed");

    assert!(
        reports
            .iter()
            .any(|r| *r.status() == VerifyStatus::Ok
                && r.path().as_str() == ".cursor/rules/alpha.mdc"),
        "the surviving rule's file is Ok"
    );
    assert!(
        reports.iter().any(|r| *r.status() == VerifyStatus::Orphan
            && r.path().as_str() == ".cursor/rules/beta.mdc"),
        "the deleted rule's generated file is reported as an orphan"
    );
}

#[test]
fn an_unmarked_file_in_an_owned_dir_is_not_an_orphan() {
    let dir = tempfile::tempdir().expect("tempdir");
    let one = validated(vec![rule("R:alpha", "A body.")]);
    let f = emit::cursor::emit(&one);
    fsio::write_all(dir.path(), &f).expect("write");

    // A hand-authored, marker-less file inside a relearn-owned dir must NOT be
    // claimed as an orphan — relearn only owns files carrying its own marker.
    let hand = dir.path().join("skills/mine/SKILL.md");
    fs::create_dir_all(hand.parent().expect("has parent")).expect("mkdir");
    fs::write(&hand, "hand-authored skill, no marker\n").expect("seed a human file");

    let reports = fsio::verify_all(dir.path(), &f).expect("verify reads succeed");
    assert!(
        reports.iter().all(|r| *r.status() != VerifyStatus::Orphan),
        "an unmarked hand-authored file is not relearn's to flag"
    );
}

/// The regression pin for 2026-10-27. Claude Code's skill sync downloads the
/// skills uploaded to an account into `skills/synced/<bucket>/<slug>/`, and a
/// relearn skill uploaded to claude.ai comes back down still carrying the
/// marker. A recursive walk of `skills/` reported every such copy as an orphan
/// "whose rule was removed" — false, and unfixable by deleting the copy, since
/// the next sync round restores it. Only the depth an emitter writes is relearn's.
#[test]
fn a_marked_skill_below_the_emitted_depth_is_not_an_orphan() {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = emit::claude::emit(&library("The body."));
    fsio::write_all(dir.path(), &f).expect("write");

    let emitted = dir.path().join(f[0].path().as_str());
    let synced = dir
        .path()
        .join("skills/synced/bucket")
        .join(f[0].path().as_str().trim_start_matches("skills/"));
    fs::create_dir_all(synced.parent().expect("has parent")).expect("mkdir");
    fs::copy(&emitted, &synced).expect("seed a synced copy carrying the marker");

    let reports = fsio::verify_all(dir.path(), &f).expect("verify reads succeed");
    assert!(
        reports.iter().all(|r| *r.status() == VerifyStatus::Ok),
        "a marked copy under skills/synced/ is not relearn's to flag: {reports:?}"
    );
}

/// The other half of the same fix: narrowing the scan to the emitted depth must
/// not hide anything an emitter actually writes. Every file the real corpus
/// emits into an owned directory, verified against an empty expectation, must
/// come back as an orphan — so an emitter that starts writing deeper than the
/// scan looks fails here instead of stranding orphans nobody reports.
#[test]
fn every_file_emitted_into_an_owned_dir_is_visible_to_the_orphan_scan() {
    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("rules");
    let lib = fsio::load_rules(&corpus)
        .expect("the corpus loads")
        .validate()
        .expect("the corpus validates");
    let mut f = emit::claude::emit(&lib);
    f.extend(emit::cursor::emit(&lib));
    f.extend(emit::claude_rules::emit(&lib));

    let dir = tempfile::tempdir().expect("tempdir");
    fsio::write_all(dir.path(), &f).expect("write");
    let reports = fsio::verify_all(dir.path(), &[]).expect("verify reads succeed");

    let owned = ["skills/", ".cursor/rules/", ".claude/rules/"];
    let emitted: Vec<&str> = f
        .iter()
        .map(|o| o.path().as_str())
        .filter(|p| owned.iter().any(|root| p.starts_with(root)))
        .collect();
    assert!(!emitted.is_empty(), "the corpus emits into the owned dirs");
    for path in emitted {
        assert!(
            reports
                .iter()
                .any(|r| *r.status() == VerifyStatus::Orphan && r.path().as_str() == path),
            "{path} is emitted but the orphan scan cannot see it"
        );
    }
}
