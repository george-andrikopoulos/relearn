//! `relearn loads` — read one or more live load logs against what `build`
//! would emit now: COLD, STALE, SHADOWED, and LOADED per home and location.
//!
//! Reads only. The logs are written by the load-log pack's hook, which
//! relearn never installs; the expected hashes come from the same emitters
//! `build` runs, hashed exactly as the marker `build` writes is computed.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;

use crate::emit::{self, HomeSlug};
use crate::fsio::{self, ProbeReadError};
use crate::loadlog::{
    ContentHash, Expected, Form, LoadLogError, ParsedLog, Window, parse_log, read,
};
use crate::loadset::{LayerSlug, Outcome};
use crate::rule::{Date, DateError};

use super::CliError;

#[derive(Debug, Args)]
pub(super) struct LoadsArgs {
    /// A load log the hook wrote. Repeat for several; they read as one.
    #[arg(long = "log", required = true)]
    logs: Vec<PathBuf>,
    /// First day of the window, YYYY-MM-DD.
    #[arg(long)]
    since: String,
    /// Last day of the window, YYYY-MM-DD, inclusive. Defaults to `--since`.
    #[arg(long)]
    until: Option<String>,
    /// Directory of `*.md` rule files: the homes, and what `build` emits now.
    #[arg(long, default_value = "rules")]
    rules: PathBuf,
}

/// Every way `relearn loads` can fail before it reports.
#[derive(Debug, thiserror::Error)]
pub enum LoadsCliError {
    #[error("{0}")]
    Read(#[from] ProbeReadError),
    #[error("{path}: {source}")]
    Log { path: PathBuf, source: LoadLogError },
    #[error("`--{flag}`: {source}")]
    Date {
        flag: &'static str,
        source: DateError,
    },
    #[error("the window ends ({until}) before it starts ({since})")]
    Window { since: Date, until: Date },
}

fn date(flag: &'static str, text: &str) -> Result<Date, LoadsCliError> {
    Date::parse(text).map_err(|source| LoadsCliError::Date { flag, source })
}

/// What `build` would emit now, hashed as its marker hashes: every home's
/// skill, and every home's rules-layer file in the layer it is written for.
fn expected(library: &crate::library::Library<crate::library::Validated>) -> Expected {
    let skills = emit::claude::emit(library).into_iter().filter_map(|f| {
        let segments: Vec<&str> = f.path().as_str().split('/').collect();
        match segments.as_slice() {
            ["skills", slug, "SKILL.md"] => LayerSlug::parse(slug)
                .zip(ContentHash::parse(&fsio::content_hash(&f)))
                .map(|(s, h)| ((s, Form::Skill), h)),
            _ => None,
        }
    });
    let rules = emit::claude_rules::layers_of(library)
        .iter()
        .flat_map(|layer| emit::claude_rules::emit(library, layer))
        .filter_map(|f| {
            let stem = f
                .path()
                .as_str()
                .rsplit('/')
                .next()?
                .strip_suffix(".md")?
                .to_owned();
            LayerSlug::parse(&stem)
                .zip(ContentHash::parse(&fsio::content_hash(&f)))
                .map(|(s, h)| ((s, Form::RulesLayer), h))
        })
        .collect::<Vec<_>>();
    Expected::new(skills.chain(rules))
}

pub(super) fn run(args: &LoadsArgs) -> Result<ExitCode, CliError> {
    let since = date("since", &args.since)?;
    let until = match &args.until {
        Some(u) => date("until", u)?,
        None => since,
    };
    let window = Window::new(since, until).ok_or(LoadsCliError::Window { since, until })?;

    let library = fsio::load_rules(&args.rules)?.validate()?;
    let mut log = ParsedLog::default();
    for path in &args.logs {
        // The probe reader is a plain text read with a path-carrying error.
        let text = fsio::read_probe_text(path).map_err(LoadsCliError::from)?;
        let parsed = parse_log(&text).map_err(|source| LoadsCliError::Log {
            path: path.clone(), // allow:clone: the error owns the path it names
            source,
        })?;
        log.extend(parsed);
    }

    let mut homes: Vec<LayerSlug> = library
        .rules()
        .iter()
        .filter_map(|r| LayerSlug::parse(HomeSlug::of(r.home()).as_str()))
        .collect();
    homes.sort();
    homes.dedup();

    let report = read(log.events(), window, &expected(&library), &homes);

    println!(
        "window {since} to {until}: {} home load(s), {} other load(s) skipped",
        report.in_window(),
        log.ignored()
    );
    println!("\nLOADED  HOME                                LOCATION        COUNT");
    for (home, location, count) in report.loaded_rows() {
        println!(
            "        {:<35} {:<15} {count}",
            home.as_str(),
            location.as_str()
        );
    }
    println!();
    for finding in report.findings() {
        println!("{finding}");
    }
    Ok(match report.outcome() {
        Outcome::Clean => {
            println!("ok: no findings");
            ExitCode::SUCCESS
        }
        Outcome::Findings => {
            println!("{} finding(s)", report.findings().len());
            ExitCode::from(1)
        }
        Outcome::Unchecked => {
            println!(
                "UNCHECKED: no relearn home loaded in the window. Either nothing loaded, or the \
                 load-log hook is not registered; a report cannot tell those apart, so it is not a pass."
            );
            ExitCode::from(2)
        }
    })
}
