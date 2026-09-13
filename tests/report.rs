//! The recurrence report: anonymous, always.
//!
//! What leaves is an upstream tag, a **bucketed** count, a month, a status kind
//! and a control kind. The field list is defined as much by what is absent —
//! no title, no incident, no body, no path, no name, no repository, no
//! language, and **no day-level date anywhere**.
//!
//! Four things the programme names as what goes wrong, each with a test here:
//! ambient state (held by `tests/solo_mode.rs`, which since B1 also forbids
//! clocks), exact counts at small n, a day-level date surviving somewhere, and
//! a local-only rule being reported — a local tag is a private name, and
//! publishing one is the leak this whole flow is shaped to avoid.

use relearn::library::Library;
use relearn::report::{Bucket, Control, InstallId, K_ANONYMITY_FLOOR, Month, Report};
use relearn::rule::{
    Approval, Approver, Authority, Body, ControlRef, Date, ErrorClass, Home, Incident, Origin,
    Recurrence, Rule, RuleTag, SourceId, Status, Title, Version,
};

fn upstream() -> Authority {
    Authority::cached(
        SourceId::parse("relearn-upstream").expect("valid source"),
        Version::new(3),
        Date::parse("2026-09-10").expect("valid date"),
    )
}

fn recurrences(dates: &[&str]) -> Vec<Recurrence> {
    dates
        .iter()
        .map(|d| {
            Recurrence::new(
                Date::parse(d).expect("valid date"),
                Incident::parse("it happened again in C:/Users/ganak/relearn, said George")
                    .expect("non-empty"),
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn rule(
    tag: &str,
    home: Home,
    origin: Origin,
    authority: Authority,
    status: Status,
    dates: &[&str],
) -> Rule {
    Rule::new(
        RuleTag::parse(tag).expect("valid tag"),
        Title::parse("A title nobody upstream needs").expect("non-empty title"),
        ErrorClass::parse("an error class").expect("non-empty error class"),
        home,
        Date::parse("2026-09-01").expect("valid date"),
        origin,
        status,
        Incident::parse("George said it in the session at C:/Users/ganak").expect("non-empty"),
        Body::parse("The body, which also never travels in a report.").expect("non-empty body"),
        recurrences(dates),
        Vec::new(),
        authority,
        None,
    )
}

fn install() -> InstallId {
    InstallId::parse("7f3c9a1e").expect("a well-formed pseudonym")
}

fn generated() -> Month {
    Month::parse("2026-09").expect("a well-formed month")
}

fn report_of(rules: Vec<Rule>) -> Report {
    let library = Library::from_rules(rules)
        .validate()
        .expect("distinct tags validate");
    Report::of(&library, install(), generated())
}

// ── nothing identifying leaves ──────────────────────────────────────────────

/// **No day-level date, anywhere in the document.** The programme names this as
/// real and easy to miss: a day can survive in `latest`, in a filename, or in a
/// field nobody re-read. Asserted over the whole rendered text rather than over
/// the fields somebody remembered to check.
#[test]
fn no_day_level_date_appears_anywhere_in_a_report() {
    let report = report_of(vec![rule(
        "R:x",
        Home::global(),
        Origin::Mined,
        upstream(),
        Status::active(),
        &["2026-08-24", "2026-08-30"],
    )]);
    let text = report.to_toml();

    let day_shaped = text
        .split(|c: char| !(c.is_ascii_digit() || c == '-'))
        .any(|token| {
            let parts: Vec<&str> = token.split('-').collect();
            parts.len() == 3 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
        });
    assert!(
        !day_shaped,
        "a day-level date survived into the report:\n{text}"
    );
    assert!(text.contains("2026-08"), "the month must survive:\n{text}");
}

/// The incident text — the rule's and every recurrence's — is a quotation from a
/// private session. None of it is in a report, and neither is the title or body.
#[test]
fn no_authored_text_of_any_kind_reaches_a_report() {
    let report = report_of(vec![rule(
        "R:x",
        Home::global(),
        Origin::Mined,
        upstream(),
        Status::active(),
        &["2026-08-24"],
    )]);
    let text = report.to_toml();

    for private in [
        "George",
        "ganak",
        "C:/Users",
        "A title nobody upstream needs",
        "The body",
        "an error class",
    ] {
        assert!(
            !text.contains(private),
            "private text reached the report: {private}\n{text}"
        );
    }
}

// ── who may be reported ─────────────────────────────────────────────────────

/// **Only upstream tags.** A local-only rule has no shared identity, so its tag
/// is a private name and publishing one is the leak. B1's `Authority` is what
/// answers the question: a cache or a fork came from upstream; a local rule did
/// not.
#[test]
fn a_local_only_rule_is_never_reported() {
    let report = report_of(vec![
        rule(
            "R:local",
            Home::global(),
            Origin::Mined,
            Authority::Local,
            Status::active(),
            &["2026-08-24"],
        ),
        rule(
            "R:upstream",
            Home::global(),
            Origin::Mined,
            upstream(),
            Status::active(),
            &["2026-08-24"],
        ),
    ]);
    let tags: Vec<&str> = report
        .observations()
        .iter()
        .map(|o| o.rule().as_str())
        .collect();
    assert_eq!(tags, vec!["R:upstream"]);
}

/// A2's exclusion in its **second** flow: an org-homed rule never appears in a
/// recurrence report, exactly as it can never be contributed.
#[test]
fn an_org_homed_rule_never_appears_in_a_report() {
    let report = report_of(vec![rule(
        "R:org",
        Home::org("acme").expect("non-empty org"),
        Origin::Mined,
        upstream(),
        Status::active(),
        &["2026-08-24"],
    )]);
    assert!(report.observations().is_empty());
}

/// A mandate is not recurrence evidence, in this flow as in the local tally.
#[test]
fn a_mandated_rule_is_never_reported() {
    let report = report_of(vec![rule(
        "R:m",
        Home::global(),
        Origin::Mandated(Approval::new(
            Approver::parse("the change board").expect("non-empty approver"),
            Date::parse("2026-07-11").expect("valid date"),
            ControlRef::parse("AC-6(9)").expect("non-empty control"),
        )),
        upstream(),
        Status::active(),
        &["2026-08-24"],
    )]);
    assert!(report.observations().is_empty());
}

/// A rule that has not recurred has nothing to observe. Reporting a zero for
/// every cached rule would publish the shape of the whole local corpus.
#[test]
fn a_rule_that_has_not_recurred_is_not_an_observation() {
    let report = report_of(vec![rule(
        "R:quiet",
        Home::global(),
        Origin::Mined,
        upstream(),
        Status::active(),
        &[],
    )]);
    assert!(report.observations().is_empty());
}

// ── buckets and constants ───────────────────────────────────────────────────

#[test]
fn counts_publish_as_buckets_with_published_boundaries() {
    assert_eq!(Bucket::of(1).as_str(), "1");
    assert_eq!(Bucket::of(2).as_str(), "2-4");
    assert_eq!(Bucket::of(4).as_str(), "2-4");
    assert_eq!(Bucket::of(5).as_str(), "5-9");
    assert_eq!(Bucket::of(9).as_str(), "5-9");
    assert_eq!(Bucket::of(10).as_str(), "10+");
    assert_eq!(Bucket::of(9_999).as_str(), "10+");
}

/// The k-floor is a **published constant a reader can check**, not a policy
/// anyone has to trust. The aggregate applies it; the constant lives here so
/// both sides read one number.
#[test]
fn the_k_anonymity_floor_is_a_readable_constant() {
    assert_eq!(K_ANONYMITY_FLOOR, 5);
}

/// The control kind is a **sealed** vocabulary: five values, no free-text
/// variant, and never the control's own name. `gate:internal-payments-lint`
/// publishes `gate` and nothing else.
#[test]
fn a_control_kind_never_carries_the_controls_name() {
    let graduated = |to: &str| {
        Status::graduated(to, Date::parse("2026-07-21").expect("valid date"))
            .expect("non-empty destination")
    };
    assert_eq!(
        Control::of(&graduated("hook:internal-payments-lint")),
        Some(Control::Hook)
    );
    assert_eq!(
        Control::of(&graduated("test:some_test")),
        Some(Control::UnitTest)
    );
    assert_eq!(
        Control::of(&graduated("gate:verify.sh")),
        Some(Control::Gate)
    );
    assert_eq!(
        Control::of(&graduated("type:SomeWitness")),
        Some(Control::Type)
    );
    assert_eq!(
        Control::of(&graduated("property:some_law")),
        Some(Control::PropertyTest)
    );
    // Active and atticked rules have no control.
    assert_eq!(Control::of(&Status::active()), None);

    let report = report_of(vec![rule(
        "R:g",
        Home::global(),
        Origin::Mined,
        upstream(),
        graduated("hook:internal-payments-lint"),
        &["2026-08-24"],
    )]);
    let text = report.to_toml();
    assert!(
        text.lines()
            .any(|l| l.starts_with("control") && l.ends_with("\"hook\"")),
        "{text}"
    );
    assert!(
        !text.contains("internal-payments-lint"),
        "the control's name reached the report:\n{text}"
    );
}

/// A destination naming two kinds cannot be represented by a single-valued
/// field, so the field is omitted rather than a kind being chosen for it.
#[test]
fn a_destination_naming_two_kinds_reports_no_control() {
    let status = Status::graduated(
        "test:font_tests.rs + gate:verify.sh emoji_ban",
        Date::parse("2026-08-16").expect("valid date"),
    )
    .expect("non-empty destination");
    assert_eq!(Control::of(&status), None);
}

// ── the pseudonym ───────────────────────────────────────────────────────────

/// The install id is a **pseudonym by construction**: eight lowercase hex
/// characters, so a name-shaped id cannot be typed in the first place.
#[test]
fn an_install_id_cannot_be_name_shaped() {
    assert!(InstallId::parse("7f3c9a1e").is_ok());
    for bad in [
        "george-laptop",
        "GEORGE",
        "7F3C9A1E",
        "7f3c9a1",
        "7f3c9a1ee",
        "",
        "zzzzzzzz",
    ] {
        assert!(
            InstallId::parse(bad).is_err(),
            "{bad} must not be a valid install id"
        );
    }
}
