//! `cli` — argument parsing (`clap`, derive), command dispatch, and
//! human-readable diagnostics. Commands (TODO.md Phase A): `check` (validate,
//! write nothing), `build --targets <list>` (emit named targets; an unknown
//! target is a `clap` parse error, so it is rejected before any write), `list
//! --home <slug>` (rules filtered by home layer).
//!
//! `build` and `verify` also take `--home <slug>`, restricting the emission to
//! one home layer. It exists for emitting into a **user** scope, where the
//! domain layer is wanted and the project layer is not: project homes are
//! always-resident, so a project layer written to a user scope would load
//! unscoped in every session in every repository. Unlike `list --home`, a slug
//! no rule is in is an **error** — an empty emission would make the paired
//! `verify --home` pass over nothing.
//!
//! **Must NOT:** contain business logic. It translates arguments into calls on
//! `library`/`emit`/`fsio` and formats their results; the rules of the domain
//! live in those modules. Library code returns typed errors ([`CliError`] wraps
//! them); this module turns them into a process exit and a diagnostic.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use crate::emit::{self, HomeSlug, OutputFile};
use crate::fsio::{self, LoadError, VerifyError, WriteError};
use crate::library::{Library, Validated, ValidationError};
use crate::lint;

/// The `relearn` command line.
#[derive(Debug, Parser)]
#[command(
    name = "relearn",
    version,
    about = "Compile correction-derived rules from one neutral source to every AI assistant's instruction layer"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// The subcommands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Validate the rule library and write nothing; non-zero exit on any failure.
    Check {
        /// Directory of `*.md` rule files.
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
    },
    /// Compile the rule library to the selected targets.
    Build {
        /// Directory of `*.md` rule files.
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
        /// Output root the emitted files are written under.
        #[arg(long, default_value = ".")]
        out: PathBuf,
        /// Comma-separated targets to emit; an unknown target is rejected here,
        /// before any file is written. Defaults to every implemented target.
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "claude,cursor,copilot,agents,claude-md"
        )]
        targets: Vec<Target>,
        /// Only emit rules homed in this layer (its slug: `global`, `domain-<name>`,
        /// `project-<slug>`). A slug no rule is in is an error, not an empty emission.
        #[arg(long)]
        home: Option<String>,
    },
    /// List rules, optionally filtered by home layer (its slug).
    List {
        /// Directory of `*.md` rule files.
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
        /// Only rules whose home slug equals this (`global`, `domain-<name>`,
        /// `project-<slug>`).
        #[arg(long)]
        home: Option<String>,
    },
    /// Report advisory lint findings (overlapping scope, home-slug collisions,
    /// dangling references). Writes nothing; exits non-zero if any are found.
    Lint {
        /// Directory of `*.md` rule files.
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
    },
    /// Verify that the generated files under `--out` match what `build` would
    /// write from the current rules. Reads only, writes nothing; exits non-zero
    /// if any generated file is missing, hand-edited, stale, or shadowed by an
    /// unversioned file — the CI check that the committed tree is in sync.
    Verify {
        /// Directory of `*.md` rule files.
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
        /// Output root the generated files were written under.
        #[arg(long, default_value = ".")]
        out: PathBuf,
        /// Comma-separated targets to verify; the same set `build` would emit.
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "claude,cursor,copilot,agents,claude-md"
        )]
        targets: Vec<Target>,
        /// Only emit rules homed in this layer (its slug: `global`, `domain-<name>`,
        /// `project-<slug>`). A slug no rule is in is an error, not an empty emission.
        #[arg(long)]
        home: Option<String>,
    },
}

/// An emission target. Only the variants that have a working emitter are
/// exposed, so selecting an unbuildable target is a `clap` parse error rather
/// than a runtime "not implemented" — the CLI never offers what it cannot do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Target {
    /// Claude skills, one per home layer (`skills/<home>/SKILL.md`).
    Claude,
    /// Cursor rules, one per rule (`.cursor/rules/<tag-body>.mdc`).
    Cursor,
    /// GitHub Copilot instructions, one concatenated file
    /// (`.github/copilot-instructions.md`).
    Copilot,
    /// A single `AGENTS.md` at the repository root.
    Agents,
    /// A project-layer `CLAUDE.md` (project-home rules only).
    ClaudeMd,
}

/// Anything the CLI can fail with. Each variant is transparent over the typed
/// error from the module that produced it; the binary edge turns it into a
/// diagnostic and a non-zero exit.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// Loading the rules directory failed.
    #[error(transparent)]
    Load(#[from] LoadError),
    /// The library failed cross-rule validation.
    #[error(transparent)]
    Validate(#[from] ValidationError),
    /// Writing the emitted files failed.
    #[error(transparent)]
    Write(#[from] WriteError),
    /// Verifying the generated tree hit an I/O failure (not mere drift, which is
    /// a report, not an error).
    #[error(transparent)]
    Verify(#[from] VerifyError),
    /// `--home` named a home layer no rule is in.
    ///
    /// Loud rather than empty **on purpose**. Emitting nothing would look like
    /// success, and `verify --home` over an empty expected set would pass
    /// trivially — a gate reporting success while checking nothing, which is the
    /// failure class this repository exists to catch. `list --home` may print
    /// nothing because printing nothing *is* its answer; a build or a gate has
    /// no such reading.
    #[error("no rule is homed in {requested} (known: {})", known.join(", "))]
    UnknownHome {
        /// The slug as given on the command line.
        requested: String,
        /// Every home slug the library does contain, sorted and deduplicated.
        known: Vec<String>,
    },
}

/// Parse the command line, dispatch, and turn the outcome into a process exit.
/// Diagnostics go to stderr; the success summary goes to stdout.
#[must_use]
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    match dispatch(cli.command) {
        Ok(code) => code,
        Err(err) => {
            report(&err);
            ExitCode::FAILURE
        }
    }
}

/// Dispatch a command. Most commands return `ExitCode::SUCCESS` on the happy
/// path; `lint` returns `FAILURE` when it reports findings (without any error).
fn dispatch(command: Command) -> Result<ExitCode, CliError> {
    match command {
        Command::Check { rules } => {
            check(&rules)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Build {
            rules,
            out,
            targets,
            home,
        } => {
            build(&rules, &out, &targets, home.as_deref())?;
            Ok(ExitCode::SUCCESS)
        }
        Command::List { rules, home } => {
            list(&rules, home.as_deref())?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Lint { rules } => lint_rules(&rules),
        Command::Verify {
            rules,
            out,
            targets,
            home,
        } => verify(&rules, &out, &targets, home.as_deref()),
    }
}

/// `check`: load and validate; write nothing.
fn check(rules: &Path) -> Result<(), CliError> {
    let validated = fsio::load_rules(rules)?.validate()?;
    println!("ok: {} rule(s) validated", validated.len());
    Ok(())
}

/// Emit the deduplicated set of files for the selected targets. Shared by
/// `build` (which writes them) and `verify` (which compares them to disk), so
/// the two commands can never disagree about what the emitted set is.
fn emit_selected(validated: &Library<Validated>, targets: &[Target]) -> Vec<OutputFile> {
    // Deduplicate so `--targets claude,claude` does not emit the same file twice.
    let unique: BTreeSet<Target> = targets.iter().copied().collect();
    let mut files = Vec::new();
    for target in unique {
        match target {
            Target::Claude => files.extend(emit::claude::emit(validated)),
            Target::Cursor => files.extend(emit::cursor::emit(validated)),
            Target::Copilot => files.extend(emit::copilot::emit(validated)),
            Target::Agents => files.extend(emit::agents::emit(validated)),
            Target::ClaudeMd => files.extend(emit::claude_md::emit(validated)),
        }
    }
    files
}

/// Restrict a validated library to one home layer, or fail loudly if that layer
/// holds no rule. `build` and `verify` share this so they can never disagree
/// about which rules are in scope — the same reason they share `emit_selected`.
fn restrict_to_home(
    validated: &Library<Validated>,
    home: Option<&str>,
) -> Result<Library<Validated>, CliError> {
    let Some(slug) = home else {
        return Ok(validated.filter(|_| true));
    };
    let filtered = validated.filter(|r| HomeSlug::of(r.home()).as_str() == slug);
    if filtered.is_empty() {
        let mut known: Vec<String> = validated
            .rules()
            .iter()
            .map(|r| HomeSlug::of(r.home()).as_str().to_owned())
            .collect();
        known.sort();
        known.dedup();
        return Err(CliError::UnknownHome {
            requested: slug.to_owned(),
            known,
        });
    }
    Ok(filtered)
}

/// `build`: load, validate, emit each unique target, and write under `out`.
fn build(rules: &Path, out: &Path, targets: &[Target], home: Option<&str>) -> Result<(), CliError> {
    let validated = restrict_to_home(&fsio::load_rules(rules)?.validate()?, home)?;
    let files = emit_selected(&validated, targets);
    let written = fsio::write_all(out, &files)?;
    println!("wrote {written} file(s) under {}", out.display());
    Ok(())
}

/// `verify`: load, validate, emit the same set `build` would, and compare it to
/// what is on disk under `out`. Writes nothing. Exits `FAILURE` if any generated
/// file drifted (missing, hand-edited, stale, or shadowed by an unversioned
/// file), so CI fails when the committed tree is out of sync with the rules.
fn verify(
    rules: &Path,
    out: &Path,
    targets: &[Target],
    home: Option<&str>,
) -> Result<ExitCode, CliError> {
    let validated = restrict_to_home(&fsio::load_rules(rules)?.validate()?, home)?;
    let files = emit_selected(&validated, targets);
    let reports = fsio::verify_all(out, &files)?;

    let drifted: Vec<_> = reports.iter().filter(|r| !r.status().is_clean()).collect();
    if drifted.is_empty() {
        println!("ok: {} generated file(s) up to date", reports.len());
        return Ok(ExitCode::SUCCESS);
    }
    for report in &drifted {
        println!("{}: {}", report.path().as_str(), report.status());
    }
    println!(
        "{} of {} generated file(s) drifted",
        drifted.len(),
        reports.len()
    );
    Ok(ExitCode::FAILURE)
}

/// `list`: print each rule as `tag  [home-slug]  title`, optionally filtered.
fn list(rules: &Path, home: Option<&str>) -> Result<(), CliError> {
    let validated = fsio::load_rules(rules)?.validate()?;
    for rule in validated.rules() {
        let slug = HomeSlug::of(rule.home());
        if home.is_some_and(|filter| filter != slug.as_str()) {
            continue;
        }
        println!(
            "{}  [{}]  {}",
            rule.tag().as_str(),
            slug.as_str(),
            rule.title().as_str()
        );
    }
    Ok(())
}

/// `lint`: load, validate, and report advisory findings. Writes nothing; exits
/// non-zero if any finding is reported so CI can catch a regression, but never
/// treats a finding as an error (findings are input to a human decision).
fn lint_rules(rules: &Path) -> Result<ExitCode, CliError> {
    let validated = fsio::load_rules(rules)?.validate()?;
    let findings = lint::lint(&validated);
    if findings.is_empty() {
        println!("ok: no lint findings");
        return Ok(ExitCode::SUCCESS);
    }
    for finding in &findings {
        println!("{}: {finding}", finding.severity().label());
    }
    // Info-level findings inform but do not fail the run; Warning and Error do,
    // so CI catches actionable regressions without tripping on advisory notes.
    let actionable = findings
        .iter()
        .filter(|f| f.severity() >= lint::Severity::Warning)
        .count();
    println!("{} finding(s), {actionable} actionable", findings.len());
    if actionable > 0 {
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Print an error and its source chain to stderr.
fn report(err: &CliError) {
    eprintln!("error: {err}");
    let mut source = std::error::Error::source(err);
    while let Some(cause) = source {
        eprintln!("  caused by: {cause}");
        source = cause.source();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule_doc(tag: &str, home_toml: &str, title: &str) -> String {
        format!(
            "+++\n\
             tag = \"{tag}\"\n\
             title = \"{title}\"\n\
             error_class = \"e\"\n\
             home = {home_toml}\n\
             created = \"2026-01-01\"\n\
             status = {{ kind = \"active\" }}\n\
             incident = \"i\"\n\
             +++\n\n\
             Body of {tag}.\n"
        )
    }

    fn write_rule(dir: &Path, file: &str, contents: &str) {
        std::fs::write(dir.join(file), contents).expect("write rule file");
    }

    #[test]
    fn build_runs_the_whole_pipeline_and_writes_skills() {
        let rules = tempfile::tempdir().expect("rules tempdir");
        let out = tempfile::tempdir().expect("out tempdir");
        write_rule(
            rules.path(),
            "g.md",
            &rule_doc("R:g", "{ kind = \"global\" }", "Global rule"),
        );
        write_rule(
            rules.path(),
            "r.md",
            &rule_doc("R:r", "{ kind = \"domain\", name = \"rust\" }", "Rust rule"),
        );

        build(rules.path(), out.path(), &[Target::Claude], None).expect("build succeeds");

        let skill = std::fs::read_to_string(out.path().join("skills/domain-rust/SKILL.md"))
            .expect("the rust skill was written");
        assert!(skill.contains("[R:r]"));
        assert!(
            skill.contains("<!-- relearn:generated"),
            "carries the guard header"
        );
        assert!(out.path().join("skills/global/SKILL.md").exists());
    }

    /// `--home` narrows the emission to one layer. The case this exists for:
    /// emitting the rules layer into a user scope must carry the domain layer
    /// and *not* the project layer, which is always-resident and would then load
    /// in every session in every repository.
    #[test]
    fn build_with_home_emits_only_that_layer() {
        let rules = tempfile::tempdir().expect("rules tempdir");
        let out = tempfile::tempdir().expect("out tempdir");
        write_rule(
            rules.path(),
            "r.md",
            &rule_doc("R:r", "{ kind = \"domain\", name = \"rust\" }", "Rust rule"),
        );
        write_rule(
            rules.path(),
            "p.md",
            &rule_doc(
                "R:p",
                "{ kind = \"project\", path = \"relearn\" }",
                "Project rule",
            ),
        );

        build(
            rules.path(),
            out.path(),
            &[Target::ClaudeMd],
            Some("domain-rust"),
        )
        .expect("build succeeds");

        assert!(out.path().join(".claude/rules/domain-rust.md").exists());
        assert!(
            !out.path().join(".claude/rules/project-relearn.md").exists(),
            "the project layer must not follow the domain layer into a user scope"
        );
    }

    /// A home no rule is in is an error, never an empty emission — an empty
    /// emission would make the paired `verify --home` pass over nothing.
    #[test]
    fn build_with_an_unknown_home_is_an_error_naming_the_known_ones() {
        let rules = tempfile::tempdir().expect("rules tempdir");
        let out = tempfile::tempdir().expect("out tempdir");
        write_rule(
            rules.path(),
            "r.md",
            &rule_doc("R:r", "{ kind = \"domain\", name = \"rust\" }", "Rust rule"),
        );

        let err = build(
            rules.path(),
            out.path(),
            &[Target::ClaudeMd],
            Some("domain-cobol"),
        )
        .expect_err("an unknown home must fail");

        match err {
            CliError::UnknownHome { requested, known } => {
                assert_eq!(requested, "domain-cobol");
                assert_eq!(known, vec!["domain-rust".to_owned()]);
            }
            other => panic!("expected UnknownHome, got {other:?}"),
        }
        assert!(
            !out.path().join(".claude").exists(),
            "nothing is written on the failure path"
        );
    }

    #[test]
    fn build_with_cursor_target_writes_mdc_rules() {
        let rules = tempfile::tempdir().expect("rules tempdir");
        let out = tempfile::tempdir().expect("out tempdir");
        write_rule(
            rules.path(),
            "r.md",
            &rule_doc(
                "R:parse-wide",
                "{ kind = \"domain\", name = \"rust\" }",
                "Parse wide",
            ),
        );

        build(rules.path(), out.path(), &[Target::Cursor], None).expect("build succeeds");

        let mdc = std::fs::read_to_string(out.path().join(".cursor/rules/parse-wide.mdc"))
            .expect("the cursor rule was written");
        assert!(mdc.contains("globs: **/*.rs"));
        assert!(
            mdc.contains("<!-- relearn:generated"),
            "carries the guard header"
        );
    }

    #[test]
    fn check_reports_a_parse_failure_by_file() {
        let rules = tempfile::tempdir().expect("rules tempdir");
        write_rule(rules.path(), "broken.md", "not a rule at all");
        assert!(matches!(
            check(rules.path()),
            Err(CliError::Load(LoadError::Parse { .. }))
        ));
    }

    #[test]
    fn check_reports_a_duplicate_tag_as_a_validation_error() {
        let rules = tempfile::tempdir().expect("rules tempdir");
        write_rule(
            rules.path(),
            "a.md",
            &rule_doc("R:dup", "{ kind = \"global\" }", "One"),
        );
        write_rule(
            rules.path(),
            "b.md",
            &rule_doc("R:dup", "{ kind = \"global\" }", "Two"),
        );
        assert!(matches!(
            check(rules.path()),
            Err(CliError::Validate(ValidationError::DuplicateTag(_)))
        ));
    }

    #[test]
    fn build_refuses_to_clobber_and_never_writes_a_partial_tree() {
        let rules = tempfile::tempdir().expect("rules tempdir");
        let out = tempfile::tempdir().expect("out tempdir");
        write_rule(
            rules.path(),
            "g.md",
            &rule_doc("R:g", "{ kind = \"global\" }", "Global rule"),
        );
        // A human file sitting exactly where the global skill would land.
        let target = out.path().join("skills/global/SKILL.md");
        std::fs::create_dir_all(target.parent().expect("has parent")).expect("mkdir");
        std::fs::write(&target, "human authored\n").expect("seed human file");

        assert!(matches!(
            build(rules.path(), out.path(), &[Target::Claude], None),
            Err(CliError::Write(WriteError::WouldClobberUnversioned { .. }))
        ));
        assert_eq!(
            std::fs::read_to_string(&target).expect("read back"),
            "human authored\n",
            "the human file is left untouched"
        );
    }

    #[test]
    fn verify_succeeds_on_a_fresh_build_and_fails_after_drift() {
        let rules = tempfile::tempdir().expect("rules tempdir");
        let out = tempfile::tempdir().expect("out tempdir");
        write_rule(
            rules.path(),
            "g.md",
            &rule_doc("R:g", "{ kind = \"global\" }", "Global rule"),
        );

        build(rules.path(), out.path(), &[Target::Agents], None).expect("build succeeds");
        assert_eq!(
            verify(rules.path(), out.path(), &[Target::Agents], None).expect("verify runs"),
            ExitCode::SUCCESS,
            "a freshly built tree verifies clean"
        );

        // Hand-edit the generated file; verify must now fail.
        let agents = out.path().join("AGENTS.md");
        let edited = format!(
            "sneaky\n{}",
            std::fs::read_to_string(&agents).expect("read")
        );
        std::fs::write(&agents, edited).expect("tamper");
        assert_eq!(
            verify(rules.path(), out.path(), &[Target::Agents], None).expect("verify runs"),
            ExitCode::FAILURE,
            "a hand-edited generated file fails verification"
        );
    }

    #[test]
    fn verify_fails_when_a_generated_file_is_missing() {
        let rules = tempfile::tempdir().expect("rules tempdir");
        let out = tempfile::tempdir().expect("out tempdir");
        write_rule(
            rules.path(),
            "g.md",
            &rule_doc("R:g", "{ kind = \"global\" }", "Global rule"),
        );
        // Never built: the expected AGENTS.md is absent.
        assert_eq!(
            verify(rules.path(), out.path(), &[Target::Agents], None).expect("verify runs"),
            ExitCode::FAILURE,
            "a never-built (missing) target fails verification"
        );
    }

    #[test]
    fn list_without_a_filter_lists_all_and_a_missing_dir_is_an_error() {
        let rules = tempfile::tempdir().expect("rules tempdir");
        write_rule(
            rules.path(),
            "g.md",
            &rule_doc("R:g", "{ kind = \"global\" }", "Global rule"),
        );
        list(rules.path(), None).expect("list all succeeds");
        list(rules.path(), Some("global")).expect("filtered list succeeds");

        let missing = rules.path().join("does-not-exist");
        assert!(matches!(
            list(&missing, None),
            Err(CliError::Load(LoadError::ReadDir { .. }))
        ));
    }
}
