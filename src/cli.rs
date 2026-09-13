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
//! `--scope <audience>` narrows the same three commands by *audience* rather
//! than by owner, and is repeatable. The two questions are independent and so
//! are the flags: `--home` asks who maintains a rule, `--scope` asks who loads
//! it. **A rule declaring no `applies_to` is emitted under every audience**, so
//! adding a scope to one rule can never remove a different rule from an
//! existing build; only a scoped rule can be withheld, and only from an
//! audience it does not name. An audience no rule declares is an error for
//! `build` and `verify` — the emission would otherwise be quietly missing every
//! scoped rule while looking like a success — and merely an empty listing for
//! `list`, exactly as `--home` is.
//!
//! **Must NOT:** contain business logic. It translates arguments into calls on
//! `library`/`emit`/`fsio` and formats their results; the rules of the domain
//! live in those modules. Library code returns typed errors ([`CliError`] wraps
//! them); this module turns them into a process exit and a diagnostic.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use crate::aggregate::{Aggregate, AggregateError};
use crate::contribute::{Contribution, NotContributable};
use crate::emit::{self, HomeSlug, OutputFile};
use crate::fsio::{self, LoadError, RuleWriteError, VerifyError, WriteError};
use crate::library::{Library, Validated, ValidationError};
use crate::lint;
use crate::poke::{self, BroadcastCap, Reach, Trigger, UnknownTrigger};
use crate::report::{InstallId, InstallIdError, K_ANONYMITY_FLOOR, Month, MonthError, Report};
use crate::rule::{
    AdoptError, CachedIsNotEditable, Date, DateError, EditableRule, RuleTag, RuleTagError,
    ScopeTag, ScopeTagError,
};
use crate::scrub::{ScrubError, TermList};

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
            default_value = "claude,cursor,copilot,agents,claude-rules"
        )]
        targets: Vec<Target>,
        /// Only emit rules homed in this layer (its slug: `global`, `domain-<name>`,
        /// `project-<slug>`). A slug no rule is in is an error, not an empty emission.
        #[arg(long)]
        home: Option<String>,
        /// Narrow to one audience; repeatable (`--scope rust --scope java`).
        /// A rule declaring no `applies_to` is emitted whatever is asked for;
        /// only a scoped rule can be withheld, and only from an audience it does
        /// not name. A scope no rule declares is an error, not a quiet drop.
        #[arg(long)]
        scope: Vec<String>,
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
        /// Only rules served by this audience; repeatable. Unlike `build` and
        /// `verify`, a scope no rule declares simply prints nothing — printing
        /// nothing *is* an answer here, as it is for `list --home`.
        #[arg(long)]
        scope: Vec<String>,
    },
    /// Take a deliberate fork of a cached rule: it becomes editable and records
    /// what it was forked from, at which upstream version, and when.
    ///
    /// The only command that writes a rule file. Editing a cache in place is a
    /// silent fork; this is the loud one.
    Adopt {
        /// Directory of `*.md` rule files.
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
        /// The rule to adopt, e.g. `R:verify-through-production-path`.
        #[arg(long)]
        tag: String,
        /// The date of the adoption, `YYYY-MM-DD`.
        ///
        /// **Given, never read from a clock.** The tool consults no ambient
        /// state — the same invariant that keeps a config file out — and a
        /// clock is ambient state that would also make this command
        /// untestable and its output unreproducible. `tests/solo_mode.rs`
        /// holds it.
        #[arg(long)]
        on: String,
    },
    /// Prepare one rule to leave this machine, and show exactly what would go.
    ///
    /// Prints the contribution and writes **nothing** unless `--confirm` is
    /// given: the default is to show, because the only real control on what a
    /// published incident says is a person reading it. The raw `incident` and
    /// every recurrence are structurally absent from the output — see
    /// `contribute::Contribution`. Writes a file; transmits nothing.
    Contribute {
        /// Directory of `*.md` rule files.
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
        /// The rule to contribute.
        #[arg(long)]
        tag: String,
        /// Path to the banned-terms list (salted digests; see
        /// `scripts/no-banned-names.sh` for the format).
        ///
        /// **Required, so a contribution with no list is unconstructible rather
        /// than a case somebody remembers to handle.** The script exits 2 when
        /// disarmed; here the disarmed state cannot be reached. The path is
        /// given rather than looked up in a home directory, because the tool
        /// consults no ambient state.
        #[arg(long)]
        terms: PathBuf,
        /// Where to write the contribution. Only used with `--confirm`.
        #[arg(long, default_value = "contributions")]
        out: PathBuf,
        /// Write the file. Without it, the contribution is printed and nothing
        /// is written — two steps, so the text is read before it exists
        /// anywhere a `git push` could reach.
        #[arg(long)]
        confirm: bool,
    },
    /// Write this install's **anonymous** recurrence report into a cloned
    /// aggregate repository.
    ///
    /// An upstream tag, a bucketed count, a month, a status kind and a control
    /// kind. Nothing else — no title, no incident, no body, no path, no name,
    /// no day-level date. Prints what would go and writes only on `--confirm`;
    /// transmits nothing, because publishing it is a pull request you open.
    Report {
        /// Directory of `*.md` rule files.
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
        /// Path to your clone of the aggregate repository. **The pseudonym
        /// lives here**, in `reports/<install-id>.toml`, never on the machine:
        /// nothing in a home directory, nothing in the environment. Delete the
        /// clone and the pseudonym is gone.
        #[arg(long)]
        aggregate: PathBuf,
        /// The month this report covers, `YYYY-MM`. Given rather than read from
        /// a clock, like every other date this tool handles.
        #[arg(long)]
        generated: String,
        /// The pseudonym to use, **required only the first time**: after that it
        /// is read from the report already in the clone, so it is typed once
        /// rather than every run — a mistyped id on a later run would fork one
        /// install's history into two and inflate every count it appears in.
        #[arg(long)]
        install: Option<String>,
        /// Write the report into the clone. Without it, nothing is written.
        #[arg(long)]
        confirm: bool,
    },
    /// Recompute an aggregate from the reports in a cloned aggregate repository.
    ///
    /// **The scheduled job's command, and it reads only the clone.** No network,
    /// no fetch: reports arrive as pull requests, a job runs this over them, and
    /// the result is committed. Applies the k-floor — a rule below it does not
    /// appear at all, tag included — and prints both confounds beside the
    /// numbers, always.
    ///
    /// Nothing a `build` does depends on the result. An aggregate is never
    /// authoritative, and `tests/aggregate.rs` reads the emitters' source to
    /// make sure it stays that way.
    Aggregate {
        /// Path to the clone holding `reports/*.toml`.
        #[arg(long)]
        clone: PathBuf,
        /// The month the aggregate covers, `YYYY-MM`. Given, never clocked.
        #[arg(long)]
        generated: String,
        /// Write `aggregate.toml` into the clone. Without it, nothing is
        /// written — the same two steps as `report` and `contribute`.
        #[arg(long)]
        confirm: bool,
    },
    /// Report advisory lint findings (overlapping scope, home-slug collisions,
    /// dangling references). Writes nothing; exits non-zero if any are found.
    ///
    /// With `--upstream`, also prints the federation's **pokes** — what the
    /// corpus knows that this install might want to. A poke has no severity and
    /// never changes the exit code: it is news from elsewhere, and a signal
    /// that can fail a run has made federation required.
    Lint {
        /// Directory of `*.md` rule files.
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
        /// Lowest severity that makes the run fail. Findings below it are still
        /// printed — they are made non-fatal, never hidden.
        #[arg(long, value_enum, default_value_t = DenyLevel::Warning)]
        deny: DenyLevel,
        /// Path to a clone of the aggregate repository, to be poked from:
        /// `rules/*.md` is the upstream corpus, `reports/*.toml` the
        /// recurrence signal. **Without it there are no pokes at all**, and no
        /// warning about their absence — a solo install is the product, and a
        /// nag is a requirement with better manners.
        #[arg(long)]
        upstream: Option<PathBuf>,
        /// Which poke triggers to run; repeatable. Naming any **replaces** the
        /// default set (`class-covered`, `cache-behind`) rather than adding to
        /// it, so one flag says exactly what will fire.
        #[arg(long)]
        poke: Vec<String>,
        /// How many broadcast pokes one run may print. Reactive pokes are not
        /// capped: they follow evidence recorded here. `0` turns broadcast off
        /// entirely.
        #[arg(long)]
        poke_cap: Option<usize>,
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
            default_value = "claude,cursor,copilot,agents,claude-rules"
        )]
        targets: Vec<Target>,
        /// Only emit rules homed in this layer (its slug: `global`, `domain-<name>`,
        /// `project-<slug>`). A slug no rule is in is an error, not an empty emission.
        #[arg(long)]
        home: Option<String>,
        /// Narrow to one audience; repeatable. Must match the `--scope` the
        /// paired `build` used, or the two disagree about what should be on disk.
        #[arg(long)]
        scope: Vec<String>,
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
    ClaudeRules,
}

/// The lowest [`lint::Severity`] that makes `relearn lint` exit non-zero.
///
/// A CLI-local enum rather than a `ValueEnum` derive on `lint::Severity`: the
/// domain type must not learn about `clap`, and the CLI's vocabulary is free to
/// differ from the domain's (there is deliberately no `--deny info`, because
/// `Info` findings are advisory by definition and a level nobody should select
/// is a level that should not exist).
///
/// **Why this flag exists** (2026-09-06). Every finding until now was a
/// structural defect fixable by editing the library, so "any finding fails the
/// run" was the whole policy. `UnheldRecurrence` is the first that is
/// legitimately long-lived: it says a rule needs promoting to a stronger
/// control, and building that control may be work in another repository
/// entirely. Blocking CI on it produces a permanently-red gate, and an
/// always-red check is a muted check (`[R:xplat-fixtures]`). The default is
/// unchanged, so no existing invocation became permissive by accident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum DenyLevel {
    /// Fail on any `Warning` or `Error` finding. The default, and the
    /// behaviour of every `relearn lint` invocation before this flag existed.
    Warning,
    /// Fail only on `Error` — a finding that makes the *emitted tree* wrong.
    /// Warnings are printed and do not fail the run.
    Error,
}

impl DenyLevel {
    /// The domain severity this level denies from.
    fn threshold(self) -> lint::Severity {
        match self {
            DenyLevel::Warning => lint::Severity::Warning,
            DenyLevel::Error => lint::Severity::Error,
        }
    }
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
    /// `adopt --tag` was not a well-formed rule tag.
    #[error("--tag: {0}")]
    Tag(RuleTagError),
    /// `adopt --on` was not a well-formed date.
    ///
    /// The date is given rather than read from a clock: the tool consults no
    /// ambient state, and a clock would also make adoption unreproducible.
    #[error("--on: {0}")]
    AdoptDate(DateError),
    /// `adopt --tag` named a rule the library does not hold.
    #[error("no rule in this library carries the tag {requested}")]
    NoSuchRule {
        /// The tag as given on the command line.
        requested: String,
    },
    /// The rule named is not a cache, so there is nothing to fork.
    #[error(transparent)]
    Adopt(AdoptError),
    /// A rule that is not editable reached the write path. Unreachable through
    /// `adopt` — adoption is what makes a rule editable — and kept because the
    /// witness returns a `Result` that no caller may discard by `unwrap`.
    #[error(transparent)]
    NotEditable(CachedIsNotEditable),
    /// Writing a rule file failed.
    #[error(transparent)]
    RuleWrite(#[from] RuleWriteError),
    /// A report in the clone could not be read into the aggregate.
    ///
    /// Reports come from strangers, so this is a refusal rather than a skip: an
    /// aggregate that ignored what it could not parse would publish a count
    /// quietly missing whoever wrote it.
    #[error(transparent)]
    Aggregate(AggregateError),
    /// `report --generated` was not a well-formed `YYYY-MM`.
    #[error("--generated: {0}")]
    ReportMonth(MonthError),
    /// `report --install` was not a well-formed pseudonym.
    #[error("--install: {0}")]
    InstallId(InstallIdError),
    /// The clone holds no report yet and no pseudonym was given.
    ///
    /// Required only the first time: after that the clone remembers, which is
    /// what keeps a mistyped id from forking one install's history into two.
    #[error(
        "this clone holds no report yet — pass --install <eight hex characters> once, \
         and it will be read from the clone from then on"
    )]
    NoInstallId,
    /// `--install` disagreed with the pseudonym already in the clone.
    ///
    /// Refused rather than honoured: silently switching pseudonyms is how one
    /// install becomes two in the aggregate, inflating every count it appears
    /// in — the confound §8 already warns about, manufactured by a typo.
    #[error(
        "this clone already reports as {found}, but --install says {given} — \
         refusing to switch pseudonyms, which would count one install as two"
    )]
    InstallIdConflict {
        /// The pseudonym found in the clone.
        found: String,
        /// The pseudonym given on the command line.
        given: String,
    },
    /// The clone holds several reports, so which install this is cannot be read
    /// from it. Two pseudonyms in one clone means two installs sharing it, and
    /// picking one would attribute this install's recurrences to the other.
    #[error(
        "this clone holds {count} reports, so it cannot say which install this is — \
         use one clone per install"
    )]
    SeveralInstallIds {
        /// How many were found.
        count: usize,
    },
    /// The banned-terms list could not be read.
    #[error("--terms {path:?}: {source}")]
    TermsUnreadable {
        /// The list path as given.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The banned-terms list could not arm the matcher.
    #[error(transparent)]
    Terms(ScrubError),
    /// The rule may not be contributed.
    #[error(transparent)]
    NotContributable(NotContributable),
    /// A protected name appears in text that would have left the machine.
    ///
    /// **The diagnostic names the field and the location, never the match** —
    /// printing it would move the exposure into a terminal, a CI log or a
    /// session transcript rather than closing it
    /// (`[R:report-the-hit-not-the-match]`).
    #[error("`{field}` contains a protected name — {detail}; rewrite it and try again")]
    BannedName {
        /// Which authored field the hit was in.
        field: &'static str,
        /// Location and length, from `scrub::Hit` — never the term.
        detail: String,
    },
    /// The contribution file could not be written.
    #[error("{path:?}: {source}")]
    ContributionWrite {
        /// The path at fault.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// `--scope` was not a well-formed scope.
    ///
    /// Deliberately **not** `#[from]`: the derived `source` would make
    /// [`report`] print the same sentence twice, once as the error and once as
    /// its own cause. The prefix is what the reader needs — which flag was
    /// wrong — so it is kept and the redundant chain is not.
    #[error("--scope: {0}")]
    Scope(ScopeTagError),
    /// `--scope` named an audience no rule in the library declares.
    ///
    /// Loud for a **different reason** than [`CliError::UnknownHome`], and the
    /// difference is worth stating because it is sharper. An unknown home
    /// produces an empty emission, which at least looks wrong. An unknown scope
    /// produces an emission that is *quietly missing every scoped rule* while
    /// every unscoped one is still there — output that looks like success. A
    /// mistyped `--scope rsut` must fail rather than ship a tree with the Rust
    /// corpus silently absent from it.
    #[error("no rule declares scope {requested} ({})", declared(known))]
    UnknownScope {
        /// The scope as given on the command line.
        requested: String,
        /// Every scope the library does declare, sorted and deduplicated.
        known: Vec<String>,
    },
    /// `--poke` or `--poke-cap` was given without `--upstream`.
    ///
    /// Refused rather than ignored. There is nothing to poke from without a
    /// clone, so the flag would do nothing — and a flag that silently does
    /// nothing reads, to whoever wrote the command, as a feature that ran.
    #[error(
        "--poke and --poke-cap need --upstream: a poke is read out of a clone of the aggregate \
         repository, and there is nothing to read without one"
    )]
    PokeWithoutUpstream,
    /// `--poke` named a trigger that does not exist.
    ///
    /// Not `#[from]`, for the same reason [`CliError::Scope`] is not: the
    /// derived source would print the same sentence twice.
    #[error("--poke: {0}")]
    PokeTrigger(UnknownTrigger),
}

/// Render the declared-scope list for [`CliError::UnknownScope`].
///
/// An empty list is its own message rather than an empty parenthesis: "declared
/// scopes: " followed by nothing tells a reader the lookup failed, not that
/// scoping is unused in this library, and those call for different fixes.
fn declared(known: &[String]) -> String {
    if known.is_empty() {
        "no rule in this library declares `applies_to`".to_owned()
    } else {
        format!("declared: {}", known.join(", "))
    }
}

/// Parse the command line, dispatch, and turn the outcome into a process exit.
/// Diagnostics go to stderr; the success summary goes to stdout.
#[must_use]
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    match dispatch(cli.command) {
        Ok(code) => code,
        Err(err) => {
            report_error(&err);
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
            scope,
        } => {
            build(&rules, &out, &targets, home.as_deref(), &scope)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::List { rules, home, scope } => {
            list(&rules, home.as_deref(), &scope)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Adopt { rules, tag, on } => {
            adopt(&rules, &tag, &on)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Contribute {
            rules,
            tag,
            terms,
            out,
            confirm,
        } => contribute(&rules, &tag, &terms, &out, confirm),
        Command::Report {
            rules,
            aggregate,
            generated,
            install,
            confirm,
        } => report(&rules, &aggregate, &generated, install.as_deref(), confirm),
        Command::Aggregate {
            clone,
            generated,
            confirm,
        } => aggregate(&clone, &generated, confirm),
        Command::Lint {
            rules,
            deny,
            upstream,
            poke,
            poke_cap,
        } => lint_rules(&rules, deny, upstream.as_deref(), &poke, poke_cap),
        Command::Verify {
            rules,
            out,
            targets,
            home,
            scope,
        } => verify(&rules, &out, &targets, home.as_deref(), &scope),
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
            Target::ClaudeRules => files.extend(emit::claude_rules::emit(validated)),
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

/// Restrict a validated library to the rules that serve `audience`, or fail
/// loudly if a requested scope is one no rule declares. `build` and `verify`
/// share this for the same reason they share [`restrict_to_home`] and
/// [`emit_selected`]: two implementations of "what is in scope" would
/// eventually disagree, and the gate would be checking a different set from the
/// one that was written.
///
/// **An empty `audience` narrows nothing**, and every unscoped rule survives
/// every audience — the safety default, held in [`Rule::serves`] rather than
/// here, so no caller can reimplement it differently.
fn restrict_to_scopes(
    validated: &Library<Validated>,
    audience: &[String],
) -> Result<Library<Validated>, CliError> {
    if audience.is_empty() {
        return Ok(validated.filter(|_| true));
    }
    // Parse at the perimeter: `--scope Rust` is rejected here as a malformed
    // scope, before anything is compared, so nothing downstream matches strings.
    let requested: Vec<ScopeTag> = audience
        .iter()
        .map(|s| ScopeTag::parse(s.as_str()).map_err(CliError::Scope))
        .collect::<Result<Vec<_>, _>>()?;

    let mut known: Vec<String> = validated
        .rules()
        .iter()
        .flat_map(|r| r.applies_to())
        .map(|s| s.as_str().to_owned())
        .collect();
    known.sort();
    known.dedup();

    if let Some(unknown) = requested
        .iter()
        .find(|s| !known.iter().any(|k| k == s.as_str()))
    {
        return Err(CliError::UnknownScope {
            requested: unknown.as_str().to_owned(),
            known,
        });
    }
    Ok(validated.filter(|r| r.serves(&requested)))
}

/// `build`: load, validate, emit each unique target, and write under `out`.
fn build(
    rules: &Path,
    out: &Path,
    targets: &[Target],
    home: Option<&str>,
    scope: &[String],
) -> Result<(), CliError> {
    let validated = restrict_to_home(&fsio::load_rules(rules)?.validate()?, home)?;
    let validated = restrict_to_scopes(&validated, scope)?;
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
    scope: &[String],
) -> Result<ExitCode, CliError> {
    let validated = restrict_to_home(&fsio::load_rules(rules)?.validate()?, home)?;
    let validated = restrict_to_scopes(&validated, scope)?;
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

/// `adopt`: convert one cached rule into a deliberate fork, and write it back.
///
/// The only command that writes a rule file, and it goes through
/// [`EditableRule`] exactly as any future write path will have to: the adopted
/// rule is editable *because* adoption made it so, and the witness is minted
/// from that fact rather than assumed.
///
/// Four refusals, each naming what to do instead: the tag is malformed, no rule
/// carries it, the rule is not a cache (`AdoptError` says whether it is already
/// local or already adopted), or the date is not a date. The library is loaded
/// and validated first, so adoption cannot run against a corpus that does not
/// parse.
fn adopt(rules: &Path, tag: &str, on: &str) -> Result<(), CliError> {
    let tag = RuleTag::parse(tag).map_err(CliError::Tag)?;
    let on = Date::parse(on).map_err(CliError::AdoptDate)?;

    // `into_rules` rather than a borrow: the rewritten rule is owned outright,
    // so no field is cloned and the original cannot be used by accident.
    let rule = fsio::load_rules(rules)?
        .validate()?
        .into_rules()
        .into_iter()
        .find(|r| r.tag() == &tag)
        .ok_or_else(|| CliError::NoSuchRule {
            requested: tag.as_str().to_owned(),
        })?;

    let adopted = rule.authority().adopt(on).map_err(CliError::Adopt)?;
    let adopted = rule.with_authority(adopted);

    let editable = EditableRule::of(&adopted).map_err(CliError::NotEditable)?;
    let path = fsio::write_rule(rules, &editable)?;
    println!(
        "adopted {} — a local fork now, recorded in {}",
        tag.as_str(),
        path.display()
    );
    Ok(())
}

/// `contribute`: show exactly what would leave, and write it only on `--confirm`.
///
/// Order matters and is deliberate. The projection is built first, so the four
/// structural refusals (withheld home, no published incident, a cache, a
/// mandate) happen before anything is read or printed. Then the matcher runs
/// over the **published** incident and the body — the two fields a contributor
/// writes — and a hit stops everything, reporting a location and a length and
/// never the term (`[R:report-the-hit-not-the-match]`). Only then is the
/// document printed, and only with `--confirm` is it written.
///
/// **Writes a file. Transmits nothing.** Publication is a person running `git`,
/// which is why §9's aggregate is a repository rather than a service, and
/// `tests/solo_mode.rs` is what keeps that true.
fn contribute(
    rules: &Path,
    tag: &str,
    terms: &Path,
    out: &Path,
    confirm: bool,
) -> Result<ExitCode, CliError> {
    let tag = RuleTag::parse(tag).map_err(CliError::Tag)?;
    let terms_text =
        std::fs::read_to_string(terms).map_err(|source| CliError::TermsUnreadable {
            path: terms.to_path_buf(),
            source,
        })?;
    let list = TermList::parse(&terms_text).map_err(CliError::Terms)?;

    let library = fsio::load_rules(rules)?.validate()?;
    let rule = library
        .rules()
        .iter()
        .find(|r| r.tag() == &tag)
        .ok_or_else(|| CliError::NoSuchRule {
            requested: tag.as_str().to_owned(),
        })?;

    let contribution = Contribution::of(rule).map_err(CliError::NotContributable)?;

    // Both fields a contributor authors, checked before either is printed. The
    // body travels too, and a name in it leaks exactly as far as one in the
    // published incident.
    for (field, text) in [
        (
            "published_incident",
            contribution.published_incident().as_str(),
        ),
        ("body", rule.body().as_str()),
    ] {
        if let Some(hit) = list.find(text) {
            return Err(CliError::BannedName {
                field,
                detail: hit.to_string(),
            });
        }
    }

    println!("{}", contribution.what_would_leave());

    if !confirm {
        println!(
            "Nothing written. {} term(s) checked and none found; the matcher cannot judge \
             whether this text identifies someone without naming them.\n\
             Re-run with --confirm to write it.",
            list.len()
        );
        return Ok(ExitCode::SUCCESS);
    }

    std::fs::create_dir_all(out).map_err(|source| CliError::ContributionWrite {
        path: out.to_path_buf(),
        source,
    })?;
    let path = out.join(format!("{}.md", tag.body()));
    std::fs::write(&path, contribution.to_document()).map_err(|source| {
        CliError::ContributionWrite {
            path: path.clone(), // allow:clone: the error owns the path for its diagnostic, and the success line below prints it
            source,
        }
    })?;
    println!(
        "written to {} — nothing has been transmitted; publishing it is a pull request you open yourself",
        path.display()
    );
    Ok(ExitCode::SUCCESS)
}

/// `report`: build this install's anonymous recurrence report, show it, and
/// write it into the clone only on `--confirm`.
///
/// **Where the pseudonym comes from is Phase 0.2's decision, implemented here.**
/// It lives in the cloned aggregate repository — `reports/<install-id>.toml` is
/// its own name — so nothing is read from a home directory or an environment
/// variable, and `tests/solo_mode.rs` needs no exemption. A clone that already
/// holds a report supplies the id, which is why `--install` is required only the
/// first time: typed once, a mistyped id is harmless; typed every run, it
/// silently forks one install's history into two and inflates every count it
/// appears in.
fn report(
    rules: &Path,
    aggregate: &Path,
    generated: &str,
    install: Option<&str>,
    confirm: bool,
) -> Result<ExitCode, CliError> {
    let generated = Month::parse(generated).map_err(CliError::ReportMonth)?;
    let reports_dir = aggregate.join("reports");

    let existing = existing_install_id(&reports_dir)?;
    let install = match (existing, install) {
        // The clone remembers. A conflicting `--install` is refused rather than
        // honoured: silently switching pseudonyms is how one install becomes
        // two in the aggregate.
        (Some(found), Some(given)) if found.as_str() != given => {
            return Err(CliError::InstallIdConflict {
                found: found.as_str().to_owned(),
                given: given.to_owned(),
            });
        }
        (Some(found), _) => found,
        (None, Some(given)) => InstallId::parse(given).map_err(CliError::InstallId)?,
        (None, None) => return Err(CliError::NoInstallId),
    };

    let library = fsio::load_rules(rules)?.validate()?;
    let report = Report::of(&library, install, generated);
    let document = report.to_toml();

    println!("{document}");
    println!(
        "{} observation(s). Nothing else leaves: no title, no incident, no body, no path, \
         no name, no day-level date.\n\
         The aggregate publishes no count until {} distinct installs have reported it.",
        report.observations().len(),
        K_ANONYMITY_FLOOR
    );

    if !confirm {
        println!("Nothing written. Re-run with --confirm to write it into the clone.");
        return Ok(ExitCode::SUCCESS);
    }

    std::fs::create_dir_all(&reports_dir).map_err(|source| CliError::ContributionWrite {
        path: reports_dir.clone(), // allow:clone: the error owns the path for its diagnostic, and the write below needs the directory again
        source,
    })?;
    let path = reports_dir.join(format!("{}.toml", report.install().as_str()));
    std::fs::write(&path, &document).map_err(|source| CliError::ContributionWrite {
        path: path.clone(), // allow:clone: as above — the success line prints the path the error would have owned
        source,
    })?;
    println!(
        "written to {} — nothing has been transmitted. Publishing it is a pull request you open, \
         and note that the commit itself records a day-level date even though this file does not.",
        path.display()
    );
    Ok(ExitCode::SUCCESS)
}

/// Every report document in a clone's `reports/` directory, in path order.
///
/// **One reader, two callers** — the `aggregate` recompute and the poke — so
/// the two can never disagree about which files count or what a missing
/// directory means. A clone with no `reports/` yet is an empty list rather than
/// an error: an aggregate nobody has reported into is a real state.
///
/// Sorted, so the same reports always produce the same document: a job that
/// committed a different byte order every run would make every recompute look
/// like a change.
fn read_reports(reports_dir: &Path) -> Result<Vec<String>, CliError> {
    let mut documents: Vec<String> = Vec::new();
    match std::fs::read_dir(reports_dir) {
        Ok(entries) => {
            let mut paths: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "toml"))
                .collect();
            paths.sort();
            for path in paths {
                documents.push(std::fs::read_to_string(&path).map_err(|source| {
                    CliError::ContributionWrite {
                        path: path.clone(), // allow:clone: the error owns the path for its diagnostic on the failure path
                        source,
                    }
                })?);
            }
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(CliError::ContributionWrite {
                path: reports_dir.to_path_buf(),
                source,
            });
        }
    }
    Ok(documents)
}

/// The pseudonym already in the clone, if there is one.
///
/// A report file's **name** is the id, which is why it can be recovered without
/// parsing the file: `reports/7f3c9a1e.toml`. More than one is an error rather
/// than a guess — two pseudonyms in one clone means two installs sharing it, and
/// picking one would attribute this install's recurrences to the other.
fn existing_install_id(reports_dir: &Path) -> Result<Option<InstallId>, CliError> {
    let entries = match std::fs::read_dir(reports_dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(CliError::ContributionWrite {
                path: reports_dir.to_path_buf(),
                source,
            });
        }
    };

    let mut found: Vec<InstallId> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "toml")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && let Ok(id) = InstallId::parse(stem)
        {
            found.push(id);
        }
    }
    found.sort();
    found.dedup();

    match found.as_slice() {
        [] => Ok(None),
        [only] => Ok(Some(only.clone())), // allow:clone: the caller owns the id for the life of the command; the vector is dropped here
        several => Err(CliError::SeveralInstallIds {
            count: several.len(),
        }),
    }
}

/// `aggregate`: recompute from the reports in a clone, show the result, and
/// write it only on `--confirm`.
///
/// **Reads the clone and nothing else.** There is no fetch here and there must
/// never be one: reports arrive as pull requests, which is what makes
/// publication deliberate by construction and every byte that ever crossed
/// auditable in public history. `tests/solo_mode.rs` is what keeps the "for
/// convenience" version from appearing later.
///
/// A clone with no `reports/` directory is an empty aggregate rather than an
/// error — an aggregate nobody has reported into is a real state, and it still
/// carries its confounds, which is exactly when they matter most.
fn aggregate(clone: &Path, generated: &str, confirm: bool) -> Result<ExitCode, CliError> {
    let generated = Month::parse(generated).map_err(CliError::ReportMonth)?;
    let documents = read_reports(&clone.join("reports"))?;

    let aggregate =
        Aggregate::of(documents.iter().map(String::as_str)).map_err(CliError::Aggregate)?;
    let document = aggregate.to_toml(generated);
    println!("{document}");
    println!(
        "{} rule(s) published from {} install(s); {} withheld below the floor of {}.",
        aggregate.rows().len(),
        aggregate.installs(),
        aggregate.suppressed(),
        K_ANONYMITY_FLOOR
    );

    if !confirm {
        println!("Nothing written. Re-run with --confirm to write aggregate.toml into the clone.");
        return Ok(ExitCode::SUCCESS);
    }

    let path = clone.join("aggregate.toml");
    std::fs::write(&path, &document).map_err(|source| CliError::ContributionWrite {
        path: path.clone(), // allow:clone: as above — the success line prints the path the error would have owned
        source,
    })?;
    println!("written to {}", path.display());
    Ok(ExitCode::SUCCESS)
}

/// `list`: print each rule as `tag  [home-slug]  title`, optionally filtered.
///
/// Deliberately **not** using [`restrict_to_scopes`]: an audience no rule
/// declares prints nothing here rather than failing, mirroring `list --home`.
/// Printing nothing *is* an answer for a listing; for a build or a gate it is
/// not, which is why the two treat the same input differently. A malformed
/// `--scope` is still a parse error — the perimeter binds everywhere.
fn list(rules: &Path, home: Option<&str>, scope: &[String]) -> Result<(), CliError> {
    let validated = fsio::load_rules(rules)?.validate()?;
    let audience: Vec<ScopeTag> = scope
        .iter()
        .map(|s| ScopeTag::parse(s.as_str()).map_err(CliError::Scope))
        .collect::<Result<Vec<_>, _>>()?;
    for rule in validated.rules() {
        let slug = HomeSlug::of(rule.home());
        if home.is_some_and(|filter| filter != slug.as_str()) {
            continue;
        }
        if !rule.serves(&audience) {
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
fn lint_rules(
    rules: &Path,
    deny: DenyLevel,
    upstream: Option<&Path>,
    triggers: &[String],
    cap: Option<usize>,
) -> Result<ExitCode, CliError> {
    // Both argument checks happen **before** any work, so a mistyped flag
    // fails on the flag rather than after a full lint has printed — the same
    // reason `build` rejects an unknown scope before it writes anything.
    //
    // A poke flag with no clone to read is a flag that does nothing, and a
    // silently inert flag reads as a feature that ran. Refused rather than
    // ignored, for the same reason a dropped field is a parse error.
    if upstream.is_none() && (!triggers.is_empty() || cap.is_some()) {
        return Err(CliError::PokeWithoutUpstream);
    }
    let enabled: Vec<Trigger> = if triggers.is_empty() {
        Trigger::defaults()
    } else {
        // Naming any trigger **replaces** the default set rather than adding to
        // it, so one flag says exactly what will fire.
        triggers
            .iter()
            .map(|t| Trigger::parse(t).map_err(CliError::PokeTrigger))
            .collect::<Result<Vec<_>, _>>()?
    };
    let cap = cap.map_or(BroadcastCap::DEFAULT, BroadcastCap::new);

    let validated = fsio::load_rules(rules)?.validate()?;
    let findings = lint::lint(&validated);
    let t = lint::tally(&validated);
    let summary = format!(
        // The mandated count is printed **only when there is one**, and that is
        // not cosmetic: "0 mandated" on every line of every run is the kind of
        // permanently-zero column a reader learns to skip, and this number is
        // worth reading precisely when it is not zero — it is the size of the
        // hold-out that the two figures before it exclude.
        "{} rule(s): {} recurred, {} inert (codified and never fired){}",
        t.total(),
        t.recurred(),
        t.inert(),
        if t.mandated() == 0 {
            String::new()
        } else {
            format!(
                "; {} mandated, held out of both over {} evidential rule(s)",
                t.mandated(),
                t.evidential()
            )
        }
    );
    // Every finding is printed, whatever the threshold. `--deny` decides what
    // is *fatal*, never what is visible: a finding suppressed from the output
    // would be a check whose verdict never reaches the reader, which is the
    // failure `[R:verdict-survives-the-channel]` names.
    if findings.is_empty() {
        println!("ok: no lint findings");
    }
    for finding in &findings {
        println!("{}: {finding}", finding.severity().label());
    }
    // Findings below the threshold inform but do not fail the run. The default
    // threshold is `Warning`, so the out-of-the-box behaviour is unchanged.
    let actionable = findings
        .iter()
        .filter(|f| f.severity() >= deny.threshold())
        .count();
    // The two-number summary: the recurrence count alone is gameable through
    // under-reporting, so it is never shown without its counter (P5).
    println!("{summary}");
    if !findings.is_empty() {
        // The threshold is named in the summary so a reader of a green log can
        // see *why* a printed warning did not fail the run, rather than having
        // to know the flag's default to interpret the outcome.
        println!(
            "{} finding(s), {actionable} fatal at --deny {}",
            findings.len(),
            deny.threshold().label()
        );
    }

    // **After the verdict, and unable to change it.** The exit code below is
    // computed from findings alone; nothing a clone contains can reach it.
    if let Some(clone) = upstream {
        print_pokes(clone, &validated, &enabled, cap)?;
    }

    if actionable > 0 {
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Read the clone, work out what the corpus has to say, and print it.
///
/// **The only place `lint` reads anything but its own rules.** Failures here
/// are failures of the *read* — a clone that is not one, a rule file upstream
/// that does not parse, a report that does not — and they are errors rather
/// than warnings for the reason every read in this tool is: a poke silently
/// missing because half the clone could not be parsed is worse than no poke.
fn print_pokes(
    clone: &Path,
    local: &Library<Validated>,
    enabled: &[Trigger],
    cap: BroadcastCap,
) -> Result<(), CliError> {
    let upstream = load_upstream_rules(&clone.join("rules"))?;
    // Recomputed from the reports rather than read from a committed
    // `aggregate.toml`: nothing verifies that a published aggregate matches the
    // reports beside it, and the recompute needs no second parser to drift.
    let reports = read_reports(&clone.join("reports"))?;
    let aggregate =
        Aggregate::of(reports.iter().map(String::as_str)).map_err(CliError::Aggregate)?;

    let pokes = poke::pokes(local, &upstream, &aggregate, enabled, cap);
    if pokes.is_empty() {
        return Ok(());
    }
    for p in pokes.shown() {
        let reach = match p.reach() {
            Reach::Reactive => "reactive",
            Reach::Broadcast => "broadcast",
        };
        println!("poke [{reach}]: {p}");
    }
    if pokes.withheld() > 0 {
        // The cap publishes what it held back, for the same reason the
        // aggregate publishes its suppressed count: a throttled run must not
        // read as a quiet one.
        println!(
            "{} more broadcast poke(s) withheld by --poke-cap {}",
            pokes.withheld(),
            pokes.cap().get()
        );
    }
    println!("Pokes are news, not findings: none of them changed the exit code above.");
    Ok(())
}

/// The upstream corpus in a clone. A clone with no `rules/` yet is an **empty**
/// corpus rather than an error — an aggregate repository nobody has contributed
/// to is a real state, and the same forgiveness `aggregate` already shows a
/// missing `reports/`.
fn load_upstream_rules(dir: &Path) -> Result<Library<Validated>, CliError> {
    if !dir.exists() {
        return Ok(Library::new().validate()?);
    }
    Ok(fsio::load_rules(dir)?.validate()?)
}

/// Print an error and its source chain to stderr.
fn report_error(err: &CliError) {
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
             origin = \"mined\"\n\
             status = {{ kind = \"active\" }}\n\
             incident = \"i\"\n\
             +++\n\n\
             Body of {tag}.\n"
        )
    }

    fn write_rule(dir: &Path, file: &str, contents: &str) {
        std::fs::write(dir.join(file), contents).expect("write rule file");
    }

    /// A rule document carrying an `applies_to` array.
    fn scoped_rule_doc(tag: &str, home_toml: &str, title: &str, scopes: &[&str]) -> String {
        let list: Vec<String> = scopes.iter().map(|s| format!("\"{s}\"")).collect();
        rule_doc(tag, home_toml, title).replace(
            "incident = \"i\"\n",
            &format!("incident = \"i\"\napplies_to = [{}]\n", list.join(", ")),
        )
    }

    /// A rules directory holding one unscoped rule and one scoped to `rust`.
    fn mixed_rules_dir() -> tempfile::TempDir {
        let rules = tempfile::tempdir().expect("rules tempdir");
        write_rule(
            rules.path(),
            "g.md",
            &rule_doc("R:g", "{ kind = \"global\" }", "Global rule"),
        );
        write_rule(
            rules.path(),
            "s.md",
            &scoped_rule_doc(
                "R:s",
                "{ kind = \"domain\", name = \"rust\" }",
                "Scoped rule",
                &["rust"],
            ),
        );
        rules
    }

    /// The four-row table at the CLI surface. Asserted on the emitted
    /// `AGENTS.md`, which carries every rule in one file, so "was this rule
    /// emitted" is a substring question with no per-target confounds.
    #[test]
    fn build_scope_withholds_only_scoped_rules_that_do_not_match() {
        let rules = mixed_rules_dir();

        for (audience, scoped_expected) in [
            (Vec::new(), true),
            (vec!["rust".to_owned()], true),
            (vec!["java".to_owned()], false),
            (vec!["rust".to_owned(), "java".to_owned()], true),
        ] {
            let out = tempfile::tempdir().expect("out tempdir");
            // `java` is declared by no rule, so on its own it would be a loud
            // error; the row is exercised by declaring it on a third rule.
            write_rule(
                rules.path(),
                "j.md",
                &scoped_rule_doc(
                    "R:j",
                    "{ kind = \"domain\", name = \"rust\" }",
                    "Java rule",
                    &["java"],
                ),
            );
            build(rules.path(), out.path(), &[Target::Agents], None, &audience)
                .expect("build succeeds");

            let agents = std::fs::read_to_string(out.path().join("AGENTS.md"))
                .expect("AGENTS.md was written");
            assert!(
                agents.contains("R:g"),
                "the unscoped rule must survive every audience ({audience:?})"
            );
            assert_eq!(
                agents.contains("R:s"),
                scoped_expected,
                "the rust-scoped rule under audience {audience:?}"
            );
        }
    }

    /// A scope no rule declares is an error — and the reason is sharper than the
    /// unknown-home one. An unknown home emits nothing, which looks wrong; an
    /// unknown scope emits a tree that is *quietly missing every scoped rule*,
    /// which looks like success.
    #[test]
    fn build_with_an_unknown_scope_is_an_error_naming_the_declared_ones() {
        let rules = mixed_rules_dir();
        let out = tempfile::tempdir().expect("out tempdir");

        let err = build(
            rules.path(),
            out.path(),
            &[Target::Agents],
            None,
            &["rsut".to_owned()],
        )
        .expect_err("an unknown scope must fail");

        match err {
            CliError::UnknownScope { requested, known } => {
                assert_eq!(requested, "rsut");
                assert_eq!(known, vec!["rust".to_owned()]);
            }
            other => panic!("expected UnknownScope, got {other:?}"),
        }
        assert!(
            !out.path().join("AGENTS.md").exists(),
            "nothing may be written before the scope is rejected"
        );
    }

    /// The message must read as an answer in the case that will be commonest
    /// for a long time: a library where nothing is scoped at all.
    #[test]
    fn an_unknown_scope_against_an_unscoped_library_says_so() {
        let rules = tempfile::tempdir().expect("rules tempdir");
        let out = tempfile::tempdir().expect("out tempdir");
        write_rule(
            rules.path(),
            "g.md",
            &rule_doc("R:g", "{ kind = \"global\" }", "Global rule"),
        );

        let err = build(
            rules.path(),
            out.path(),
            &[Target::Agents],
            None,
            &["rust".to_owned()],
        )
        .expect_err("an unknown scope must fail");
        assert!(
            err.to_string().contains("no rule in this library declares"),
            "an empty declared-set needs its own message, got: {err}"
        );
    }

    /// `--scope Rust` is rejected at the perimeter as a malformed scope, before
    /// anything is compared — so nothing downstream ever matches raw strings.
    #[test]
    fn a_malformed_scope_is_rejected_before_the_lookup() {
        let rules = mixed_rules_dir();
        let out = tempfile::tempdir().expect("out tempdir");

        let err = build(
            rules.path(),
            out.path(),
            &[Target::Agents],
            None,
            &["Rust".to_owned()],
        )
        .expect_err("a malformed scope must fail");
        assert!(matches!(err, CliError::Scope(_)), "got {err:?}");
    }

    /// `list --scope` mirrors `list --home`: an audience nothing declares prints
    /// nothing rather than failing, because printing nothing *is* an answer for
    /// a listing. A build or a gate has no such reading, which is why the two
    /// commands treat the same input differently.
    #[test]
    fn list_with_an_undeclared_scope_prints_nothing_and_succeeds() {
        let rules = mixed_rules_dir();
        list(rules.path(), None, &["java".to_owned()]).expect("list succeeds");
        list(rules.path(), None, &["rust".to_owned()]).expect("list succeeds");
        // A malformed scope is still a parse error here — the perimeter binds
        // everywhere, even where an empty result is legitimate.
        assert!(list(rules.path(), None, &["Rust".to_owned()]).is_err());
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

        build(rules.path(), out.path(), &[Target::Claude], None, &[]).expect("build succeeds");

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
            &[Target::ClaudeRules],
            Some("domain-rust"),
            &[],
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
            &[Target::ClaudeRules],
            Some("domain-cobol"),
            &[],
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

        build(rules.path(), out.path(), &[Target::Cursor], None, &[]).expect("build succeeds");

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
            build(rules.path(), out.path(), &[Target::Claude], None, &[]),
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

        build(rules.path(), out.path(), &[Target::Agents], None, &[]).expect("build succeeds");
        assert_eq!(
            verify(rules.path(), out.path(), &[Target::Agents], None, &[]).expect("verify runs"),
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
            verify(rules.path(), out.path(), &[Target::Agents], None, &[]).expect("verify runs"),
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
            verify(rules.path(), out.path(), &[Target::Agents], None, &[]).expect("verify runs"),
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
        list(rules.path(), None, &[]).expect("list all succeeds");
        list(rules.path(), Some("global"), &[]).expect("filtered list succeeds");

        let missing = rules.path().join("does-not-exist");
        assert!(matches!(
            list(&missing, None, &[]),
            Err(CliError::Load(LoadError::ReadDir { .. }))
        ));
    }
}
