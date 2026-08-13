//! `cli` — argument parsing (`clap`, derive), command dispatch, and
//! human-readable diagnostics. Commands (TODO.md Phase A): `check` (validate,
//! write nothing), `build --targets <list>` (emit named targets; an unknown
//! target is a `clap` parse error, so it is rejected before any write), `list
//! --home <slug>` (rules filtered by home layer).
//!
//! **Must NOT:** contain business logic. It translates arguments into calls on
//! `library`/`emit`/`fsio` and formats their results; the rules of the domain
//! live in those modules. Library code returns typed errors ([`CliError`] wraps
//! them); this module turns them into a process exit and a diagnostic.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use crate::emit::{self, HomeSlug};
use crate::fsio::{self, LoadError, WriteError};
use crate::library::ValidationError;

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
}

/// Parse the command line, dispatch, and turn the outcome into a process exit.
/// Diagnostics go to stderr; the success summary goes to stdout.
#[must_use]
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    match dispatch(cli.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            report(&err);
            ExitCode::FAILURE
        }
    }
}

fn dispatch(command: Command) -> Result<(), CliError> {
    match command {
        Command::Check { rules } => check(&rules),
        Command::Build {
            rules,
            out,
            targets,
        } => build(&rules, &out, &targets),
        Command::List { rules, home } => list(&rules, home.as_deref()),
    }
}

/// `check`: load and validate; write nothing.
fn check(rules: &Path) -> Result<(), CliError> {
    let validated = fsio::load_rules(rules)?.validate()?;
    println!("ok: {} rule(s) validated", validated.len());
    Ok(())
}

/// `build`: load, validate, emit each unique target, and write under `out`.
fn build(rules: &Path, out: &Path, targets: &[Target]) -> Result<(), CliError> {
    let validated = fsio::load_rules(rules)?.validate()?;

    // Deduplicate so `--targets claude,claude` does not emit the same file twice.
    let unique: BTreeSet<Target> = targets.iter().copied().collect();
    let mut files = Vec::new();
    for target in unique {
        match target {
            Target::Claude => files.extend(emit::claude::emit(&validated)),
            Target::Cursor => files.extend(emit::cursor::emit(&validated)),
            Target::Copilot => files.extend(emit::copilot::emit(&validated)),
            Target::Agents => files.extend(emit::agents::emit(&validated)),
            Target::ClaudeMd => files.extend(emit::claude_md::emit(&validated)),
        }
    }

    let written = fsio::write_all(out, &files)?;
    println!("wrote {written} file(s) under {}", out.display());
    Ok(())
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

        build(rules.path(), out.path(), &[Target::Claude]).expect("build succeeds");

        let skill = std::fs::read_to_string(out.path().join("skills/domain-rust/SKILL.md"))
            .expect("the rust skill was written");
        assert!(skill.contains("[R:r]"));
        assert!(
            skill.contains("<!-- relearn:generated"),
            "carries the guard header"
        );
        assert!(out.path().join("skills/global/SKILL.md").exists());
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

        build(rules.path(), out.path(), &[Target::Cursor]).expect("build succeeds");

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
            build(rules.path(), out.path(), &[Target::Claude]),
            Err(CliError::Write(WriteError::WouldClobberUnversioned { .. }))
        ));
        assert_eq!(
            std::fs::read_to_string(&target).expect("read back"),
            "human authored\n",
            "the human file is left untouched"
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
