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

// ── the plan: what --all would take, refresh, drop, and leave alone ──────────

use relearn::pull::{Plan, Prune, Skipped};

fn lib(rules: Vec<Rule>) -> relearn::library::Library<relearn::library::Validated> {
    relearn::library::Library::from_rules(rules)
        .validate()
        .expect("a valid library")
}

fn retired_upstream(tag: &str, version: u32) -> Rule {
    let doc = upstream_doc(tag, "{ kind = \"global\" }", Some(version)).replace(
        "status = { kind = \"active\" }",
        "status = { kind = \"attic\", reason = \"cold surface\", date = \"2026-09-02\" }",
    );
    parse_document(&doc).expect("a retired upstream rule parses")
}

/// **A plan is a report before it is an action.** Everything `--all` would do,
/// sorted into what it takes, what it refreshes, and what it leaves alone —
/// printed with no `--confirm`, which is what makes it the status check to run
/// before starting work.
#[test]
fn a_plan_takes_what_is_new_and_refreshes_what_is_behind() {
    let local = lib(vec![
        local("R:held", cached_at(2)),
        local("R:current", cached_at(7)),
    ]);
    let upstream = lib(vec![
        upstream("R:held", Some(5)),
        upstream("R:current", Some(7)),
        upstream("R:new", Some(1)),
    ]);

    let plan = Plan::of(&local, &upstream, source(), on(), &[], Prune::Keep);

    let taken: Vec<&str> = plan
        .take()
        .iter()
        .map(|p| p.rule().tag().as_str())
        .collect();
    assert_eq!(taken, ["R:new"], "a tag this install does not hold");

    let refreshed: Vec<&str> = plan
        .refresh()
        .iter()
        .map(|p| p.rule().tag().as_str())
        .collect();
    assert_eq!(refreshed, ["R:held"], "a cache behind upstream");

    assert!(
        plan.skipped()
            .iter()
            .any(|(tag, why)| tag.as_str() == "R:current" && *why == Skipped::AlreadyCurrent),
        "a cache already at upstream's revision is reported, not silently omitted: {:?}",
        plan.skipped()
    );
}

/// **What it will not touch, and says so.** A rule you own, a fork you took, a
/// withheld home and an unnumbered upstream rule are each left alone *and
/// named* — an omission nobody is told about is the failure a plan exists to
/// prevent.
#[test]
fn a_plan_names_everything_it_leaves_alone() {
    let local = lib(vec![
        local("R:mine", Authority::local()),
        local(
            "R:forked",
            Authority::adopted(source(), Version::new(1), on(), on()),
        ),
    ]);
    let withheld = parse_document(&upstream_doc(
        "R:theirs",
        "{ kind = \"project\", path = \"/somewhere/else\" }",
        Some(1),
    ))
    .expect("parses");
    let upstream = lib(vec![
        upstream("R:mine", Some(9)),
        upstream("R:forked", Some(9)),
        upstream("R:unnumbered", None),
        withheld,
    ]);

    let plan = Plan::of(&local, &upstream, source(), on(), &[], Prune::Keep);
    assert!(
        plan.take().is_empty() && plan.refresh().is_empty(),
        "{plan:?}"
    );

    let reasons: Vec<(&str, Skipped)> = plan
        .skipped()
        .iter()
        .map(|(tag, why)| (tag.as_str(), *why))
        .collect();
    for expected in [
        ("R:mine", Skipped::YoursToKeep),
        ("R:forked", Skipped::ADeliberateFork),
        ("R:unnumbered", Skipped::NoUpstreamVersion),
        ("R:theirs", Skipped::HomeIsWithheld),
    ] {
        assert!(
            reasons.contains(&expected),
            "{expected:?} missing from {reasons:?}"
        );
    }
}

/// `--scope` bounds what a bulk pull takes, which is §10's own mechanism for
/// keeping a local corpus small enough to compile. An **unscoped** upstream
/// rule serves every audience and is taken regardless — the same safety default
/// `build --scope` has, where narrowing can never remove a rule that declared
/// no audience.
#[test]
fn an_audience_bounds_what_a_bulk_pull_takes() {
    let upstream = lib(vec![
        scoped_upstream("R:rusty", 1, &["rust"]),
        scoped_upstream("R:javan", 1, &["java"]),
        upstream("R:everyone", Some(1)),
    ]);
    let audience = [relearn::rule::ScopeTag::parse("rust").expect("a valid scope")];
    let plan = Plan::of(
        &lib(Vec::new()),
        &upstream,
        source(),
        on(),
        &audience,
        Prune::Keep,
    );

    let mut taken: Vec<&str> = plan
        .take()
        .iter()
        .map(|p| p.rule().tag().as_str())
        .collect();
    taken.sort_unstable();
    assert_eq!(taken, ["R:everyone", "R:rusty"]);
    assert!(
        plan.skipped()
            .iter()
            .any(|(tag, why)| tag.as_str() == "R:javan" && *why == Skipped::NotInYourAudience)
    );
}

// ── dropping the unwanted ───────────────────────────────────────────────────

/// **Only a cache is droppable, and that is the whole safety argument.** A cache
/// is regenerable — dropping one loses nothing a `pull` cannot restore. A rule
/// you own and a fork you took are *source*, and nothing regenerates either.
#[test]
fn only_a_cache_can_be_dropped() {
    let gone_upstream = lib(vec![
        local("R:orphan", cached_at(1)),
        local("R:mine", Authority::local()),
        local(
            "R:forked",
            Authority::adopted(source(), Version::new(1), on(), on()),
        ),
    ]);
    let plan = Plan::of(
        &gone_upstream,
        &lib(Vec::new()),
        source(),
        on(),
        &[],
        Prune::Drop,
    );

    let dropped: Vec<&str> = plan.drop().iter().map(|d| d.tag().as_str()).collect();
    assert_eq!(
        dropped,
        ["R:orphan"],
        "a rule you own and a fork you took are source, and survive a prune"
    );
}

/// Two reasons a cache is unwanted, and both are reported as what they are:
/// the tag is **gone** from the drive, or upstream **retired** it — §12.6's
/// third resolution, taken deliberately rather than applied behind your back.
#[test]
fn a_cache_is_unwanted_when_it_is_gone_or_retired_upstream() {
    let local = lib(vec![
        local("R:orphan", cached_at(1)),
        local("R:retired", cached_at(1)),
        local("R:live", cached_at(1)),
    ]);
    let upstream = lib(vec![
        retired_upstream("R:retired", 1),
        upstream("R:live", Some(1)),
    ]);

    let plan = Plan::of(&local, &upstream, source(), on(), &[], Prune::Drop);
    let mut dropped: Vec<&str> = plan.drop().iter().map(|d| d.tag().as_str()).collect();
    dropped.sort_unstable();
    assert_eq!(dropped, ["R:orphan", "R:retired"]);
    assert!(
        plan.drop().iter().any(|d| d.why().contains("gone")),
        "the reason travels with the decision: {:?}",
        plan.drop()
    );
}

/// **Prune is opt-in, and `Prune::Keep` is the default everywhere.** Dropping is
/// the one thing here that removes a file, so it never happens because somebody
/// ran the ordinary command.
#[test]
fn nothing_is_dropped_unless_pruning_was_asked_for() {
    let local = lib(vec![local("R:orphan", cached_at(1))]);
    let plan = Plan::of(&local, &lib(Vec::new()), source(), on(), &[], Prune::Keep);
    assert!(plan.drop().is_empty());
    assert!(
        plan.skipped()
            .iter()
            .any(|(tag, why)| tag.as_str() == "R:orphan" && *why == Skipped::UnwantedButKept),
        "it is still reported, so the status check tells you what a prune would take: {:?}",
        plan.skipped()
    );
}

/// A cache of `relearn-upstream` at a given revision.
fn cached_at(revision: u32) -> Authority {
    Authority::cached(source(), Version::new(revision), on())
}

/// An upstream rule declaring the audiences it serves.
fn scoped_upstream(tag: &str, version: u32, scopes: &[&str]) -> Rule {
    let list: Vec<String> = scopes.iter().map(|s| format!("\"{s}\"")).collect();
    let doc = upstream_doc(tag, "{ kind = \"global\" }", Some(version)).replace(
        "created = ",
        &format!("applies_to = [{}]\ncreated = ", list.join(", ")),
    );
    parse_document(&doc).expect("a scoped upstream rule parses")
}

/// The whole workflow through the real binary: status, apply, converge. **The
/// second run is the assertion that matters** — a bulk operation that is not
/// idempotent is one nobody can run twice without reading the output first.
#[test]
fn pull_all_takes_refreshes_prunes_and_then_converges() {
    let dir = tempfile::tempdir().expect("tempdir");
    let drive = dir.path().join("drive");
    let rules = dir.path().join("rules");
    fs::create_dir_all(drive.join("rules")).expect("create the drive");
    fs::create_dir_all(&rules).expect("create the local rules dir");

    fs::write(
        drive.join("rules").join("new-one.md"),
        upstream_doc("R:new-one", "{ kind = \"global\" }", Some(1)),
    )
    .expect("write");
    fs::write(
        drive.join("rules").join("behind.md"),
        upstream_doc("R:behind", "{ kind = \"global\" }", Some(5)),
    )
    .expect("write");
    fs::write(
        rules.join("behind.md"),
        relearn::rule::to_document(&local("R:behind", cached_at(2))),
    )
    .expect("write");
    fs::write(
        rules.join("orphan.md"),
        relearn::rule::to_document(&local("R:orphan", cached_at(1))),
    )
    .expect("write");
    fs::write(
        rules.join("mine.md"),
        relearn::rule::to_document(&local("R:mine", Authority::local())),
    )
    .expect("write");

    let run = |extra: &[&str]| {
        let mut cmd = relearn();
        cmd.args(["pull", "--rules"])
            .arg(&rules)
            .arg("--upstream")
            .arg(&drive)
            .args(["--all", "--from", "drive", "--on", "2026-09-13"]);
        for arg in extra {
            cmd.arg(arg);
        }
        let out = cmd.output().expect("the binary runs");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    };

    // The status check writes nothing.
    let status = run(&["--prune"]);
    assert!(status.contains("take     R:new-one"), "{status}");
    assert!(status.contains("refresh  R:behind"), "{status}");
    assert!(status.contains("drop     R:orphan"), "{status}");
    assert!(status.contains("keep     R:mine"), "{status}");
    assert!(rules.join("orphan.md").exists(), "nothing written yet");

    run(&["--prune", "--confirm"]);
    assert!(rules.join("new-one.md").exists());
    assert!(
        !rules.join("orphan.md").exists(),
        "the orphan cache is gone"
    );
    assert!(
        rules.join("mine.md").exists(),
        "a rule this install owns is source, and survives a prune"
    );
    let refreshed = fs::read_to_string(rules.join("behind.md")).expect("read");
    assert!(refreshed.contains("version = 5"), "{refreshed}");

    // **Twice is the same as once.**
    let again = run(&["--prune", "--confirm"]);
    assert!(again.contains("Nothing to do."), "{again}");
}

/// A command that would do nothing is refused, because to whoever typed it a
/// silent no-op reads as a command that ran.
#[test]
fn pull_with_neither_a_tag_nor_all_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let common = dir.path().join("common-drive");
    drive(&common, Some(1));

    let out = relearn()
        .args(["pull", "--rules"])
        .arg(dir.path().join("rules"))
        .arg("--upstream")
        .arg(&common)
        .args(["--from", "drive", "--on", "2026-09-13"])
        .output()
        .expect("the binary runs");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--tag") && stderr.contains("--all"),
        "{stderr}"
    );
}
