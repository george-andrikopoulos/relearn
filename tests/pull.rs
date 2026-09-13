//! `pull` — the cache-creating half, and the second witness that keeps it from
//! being the editing half.
//!
//! Until now this install could **publish** a rule and could not **receive**
//! one. `Authority::Cached` existed, `adopt` existed, three poke triggers read
//! caches — and nothing anywhere created one, so a cache could only be made by
//! hand-writing an `authority` table. Everything downstream of that was
//! dormant.
//!
//! **The type-level point is decision 3, and it is the reason this is not a
//! flag on `write_rule`.** B1 made "a cached rule is never edited in place"
//! structural: `fsio::write_rule` takes an [`EditableRule`] whose constructor
//! *refuses a cache*. `pull` is the one legitimate writer of a cache, so it
//! cannot reuse that path — and must not be able to. It takes a second witness,
//! [`PulledRule`], minted only from an upstream document, and the two write
//! paths are therefore incapable of being confused for one another. A `bool` on
//! one path would have made "edit a cache in place" reachable by passing
//! `true`, which is exactly the state B1 spent a phase making unconstructible.

use std::fs;
use std::path::Path;

use relearn::rule::{
    Authority, Date, NotPullable, PulledRule, Rule, SourceId, Version, parse_document,
};

/// An upstream rule document, as it would sit on the common drive.
fn upstream_doc(tag: &str, home: &str, version: Option<u32>) -> String {
    let authority = match version {
        Some(v) => format!("authority = {{ kind = \"local\", version = {v} }}\n"),
        None => String::new(),
    };
    format!(
        r#"+++
tag = "{tag}"
title = "An upstream rule"
error_class = "a class somebody upstream wrote down"
home = {home}
created = "2026-09-01"
origin = "mined"
status = {{ kind = "active" }}
{authority}incident = "A published account with no quotation."
+++

Do the upstream thing.
"#
    )
}

fn upstream(tag: &str, version: Option<u32>) -> Rule {
    parse_document(&upstream_doc(tag, "{ kind = \"global\" }", version))
        .expect("the upstream document parses")
}

fn source() -> SourceId {
    SourceId::parse("relearn-upstream").expect("a valid source")
}

fn on() -> Date {
    Date::parse("2026-09-13").expect("a valid date")
}

fn local(tag: &str, authority: Authority) -> Rule {
    let doc = upstream_doc(tag, "{ kind = \"global\" }", None);
    let mut rule = parse_document(&doc).expect("parses");
    rule = Rule::new(
        rule.tag().clone(),
        relearn::rule::Title::parse("A local rule").expect("non-empty"),
        rule.error_class().clone(),
        rule.home().clone(),
        rule.created(),
        rule.origin().clone(),
        rule.status().clone(),
        rule.incident().clone(),
        rule.body().clone(),
        Vec::new(),
        Vec::new(),
        authority,
        None,
    );
    rule
}

// ── what a pull produces ────────────────────────────────────────────────────

/// The cache records **what it is a copy of, at which revision, and when** —
/// the three facts `cache-behind` and `adopt` are both built on.
#[test]
fn a_pull_mints_a_cache_carrying_its_provenance() {
    let pulled = PulledRule::of(&upstream("R:x", Some(4)), None, source(), on())
        .expect("a versioned global rule can be pulled");

    match pulled.rule().authority() {
        Authority::Cached {
            from,
            version,
            pulled,
        } => {
            assert_eq!(from.as_str(), "relearn-upstream");
            assert_eq!(*version, Version::new(4));
            assert_eq!(pulled.to_string(), "2026-09-13");
        }
        other => panic!("a pull must mint a cache, got {other:?}"),
    }
    assert!(
        !pulled.rule().is_editable(),
        "a freshly pulled rule is a cache, and a cache is not editable"
    );
}

/// The rule keeps **upstream's** home, which is the whole point of caching it:
/// it emits into this install's global layer exactly like a local rule. Home
/// says which layer; `Authority` says who maintains it. They are independent.
#[test]
fn a_cache_keeps_the_home_it_arrived_with() {
    let pulled = PulledRule::of(&upstream("R:x", Some(1)), None, source(), on()).expect("pulls");
    assert_eq!(pulled.rule().home(), &relearn::rule::Home::global());
}

/// **No upstream revision, no pull.** A cache of an unnumbered rule could never
/// be told it is stale — `cache-behind` would be silently dead for it forever —
/// so the refusal is at the door rather than a surprise months later.
#[test]
fn an_unnumbered_upstream_rule_cannot_be_pulled() {
    assert_eq!(
        PulledRule::of(&upstream("R:x", None), None, source(), on())
            .expect_err("an unnumbered rule has no comparison point"),
        NotPullable::NoUpstreamVersion
    );
}

/// A home that never leaves a machine never arrives on one either. A
/// project-homed rule names **somebody else's filesystem path**; an org-homed
/// one is an organisation's own. `Home::federation` decides both, in the one
/// exhaustive match that already refuses to let them out.
#[test]
fn a_withheld_home_cannot_arrive_any_more_than_it_can_leave() {
    for home in [
        "{ kind = \"project\", path = \"/home/someone/their-repo\" }",
        "{ kind = \"org\", name = \"their-employer\" }",
    ] {
        let doc = upstream_doc("R:x", home, Some(2));
        let rule = parse_document(&doc).expect("parses");
        assert_eq!(
            PulledRule::of(&rule, None, source(), on()).expect_err("a withheld home is refused"),
            NotPullable::HomeIsWithheld,
            "{home} should never arrive"
        );
    }
}

// ── what a pull refuses to overwrite ────────────────────────────────────────

/// Re-pulling **is** the update path: a cache is regenerable, that is what makes
/// it a cache, and the new copy carries the new revision.
#[test]
fn re_pulling_over_a_cache_updates_it() {
    let held = local(
        "R:x",
        Authority::cached(
            source(),
            Version::new(2),
            Date::parse("2026-09-01").expect("date"),
        ),
    );
    let pulled = PulledRule::of(&upstream("R:x", Some(5)), Some(&held), source(), on())
        .expect("a cache may be replaced by a newer copy of itself");
    assert_eq!(
        pulled.rule().authority().version(),
        Some(Version::new(5)),
        "the refreshed cache holds the revision it just arrived at"
    );
}

/// **The refusal that matters.** Pulling over a rule this install owns would
/// destroy hand-authored source and replace it with a stranger's, silently.
#[test]
fn pulling_over_your_own_rule_is_refused() {
    let mine = local("R:x", Authority::local());
    assert_eq!(
        PulledRule::of(&upstream("R:x", Some(3)), Some(&mine), source(), on())
            .expect_err("a local rule is not a cache to be refreshed"),
        NotPullable::WouldClobberLocal
    );
}

/// A fork was taken deliberately and **remembers what it was forked from**;
/// overwriting it with upstream's copy would discard both the change and the
/// provenance, which is the silent fork running backwards.
#[test]
fn pulling_over_a_deliberate_fork_is_refused() {
    let fork = local(
        "R:x",
        Authority::adopted(
            source(),
            Version::new(2),
            Date::parse("2026-09-01").expect("date"),
            Date::parse("2026-09-02").expect("date"),
        ),
    );
    assert_eq!(
        PulledRule::of(&upstream("R:x", Some(9)), Some(&fork), source(), on())
            .expect_err("a fork is not a cache"),
        NotPullable::WouldClobberFork
    );
}

// ── the two write paths cannot be confused ──────────────────────────────────

/// `write_rule` takes an `EditableRule` and `write_cache` takes a `PulledRule`,
/// and **neither witness can be minted for the other's subject**: a cache
/// cannot become editable, and a pull only ever produces a cache. The compile-
/// fail pin beside `tests/compile_fail/` holds the first half; this holds the
/// second at runtime, over the same rule.
#[test]
fn a_pulled_rule_is_never_editable_and_an_editable_rule_is_never_a_cache() {
    let pulled = PulledRule::of(&upstream("R:x", Some(1)), None, source(), on()).expect("pulls");
    assert!(
        relearn::rule::EditableRule::of(pulled.rule()).is_err(),
        "the rule a pull produced must not be mintable as editable"
    );

    let mine = local("R:y", Authority::local());
    assert!(relearn::rule::EditableRule::of(&mine).is_ok());
}

// ── through the real binary ─────────────────────────────────────────────────

fn relearn() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_relearn"))
}

/// A common drive with one rule on it, and an empty local library.
fn drive(dir: &Path, version: Option<u32>) {
    let rules = dir.join("rules");
    fs::create_dir_all(&rules).expect("create the upstream rules dir");
    fs::write(
        rules.join("x.md"),
        upstream_doc("R:x", "{ kind = \"global\" }", version),
    )
    .expect("write the upstream rule");
}

/// **Two steps, like every other flow that writes: print, then `--confirm`.**
#[test]
fn pull_writes_nothing_without_confirm_and_then_writes_the_cache() {
    let dir = tempfile::tempdir().expect("tempdir");
    let common = dir.path().join("common-drive");
    drive(&common, Some(4));
    let rules = dir.path().join("rules");
    fs::create_dir_all(&rules).expect("create the local rules dir");

    let out = relearn()
        .args(["pull", "--rules"])
        .arg(&rules)
        .arg("--upstream")
        .arg(&common)
        .args([
            "--tag",
            "R:x",
            "--from",
            "relearn-upstream",
            "--on",
            "2026-09-13",
        ])
        .output()
        .expect("the binary runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !rules.join("x.md").exists(),
        "a pull without --confirm must write nothing"
    );

    let out = relearn()
        .args(["pull", "--rules"])
        .arg(&rules)
        .arg("--upstream")
        .arg(&common)
        .args([
            "--tag",
            "R:x",
            "--from",
            "relearn-upstream",
            "--on",
            "2026-09-13",
            "--confirm",
        ])
        .output()
        .expect("the binary runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let written = fs::read_to_string(rules.join("x.md")).expect("the cache was written");
    assert!(written.contains("kind = \"cached\""), "{written}");
    assert!(written.contains("from = \"relearn-upstream\""), "{written}");
    assert!(written.contains("version = 4"), "{written}");
    assert!(written.contains("pulled = \"2026-09-13\""), "{written}");

    // And the corpus it landed in still validates, which is the only thing that
    // makes a pull useful: the cache compiles exactly like a local rule.
    let out = relearn()
        .args(["check", "--rules"])
        .arg(&rules)
        .output()
        .expect("the binary runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// A tag the common drive does not carry is an error naming it, not an empty
/// success — the shape every lookup in this tool takes.
#[test]
fn pulling_a_tag_the_drive_does_not_have_is_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let common = dir.path().join("common-drive");
    drive(&common, Some(1));
    let rules = dir.path().join("rules");
    fs::create_dir_all(&rules).expect("create the local rules dir");

    let out = relearn()
        .args(["pull", "--rules"])
        .arg(&rules)
        .arg("--upstream")
        .arg(&common)
        .args([
            "--tag",
            "R:absent",
            "--from",
            "relearn-upstream",
            "--on",
            "2026-09-13",
        ])
        .output()
        .expect("the binary runs");
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("R:absent"),
        "the error names the tag that was asked for"
    );
}
