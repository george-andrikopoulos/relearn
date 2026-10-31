//! `relearn probe` — the read-only half of a probe run.
//!
//! The runner (`scripts/probe-run.sh`) starts every model session; these
//! subcommands tell it what to run and judge what came back, so no checker is
//! written twice. Nothing here writes a file, spawns a process or opens a
//! socket.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Subcommand;

use crate::emit::{HomeSlug, LoadSemantics};
use crate::fsio::{self, ProbeReadError};
use crate::probe::{
    Delivery, ResultsError, SkillName, TranscriptError, parse_results, render,
    session_from_stream_json, tally,
};
use crate::rule::{Checker, Home, Rule, RuleTag, RuleTagError, Verdict};

use super::CliError;

#[derive(Debug, Subcommand)]
pub(super) enum ProbeCommand {
    /// Every rule carrying a probe, as TSV: tag, home slug, the build target
    /// that emits its layer for the WITH arm, workspace, checker kind.
    List {
        /// Directory of `*.md` rule files.
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
    },
    /// Print one rule's probe task, verbatim, for the runner to hand the model.
    Task {
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
        #[arg(long)]
        tag: String,
    },
    /// Judge one session and print one TSV row: verdict, reported cost,
    /// tokens, what the session started with, and the skills it opened.
    Judge {
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
        #[arg(long)]
        tag: String,
        /// The session's `stream-json` transcript. Required for a `trace`
        /// checker; read for its metadata by every kind when given.
        #[arg(long)]
        transcript: Option<PathBuf>,
        /// The fixture workspace the session started from (`diff`). Absent
        /// means an empty start.
        #[arg(long)]
        fixture: Option<PathBuf>,
        /// The session's final workspace (`diff`).
        #[arg(long)]
        workspace: Option<PathBuf>,
        /// The exit status of the probe's `check.sh` (`run`).
        #[arg(long)]
        run_exit: Option<i32>,
    },
    /// Turn a results file into the pilot table. The library says how each
    /// rule's layer reaches a WITH session, which decides whether its WITH arm
    /// is split by load.
    Tally {
        #[arg(long)]
        results: PathBuf,
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
    },
}

/// Every way a probe subcommand can fail.
#[derive(Debug, thiserror::Error)]
pub enum ProbeCliError {
    #[error("`--tag`: {0}")]
    Tag(#[from] RuleTagError),
    #[error("no rule `{0}` in the library")]
    NoSuchRule(String),
    #[error("rule `{0}` carries no `[probe]`")]
    NoProbe(String),
    #[error("a `{kind}` checker needs `--{arg}`")]
    MissingInput {
        kind: &'static str,
        arg: &'static str,
    },
    #[error(
        "check.sh exited {0}: a run checker exits 0 (held), 1 (violated) or 2 \
         (inconclusive), and anything else is a checker error, not a verdict"
    )]
    RunExit(i32),
    #[error(transparent)]
    Read(#[from] ProbeReadError),
    #[error("transcript: {0}")]
    Transcript(#[from] TranscriptError),
    #[error("results: {0}")]
    Results(#[from] ResultsError),
}

pub(super) fn dispatch(command: ProbeCommand) -> Result<ExitCode, CliError> {
    match command {
        ProbeCommand::List { rules } => list(&rules),
        ProbeCommand::Task { rules, tag } => {
            let library = fsio::load_rules(&rules)?.validate()?;
            let rule = find(library.rules(), &tag)?;
            let probe = rule
                .probe()
                .ok_or_else(|| ProbeCliError::NoProbe(tag.clone()))?; // allow:clone: the error owns the tag it names
            println!("{}", probe.task().as_str());
            Ok(ExitCode::SUCCESS)
        }
        ProbeCommand::Judge {
            rules,
            tag,
            transcript,
            fixture,
            workspace,
            run_exit,
        } => judge(&Judge {
            rules: &rules,
            tag: &tag,
            transcript: transcript.as_deref(),
            fixture: fixture.as_deref(),
            workspace: workspace.as_deref(),
            run_exit,
        }),
        ProbeCommand::Tally { results, rules } => {
            let library = fsio::load_rules(&rules)?.validate()?;
            let text = fsio::read_probe_text(&results).map_err(ProbeCliError::from)?;
            let rows = parse_results(&text).map_err(ProbeCliError::from)?;
            let (tallies, cost) = tally(&rows, |tag| delivery(library.rules(), tag));
            print!("{}", render(&tallies, cost));
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn find<'a>(rules: &'a [Rule], tag: &str) -> Result<&'a Rule, ProbeCliError> {
    let tag = RuleTag::parse(tag)?;
    rules
        .iter()
        .find(|r| r.tag() == &tag)
        .ok_or_else(|| ProbeCliError::NoSuchRule(tag.as_str().to_owned()))
}

/// Which build target writes a home's layer the way a session loads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayerTarget {
    /// A skill, reached by description: `global`, and a domain with no known
    /// file glob.
    Skill,
    /// A `.claude/rules/` file, resident or attached on reading: a project,
    /// and a known-language domain.
    RulesLayer,
}

impl LayerTarget {
    fn of(home: &Home) -> Self {
        match (home, LoadSemantics::for_home(home)) {
            (Home::Global | Home::Org { .. }, _) | (_, LoadSemantics::OnRequest) => {
                LayerTarget::Skill
            }
            (_, LoadSemantics::Always | LoadSemantics::WhenReading(_)) => LayerTarget::RulesLayer,
        }
    }

    /// The `--targets` name the runner builds the WITH arm's layer with.
    fn as_str(self) -> &'static str {
        match self {
            LayerTarget::Skill => "claude",
            LayerTarget::RulesLayer => "claude-rules",
        }
    }
}

/// How a rule's layer reaches a WITH session, for the load split.
fn delivery(rules: &[Rule], tag: &RuleTag) -> Delivery {
    let Some(rule) = rules.iter().find(|r| r.tag() == tag) else {
        return Delivery::NotInLibrary;
    };
    match LayerTarget::of(rule.home()) {
        LayerTarget::Skill => SkillName::new(HomeSlug::of(rule.home()).as_str())
            .map_or(Delivery::NotInLibrary, Delivery::Skill),
        LayerTarget::RulesLayer => Delivery::RulesLayer,
    }
}

fn list(rules: &Path) -> Result<ExitCode, CliError> {
    let library = fsio::load_rules(rules)?.validate()?;
    println!("#tag\thome\tlayer\tworkspace\tchecker");
    for rule in library.rules() {
        if let Some(probe) = rule.probe() {
            println!(
                "{}\t{}\t{}\t{}\t{}",
                rule.tag().as_str(),
                HomeSlug::of(rule.home()).as_str(),
                LayerTarget::of(rule.home()).as_str(),
                probe.workspace().as_str(),
                probe.checker().kind()
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}

struct Judge<'a> {
    rules: &'a Path,
    tag: &'a str,
    transcript: Option<&'a Path>,
    fixture: Option<&'a Path>,
    workspace: Option<&'a Path>,
    run_exit: Option<i32>,
}

fn judge(args: &Judge<'_>) -> Result<ExitCode, CliError> {
    let library = fsio::load_rules(args.rules)?.validate()?;
    let rule = find(library.rules(), args.tag)?;
    let probe = rule
        .probe()
        .ok_or_else(|| ProbeCliError::NoProbe(args.tag.to_owned()))?;
    let session = match args.transcript {
        Some(path) => {
            let text = fsio::read_probe_text(path).map_err(ProbeCliError::from)?;
            Some(session_from_stream_json(&text).map_err(ProbeCliError::from)?)
        }
        None => None,
    };
    let verdict = match probe.checker() {
        Checker::Trace(t) => {
            let session = session.as_ref().ok_or(ProbeCliError::MissingInput {
                kind: "trace",
                arg: "transcript",
            })?;
            t.judge(session.events())
        }
        Checker::Diff(d) => {
            let workspace = args.workspace.ok_or(ProbeCliError::MissingInput {
                kind: "diff",
                arg: "workspace",
            })?;
            let change =
                fsio::workspace_change(args.fixture, workspace).map_err(ProbeCliError::from)?;
            d.judge(&change, rule.detectors())
        }
        Checker::Run(_) => match args.run_exit {
            Some(0) => Verdict::Held,
            Some(1) => Verdict::Violated,
            Some(2) => Verdict::Inconclusive,
            Some(other) => return Err(ProbeCliError::RunExit(other).into()),
            None => {
                return Err(ProbeCliError::MissingInput {
                    kind: "run",
                    arg: "run-exit",
                }
                .into());
            }
        },
    };
    let (cost, tokens, loaded, invoked) = match &session {
        Some(s) => (
            s.cost().map(|c| c.to_string()).unwrap_or_default(),
            s.tokens().map(|t| t.to_string()).unwrap_or_default(),
            s.loaded().join(","),
            s.skills_invoked().join(","),
        ),
        None => Default::default(),
    };
    println!("{verdict}\t{cost}\t{tokens}\t{loaded}\t{invoked}");
    Ok(ExitCode::SUCCESS)
}
