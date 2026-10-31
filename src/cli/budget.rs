//! `relearn budget` — what a Claude Code session in a project loads at start,
//! what it costs against the loader's limit, and whether a layer loads twice
//! or for the wrong project. Read-only: it fetches nothing and writes nothing.
//!
//! A subcommand rather than a `lint` flag (decisions log, 2026-10-31): `lint`
//! reads the rule corpus and this reads two directories the corpus knows
//! nothing about, so the two share no input and should not share an exit code.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;

use crate::fsio::{self, Ancestors, LoadSetReadError};
use crate::loadset::{
    self, Authorship, Coverage, LayerSlug, Limits, LimitsError, LoadSet, LoaderChars, Outcome,
    Residency, Share,
};

/// The arguments of `relearn budget`.
#[derive(Debug, Args)]
pub(super) struct BudgetArgs {
    /// The project directory a session would start in.
    #[arg(long, default_value = ".")]
    project: PathBuf,
    /// The user configuration directory (`~/.claude`). Without it the user
    /// share is UNCHECKED — never counted as zero — and the exit status is 2
    /// unless `--project-only` says the user share is not the question.
    #[arg(long, conflicts_with = "project_only")]
    user: Option<PathBuf>,
    /// Measure the project's own share only: no user directory, no ancestor
    /// directories. The hermetic mode `scripts/check.sh` runs.
    #[arg(long)]
    project_only: bool,
    /// The project's name as its relearn home spells it (`relearn` for
    /// `project-relearn`). Defaults to the project directory's own name.
    #[arg(long = "as")]
    project_name: Option<String>,
    /// The loader's limit, in UTF-16 code units.
    #[arg(long, default_value_t = loadset::DEFAULT_LIMIT)]
    limit: usize,
    /// The budget beneath it. Defaults to the smaller of the limit and the
    /// loader's combined limit on its smallest window.
    #[arg(long)]
    budget: Option<usize>,
    /// A budget for the project's own always-resident share.
    #[arg(long)]
    project_budget: Option<usize>,
}

/// Why `relearn budget` could not measure.
#[derive(Debug, thiserror::Error)]
pub enum BudgetCliError {
    /// The load set could not be read.
    #[error(transparent)]
    Read(#[from] LoadSetReadError),
    /// The limit and budget given are not a usable pair.
    #[error(transparent)]
    Limits(#[from] LimitsError),
    /// No project name was given and the project directory has none.
    #[error("{path:?} has no directory name to take the project's name from; pass --as")]
    NoProjectName {
        /// The project path given.
        path: PathBuf,
    },
}

/// Run `relearn budget`: 0 clean, 1 any finding, 2 the user share unchecked.
pub(super) fn run(args: &BudgetArgs) -> Result<ExitCode, BudgetCliError> {
    let coverage = match (&args.user, args.project_only) {
        (Some(_), _) => Coverage::Whole,
        (None, true) => Coverage::ProjectOnly,
        (None, false) => Coverage::UserUnchecked,
    };
    let ancestors = match coverage {
        Coverage::Whole => Ancestors::All,
        Coverage::ProjectOnly | Coverage::UserUnchecked => Ancestors::Skip,
    };
    let limit = LoaderChars::new(args.limit);
    let budget = LoaderChars::new(
        args.budget
            .unwrap_or_else(|| args.limit.min(loadset::DEFAULT_BUDGET)),
    );
    let limits = Limits::new(limit, budget)?;
    let session = session_layer(&args.project, args.project_name.as_deref())?;

    let set = LoadSet::new(fsio::discover_load_set(
        &args.project,
        args.user.as_deref(),
        ancestors,
    )?);
    let assessment = loadset::assess(
        &set,
        &session,
        coverage,
        limits,
        args.project_budget.map(LoaderChars::new),
    );

    print_table(&set);
    println!();
    println!(
        "always-resident: {} chars (counts toward the loader's combined total)",
        set.always_total()
    );
    println!(
        "path-scoped:     {} chars more if every scope attaches",
        set.scoped_total()
    );
    for share in [Share::User, Share::Ancestor, Share::Project] {
        let total = set.share_total(share);
        match (share, coverage) {
            (Share::User | Share::Ancestor, Coverage::UserUnchecked) => {
                println!("{:<9} share: UNCHECKED (no --user given)", share.label());
            }
            (Share::User | Share::Ancestor, Coverage::ProjectOnly) => {
                println!("{:<9} share: not measured (--project-only)", share.label());
            }
            _ => println!("{:<9} share: {total} chars always-resident", share.label()),
        }
    }
    println!(
        "limit {} / budget {}{}; session layer {session}",
        limits.limit(),
        limits.budget(),
        args.project_budget
            .map(|b| format!(" / project budget {b}"))
            .unwrap_or_default(),
    );
    for finding in assessment.findings() {
        println!("{finding}");
    }
    Ok(match assessment.outcome() {
        Outcome::Clean => {
            println!("ok: no findings");
            ExitCode::SUCCESS
        }
        Outcome::Findings => {
            println!("{} finding(s)", assessment.findings().len());
            ExitCode::FAILURE
        }
        Outcome::Unchecked => {
            println!(
                "UNCHECKED: no findings in the project share, but the user share was not measured; pass --user, or --project-only if it is not the question"
            );
            ExitCode::from(2)
        }
    })
}

fn session_layer(project: &Path, name: Option<&str>) -> Result<LayerSlug, BudgetCliError> {
    if let Some(name) = name {
        return Ok(LayerSlug::project(name));
    }
    let canonical = std::fs::canonicalize(project).map_err(|source| {
        BudgetCliError::Read(LoadSetReadError::Io {
            path: project.to_path_buf(),
            source,
        })
    })?;
    canonical
        .file_name()
        .and_then(|n| n.to_str())
        .map(LayerSlug::project)
        .ok_or_else(|| BudgetCliError::NoProjectName {
            path: project.to_path_buf(),
        })
}

fn print_table(set: &LoadSet) {
    // LOADS is last because a glob list has no useful width bound.
    println!(
        "{:<9} {:>7}  {:<34} {:<52} LOADS",
        "SHARE", "CHARS", "GENERATED", "PATH"
    );
    for file in set.files() {
        let loads = match file.residency() {
            Residency::Always => "always".to_owned(),
            Residency::PathScoped(globs) => format!("paths: {}", globs.join(", ")),
        };
        let generated = match file.origin() {
            Authorship::HandAuthored => "-".to_owned(),
            Authorship::Generated(Some(layer)) => layer.to_string(),
            Authorship::Generated(None) => "yes (no layer)".to_owned(),
        };
        println!(
            "{:<9} {:>7}  {:<34} {:<52} {loads}",
            file.share().label(),
            file.chars(),
            generated,
            file.shown()
        );
    }
}
