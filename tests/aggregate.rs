//! The aggregate: many installs' reports, recomputed into one document.
//!
//! Three properties the programme names as what goes wrong, each with a test
//! here. **The k-floor is applied** — no rule's counts publish until enough
//! distinct installs have reported it, and below the floor a count of one is a
//! finger pointing at someone. **The confound sentences print beside the
//! numbers** — cross-install recurrence measures frequency *and* diligence,
//! inseparably, and since the catalogue was dropped the aggregate counts only
//! classes somebody published a rule for; a headline figure without both is the
//! overclaim this project exists to prevent. And **the aggregate is never
//! authoritative**: no build behaves differently for having seen one, which is
//! asserted by reading the source rather than hoped for.
//!
//! The fourth — someone adding a fetch "for convenience" — is
//! `tests/solo_mode.rs`, which has held it since before any of this existed.

use relearn::aggregate::{Aggregate, AggregateError, K_ANONYMITY_FLOOR};
use relearn::report::Month;

/// A report document from one install, in the format `relearn report` writes.
fn report(install: &str, observations: &[(&str, &str, &str, Option<&str>)]) -> String {
    let mut out = format!("schema    = 1\ninstall   = \"{install}\"\ngenerated = \"2026-09\"\n");
    for (rule, bucket, status, control) in observations {
        out.push_str("\n[[observation]]\n");
        out.push_str(&format!("rule        = \"{rule}\"\n"));
        out.push_str(&format!("recurrences = \"{bucket}\"\n"));
        out.push_str("latest      = \"2026-08\"\n");
        out.push_str(&format!("status      = \"{status}\"\n"));
        if let Some(control) = control {
            out.push_str(&format!("control     = \"{control}\"\n"));
        }
    }
    out
}

/// `n` installs all reporting the same rule, with distinct pseudonyms.
fn installs_reporting(n: usize, rule: &str) -> Vec<String> {
    (0..n)
        .map(|i| {
            report(
                &format!("{i:08x}"),
                &[(rule, "2-4", "active", Some("hook"))],
            )
        })
        .collect()
}

fn aggregate_of(reports: &[String]) -> Aggregate {
    let month = Month::parse("2026-09").expect("a well-formed month");
    Aggregate::of(reports.iter().map(String::as_str), month).expect("well-formed reports")
}

// ── the k-floor ─────────────────────────────────────────────────────────────

/// Below the floor, nothing about the rule publishes — **not the count, not the
/// tag**. A row saying "one install hit this" is a finger pointing at someone,
/// and naming the rule while withholding the number would point just as well.
#[test]
fn below_the_floor_a_rule_does_not_appear_at_all() {
    for n in 1..K_ANONYMITY_FLOOR {
        let aggregate = aggregate_of(&installs_reporting(n, "R:x"));
        assert!(
            aggregate.rows().is_empty(),
            "{n} install(s) is below the floor of {K_ANONYMITY_FLOOR}"
        );
        assert!(
            !aggregate.to_toml().contains("R:x"),
            "the tag leaked while its count was withheld"
        );
    }
}

#[test]
fn at_the_floor_the_rule_publishes() {
    let aggregate = aggregate_of(&installs_reporting(K_ANONYMITY_FLOOR, "R:x"));
    assert_eq!(aggregate.rows().len(), 1);
    assert_eq!(aggregate.rows()[0].installs(), K_ANONYMITY_FLOOR);
    assert!(aggregate.to_toml().contains("R:x"));
}

/// **The reader is told how much was withheld, without being told what.** An
/// aggregate that silently omits its suppressed rules reads as complete, and a
/// reader cannot tell a corpus of five rules from one of five hundred with 495
/// below the floor.
#[test]
fn the_suppressed_count_is_published_but_never_the_tags() {
    let mut reports = installs_reporting(K_ANONYMITY_FLOOR, "R:published");
    reports.push(report("ffffffff", &[("R:too-few", "1", "active", None)]));
    let aggregate = aggregate_of(&reports);

    assert_eq!(aggregate.suppressed(), 1);
    let text = aggregate.to_toml();
    assert!(text.contains("suppressed"), "{text}");
    assert!(
        !text.contains("R:too-few"),
        "a suppressed tag leaked:\n{text}"
    );
}

/// One install reporting the same rule twice cannot lift it over the floor: the
/// floor counts **distinct installs**, and duplicate pseudonyms are the obvious
/// way to fake a population.
#[test]
fn one_install_cannot_reach_the_floor_by_repetition() {
    let same = report("7f3c9a1e", &[("R:x", "2-4", "active", None)]);
    let reports: Vec<String> = (0..K_ANONYMITY_FLOOR + 2).map(|_| same.clone()).collect();
    let aggregate = aggregate_of(&reports);
    assert!(aggregate.rows().is_empty());
}

// ── the confounds ───────────────────────────────────────────────────────────

/// **Both sentences, in every rendered aggregate, unconditionally.** They are
/// not a flag and not a verbosity level: a headline figure without them is the
/// overclaim this project exists to prevent, and the way they get dropped is by
/// being droppable.
#[test]
fn both_confounds_print_beside_the_numbers() {
    let aggregate = aggregate_of(&installs_reporting(K_ANONYMITY_FLOOR, "R:x"));
    let text = aggregate.to_toml();

    assert!(
        text.contains("diligence"),
        "the frequency-and-diligence confound is missing:\n{text}"
    );
    assert!(
        text.contains("published a rule for"),
        "the published-rule confound is missing:\n{text}"
    );
}

/// An **empty** aggregate still carries them. This is the case where dropping
/// them is most tempting and most misleading: a document with no rows reads as
/// "nothing recurs anywhere" unless it says what it cannot see.
#[test]
fn an_empty_aggregate_still_carries_its_confounds() {
    let aggregate = aggregate_of(&[]);
    assert!(aggregate.rows().is_empty());
    let text = aggregate.to_toml();
    assert!(text.contains("diligence"), "{text}");
    assert!(text.contains("published a rule for"), "{text}");
}

// ── what an aggregate carries ───────────────────────────────────────────────

#[test]
fn a_row_carries_the_distribution_and_the_control_kinds_seen() {
    let reports = vec![
        report("00000001", &[("R:x", "1", "active", None)]),
        report("00000002", &[("R:x", "2-4", "graduated", Some("hook"))]),
        report("00000003", &[("R:x", "2-4", "graduated", Some("gate"))]),
        report("00000004", &[("R:x", "10+", "graduated", Some("hook"))]),
        report("00000005", &[("R:x", "5-9", "attic", None)]),
    ];
    let aggregate = aggregate_of(&reports);
    let row = &aggregate.rows()[0];

    assert_eq!(row.installs(), 5);
    assert_eq!(row.in_bucket("2-4"), 2);
    assert_eq!(row.in_bucket("1"), 1);
    assert_eq!(row.controls(), ["gate", "hook"]);
}

/// No day-level date survives aggregation either — the inputs carry months, and
/// nothing here adds a day back.
#[test]
fn no_day_level_date_appears_anywhere_in_an_aggregate() {
    let text = aggregate_of(&installs_reporting(K_ANONYMITY_FLOOR, "R:x")).to_toml();
    let day_shaped = text
        .split(|c: char| !(c.is_ascii_digit() || c == '-'))
        .any(|token| {
            let parts: Vec<&str> = token.split('-').collect();
            parts.len() == 3 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
        });
    assert!(!day_shaped, "a day-level date appeared:\n{text}");
}

// ── untrusted input ─────────────────────────────────────────────────────────

/// Reports come from strangers. A schema version this build does not understand
/// is refused rather than read optimistically: silently ignoring fields it
/// cannot interpret is how an aggregate miscounts without anyone noticing.
#[test]
fn an_unknown_schema_version_is_refused() {
    let future = "schema = 2\ninstall = \"7f3c9a1e\"\ngenerated = \"2026-09\"\n";
    let month = Month::parse("2026-09").expect("valid month");
    assert!(matches!(
        Aggregate::of([future].into_iter(), month),
        Err(AggregateError::UnknownSchema { .. })
    ));
}

#[test]
fn a_malformed_install_id_is_refused() {
    let bad = "schema = 1\ninstall = \"george-laptop\"\ngenerated = \"2026-09\"\n";
    let month = Month::parse("2026-09").expect("valid month");
    assert!(Aggregate::of([bad].into_iter(), month).is_err());
}

// ── the aggregate is never authoritative ────────────────────────────────────

/// **No build behaves differently for having seen an aggregate.** The emitters
/// and the build path must not reference the aggregate at all — not to annotate,
/// not to order, not to filter. Read from the source, because nothing else can
/// see it: a build that quietly consults a downloaded file is exactly the
/// failure the programme names, and it would arrive as an improvement.
#[test]
fn no_emitter_or_build_path_reads_the_aggregate() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut checked = 0;

    let mut scan = |path: std::path::PathBuf| {
        let source = std::fs::read_to_string(&path).expect("a readable source file");
        // Comments stripped, and the markers are what a *dependency* looks like
        // rather than the English word: a test named
        // `home_skill_aggregates_its_rules_sorted_by_tag` is not a build
        // consulting a downloaded file, and a detector that cannot tell the
        // difference gets muted (`[R:detector-excludes-own-definitions]`).
        let code: String = source
            .lines()
            .map(|line| match line.find("//") {
                Some(at) => &line[..at],
                None => line,
            })
            .collect::<Vec<_>>()
            .join("\n");
        for marker in ["crate::aggregate", "aggregate::", "Aggregate"] {
            assert!(
                !code.contains(marker),
                "{} depends on the aggregate ({marker}); a build must never behave \
                 differently for having seen one",
                path.display()
            );
        }
        checked += 1;
    };

    for path in ["src/emit.rs", "src/library.rs", "src/fsio.rs"] {
        scan(root.join(path));
    }
    for entry in std::fs::read_dir(root.join("src/emit")).expect("the emit directory") {
        scan(entry.expect("a readable entry").path());
    }
    assert!(
        checked >= 6,
        "only {checked} files checked — too few to mean anything"
    );
}
