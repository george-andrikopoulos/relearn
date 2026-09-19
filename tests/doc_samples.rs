//! Every rule document shown in the documentation is a rule the parser accepts.
//!
//! **Why this exists.** On 2026-09-19 the front matter in `README.md` and
//! `docs/worked-example.md` was extracted and run through `relearn check`, and
//! both were rejected: `missing field origin`. The project's public face had
//! been showing a document the tool refuses, on both counts, for months — while
//! `tests/pack_counts.rs` was catching a stale *line count* in the same session.
//!
//! The reason nothing caught it is worth stating, because it is the general
//! shape: a sample is prose to everyone who reads it and a rule to nobody, so no
//! existing gate had a reason to look at it. `verify` compares generated files
//! against the library and never opens a hand-authored document; `check` reads
//! `rules/` and nothing else. A documentation example sits in the one place both
//! of them are blind to, and it is the first thing a new reader copies.
//!
//! **Scope, stated rather than implied** (`[R:measure-the-claim-not-a-subset]`).
//! This reads the four hand-authored documents that carry `+++` blocks —
//! `README.md`, `ARCHITECTURE.md` and the two worked examples — and every
//! `+++`-delimited block inside them. It does **not** read the emitted tree
//! (`verify` owns that), the packs (`pack_counts` owns those), or prose that
//! merely mentions a field name. A sample whose body is omitted for brevity is
//! given one before parsing, so what this asserts about such a block is that its
//! **front matter** is valid; the body half is only asserted where the document
//! actually shows one.

use std::fs;
use std::path::{Path, PathBuf};

use relearn::rule::parse_document;

/// The hand-authored documents that carry rule samples.
///
/// Named rather than globbed, deliberately: a glob over `docs/` would silently
/// start covering a new file and silently stop covering a renamed one, and this
/// gate's whole subject is a claim nobody was checking. A new document with a
/// sample in it is added here by the person who writes it — and
/// `every_listed_document_exists` fails if one is renamed away.
const DOCUMENTED: &[&str] = &[
    "README.md",
    "ARCHITECTURE.md",
    "docs/worked-example.md",
    "docs/worked-example-two-installs.md",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// One rule document found in a markdown file, with the line its `+++` opened on
/// so a failure can name it.
struct Sample {
    line: usize,
    document: String,
}

/// Pull every `+++`-delimited block out of a markdown file, with whatever body
/// follows it up to the end of the enclosing fenced code block.
///
/// Carriage returns are stripped first. These documents are not pinned to LF in
/// `.gitattributes` — only `rules/**` and the emitted tree are — so a Windows
/// checkout hands us CRLF, a `+++\r` line matches no delimiter, and this gate
/// would find nothing and pass on the platform it was written on
/// (`[R:xplat-fixtures]`).
fn samples(text: &str) -> Vec<Sample> {
    let lines: Vec<&str> = text.lines().map(|l| l.trim_end_matches('\r')).collect();
    let mut found = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i] != "+++" {
            i += 1;
            continue;
        }
        let opened_at = i + 1;
        let Some(close) = (i + 1..lines.len()).find(|&j| lines[j] == "+++") else {
            break;
        };
        // The body runs to the end of the fenced block the sample sits in.
        let body_end = (close + 1..lines.len())
            .find(|&j| lines[j].starts_with("```"))
            .unwrap_or(lines.len());
        let front = lines[i..=close].join("\n");
        let body = lines[close + 1..body_end].join("\n");
        let body = if body.trim().is_empty() {
            // A sample that elides its body still has front matter worth
            // checking; give it one rather than skip the block.
            "Body omitted in the documentation; supplied by tests/doc_samples.rs."
        } else {
            body.trim()
        };
        found.push(Sample {
            line: opened_at,
            document: format!("{front}\n\n{body}\n"),
        });
        i = body_end;
    }
    found
}

/// **The assertion.** Every sample parses as a rule.
#[test]
fn every_rule_sample_in_the_documentation_parses() {
    let root = repo_root();
    let mut checked = 0usize;
    for name in DOCUMENTED {
        let path = root.join(name);
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {name}: {e}"));
        for sample in samples(&text) {
            checked += 1;
            if let Err(err) = parse_document(&sample.document) {
                panic!(
                    "{name}:{} is shown as a rule and does not parse: {err}\n\
                     The documentation is showing a document the tool refuses.",
                    sample.line
                );
            }
        }
    }
    // A gate that finds nothing passes, which is the failure this whole file is
    // about. `[R:wired-artifact]`
    assert!(
        checked >= DOCUMENTED.len(),
        "found only {checked} sample(s) across {} documents — the extractor is \
         matching nothing and this gate is measuring nothing",
        DOCUMENTED.len()
    );
}

/// The list above is a list of paths, and a path can be renamed out from under
/// it. Without this, dropping a document from the repository would make this
/// gate quieter rather than louder.
#[test]
fn every_listed_document_exists_and_carries_a_sample() {
    let root = repo_root();
    for name in DOCUMENTED {
        let path = root.join(name);
        let text = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("{name} is listed in DOCUMENTED and could not be read: {e}")
        });
        assert!(
            !samples(&text).is_empty(),
            "{name} is listed as carrying a rule sample and carries none — \
             remove it from the list, or the list is describing the repository it used to be"
        );
    }
}

/// The extractor itself, pinned on both line endings.
///
/// Written because the failure it guards against is invisible: on LF it works,
/// on CRLF it silently finds nothing, and "found nothing" is indistinguishable
/// from "everything passed" without the count assertion above.
#[test]
fn the_extractor_finds_a_sample_whatever_the_line_endings() {
    let lf = "text\n\n```\n+++\ntag = \"R:x\"\n+++\n\nA body.\n```\n\nmore text\n";
    assert_eq!(samples(lf).len(), 1, "LF");
    let crlf = lf.replace('\n', "\r\n");
    assert_eq!(samples(&crlf).len(), 1, "CRLF");
    assert!(samples(&crlf)[0].document.contains("tag = \"R:x\""));
}
