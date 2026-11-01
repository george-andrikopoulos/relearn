//! Repository gates: `[[detector]]` tables whose engine reads a whole checkout
//! rather than one write (ARCHITECTURE.md, "Proposed 2026-10-31: detector
//! engines", decision 3, approved 2026-11-01).
//!
//! A gate's fixtures are **records**, the lines its generated script's collect
//! stage emits and its judge stage reads, so they run at parse exactly as a
//! write detector's do. These tests pin the parse refusals, each engine's
//! judgement, the laws the judgement must obey, and the shape of the pack.

use proptest::prelude::*;
use relearn::emit::repo;
use relearn::library::{Library, Validated};
use relearn::rule::{Detection, Gate, parse_document};

/// A rule whose only table is `detector`, a TOML fragment.
fn document(detector: &str) -> String {
    format!(
        r#"+++
tag = "R:synthetic-gate"
title = "t"
error_class = "e"
home = {{ kind = "global" }}
created = "2026-11-01"
origin = "mined"
status = {{ kind = "active" }}
incident = "synthetic"
{detector}
+++

Body.
"#
    )
}

fn parse_err(detector: &str) -> String {
    match parse_document(&document(detector)) {
        Ok(_) => panic!("expected a refusal:\n{detector}"),
        Err(e) => e.to_string(),
    }
}

fn gate(detector: &str) -> Gate {
    let rule = parse_document(&document(detector)).expect("the synthetic gate parses");
    assert!(
        rule.detectors().is_empty(),
        "a gate is not a write detector"
    );
    rule.gates()
        .first()
        .cloned() // allow:clone: the test owns one gate beyond the parsed rule's lifetime
        .expect("one gate")
}

fn library(detector: &str) -> Library<Validated> {
    let rule = parse_document(&document(detector)).expect("parses");
    let mut library = Library::new();
    library.push(rule);
    library.validate().expect("validates")
}

const EOL: &str = r#"
[[detector]]
name = "eol-pinned"
engine = "eol-attribute"
files = ["**"]
always = ["generated/**"]
events = ["repository-gate"]
action = "block"
message = "Pin it in .gitattributes."

[[detector.fixture]]
records = ["bin/run\t#!/bin/sh\tunspecified\tunspecified"]
expect = "hit"

[[detector.fixture]]
records = ["bin/run\t#!/bin/sh\tset\tlf", "README.md\t# Title\tunspecified\tunspecified"]
expect = "clean"
"#;

const ROOT_DOCS: &str = r#"
[[detector]]
name = "root-docs"
engine = "root-documents"
files = ["*.md"]
exclude = ["README.md", "CLAUDE.md", "ARCHITECTURE.md", "FEATURES.md", "TODO.md"]
events = ["repository-gate"]
action = "block"
message = "Fold it into one of the five."

[[detector.fixture]]
records = ["README.md", "NOTES.md"]
expect = "hit"

[[detector.fixture]]
records = ["README.md", "docs/notes.md"]
expect = "clean"
"#;

const LINT: &str = r#"
[[detector]]
name = "lint-on"
engine = "lint-enabled"
lint = "clippy::needless_pass_by_value"
groups = ["clippy::pedantic"]
default = "allow"
files = ["Cargo.toml", "src/lib.rs", "src/main.rs"]
events = ["repository-gate"]
action = "block"
message = "Enable the lint."

[[detector.fixture]]
records = ["Cargo.toml\tclippy::all\twarn"]
expect = "hit"

[[detector.fixture]]
records = ["Cargo.toml\tclippy::pedantic\twarn"]
expect = "clean"
"#;

// ---------------------------------------------------------------------------
// Parse.
// ---------------------------------------------------------------------------

#[test]
fn each_gate_engine_parses_as_a_gate_and_not_a_write_detector() {
    for (src, engine) in [
        (EOL, "eol-attribute"),
        (ROOT_DOCS, "root-documents"),
        (LINT, "lint-enabled"),
    ] {
        assert_eq!(gate(src).engine().as_str(), engine);
    }
}

#[test]
fn a_gate_fixture_that_disagrees_stops_the_parse() {
    // A third fixture claiming an unpinned shebang is clean.
    let wrong = format!(
        "{EOL}\n[[detector.fixture]]\nrecords = [\"bin/x\\t#!/bin/sh\\tunspecified\\tunspecified\"]\nexpect = \"clean\"\n"
    );
    let err = parse_err(&wrong);
    assert!(err.contains("fixture 3"), "{err}");
    assert!(err.contains("expects clean"), "{err}");
}

#[test]
fn a_gate_needs_a_hit_and_a_clean_fixture() {
    let only_hit = &EOL[..EOL.rfind("[[detector.fixture]]").expect("two fixtures")];
    assert!(parse_err(only_hit).contains("no fixture expects `clean`"));
}

#[test]
fn a_gate_engine_runs_only_at_the_repository_gate_event() {
    let at_write = EOL.replace(
        "events = [\"repository-gate\"]",
        "events = [\"claude-code-pre-tool-use\"]",
    );
    let err = parse_err(&at_write);
    assert!(err.contains("repository-gate"), "{err}");
}

#[test]
fn a_write_engine_does_not_run_at_the_repository_gate_event() {
    let err = parse_err(
        r#"
[[detector]]
name = "w"
engine = "line-regex"
pattern = 'x'
files = ["**"]
events = ["repository-gate"]
action = "block"
message = "m"

[[detector.fixture]]
path = "a"
input = 'x'
expect = "hit"

[[detector.fixture]]
path = "a"
input = 'y'
expect = "clean"
"#,
    );
    assert!(err.contains("repository-gate"), "{err}");
}

#[test]
fn a_gate_fixture_takes_records_and_no_path() {
    let with_path = EOL.replacen("records = [", "path = \"x\"\nrecords = [", 1);
    assert!(parse_err(&with_path).contains("records"));
}

#[test]
fn a_record_with_the_wrong_field_count_is_refused() {
    let short = EOL.replacen(
        "bin/run\\t#!/bin/sh\\tunspecified\\tunspecified",
        "bin/run\\tunspecified",
        1,
    );
    let err = parse_err(&short);
    assert!(err.contains("record"), "{err}");
}

#[test]
fn an_engine_field_on_the_wrong_engine_is_refused() {
    let stray = ROOT_DOCS.replacen("files = [", "always = [\"x\"]\nfiles = [", 1);
    assert!(parse_err(&stray).contains("always"));
}

#[test]
fn a_lint_name_outside_the_lint_alphabet_is_refused() {
    let bad = LINT.replacen("clippy::needless_pass_by_value", "clippy::Needless", 1);
    assert!(parse_err(&bad).contains("lint"));
}

#[test]
fn a_gate_and_a_write_detector_share_one_namespace() {
    // A write detector of the same name: one file stem, one `hook:`/gate name.
    let doc = document(&format!(
        r#"{EOL}
[[detector]]
name = "eol-pinned"
engine = "line-regex"
pattern = 'x'
files = ["**"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "m"

[[detector.fixture]]
path = "a"
input = 'x'
expect = "hit"

[[detector.fixture]]
path = "a"
input = 'y'
expect = "clean"
"#
    ));
    let rule = parse_document(&doc).expect("the rule parses; names are checked by the library");
    let mut library = Library::new();
    library.push(rule);
    assert!(
        library.validate().is_err(),
        "a gate and a detector of one name"
    );
}

// ---------------------------------------------------------------------------
// Judgement, one engine at a time.
// ---------------------------------------------------------------------------

fn judge(g: &Gate, records: &[&str]) -> Detection {
    g.judge_lines(records).expect("well-formed records")
}

#[test]
fn eol_a_shebang_needs_lf_or_no_conversion() {
    let g = gate(EOL);
    let at = |text: &str, eol: &str| judge(&g, &[&format!("bin/run\t#!/bin/sh\t{text}\t{eol}")]);
    assert_eq!(at("unspecified", "unspecified"), Detection::Hit);
    assert_eq!(at("set", "unspecified"), Detection::Hit);
    assert_eq!(at("unspecified", "lf"), Detection::Clean);
    assert_eq!(at("unset", "unspecified"), Detection::Clean);
    // CRLF on a shebang is exactly the break the rule is about.
    assert_eq!(at("set", "crlf"), Detection::Hit);
}

#[test]
fn eol_an_always_path_is_pinned_by_either_ending_and_a_plain_file_needs_nothing() {
    let g = gate(EOL);
    let at =
        |path: &str, eol: &str| judge(&g, &[&format!("{path}\tplain text\tunspecified\t{eol}")]);
    assert_eq!(at("generated/a.md", "unspecified"), Detection::Hit);
    assert_eq!(at("generated/a.md", "crlf"), Detection::Clean);
    assert_eq!(at("generated/a.md", "lf"), Detection::Clean);
    assert_eq!(at("notes/a.md", "unspecified"), Detection::Clean);
}

#[test]
fn root_documents_counts_the_root_only() {
    let g = gate(ROOT_DOCS);
    assert_eq!(judge(&g, &["CHANGELOG.md"]), Detection::Hit);
    assert_eq!(judge(&g, &["docs/CHANGELOG.md"]), Detection::Clean);
    assert_eq!(judge(&g, &["LICENSE", "Cargo.toml"]), Detection::Clean);
    assert_eq!(judge(&g, &["TODO.md", "FEATURES.md"]), Detection::Clean);
}

#[test]
fn lint_explicit_allow_wins_over_any_group() {
    let g = gate(LINT);
    assert_eq!(
        judge(
            &g,
            &[
                "Cargo.toml\tclippy::pedantic\twarn",
                "Cargo.toml\tclippy::needless_pass_by_value\tallow",
            ]
        ),
        Detection::Hit
    );
}

#[test]
fn lint_named_directly_or_through_a_crate_root_attribute_is_enabled() {
    let g = gate(LINT);
    assert_eq!(
        judge(&g, &["src/lib.rs\tclippy::needless_pass_by_value\tdeny"]),
        Detection::Clean
    );
    assert_eq!(
        judge(&g, &["src/main.rs\tclippy::pedantic\tforbid"]),
        Detection::Clean
    );
}

#[test]
fn lint_with_no_record_is_off_when_its_default_is_allow_and_on_when_warn() {
    assert_eq!(judge(&gate(LINT), &[]), Detection::Hit);
    let on_by_default = LINT
        .replace("default = \"allow\"", "default = \"warn\"")
        .replace(
            "records = [\"Cargo.toml\\tclippy::all\\twarn\"]",
            "records = [\"Cargo.toml\\tclippy::needless_pass_by_value\\tallow\"]",
        );
    let g = gate(&on_by_default);
    assert_eq!(judge(&g, &[]), Detection::Clean);
    // A default-on lint is switched off by allowing its group.
    assert_eq!(
        judge(&g, &["Cargo.toml\tclippy::pedantic\tallow"]),
        Detection::Hit
    );
}

#[test]
fn a_lint_record_outside_the_scope_is_not_read() {
    let g = gate(LINT);
    assert_eq!(
        judge(&g, &["build.rs\tclippy::pedantic\twarn"]),
        Detection::Hit
    );
}

// ---------------------------------------------------------------------------
// Laws.
// ---------------------------------------------------------------------------

fn eol_record() -> impl Strategy<Value = String> {
    (
        prop_oneof![
            Just("bin/run"),
            Just("generated/a.md"),
            Just("src/x.rs"),
            Just("run.sh")
        ],
        prop_oneof![Just("#!/bin/sh"), Just("plain"), Just("")],
        prop_oneof![Just("set"), Just("unset"), Just("unspecified")],
        prop_oneof![Just("lf"), Just("crlf"), Just("unspecified")],
    )
        .prop_map(|(p, f, t, e)| format!("{p}\t{f}\t{t}\t{e}"))
}

fn lint_record() -> impl Strategy<Value = String> {
    (
        prop_oneof![Just("Cargo.toml"), Just("src/lib.rs"), Just("build.rs")],
        prop_oneof![
            Just("clippy::needless_pass_by_value"),
            Just("clippy::pedantic"),
            Just("clippy::all"),
            Just("unreachable_pub")
        ],
        prop_oneof![
            Just("allow"),
            Just("expect"),
            Just("warn"),
            Just("deny"),
            Just("forbid")
        ],
    )
        .prop_map(|(p, l, v)| format!("{p}\t{l}\t{v}"))
}

proptest! {
    /// A gate judges a set of records: the order the collect stage happened
    /// to list them in cannot change the verdict.
    #[test]
    fn the_verdict_does_not_depend_on_record_order(
        records in proptest::collection::vec(eol_record(), 0..8),
        lints in proptest::collection::vec(lint_record(), 0..8),
    ) {
        for (g, set) in [(gate(EOL), &records), (gate(LINT), &lints)] {
            let forward: Vec<&str> = set.iter().map(String::as_str).collect();
            let mut reversed = forward.clone(); // allow:clone: a second ordering of the same borrowed records
            reversed.reverse();
            prop_assert_eq!(judge(&g, &forward), judge(&g, &reversed));
        }
    }

    /// An eol hit is a per-record fact: the set hits exactly when one of its
    /// records hits alone.
    #[test]
    fn an_eol_set_hits_exactly_when_some_record_hits_alone(
        records in proptest::collection::vec(eol_record(), 0..8),
    ) {
        let g = gate(EOL);
        let all: Vec<&str> = records.iter().map(String::as_str).collect();
        let any = records.iter().any(|r| judge(&g, &[r.as_str()]) == Detection::Hit);
        prop_assert_eq!(judge(&g, &all) == Detection::Hit, any);
    }

    /// Every record a fixture can hold renders back to the line it was read
    /// from, so the pack's runner feeds the generated judge exactly what the
    /// Rust judge read.
    #[test]
    fn a_record_renders_back_to_its_line(
        record in prop_oneof![eol_record(), lint_record()],
    ) {
        let engine = if record.split('\t').count() == 4 { gate(EOL) } else { gate(LINT) };
        prop_assert_eq!(engine.render_lines(&[record.as_str()]).expect("parses"), vec![record.clone()]); // allow:clone: the expected value is compared to a rendering that owns its own
    }
}

// ---------------------------------------------------------------------------
// The pack.
// ---------------------------------------------------------------------------

#[test]
fn the_repo_target_emits_one_script_per_gate_and_a_runner() {
    let library = library(EOL);
    let files = repo::emit(&library);
    let paths: Vec<&str> = files.iter().map(|f| f.path().as_str()).collect();
    assert_eq!(paths, ["gates/eol-pinned.sh", "gates/test-gates.sh"]);
}

#[test]
fn a_library_with_no_gate_emits_nothing() {
    let library = library(
        r#"
[[detector]]
name = "w"
engine = "line-regex"
pattern = 'x'
files = ["**"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "m"

[[detector.fixture]]
path = "a"
input = 'x'
expect = "hit"

[[detector.fixture]]
path = "a"
input = 'y'
expect = "clean"
"#,
    );
    assert!(repo::emit(&library).is_empty());
}

#[test]
fn the_hooks_target_emits_nothing_for_a_gate() {
    assert!(relearn::emit::hooks::emit(&library(EOL)).is_empty());
}

#[test]
fn every_gate_fixture_reaches_the_runner_as_records_with_its_verdict() {
    let files = repo::emit(&library(EOL));
    let runner = files
        .iter()
        .find(|f| f.path().as_str() == "gates/test-gates.sh")
        .expect("a runner");
    assert!(runner.contents().contains("judge_case 'eol-pinned.sh' hit"));
    assert!(
        runner
            .contents()
            .contains("judge_case 'eol-pinned.sh' clean")
    );
    assert!(runner.contents().contains("bin/run"));
}

// ---------------------------------------------------------------------------
// A gate travels with its rule, and its authored text is scanned.
// ---------------------------------------------------------------------------

#[test]
fn a_contributed_rule_carries_its_gate_and_offers_its_text_to_the_scan() {
    use relearn::contribute::{Contribution, SupersedingVersion};
    use relearn::rule::Version;

    let doc = document(EOL).replace(
        "incident = \"synthetic\"",
        "incident = \"synthetic\"\npublished_incident = \"A synthetic published account.\"",
    );
    let rule = parse_document(&doc).expect("parses");
    let contribution = Contribution::of(&rule).expect("contributable");
    let fields: Vec<(&str, &str)> = contribution.authored_texts().collect();
    assert!(
        fields.contains(&("detector.message", "Pin it in .gitattributes.")),
        "{fields:?}"
    );
    assert!(
        fields.contains(&("detector.fixture.records", "bin/run")),
        "{fields:?}"
    );
    assert!(
        fields.contains(&("detector.fixture.records", "#!/bin/sh")),
        "{fields:?}"
    );
    let version = SupersedingVersion::of(Version::new(1), None).expect("forwards");
    let rendered = contribution.to_document(version);
    let back = parse_document(&rendered).expect("the contribution parses");
    assert_eq!(back.gates(), rule.gates());
}

// ---------------------------------------------------------------------------
// The committed pack's README table, read back.
// ---------------------------------------------------------------------------

/// `repo-pack/README.md` lists every gate script, the rule it was generated
/// for and whether it is installable — `yes` exactly when the rule's status
/// names `gate:<name>`, `draft` otherwise — and nothing but this test reads it.
#[test]
fn the_repo_pack_readme_lists_every_gate_script_its_rule_and_install_state() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(root.join("repo-pack/README.md")).expect("a README");
    let rows: Vec<(String, String, String)> = readme
        .lines()
        .filter_map(|line| {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            let script = cells.get(1)?.strip_prefix('`')?.strip_suffix(".sh`")?;
            let tag = cells.get(2)?.strip_prefix("`[")?.strip_suffix("]`")?;
            Some((
                script.to_owned(),
                tag.to_owned(),
                (*cells.get(5)?).to_owned(),
            ))
        })
        .collect();
    assert!(!rows.is_empty(), "repo-pack/README.md has no script table");

    let mut scripts: Vec<String> = std::fs::read_dir(root.join("repo-pack/gates"))
        .expect("the pack's gates directory")
        .map(|e| {
            e.expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter_map(|n| n.strip_suffix(".sh").map(str::to_owned))
        .filter(|n| n != "test-gates")
        .collect();
    scripts.sort();
    let mut listed: Vec<&str> = rows.iter().map(|(s, _, _)| s.as_str()).collect();
    listed.sort_unstable();
    assert_eq!(
        listed, scripts,
        "repo-pack/README.md's table and repo-pack/gates/ disagree"
    );

    let library = relearn::fsio::load_rules(&root.join("rules"))
        .expect("the corpus loads")
        .validate()
        .expect("the corpus validates");
    for (script, tag, install) in &rows {
        let text =
            std::fs::read_to_string(root.join("repo-pack/gates").join(format!("{script}.sh")))
                .expect("a listed script");
        assert!(
            text.lines().any(|l| l == format!("tag='{tag}'")),
            "the README says {script}.sh holds [{tag}], and it was generated for another rule"
        );
        let rule = library
            .rules()
            .iter()
            .find(|r| r.tag().as_str() == tag)
            .expect("the row's rule is in the corpus");
        let named = rule.status().named_controls().is_some_and(|c| {
            c.as_slice().iter().any(|c| {
                c.kind() == relearn::rule::ControlKind::Gate && c.name() == script.as_str()
            })
        });
        let expected = if named { "yes" } else { "draft" };
        assert_eq!(
            install, expected,
            "{script}.sh's Install cell must read {expected}"
        );
    }
}

/// A gate reaches no instruction layer: only `emit::repo` reads one, as only
/// `emit::hooks` reads a write detector. A gate's fixtures quote what it
/// finds, and an instruction layer quoting them would trip the gate.
#[test]
fn no_emitter_but_repo_reads_a_gate() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut checked = 0;
    let mut paths = vec![root.join("src/emit.rs")];
    for entry in std::fs::read_dir(root.join("src/emit")).expect("the emit directory") {
        paths.push(entry.expect("a readable entry").path());
    }
    for path in paths {
        if path.file_name().is_some_and(|n| n == "repo.rs") || path.is_dir() {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("a readable source file");
        let code: String = source
            .lines()
            .map(|line| line.find("//").map_or(line, |at| &line[..at]))
            .collect::<Vec<_>>()
            .join("\n");
        for marker in ["gates()", "Gate"] {
            assert!(
                !code.contains(marker),
                "{} reads a gate ({marker}); only emit::repo may",
                path.display()
            );
        }
        checked += 1;
    }
    assert!(checked >= 6, "the scan reached only {checked} file(s)");
}
