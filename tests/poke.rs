//! The poke: what the corpus has to say to this install, surfaced in `lint`.
//!
//! Four triggers, and the shape of the design is that **one of them is worth
//! reading**. The reactive poke follows evidence recorded here — you wrote down
//! that a rule of yours failed, and upstream already has a rule for that class.
//! The other three follow things that happened elsewhere, which is how a
//! notification channel teaches people to ignore it, so they are capped and two
//! of them are off until asked for.
//!
//! What the programme names as what goes wrong, with a test each: broadcast
//! triggers defaulting on, the poke firing during `build`, and — the one the
//! programme does not name and this file adds — a poke changing a verdict. A
//! signal from strangers that can fail a run has made federation required,
//! which invariant 3 forbids outright.

use std::fs;
use std::path::Path;
use std::process::Command;

use relearn::aggregate::{Aggregate, K_ANONYMITY_FLOOR};
use relearn::library::{Library, Validated};
use relearn::poke::{BroadcastCap, Poke, Pokes, Reach, Trigger, pokes};
use relearn::rule::{
    Authority, Body, Date, ErrorClass, Home, Incident, Origin, Recurrence, Rule, RuleTag, ScopeTag,
    SourceId, Status, Title, Version,
};

// ── fixtures ────────────────────────────────────────────────────────────────

/// A rule, with the four things a poke can read varied and everything else
/// held constant.
fn rule(
    tag: &str,
    error_class: &str,
    authority: Authority,
    recurrences: usize,
    applies_to: &[&str],
) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("a valid tag"),
        Title::parse("A title").expect("non-empty"),
        ErrorClass::parse(error_class).expect("non-empty"),
        Home::global(),
        Date::parse("2026-09-13").expect("a valid date"),
        Origin::Mined,
        Status::active(),
        Incident::parse("The triggering incident.").expect("non-empty"),
        Body::parse("Do the thing.").expect("non-empty"),
        (0..recurrences)
            .map(|i| {
                Recurrence::new(
                    Date::parse("2026-09-01").expect("a valid date"),
                    Incident::parse(format!("It happened again ({i}).")).expect("non-empty"),
                )
            })
            .collect(),
        applies_to
            .iter()
            .map(|s| ScopeTag::parse(*s).expect("a valid scope"))
            .collect(),
        authority,
        None,
    )
}

fn library(rules: Vec<Rule>) -> Library<Validated> {
    Library::from_rules(rules)
        .validate()
        .expect("a valid library")
}

fn cached_at(revision: u32) -> Authority {
    Authority::cached(
        SourceId::parse("relearn-upstream").expect("a valid source"),
        Version::new(revision),
        Date::parse("2026-09-10").expect("a valid date"),
    )
}

/// An aggregate in which `installs` distinct installs report `tag` in `bucket`.
fn aggregate_of(tag: &str, installs: usize, bucket: &str) -> Aggregate {
    let reports: Vec<String> = (0..installs)
        .map(|i| {
            format!(
                "schema = 1\ninstall = \"{i:08x}\"\ngenerated = \"2026-09\"\n\n\
                 [[observation]]\nrule = \"{tag}\"\nrecurrences = \"{bucket}\"\n\
                 latest = \"2026-08\"\nstatus = \"active\"\n"
            )
        })
        .collect();
    Aggregate::of(reports.iter().map(String::as_str)).expect("well-formed reports")
}

fn empty_aggregate() -> Aggregate {
    Aggregate::of(std::iter::empty()).expect("an empty aggregate")
}

fn with(
    local: &Library<Validated>,
    upstream: &Library<Validated>,
    aggregate: &Aggregate,
    triggers: &[Trigger],
) -> Pokes {
    pokes(
        local,
        upstream,
        aggregate,
        triggers,
        BroadcastCap::new(usize::MAX),
    )
}

// ── the reactive trigger ────────────────────────────────────────────────────

/// The one that makes this a collective memory rather than a mailing list: it
/// arrives at the moment somebody has just demonstrated they needed it.
#[test]
fn a_class_that_fired_here_and_is_covered_upstream_pokes() {
    let local = library(vec![rule(
        "R:mine",
        "a shared class",
        Authority::local(),
        2,
        &[],
    )]);
    let upstream = library(vec![rule(
        "R:theirs",
        "A Shared Class",
        Authority::local(),
        0,
        &[],
    )]);

    let raised = with(
        &local,
        &upstream,
        &empty_aggregate(),
        &[Trigger::ClassCoveredUpstream],
    );
    assert_eq!(raised.shown().len(), 1, "{:?}", raised.shown());
    match &raised.shown()[0] {
        Poke::ClassCoveredUpstream {
            fired,
            times,
            upstream,
            ..
        } => {
            assert_eq!(fired.as_str(), "R:mine");
            assert_eq!(upstream.as_str(), "R:theirs");
            assert_eq!(*times, 2);
        }
        other => panic!("expected the reactive poke, got {other:?}"),
    }
}

/// A rule nobody has seen fire is not evidence, and the poke exists to follow
/// evidence. Without this the reactive trigger degenerates into the broadcast
/// one it was distinguished from.
#[test]
fn a_rule_that_has_not_fired_raises_nothing() {
    let local = library(vec![rule(
        "R:mine",
        "a shared class",
        Authority::local(),
        0,
        &[],
    )]);
    let upstream = library(vec![rule(
        "R:theirs",
        "a shared class",
        Authority::local(),
        0,
        &[],
    )]);
    assert!(
        with(
            &local,
            &upstream,
            &empty_aggregate(),
            &[Trigger::ClassCoveredUpstream]
        )
        .is_empty()
    );
}

/// Holding the upstream rule **is** the answer the poke would give, so saying
/// it is noise — and noise is the failure mode the whole cap exists for.
#[test]
fn an_upstream_rule_you_already_hold_is_not_news() {
    let local = library(vec![
        rule("R:mine", "a shared class", Authority::local(), 1, &[]),
        rule("R:theirs", "a shared class", cached_at(1), 0, &[]),
    ]);
    let upstream = library(vec![rule(
        "R:theirs",
        "a shared class",
        Authority::local(),
        0,
        &[],
    )]);
    assert!(
        with(
            &local,
            &upstream,
            &empty_aggregate(),
            &[Trigger::ClassCoveredUpstream]
        )
        .is_empty()
    );
}

/// A mandate was never mined, so its recurrences are not evidence that prose
/// failed — here for the same reason the local tally and the report flow hold
/// mandates out, and through the same predicate.
#[test]
fn a_mandate_that_recurred_raises_no_reactive_poke() {
    let approval = relearn::rule::Approval::new(
        relearn::rule::Approver::parse("the control board").expect("non-empty"),
        Date::parse("2026-07-11").expect("a valid date"),
        relearn::rule::ControlRef::parse("CC-6.1").expect("non-empty"),
    );
    let mandated = Rule::new(
        RuleTag::parse("R:mandated").expect("a valid tag"),
        Title::parse("A title").expect("non-empty"),
        ErrorClass::parse("a shared class").expect("non-empty"),
        Home::global(),
        Date::parse("2026-09-13").expect("a valid date"),
        Origin::Mandated(approval),
        Status::active(),
        Incident::parse("The approval's own account.").expect("non-empty"),
        Body::parse("Do the thing.").expect("non-empty"),
        vec![Recurrence::new(
            Date::parse("2026-09-01").expect("a valid date"),
            Incident::parse("It happened again.").expect("non-empty"),
        )],
        Vec::new(),
        Authority::local(),
        None,
    );
    let upstream = library(vec![rule(
        "R:theirs",
        "a shared class",
        Authority::local(),
        0,
        &[],
    )]);
    assert!(
        with(
            &library(vec![mandated]),
            &upstream,
            &empty_aggregate(),
            &[Trigger::ClassCoveredUpstream]
        )
        .is_empty()
    );
}

// ── the stale-cache trigger ─────────────────────────────────────────────────

#[test]
fn a_cache_behind_the_upstream_revision_pokes() {
    let local = library(vec![rule("R:x", "a class", cached_at(2), 0, &[])]);
    let upstream = library(vec![rule(
        "R:x",
        "a class",
        Authority::local_at(Version::new(5)),
        0,
        &[],
    )]);

    let raised = with(
        &local,
        &upstream,
        &empty_aggregate(),
        &[Trigger::CacheBehind],
    );
    match &raised.shown()[0] {
        Poke::CacheBehind {
            tag,
            held,
            upstream,
        } => {
            assert_eq!(tag.as_str(), "R:x");
            assert_eq!(*held, Version::new(2));
            assert_eq!(*upstream, Version::new(5));
        }
        other => panic!("expected a stale-cache poke, got {other:?}"),
    }
}

#[test]
fn a_cache_at_the_upstream_revision_is_quiet() {
    let local = library(vec![rule("R:x", "a class", cached_at(5), 0, &[])]);
    let upstream = library(vec![rule(
        "R:x",
        "a class",
        Authority::local_at(Version::new(5)),
        0,
        &[],
    )]);
    assert!(
        with(
            &local,
            &upstream,
            &empty_aggregate(),
            &[Trigger::CacheBehind]
        )
        .is_empty()
    );
}

/// **No revision upstream, no comparison point, no poke.** The alternative —
/// treating an absent revision as zero — would tell every cache it is stale
/// forever, which is the sentinel-value failure this repository refuses by
/// construction elsewhere.
#[test]
fn an_upstream_rule_with_no_revision_pokes_nobody() {
    let local = library(vec![rule("R:x", "a class", cached_at(2), 0, &[])]);
    let upstream = library(vec![rule("R:x", "a class", Authority::local(), 0, &[])]);
    assert!(
        with(
            &local,
            &upstream,
            &empty_aggregate(),
            &[Trigger::CacheBehind]
        )
        .is_empty()
    );
}

/// A home is not behind its own caches. The rule this pins is in `Authority`,
/// where `is_behind` matches rather than reading the revision through
/// `version()` — the accessor that deliberately answers a different question.
#[test]
fn a_local_rule_is_never_behind_anything() {
    let local = library(vec![rule(
        "R:x",
        "a class",
        Authority::local_at(Version::new(2)),
        0,
        &[],
    )]);
    let upstream = library(vec![rule(
        "R:x",
        "a class",
        Authority::local_at(Version::new(9)),
        0,
        &[],
    )]);
    assert!(
        with(
            &local,
            &upstream,
            &empty_aggregate(),
            &[Trigger::CacheBehind]
        )
        .is_empty()
    );
}

// ── the contribution trigger ────────────────────────────────────────────────

#[test]
fn a_contributed_rule_serving_your_audience_pokes_when_asked_for() {
    let local = library(vec![rule(
        "R:mine",
        "a class",
        Authority::local(),
        0,
        &["rust"],
    )]);
    let upstream = library(vec![
        rule("R:rusty", "another class", Authority::local(), 0, &["rust"]),
        rule("R:javan", "a third class", Authority::local(), 0, &["java"]),
    ]);

    let raised = with(
        &local,
        &upstream,
        &empty_aggregate(),
        &[Trigger::ContributedInAudience],
    );
    let tags: Vec<String> = raised
        .shown()
        .iter()
        .map(|p| match p {
            Poke::ContributedInAudience { tag, .. } => tag.as_str().to_owned(),
            other => panic!("expected a contribution poke, got {other:?}"),
        })
        .collect();
    assert_eq!(tags, ["R:rusty"], "a java rule is not this install's news");
}

// ── the population trigger ──────────────────────────────────────────────────

/// The k-floor protects reporters here exactly as it does in the published
/// document, because this reads the aggregate's rows rather than the reports —
/// one floor, not two policies.
#[test]
fn a_rule_below_the_floor_is_invisible_to_the_poke() {
    let local = library(vec![rule("R:mine", "a class", Authority::local(), 0, &[])]);
    let upstream = library(Vec::new());
    let thin = aggregate_of("R:popular", K_ANONYMITY_FLOOR - 1, "10+");
    assert!(with(&local, &upstream, &thin, &[Trigger::HighRecurrenceUnheld]).is_empty());

    let enough = aggregate_of("R:popular", K_ANONYMITY_FLOOR, "10+");
    let raised = with(&local, &upstream, &enough, &[Trigger::HighRecurrenceUnheld]);
    match &raised.shown()[0] {
        Poke::HighRecurrenceUnheld { tag, installs } => {
            assert_eq!(tag, "R:popular");
            assert_eq!(*installs, K_ANONYMITY_FLOOR);
        }
        other => panic!("expected a population poke, got {other:?}"),
    }
}

/// "High" is a published bucket boundary, not a threshold somebody tuned: a
/// corpus-wide count of installs in the middle bucket is not news.
#[test]
fn high_recurrence_means_the_top_bucket() {
    let local = library(vec![rule("R:mine", "a class", Authority::local(), 0, &[])]);
    let upstream = library(Vec::new());
    let middling = aggregate_of("R:popular", K_ANONYMITY_FLOOR + 3, "2-4");
    assert!(
        with(
            &local,
            &upstream,
            &middling,
            &[Trigger::HighRecurrenceUnheld]
        )
        .is_empty()
    );
}

// ── the defaults, and the cap ───────────────────────────────────────────────

/// **§6's table, observed rather than described.** With data present for all
/// four triggers and nothing asked for, exactly the two default-on triggers
/// fire — the reactive one, and the stale cache. The two that broadcast about
/// things nobody here has touched stay silent, which is the failure the
/// programme names first: *broadcast triggers default on, because they are
/// easier to demo*.
#[test]
fn only_the_default_triggers_fire_when_none_is_named() {
    let local = library(vec![
        rule("R:mine", "a shared class", Authority::local(), 1, &["rust"]),
        rule("R:cached", "a cached class", cached_at(1), 0, &[]),
    ]);
    let upstream = library(vec![
        rule(
            "R:theirs",
            "a shared class",
            Authority::local(),
            0,
            &["rust"],
        ),
        rule(
            "R:cached",
            "a cached class",
            Authority::local_at(Version::new(4)),
            0,
            &[],
        ),
    ]);
    let aggregate = aggregate_of("R:popular", K_ANONYMITY_FLOOR, "10+");

    let raised = pokes(
        &local,
        &upstream,
        &aggregate,
        &Trigger::defaults(),
        BroadcastCap::new(usize::MAX),
    );
    let triggers: Vec<Trigger> = raised.shown().iter().map(Poke::trigger).collect();
    assert_eq!(
        triggers,
        vec![Trigger::ClassCoveredUpstream, Trigger::CacheBehind],
        "the off-by-default triggers had data and must still have stayed quiet: {:?}",
        raised.shown()
    );

    // And they fire when they are asked for, so the silence above is a
    // decision rather than an absence of data.
    let all = pokes(
        &local,
        &upstream,
        &aggregate,
        &Trigger::ALL,
        BroadcastCap::new(usize::MAX),
    );
    assert_eq!(all.shown().len(), 4, "{:?}", all.shown());
}

#[test]
fn the_cap_withholds_broadcast_pokes_and_publishes_how_many() {
    let local = library(vec![rule("R:mine", "a class", Authority::local(), 0, &[])]);
    let upstream = library(vec![
        rule("R:a", "class a", Authority::local(), 0, &[]),
        rule("R:b", "class b", Authority::local(), 0, &[]),
        rule("R:c", "class c", Authority::local(), 0, &[]),
    ]);

    let raised = pokes(
        &local,
        &upstream,
        &empty_aggregate(),
        &[Trigger::ContributedInAudience],
        BroadcastCap::new(1),
    );
    assert_eq!(raised.shown().len(), 1);
    assert_eq!(raised.withheld(), 2);
}

/// A cap of zero silences broadcast entirely and leaves the reactive poke
/// alone — the shape of the failure the cap exists for, and the reason the cap
/// is not simply "how many pokes".
#[test]
fn the_cap_never_withholds_a_reactive_poke() {
    let local = library(vec![rule(
        "R:mine",
        "a shared class",
        Authority::local(),
        1,
        &[],
    )]);
    let upstream = library(vec![
        rule("R:theirs", "a shared class", Authority::local(), 0, &[]),
        rule("R:other", "another class", Authority::local(), 0, &[]),
    ]);

    let raised = pokes(
        &local,
        &upstream,
        &empty_aggregate(),
        &Trigger::ALL,
        BroadcastCap::new(0),
    );
    assert_eq!(raised.shown().len(), 1);
    assert_eq!(raised.shown()[0].reach(), Reach::Reactive);
    // Both upstream rules are withheld, and the pair is the documented
    // consequence of an install whose corpus declares no scope: its audience is
    // empty, `Rule::serves` reads that as "no filter", and every upstream rule
    // matches. Loud rather than wrong — and the reason this trigger is off
    // until asked for, and capped when it is not.
    assert_eq!(raised.withheld(), 2);
}

/// Reactive first, then broadcast by the rank the `Trigger` declaration states
/// — never an incidental sort of the rendered text
/// (`[R:order-by-explicit-rank]`).
#[test]
fn pokes_are_ordered_by_rank_then_tag() {
    let local = library(vec![
        rule("R:zebra", "a shared class", Authority::local(), 1, &[]),
        rule("R:cached", "a cached class", cached_at(1), 0, &[]),
    ]);
    let upstream = library(vec![
        rule("R:aardvark", "a shared class", Authority::local(), 0, &[]),
        rule(
            "R:cached",
            "a cached class",
            Authority::local_at(Version::new(9)),
            0,
            &[],
        ),
    ]);
    let raised = with(&local, &upstream, &empty_aggregate(), &Trigger::ALL);
    let order: Vec<Trigger> = raised.shown().iter().map(Poke::trigger).collect();
    assert_eq!(order[0], Trigger::ClassCoveredUpstream);
    assert!(order.windows(2).all(|w| w[0] <= w[1]), "{order:?}");
}

// ── what the poke must never do ─────────────────────────────────────────────

/// **No build behaves differently for having been poked.** The emitters and the
/// build path must not reference the poke at all — `lint` reports, `build`
/// emits, and mixing them means a build that talks. Read from the source,
/// because nothing else can see it: the failure arrives as a convenience.
///
/// The same shape as `tests/aggregate.rs`'s scan, and comments are stripped
/// before matching so a doc comment naming the module cannot turn the check
/// permanently red (`[R:detector-excludes-own-definitions]`).
#[test]
fn no_emitter_or_build_path_reads_the_poke() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut checked = 0;

    let mut scan = |path: std::path::PathBuf| {
        let source = fs::read_to_string(&path).expect("a readable source file");
        let code: String = source
            .lines()
            .map(|line| match line.find("//") {
                Some(at) => &line[..at],
                None => line,
            })
            .collect::<Vec<_>>()
            .join("\n");
        for marker in ["crate::poke", "poke::", "Poke"] {
            assert!(
                !code.contains(marker),
                "{} depends on the poke ({marker}); `lint` reports and `build` emits",
                path.display()
            );
        }
        checked += 1;
    };

    for path in ["src/emit.rs", "src/library.rs", "src/fsio.rs"] {
        scan(root.join(path));
    }
    for entry in fs::read_dir(root.join("src/emit")).expect("the emit directory") {
        scan(entry.expect("a readable entry").path());
    }
    assert!(
        checked >= 6,
        "only {checked} files checked — too few to mean anything"
    );
}

// ── through the real binary ─────────────────────────────────────────────────

/// A one-rule library, with `recurrence` spliced in verbatim when non-empty.
fn rule_file(dir: &Path, tag: &str, class: &str, extra: &str) {
    fs::create_dir_all(dir).expect("create the directory");
    let doc = format!(
        r#"+++
tag = "{tag}"
title = "A demo rule"
error_class = "{class}"
home = {{ kind = "global" }}
created = "2026-08-16"
origin = "mined"
status = {{ kind = "active" }}
incident = "The triggering incident."
{extra}+++

Do the thing.
"#
    );
    fs::write(dir.join(format!("{}.md", tag.replace("R:", ""))), doc).expect("write the rule");
}

const A_RECURRENCE: &str = r#"
[[recurrence]]
date = "2026-08-24"
incident = "It happened again."
"#;

fn lint(rules: &Path, extra: &[&str]) -> (bool, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_relearn"));
    cmd.arg("lint").arg("--rules").arg(rules);
    for arg in extra {
        cmd.arg(arg);
    }
    let out = cmd.output().expect("the relearn binary runs");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// **The load-bearing one, through the channel CI would run.** A poke is news
/// from strangers: it must reach the reader and must not touch the verdict. A
/// federated signal that can fail a run has made federation required, which
/// invariant 3 forbids — and the exit code is only observable from outside the
/// process (`[R:verify-through-production-path]`).
#[test]
fn a_poke_reaches_the_reader_and_never_changes_the_exit_code() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    rule_file(&rules, "R:mine", "a shared class", A_RECURRENCE);

    let clone = dir.path().join("clone");
    rule_file(&clone.join("rules"), "R:theirs", "a shared class", "");

    let (bare_ok, bare) = lint(&rules, &[]);
    let (poked_ok, poked) = lint(
        &rules,
        &["--upstream", clone.to_str().expect("a utf-8 path")],
    );

    assert_eq!(
        bare_ok, poked_ok,
        "the poke changed the verdict:\n{bare}\n---\n{poked}"
    );
    assert!(!bare.contains("poke"), "{bare}");
    assert!(poked.contains("R:theirs"), "{poked}");
    assert!(poked.contains("reactive"), "{poked}");
}

/// A run with a finding keeps failing, and says so, with a clone attached.
#[test]
fn a_failing_lint_still_fails_when_it_is_poked() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    rule_file(&rules, "R:mine", "a shared class", A_RECURRENCE);
    let clone = dir.path().join("clone");
    rule_file(&clone.join("rules"), "R:theirs", "a shared class", "");

    // An unheld recurrence is a Warning, and `--deny warning` is the default.
    let (ok, text) = lint(
        &rules,
        &["--upstream", clone.to_str().expect("a utf-8 path")],
    );
    assert!(!ok, "an unheld recurrence must still fail the run:\n{text}");
    assert!(text.contains("unheld recurrence"), "{text}");
    assert!(text.contains("poke"), "{text}");
}

/// No clone, no poke, and **no warning about its absence**: a solo install is
/// the product, and a nag is a requirement with better manners.
#[test]
fn without_a_clone_nothing_is_poked_and_nothing_is_said_about_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    rule_file(&rules, "R:mine", "a class", "");
    let (ok, text) = lint(&rules, &[]);
    assert!(ok, "{text}");
    assert!(!text.to_lowercase().contains("poke"), "{text}");
    assert!(!text.to_lowercase().contains("upstream"), "{text}");
}

/// A poke flag with no clone to read is a flag that does nothing, and a
/// silently inert flag reads as a feature that ran.
#[test]
fn a_poke_flag_without_a_clone_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    rule_file(&rules, "R:mine", "a class", "");
    let (ok, _) = lint(&rules, &["--poke", "cache-behind"]);
    assert!(!ok, "--poke without --upstream must be refused");
    let (ok, _) = lint(&rules, &["--poke-cap", "0"]);
    assert!(!ok, "--poke-cap without --upstream must be refused");
}

/// A clone that is not one — no `rules/`, no `reports/` — is an empty corpus
/// rather than an error: an aggregate repository nobody has contributed to yet
/// is a real state, and it is the state every new one starts in.
#[test]
fn an_empty_clone_is_quiet_rather_than_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    rule_file(&rules, "R:mine", "a class", A_RECURRENCE);
    let clone = dir.path().join("empty-clone");
    fs::create_dir_all(&clone).expect("create the clone");

    let (_, text) = lint(
        &rules,
        &["--upstream", clone.to_str().expect("a utf-8 path")],
    );
    assert!(!text.contains("poke ["), "{text}");
}
