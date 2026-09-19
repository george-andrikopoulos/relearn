//! A codified rule may name the artefact it was written down from, and only a
//! codified rule may.
//!
//! **Why this exists.** `Origin::Codified` carried no payload until 2026-09-18,
//! on the stated ground that "a practice written down from standing doctrine is
//! meaningful without naming a document". That holds for a rule ported out of
//! the author's own always-loaded instruction file, which is where every
//! codified rule in the corpus had come from. It stopped holding the moment a
//! rule was codified from **someone else's** published artefact — a paper, a
//! specification, an algorithm's original page — because there the document is
//! not incidental to the provenance, it *is* the provenance, and a reader who
//! cannot reach it cannot check the practice against the thing that defines it.
//! `[R:source-practice-from-its-artefact]`
//!
//! **What the shape has to hold.** Three things, and the third is the one that
//! needed a type rather than a convention:
//!
//! * A codified rule may carry a source, and round-trips with it.
//! * A codified rule may omit one. Twenty-two rule files predate the field and
//!   none of them needs editing — the absent case stays the common one.
//! * A source on a rule that is **not** codified is refused, not ignored. This
//!   mirrors `ApprovalWithoutMandate` and for the same reason: a silently
//!   dropped source reads, to the next person, as a rule that cited its
//!   artefact when it did not. Provenance that can evaporate is worse than
//!   provenance that was never claimed.
//!
//! The mined case is the one that matters in practice. A mined rule's
//! provenance is its incident, and a `source` beside it is either a mistake or
//! a rule whose origin was mislabelled — both of which are worth stopping the
//! build for.

use relearn::rule::{Origin, OriginError, ParseError, parse_document, to_document};

/// A rule document with the given `origin` line and an optional `source` line,
/// so each test states exactly the pair it is about.
fn doc(origin: &str, source: Option<&str>) -> String {
    let mut front = String::from("+++\n");
    front.push_str("tag = \"R:x\"\n");
    front.push_str("title = \"t\"\n");
    front.push_str("error_class = \"e\"\n");
    front.push_str("home = { kind = \"global\" }\n");
    front.push_str("created = \"2026-01-01\"\n");
    front.push_str(&format!("origin = {origin}\n"));
    if let Some(s) = source {
        front.push_str(&format!("source = {s}\n"));
    }
    front.push_str("status = { kind = \"active\" }\n");
    front.push_str("incident = \"i\"\n");
    front.push_str("+++\n\nBody.\n");
    front
}

#[test]
fn a_codified_rule_carries_the_artefact_it_was_written_down_from() {
    let rule = parse_document(&doc(
        "\"codified\"",
        Some("\"Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores\""),
    ))
    .expect("a codified rule may name its source");

    let source = rule.origin().source().expect("the source survives parsing");
    assert_eq!(
        source.as_str(),
        "Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores"
    );
}

/// The absent case is the common one and must stay free: twenty-two rule files
/// predate the field.
#[test]
fn a_codified_rule_without_a_source_still_parses() {
    let rule = parse_document(&doc("\"codified\"", None)).expect("a source is optional");
    assert_eq!(rule.origin(), &Origin::Codified(None));
    assert!(rule.origin().source().is_none());
}

/// Round-trip, because a provenance field that parses and does not serialize is
/// a provenance field that disappears on the next `adopt` or `contribute`.
#[test]
fn the_source_survives_a_round_trip() {
    let original = parse_document(&doc("\"codified\"", Some("\"a paper, section 4\"")))
        .expect("valid document");
    let rendered = to_document(&original);
    let reparsed = parse_document(&rendered).expect("the rendered document parses");

    assert_eq!(original.origin(), reparsed.origin());
    assert!(
        rendered.contains("source = \"a paper, section 4\""),
        "the source reaches the neutral format: {rendered}"
    );
}

/// A rule with no source renders no `source` line — the field comes from
/// matching the payload, not from an always-written empty string.
#[test]
fn a_rule_without_a_source_renders_no_source_line() {
    let rule = parse_document(&doc("\"codified\"", None)).expect("valid document");
    let rendered = to_document(&rule);
    assert!(!rendered.contains("source ="), "{rendered}");
}

/// A mined rule's provenance is its incident. A `source` beside it is a
/// mislabelled origin, and is refused rather than dropped.
#[test]
fn a_source_on_a_mined_rule_stops_the_build() {
    let err = parse_document(&doc("\"mined\"", Some("\"a paper\"")))
        .expect_err("a mined rule may not name a source");
    assert_eq!(
        err,
        ParseError::Origin(OriginError::SourceWithoutCodification("mined".to_owned()))
    );
}

/// A mandate's provenance is its approval, which is already a required payload.
#[test]
fn a_source_on_a_mandated_rule_stops_the_build() {
    let mut d = doc("\"mandated\"", Some("\"a paper\""));
    d = d.replace(
        "incident = \"i\"\n",
        "incident = \"i\"\napproval = { by = \"b\", date = \"2026-07-11\", control = \"c\" }\n",
    );
    let err = parse_document(&d).expect_err("a mandated rule may not name a source");
    assert_eq!(
        err,
        ParseError::Origin(OriginError::SourceWithoutCodification(
            "mandated".to_owned()
        ))
    );
}

/// Present-but-blank is the state the newtype exists to refuse: it reads as a
/// cited artefact in every listing and resolves to nothing.
#[test]
fn a_blank_source_stops_the_build() {
    let err = parse_document(&doc("\"codified\"", Some("\"   \"")))
        .expect_err("a blank source is not a source");
    assert!(
        matches!(err, ParseError::Text(_)),
        "expected an empty-text error naming the field, got {err:?}"
    );
}
