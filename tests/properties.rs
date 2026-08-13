//! Property tests over generated rule sets (rust-typedd tier 2). These exercise
//! the whole public pipeline — parse/serialize round-trip and emitter
//! determinism — with `proptest`, which types alone cannot express: "for all
//! rules, serialize-then-parse is the identity" and "for all libraries, every
//! emitter is a deterministic function".

use proptest::prelude::*;

use relearn::emit;
use relearn::library::Library;
use relearn::rule::{
    Body, Date, ErrorClass, Home, Incident, Rule, RuleTag, Status, Title, parse_document,
    to_document,
};

/// Non-empty, edge-trimmed text (parsing trims, so generated values must have no
/// leading/trailing whitespace to round-trip). Includes `"` to exercise the
/// serializer's TOML escaping under proptest.
fn arb_text() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 .,()\"-]{1,40}"
        .prop_map(|s| s.trim().to_owned())
        .prop_filter("non-empty after trim", |s| !s.is_empty())
}

/// A well-shaped tag body (`R:` is added by the caller).
fn arb_tag_body() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-z0-9][a-z0-9-]{0,20}").expect("valid tag-body regex")
}

/// A valid calendar date, generated safely (day <= 28 so every month is valid).
fn arb_date() -> impl Strategy<Value = Date> {
    (2000i32..=2100, 1u32..=12, 1u32..=28).prop_map(|(y, m, d)| {
        Date::parse(&format!("{y:04}-{m:02}-{d:02}")).expect("generated date is valid")
    })
}

fn arb_home() -> impl Strategy<Value = Home> {
    prop_oneof![
        Just(Home::global()),
        arb_text().prop_map(|n| Home::domain(n).expect("non-empty domain")),
        arb_text().prop_map(|p| Home::project(p).expect("non-empty project")),
    ]
}

fn arb_status() -> impl Strategy<Value = Status> {
    prop_oneof![
        Just(Status::active()),
        arb_text().prop_map(|t| Status::graduated(t).expect("non-empty destination")),
        (arb_text(), arb_date()).prop_map(|(r, d)| Status::attic(r, d).expect("non-empty reason")),
    ]
}

/// A fully-arbitrary rule with the given tag body.
fn arb_rule(tag_body: String) -> impl Strategy<Value = Rule> {
    (
        arb_text(),
        arb_text(),
        arb_home(),
        arb_date(),
        arb_status(),
        arb_text(),
        arb_text(),
    )
        .prop_map(
            move |(title, error_class, home, created, status, incident, body)| {
                Rule::new(
                    RuleTag::parse(format!("R:{tag_body}")).expect("valid tag"),
                    Title::parse(title).expect("non-empty title"),
                    ErrorClass::parse(error_class).expect("non-empty error class"),
                    home,
                    created,
                    status,
                    Incident::parse(incident).expect("non-empty incident"),
                    Body::parse(body).expect("non-empty body"),
                )
            },
        )
}

/// A validated library of up to five rules with distinct tags and varied homes.
fn arb_library() -> impl Strategy<Value = Library<relearn::library::Validated>> {
    proptest::collection::vec((arb_tag_body(), arb_home(), arb_text()), 0..5).prop_map(|items| {
        let mut seen = std::collections::BTreeSet::new();
        let mut rules = Vec::new();
        for (tag_body, home, text) in items {
            if !seen.insert(tag_body.clone()) {
                continue; // keep tags distinct so validation succeeds
            }
            rules.push(Rule::new(
                RuleTag::parse(format!("R:{tag_body}")).expect("valid tag"),
                Title::parse(text).expect("non-empty title"),
                ErrorClass::parse("error class").expect("non-empty error class"),
                home,
                Date::parse("2026-08-13").expect("valid date"),
                Status::active(),
                Incident::parse("incident").expect("non-empty incident"),
                Body::parse("body").expect("non-empty body"),
            ));
        }
        Library::from_rules(rules)
            .validate()
            .expect("distinct tags validate")
    })
}

proptest! {
    /// Serialize-then-parse is the identity: the neutral format is lossless over
    /// tag, title, error class, home, created date, status (with payload), and
    /// body — including quotes that must be TOML-escaped.
    #[test]
    fn neutral_round_trip_preserves_the_rule(rule in arb_tag_body().prop_flat_map(arb_rule)) {
        let doc = to_document(&rule);
        let reparsed = parse_document(&doc).expect("a serialized rule must re-parse");
        prop_assert_eq!(reparsed, rule);
    }

    /// Every emitter is a deterministic function of the library: re-emitting the
    /// same library yields byte-identical output. This is the property the
    /// overwrite guard and "rebuild is a no-op" both rely on.
    #[test]
    fn emission_is_idempotent(lib in arb_library()) {
        prop_assert_eq!(emit::claude::emit(&lib), emit::claude::emit(&lib));
        prop_assert_eq!(emit::cursor::emit(&lib), emit::cursor::emit(&lib));
        prop_assert_eq!(emit::copilot::emit(&lib), emit::copilot::emit(&lib));
        prop_assert_eq!(emit::agents::emit(&lib), emit::agents::emit(&lib));
        prop_assert_eq!(emit::claude_md::emit(&lib), emit::claude_md::emit(&lib));
    }
}
