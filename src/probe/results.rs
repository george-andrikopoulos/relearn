//! A probe run's results file, and the table a pilot reports.
//!
//! The runner appends one tab-separated row per session, these columns in order:
//!
//! ```text
//! #tag, arm, sample, outcome, cost_usd, tokens, loaded, skills_invoked
//! ```
//!
//! `outcome` is a [`Verdict`], or `contaminated` (the session's `init` listed
//! something its arm did not put there) or `error` (the session or its checker
//! failed). Neither of the last two is a verdict: both are counted and shown,
//! and neither enters a rate. Rules are reported in the order the run first
//! recorded them, which is the order `relearn probe list` gave the runner.
//!
//! **Split by load.** A rule delivered as a skill reaches a WITH session only
//! if the model opens it, so the WITH arm is split by whether the rule's own
//! skill is in `skills_invoked`: a violation where it was never opened is a
//! delivery failure, not a prose failure. A rule whose layer is a
//! `.claude/rules/` file loads with no tool call, and is never split — it is
//! given no load rate rather than a load rate of zero.

use super::stats::{Count, LoadEvidence, LoadRate, Recommendation, newcombe, recommend, wilson};
use super::transcript::MicroUsd;
use crate::rule::{RuleTag, Verdict};

/// Which arm a session ran in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arm {
    /// The rule's home layer was in the workspace.
    With,
    /// The same workspace without it.
    Without,
}

impl Arm {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Arm::With => "with",
            Arm::Without => "without",
        }
    }
}

/// What one recorded session came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Judged(Verdict),
    /// A user layer, or anything else the arm did not add, reached the session.
    Contaminated,
    /// The session or its checker failed; nothing was judged.
    Failed,
}

/// Why a results file could not be read. Each names the line.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResultsError {
    #[error("line {line}: expected at least 4 tab-separated columns, found {found}")]
    Columns { line: usize, found: usize },
    #[error("line {line}: bad tag: {reason}")]
    Tag { line: usize, reason: String },
    #[error("line {line}: unknown arm {arm:?} (expected with | without)")]
    Arm { line: usize, arm: String },
    #[error(
        "line {line}: unknown outcome {outcome:?} \
         (expected held | violated | inconclusive | contaminated | error)"
    )]
    Outcome { line: usize, outcome: String },
    #[error("line {line}: cost {cost:?} is not a non-negative decimal")]
    Cost { line: usize, cost: String },
}

/// One session's row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    tag: RuleTag,
    arm: Arm,
    outcome: Outcome,
    cost: Option<MicroUsd>,
    /// The skills the session opened, as invoked. Empty when the column is
    /// absent: a file written before the column existed opened nothing that
    /// was recorded, which is what an empty list says.
    skills_invoked: Vec<String>,
}

/// A skill's name as relearn emits it: the home slug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillName(String);

impl SkillName {
    /// `None` for an empty name, which no skill has.
    #[must_use]
    pub fn new(slug: &str) -> Option<Self> {
        (!slug.is_empty()).then(|| SkillName(slug.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether a session that invoked `invoked` opened this skill: by its own
    /// name, or under the `<plugin>:<skill>` name an account-synced copy loads
    /// as.
    #[must_use]
    pub fn is_invoked_as(&self, invoked: &str) -> bool {
        invoked == self.0
            || invoked
                .rsplit_once(':')
                .is_some_and(|(_, name)| name == self.0)
    }
}

/// How a rule's home layer reaches a WITH session — the input to the split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    /// A skill the model opens with the `Skill` tool.
    Skill(SkillName),
    /// A `.claude/rules/` file: resident, or attached on reading. No tool call.
    RulesLayer,
    /// The results name a rule the library no longer holds.
    NotInLibrary,
}

/// A WITH arm split by whether the rule's skill was opened.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LoadSplit {
    loaded: ArmTally,
    not_loaded: ArmTally,
}

impl LoadSplit {
    /// Sessions that opened the rule's skill.
    #[must_use]
    pub fn loaded(self) -> ArmTally {
        self.loaded
    }

    /// Sessions that did not.
    #[must_use]
    pub fn not_loaded(self) -> ArmTally {
        self.not_loaded
    }

    /// Loaded over judged sessions (held, violated, inconclusive). A
    /// contaminated or failed session is not trusted to say what it opened.
    #[must_use]
    pub fn rate(self) -> Option<LoadRate> {
        LoadRate::new(
            self.loaded.judged(),
            self.loaded.judged() + self.not_loaded.judged(),
        )
    }

    fn evidence(self) -> Option<LoadEvidence> {
        self.rate()
            .map(|rate| LoadEvidence::new(rate, self.loaded.count()))
    }
}

/// What the tally can say about delivery, for one rule — the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Load {
    /// Delivered as a skill: the WITH arm, split.
    Split(LoadSplit),
    /// A `.claude/rules/` layer: the split does not apply.
    RulesLayer,
    /// Not in the library, so its delivery is unknown.
    NotInLibrary,
}

/// Parse a results file. `#` lines and blank lines are skipped.
///
/// # Errors
/// [`ResultsError`] naming the first malformed line.
pub fn parse_results(text: &str) -> Result<Vec<Row>, ResultsError> {
    let mut rows = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line_no = i + 1;
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 4 {
            return Err(ResultsError::Columns {
                line: line_no,
                found: cols.len(),
            });
        }
        let tag = RuleTag::parse(cols[0]).map_err(|e| ResultsError::Tag {
            line: line_no,
            reason: e.to_string(),
        })?;
        let arm = match cols[1] {
            "with" => Arm::With,
            "without" => Arm::Without,
            other => {
                return Err(ResultsError::Arm {
                    line: line_no,
                    arm: other.to_owned(),
                });
            }
        };
        let outcome = match cols[3] {
            "contaminated" => Outcome::Contaminated,
            "error" => Outcome::Failed,
            other => Outcome::Judged(Verdict::parse(other).map_err(|_| ResultsError::Outcome {
                line: line_no,
                outcome: other.to_owned(),
            })?),
        };
        let cost = match cols.get(4).copied().filter(|c| !c.is_empty()) {
            Some(c) => Some(MicroUsd::parse_decimal(c).ok_or(ResultsError::Cost {
                line: line_no,
                cost: c.to_owned(),
            })?),
            None => None,
        };
        let skills_invoked = cols
            .get(7)
            .map(|c| {
                c.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        rows.push(Row {
            tag,
            arm,
            outcome,
            cost,
            skills_invoked,
        });
    }
    Ok(rows)
}

/// One arm's sessions, by outcome.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ArmTally {
    held: u32,
    violated: u32,
    inconclusive: u32,
    contaminated: u32,
    failed: u32,
}

impl ArmTally {
    fn add(&mut self, outcome: Outcome) {
        let slot = match outcome {
            Outcome::Judged(Verdict::Held) => &mut self.held,
            Outcome::Judged(Verdict::Violated) => &mut self.violated,
            Outcome::Judged(Verdict::Inconclusive) => &mut self.inconclusive,
            Outcome::Contaminated => &mut self.contaminated,
            Outcome::Failed => &mut self.failed,
        };
        *slot += 1;
    }

    #[must_use]
    pub fn held(self) -> u32 {
        self.held
    }

    #[must_use]
    pub fn violated(self) -> u32 {
        self.violated
    }

    /// Sessions that reached a verdict: held, violated or inconclusive.
    #[must_use]
    pub fn judged(self) -> u32 {
        self.held + self.violated + self.inconclusive
    }

    /// Violations over judged-and-scored sessions, if there were any.
    #[must_use]
    pub fn count(self) -> Option<Count> {
        Count::new(self.violated, self.held + self.violated)
    }

    /// Sessions recorded but not scored: inconclusive, contaminated, failed.
    #[must_use]
    pub fn unscored(self) -> (u32, u32, u32) {
        (self.inconclusive, self.contaminated, self.failed)
    }
}

/// One rule's two arms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleTally {
    tag: RuleTag,
    without: ArmTally,
    with: ArmTally,
    load: Load,
}

impl RuleTally {
    #[must_use]
    pub fn tag(&self) -> &RuleTag {
        &self.tag
    }

    #[must_use]
    pub fn arm(&self, arm: Arm) -> ArmTally {
        match arm {
            Arm::With => self.with,
            Arm::Without => self.without,
        }
    }

    /// What the tally can say about delivery.
    #[must_use]
    pub fn load(&self) -> Load {
        self.load
    }

    #[must_use]
    pub fn recommendation(&self) -> Recommendation {
        let evidence = match self.load {
            Load::Split(split) => split.evidence(),
            Load::RulesLayer | Load::NotInLibrary => None,
        };
        recommend(self.without.count(), self.with.count(), evidence)
    }
}

/// Every rule's tally, in the order the run first recorded each, and the
/// summed reported cost. `delivery` says how each rule's layer reaches a
/// session; it is asked once per rule.
#[must_use]
pub fn tally(rows: &[Row], delivery: impl Fn(&RuleTag) -> Delivery) -> (Vec<RuleTally>, MicroUsd) {
    let mut out: Vec<(RuleTally, Delivery)> = Vec::new();
    let mut cost = 0u64;
    for row in rows {
        cost = cost.saturating_add(row.cost.map_or(0, MicroUsd::micros));
        let index = match out.iter().position(|(t, _)| t.tag == row.tag) {
            Some(i) => i,
            None => {
                let how = delivery(&row.tag);
                let load = match how {
                    Delivery::Skill(_) => Load::Split(LoadSplit::default()),
                    Delivery::RulesLayer => Load::RulesLayer,
                    Delivery::NotInLibrary => Load::NotInLibrary,
                };
                out.push((
                    RuleTally {
                        tag: row.tag.clone(), // allow:clone: the tally owns its key, outliving the borrowed rows
                        without: ArmTally::default(),
                        with: ArmTally::default(),
                        load,
                    },
                    how,
                ));
                out.len() - 1
            }
        };
        let (entry, how) = &mut out[index];
        match row.arm {
            Arm::Without => entry.without.add(row.outcome),
            Arm::With => {
                entry.with.add(row.outcome);
                match (&mut entry.load, &*how) {
                    (Load::Split(split), Delivery::Skill(name)) => {
                        let opened = row.skills_invoked.iter().any(|s| name.is_invoked_as(s));
                        let side = if opened {
                            &mut split.loaded
                        } else {
                            &mut split.not_loaded
                        };
                        side.add(row.outcome);
                    }
                    // `load` was derived from `how` when the entry was made, so
                    // only the pair above occurs; the rest carry no split.
                    (Load::Split(_), Delivery::RulesLayer | Delivery::NotInLibrary)
                    | (Load::RulesLayer | Load::NotInLibrary, _) => {}
                }
            }
        }
    }
    (
        out.into_iter().map(|(t, _)| t).collect(),
        MicroUsd::from_micros(cost),
    )
}

fn arm_cell(count: Option<Count>) -> String {
    match count {
        Some(c) => {
            let i = wilson(c);
            format!(
                "{}/{} [{:.2}, {:.2}]",
                c.violations(),
                c.scored(),
                i.lo(),
                i.hi()
            )
        }
        None => "0/0".to_owned(),
    }
}

/// The three load cells: the load rate, and the WITH violation rate given
/// the skill loaded and given it did not. A rules layer and an unknown rule
/// say why there is no split, never `0/0`.
fn load_cells(load: Load) -> (String, String, String) {
    match load {
        Load::Split(split) => {
            let rate = match split.rate() {
                Some(r) => {
                    let i = r.interval();
                    format!(
                        "{}/{} [{:.2}, {:.2}]",
                        r.loaded(),
                        r.sessions(),
                        i.lo(),
                        i.hi()
                    )
                }
                None => "0/0".to_owned(),
            };
            (
                rate,
                arm_cell(split.loaded().count()),
                arm_cell(split.not_loaded().count()),
            )
        }
        Load::RulesLayer => (
            "n/a: rules layer, no Skill call".to_owned(),
            "—".to_owned(),
            "—".to_owned(),
        ),
        Load::NotInLibrary => (
            "n/a: not in the library".to_owned(),
            "—".to_owned(),
            "—".to_owned(),
        ),
    }
}

/// The pilot table, as markdown.
#[must_use]
pub fn render(tallies: &[RuleTally], cost: MicroUsd) -> String {
    let mut out = String::from(
        "| Rule | WITHOUT violated [95% Wilson] | WITH violated [95% Wilson] | \
         Reduction [95% Newcombe] | Unscored w/o (inc, cont, err) | \
         Unscored with (inc, cont, err) | Load rate [95% Wilson] | \
         WITH violated, loaded | WITH violated, not loaded | Recommendation |\n\
         |---|---|---|---|---|---|---|---|---|---|\n",
    );
    for t in tallies {
        let diff = match (t.without.count(), t.with.count()) {
            (Some(a), Some(b)) => {
                let d = newcombe(a, b);
                format!(
                    "{:+.2} [{:+.2}, {:+.2}]",
                    a.rate() - b.rate(),
                    d.lo(),
                    d.hi()
                )
            }
            _ => "—".to_owned(),
        };
        let unscored = |a: ArmTally| {
            let (i, c, f) = a.unscored();
            format!("{i}, {c}, {f}")
        };
        let (rate, loaded, not_loaded) = load_cells(t.load);
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            t.tag.as_str(),
            arm_cell(t.without.count()),
            arm_cell(t.with.count()),
            diff,
            unscored(t.without),
            unscored(t.with),
            rate,
            loaded,
            not_loaded,
            t.recommendation().as_str()
        ));
    }
    out.push_str(&format!("\nReported cost, summed: ${cost}\n"));
    out
}
