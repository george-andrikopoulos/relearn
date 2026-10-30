//! A rule's `[[detector]]` tables: what parses, what is refused, and the laws
//! the evaluator obeys.
//!
//! Every test goes through `parse_document`, the perimeter, because a detector
//! is only ever constructed there — and because the guarantee under test is that
//! a detector whose fixtures do not discriminate, or disagree with its own
//! pattern, **does not parse at all** (`[R:evidence-must-discriminate]`).

use proptest::prelude::*;
use relearn::library::{Library, ValidationError};
use relearn::rule::{
    Detection, DetectorError, Glob, ParseError, Subject, parse_document, to_document,
};

/// A complete rule document around the given detector tables.
fn doc(detectors: &str) -> String {
    format!(
        "+++\ntag = \"R:synthetic\"\ntitle = \"t\"\nerror_class = \"e\"\n\
         home = {{ kind = \"global\" }}\ncreated = \"2026-10-30\"\norigin = \"mined\"\n\
         status = {{ kind = \"active\" }}\nincident = \"synthetic\"\n{detectors}+++\n\nBody.\n"
    )
}

/// A line-regex detector shaped like the unwrap pilot, with the given extra
/// fixtures appended after one hit and one clean.
fn unwrap_detector(extra: &str) -> String {
    format!(
        r#"
[[detector]]
name = "no-unwrap"
engine = "line-regex"
pattern = '\.unwrap[[:space:]]*\('
comment = "//"
allow = "allow:unwrap"
files = ["src/**/*.rs"]
exclude = ["**/tests/**", "**/*_tests.rs"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "Return the error with ? instead."

[[detector.fixture]]
path = "src/lib.rs"
input = 'let x = f().unwrap();'
expect = "hit"

[[detector.fixture]]
path = "src/lib.rs"
input = 'let x = f()?;'
expect = "clean"
{extra}"#
    )
}

fn detector_error(document: &str) -> DetectorError {
    match parse_document(document) {
        Err(ParseError::Detector { source, .. }) => source,
        other => panic!("expected a detector error, got {other:?}"),
    }
}

#[test]
fn a_detector_with_a_hit_and_a_clean_fixture_parses() {
    let rule = parse_document(&doc(&unwrap_detector(""))).expect("the pilot shape parses");
    assert_eq!(rule.detectors().len(), 1);
    assert_eq!(rule.detectors()[0].name().as_str(), "no-unwrap");
}

#[test]
fn the_evaluator_honours_scope_comments_and_the_override_token() {
    let rule = parse_document(&doc(&unwrap_detector(""))).expect("parses");
    let d = &rule.detectors()[0];
    let at = |path: &str, text: &str| d.evaluate(&Subject::new(path, text, &[]));

    assert_eq!(at("/abs/proj/src/a/b.rs", "f().unwrap()"), Detection::Hit);
    assert_eq!(at("/abs/proj/srcx/b.rs", "f().unwrap()"), Detection::Clean);
    assert_eq!(
        at("/abs/proj/src/tests/b.rs", "f().unwrap()"),
        Detection::Clean
    );
    assert_eq!(
        at("/abs/proj/src/foo_tests.rs", "f().unwrap()"),
        Detection::Clean
    );
    assert_eq!(
        at("/abs/proj/src/b.rs", "  // f().unwrap()"),
        Detection::Clean
    );
    assert_eq!(
        at("/abs/proj/src/b.rs", "f().unwrap(); // allow:unwrap"),
        Detection::Clean
    );
    assert_eq!(
        at("C:\\abs\\proj\\src\\b.rs", "ok\nf().unwrap ()\n"),
        Detection::Hit,
        "a Windows path is normalised before the scope is read"
    );
}

#[test]
fn a_detector_without_a_clean_fixture_does_not_parse() {
    let only_hit = doc(&unwrap_detector("")
        .replace("expect = \"clean\"", "expect = \"hit\"")
        .replace("input = 'let x = f()?;'", "input = 'g().unwrap()'"));
    assert_eq!(detector_error(&only_hit), DetectorError::NoCleanFixture);
}

#[test]
fn a_detector_without_a_hit_fixture_does_not_parse() {
    let only_clean = doc(&unwrap_detector("").replace(
        "input = 'let x = f().unwrap();'\nexpect = \"hit\"",
        "input = 'let y = 1;'\nexpect = \"clean\"",
    ));
    assert_eq!(detector_error(&only_clean), DetectorError::NoHitFixture);
}

#[test]
fn a_fixture_that_disagrees_with_the_pattern_stops_the_parse() {
    let wrong = doc(&unwrap_detector(
        "\n[[detector.fixture]]\npath = \"src/lib.rs\"\ninput = 'f().unwrap() // allow:unwrap'\nexpect = \"hit\"\n",
    ));
    match detector_error(&wrong) {
        DetectorError::FixtureDisagrees {
            index,
            expected,
            got,
            ..
        } => {
            assert_eq!(index, 3);
            assert_eq!(expected, Detection::Hit);
            assert_eq!(got, Detection::Clean);
        }
        other => panic!("expected FixtureDisagrees, got {other:?}"),
    }
}

#[test]
fn patterns_outside_the_shared_dialect_are_refused() {
    for bad in [
        r"\d+",        // a class grep -E does not read the same way
        r"(?i)unwrap", // inline flags
        r"a+?b",       // lazy quantifier
        r#"[^"\\]"#,   // backslash inside a bracket: literal to grep, escape to Rust
        "caf\u{e9}",   // non-ASCII
        "x*",          // matches the empty line, so every write would hit
        "",            // empty
    ] {
        let d = doc(&unwrap_detector("").replace(r"'\.unwrap[[:space:]]*\('", &format!("'{bad}'")));
        match detector_error(&d) {
            DetectorError::Pattern { .. } => {}
            other => panic!("pattern {bad:?}: expected a pattern error, got {other:?}"),
        }
    }
}

#[test]
fn a_named_but_unbuilt_event_is_refused_by_name() {
    let d = doc(&unwrap_detector("").replace(
        "events = [\"claude-code-pre-tool-use\"]",
        "events = [\"git-pre-commit\"]",
    ));
    assert_eq!(
        detector_error(&d),
        DetectorError::EventNotBuilt("git-pre-commit".to_owned())
    );
}

#[test]
fn a_glob_outside_the_closed_alphabet_is_refused() {
    let d = doc(&unwrap_detector("").replace("[\"src/**/*.rs\"]", "[\"src/[ab].rs\"]"));
    match detector_error(&d) {
        DetectorError::Glob { .. } => {}
        other => panic!("expected a glob error, got {other:?}"),
    }
}

const SIBLING_CASE: &str = r#"
[[detector]]
name = "case-sibling"
engine = "sibling-case"
files = ["**"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "Merge into the existing file or choose a distinct name."

[[detector.fixture]]
path = "docs/README.md"
siblings = ["readme.md"]
expect = "hit"

[[detector.fixture]]
path = "docs/NOTES.md"
siblings = ["readme.md"]
expect = "clean"

[[detector.fixture]]
path = "docs/readme.md"
siblings = ["readme.md"]
expect = "clean"
"#;

#[test]
fn the_sibling_case_engine_reads_names_not_text() {
    let rule = parse_document(&doc(SIBLING_CASE)).expect("parses");
    let d = &rule.detectors()[0];
    let siblings = ["Cargo.toml".to_owned()];
    assert_eq!(
        d.evaluate(&Subject::new("/x/cargo.toml", "", &siblings)),
        Detection::Hit
    );
    assert_eq!(
        d.evaluate(&Subject::new("/x/Cargo.toml", "cargo.toml", &siblings)),
        Detection::Clean,
        "an exact-name sibling is the same file, and the text is never read"
    );
}

#[test]
fn a_sibling_case_detector_refuses_a_pattern() {
    let d = doc(&SIBLING_CASE.replace(
        "engine = \"sibling-case\"",
        "engine = \"sibling-case\"\npattern = 'x'",
    ));
    assert_eq!(
        detector_error(&d),
        DetectorError::FieldNotUsed {
            engine: "sibling-case",
            field: "pattern"
        }
    );
}

#[test]
fn a_line_regex_fixture_with_siblings_is_refused() {
    let d = doc(&unwrap_detector(
        "\n[[detector.fixture]]\npath = \"src/lib.rs\"\nsiblings = [\"x\"]\nexpect = \"clean\"\n",
    ));
    assert_eq!(
        detector_error(&d),
        DetectorError::FixtureInputMismatch {
            index: 3,
            engine: "line-regex"
        }
    );
}

/// `[R:detector-excludes-own-definitions]`: a content detector never fires on
/// a rule file, generated pack, or installed hook — wherever the scope points.
#[test]
fn a_content_detector_never_reads_rule_files_or_generated_hooks() {
    let everything = unwrap_detector("")
        .replace("[\"src/**/*.rs\"]", "[\"**\"]")
        .replace("exclude = [\"**/tests/**\", \"**/*_tests.rs\"]\n", "");
    let rule = parse_document(&doc(&everything)).expect("parses");
    let d = &rule.detectors()[0];
    for path in [
        "/repo/rules/no-unwrap.md",
        "/repo/hooks-pack/hooks/no-unwrap.sh",
        "/home/u/.claude/hooks/relearn/no-unwrap.sh",
    ] {
        assert_eq!(
            d.evaluate(&Subject::new(path, "f().unwrap()", &[])),
            Detection::Clean,
            "{path}"
        );
    }
    assert_eq!(
        d.evaluate(&Subject::new("/repo/notes.txt", "f().unwrap()", &[])),
        Detection::Hit
    );
}

#[test]
fn a_detector_survives_a_serialize_parse_round_trip() {
    for tables in [unwrap_detector(""), SIBLING_CASE.to_owned()] {
        let rule = parse_document(&doc(&tables)).expect("parses");
        let rendered = to_document(&rule);
        let again = parse_document(&rendered).expect("the rendering parses");
        assert_eq!(again, rule);
        assert_eq!(to_document(&again), rendered, "rendering is a fixed point");
    }
}

#[test]
fn two_rules_may_not_name_the_same_detector() {
    let a = parse_document(&doc(&unwrap_detector(""))).expect("parses");
    let b = parse_document(&doc(&unwrap_detector("")).replace("R:synthetic", "R:synthetic-two"))
        .expect("parses");
    let mut library = Library::new();
    library.push(a);
    library.push(b);
    match library.validate() {
        Err(ValidationError::DuplicateDetector { name, .. }) => assert_eq!(name, "no-unwrap"),
        other => panic!("expected DuplicateDetector, got {other:?}"),
    }
}

/// A wildcard stays inside one path component. The property below rarely lines
/// a `?` or `*` up against a `/`, so a translation that let them cross one went
/// uncaught by it in a mutation probe; this pins the case it misses.
#[test]
fn wildcards_never_cross_a_separator() {
    for (glob, path) in [
        ("a?b", "x/a/b"),
        ("a*b", "x/a/b"),
        ("src/*.rs", "src/a/b.rs"),
    ] {
        let parsed = Glob::parse(glob).expect("a legal glob");
        assert!(!parsed.matches(path), "{glob} must not match {path}");
    }
    assert!(Glob::parse("a?b").expect("legal").matches("x/a.b"));
    assert!(
        Glob::parse("src/**/*.rs")
            .expect("legal")
            .matches("/abs/src/a/b.rs")
    );
}

/// The reference the compiled ERE is checked against: a direct recursive glob
/// match on path components, sharing no code with the translation.
fn reference(glob: &[&str], path: &[&str]) -> bool {
    match (glob.first(), path.first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some(&"**"), _) if glob.len() == 1 => !path.is_empty(),
        (Some(&"**"), _) => (0..=path.len()).any(|skip| reference(&glob[1..], &path[skip..])),
        (Some(_), None) => false,
        (Some(seg), Some(part)) => {
            segment(seg.as_bytes(), part.as_bytes()) && reference(&glob[1..], &path[1..])
        }
    }
}

fn segment(glob: &[u8], name: &[u8]) -> bool {
    match (glob.first(), name.first()) {
        (None, None) => true,
        (Some(b'*'), _) => (0..=name.len()).any(|skip| segment(&glob[1..], &name[skip..])),
        (Some(b'?'), Some(_)) => segment(&glob[1..], &name[1..]),
        (Some(g), Some(n)) if g == n => segment(&glob[1..], &name[1..]),
        _ => false,
    }
}

/// A glob matches a path when it matches some component-aligned suffix of it.
fn suffix_reference(glob: &str, path: &str) -> bool {
    let g: Vec<&str> = glob.split('/').collect();
    let p: Vec<&str> = path.split('/').collect();
    (0..p.len()).any(|start| reference(&g, &p[start..]))
}

proptest! {
    /// The ERE a glob compiles to — the one string both `relearn check` and the
    /// generated hook read — agrees with a direct glob matcher on every path.
    #[test]
    fn a_compiled_glob_agrees_with_a_direct_match(
        glob in prop::collection::vec(
            prop_oneof![Just("**".to_owned()), "[ab*?.]{1,3}"], 1..4
        ),
        path in prop::collection::vec("[ab.]{1,3}", 1..5),
    ) {
        // `**` is legal only as a whole segment, and is never doubled.
        prop_assume!(!glob.windows(2).any(|w| w[0] == "**" && w[1] == "**"));
        prop_assume!(glob.iter().all(|s| s == "**" || !s.contains("**")));
        let glob = glob.join("/");
        let path = path.join("/");
        let parsed = Glob::parse(&glob).expect("the generator only builds legal globs");
        prop_assert_eq!(parsed.matches(&path), suffix_reference(&glob, &path), "glob {} path {}", glob, path);
    }
}
