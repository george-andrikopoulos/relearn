//! `FEATURES.md` names the artefact that enforces each guarantee. This reads
//! the names.
//!
//! The ledger makes seventy claims of the form *"Enforced by: `X`"*, and until
//! now **nothing checked that `X` exists**. A test renamed in a refactor leaves
//! the row behind, still naming it, still reading as a guarantee somebody holds
//! — which is `[R:guarantee-needs-a-reader]` and `[R:wired-artifact]` at once:
//! a claim with no reader, and a claim whose evidence is a string anybody can
//! typo. It is also `[R:doc-currency]`, the rule that fired twice in this file
//! on 2026-09-13 and now carries a recurrence for it.
//!
//! **Two directions, as with the pack counts, and each owns its denominator.**
//!
//! * Every **path-qualified citation** — `` `tests/x.rs::some_test` `` —
//!   resolves: the file exists and defines that function.
//! * Every **`Enforced by:` row** names something in backticks, or says
//!   `NOTHING YET` — so a row cannot claim enforcement in prose alone, which is
//!   the shape the ledger exists to prevent.
//!
//! **Scope, and what it deliberately does not cover** (`[R:measure-the-claim-not-a-subset]`,
//! because a check whose reach is assumed rather than stated is how a subset
//! comes to stand for a superset). Only the path-qualified form is resolved.
//! Bare `module::item` citations — `lint::tally`, `Rule::serves`,
//! `Status::emittability` — are **not**: resolving them means searching the
//! tree for a definition, and the same citation shape is used for `env::var`
//! and `ExitCode::FAILURE`, which belong to the standard library. A checker
//! that had to carry an exclusion list of foreign names would grow one entry
//! per release until somebody muted it. The path-qualified form carries its own
//! file, so there is nothing to search and nothing to exclude — and it is the
//! form the load-bearing citations use, because a test is what most rows are
//! enforced by.

use std::fs;
use std::path::{Path, PathBuf};

/// The ledger this reads. One file, named once.
const LEDGER: &str = "FEATURES.md";

/// The marker a row uses to name what holds it.
///
/// **Bold, and that is load-bearing rather than cosmetic.** The ledger's own
/// header explains the *"Enforced by"* column in prose and in quotation marks;
/// the first version of this check matched the bare words and failed on that
/// sentence — documentation *about* the marker read as a use *of* it, which is
/// `[R:detector-excludes-own-definitions]` inside the artefact meant to hold
/// the ledger honest. A row states the marker in bold; prose about rows does
/// not.
const ENFORCED_BY: &str = "**Enforced by";

/// A row with no artefact says this, exactly, and is carried in `TODO.md`.
const EXPOSED: &str = "NOTHING YET";

/// How far past the marker a row may go before naming what holds it.
///
/// Not a taste: measured over the ledger. Fifty-eight rows open with the
/// citation, the rest reach it within thirty characters, and the single row
/// that ran to 378 was claiming a human review process rather than an artefact.
/// Generous enough that no honest row has to be reworded, tight enough that a
/// row claiming enforcement in prose cannot be rescued by a backtick three
/// clauses later — which is what made the first version of this check vacuous.
const NAMED_WITHIN: usize = 60;

/// A citation that names both a file and an item in it.
#[derive(Debug)]
struct Citation {
    /// 1-based line in the ledger, so a failure names the line to edit.
    line: usize,
    /// The file, repository-relative, as the ledger writes it.
    path: String,
    /// The function it claims that file defines.
    item: String,
}

/// Rust source with line comments removed.
///
/// A test name that appears only in a comment must not satisfy a citation:
/// the ledger claims the artefact **exists**, and a sentence about it is what
/// the claim was supposed to replace. Same reason the aggregate and poke scans
/// strip comments before matching (`[R:detector-excludes-own-definitions]`).
fn code_of(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
        .lines()
        .map(|line| match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether `code` defines `fn item`.
fn defines_fn(code: &str, item: &str) -> bool {
    code.match_indices(item).any(|(at, _)| {
        let before = code[..at].trim_end();
        let after = &code[at + item.len()..];
        before.ends_with("fn")
            && after
                .chars()
                .next()
                .is_some_and(|c| c == '(' || c == '<' || c == ' ')
    })
}

/// Every `` `<path>.rs::<item>` `` the ledger states.
fn citations(text: &str) -> Vec<Citation> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        // Odd-indexed segments of a backtick split are the code spans.
        for span in line.split('`').skip(1).step_by(2) {
            let Some((path, item)) = span.split_once("::") else {
                continue;
            };
            if !path.ends_with(".rs") || item.is_empty() {
                continue;
            }
            let well_formed = item.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !item.starts_with(|c: char| c.is_ascii_digit());
            if well_formed {
                out.push(Citation {
                    line: i + 1,
                    path: path.to_owned(),
                    item: item.to_owned(),
                });
            }
        }
    }
    out
}

fn ledger() -> (PathBuf, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join(LEDGER);
    let text = fs::read_to_string(&path).expect("the feature ledger is readable");
    (root, text)
}

/// Every artefact the ledger names by file **and** item is really there.
///
/// The failure this closes is a rename: `cargo test` stays green, the ledger
/// goes on naming the old function, and the guarantee reads as held by
/// something that no longer exists.
#[test]
fn every_artefact_the_ledger_names_by_path_exists() {
    let (root, text) = ledger();
    let citations = citations(&text);

    // A parser that has stopped recognising the ledger's citation form would
    // otherwise report success having checked nothing. The floor is well below
    // the current count, so adding rows never has to touch this number —
    // a check that must be edited to stay true is one that gets edited away.
    assert!(
        citations.len() >= 20,
        "only {} path-qualified citation(s) found in {LEDGER} — either the ledger \
         stopped naming its artefacts, or this parser stopped recognising them",
        citations.len()
    );

    for citation in &citations {
        let path = root.join(&citation.path);
        assert!(
            path.is_file(),
            "{LEDGER}:{} names {}, and that file does not exist",
            citation.line,
            citation.path
        );
        assert!(
            defines_fn(&code_of(&path), &citation.item),
            "{LEDGER}:{} says {} is enforced by `{}`, and {} defines no such function. \
             A renamed test leaves the guarantee reading as held by something that is gone",
            citation.line,
            citation.path,
            citation.item,
            citation.path
        );
    }
}

/// **The other direction.** A row may be honestly unenforced — four are, and
/// say `NOTHING YET` — but it may not claim enforcement in prose alone. That is
/// the shape the ledger exists to prevent: a guarantee whose evidence is a
/// sentence.
#[test]
fn every_enforced_by_row_names_an_artefact_or_declares_itself_exposed() {
    let (_, text) = ledger();
    let mut rows = 0;

    for (i, line) in text.lines().enumerate() {
        if !line.contains(ENFORCED_BY) {
            continue;
        }
        rows += 1;
        let after = line
            .split_once(ENFORCED_BY)
            .map(|(_, rest)| rest)
            .unwrap_or(line);
        // **A window, because these rows are long paragraphs.** The first
        // version asked only whether a backtick appeared anywhere after the
        // marker, which every row satisfies by accident — a check that cannot
        // fail. Measured over the real ledger, every row that names an artefact
        // names it within thirty characters of the marker; the one row that did
        // not was claiming a human review process, and now says `NOTHING YET`
        // as the ledger's own header always said it should.
        let window: String = after.chars().take(NAMED_WITHIN).collect();
        assert!(
            window.contains('`') || window.contains(EXPOSED),
            "{LEDGER}:{} claims enforcement and names nothing: a guarantee whose \
             evidence is a sentence is the thing this ledger exists to prevent. \
             Name the artefact, or write `{EXPOSED} — exposed` and carry it in TODO.md",
            i + 1
        );
    }

    assert!(
        rows >= 20,
        "only {rows} enforced-by row(s) found in {LEDGER} — the marker has changed \
         and this check is reading nothing"
    );
}
