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

use super::stats::{Count, Recommendation, newcombe, recommend, wilson};
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
        rows.push(Row {
            tag,
            arm,
            outcome,
            cost,
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

    #[must_use]
    pub fn recommendation(&self) -> Recommendation {
        recommend(self.without.count(), self.with.count())
    }
}

/// Every rule's tally, in the order the run first recorded each, and the
/// summed reported cost.
#[must_use]
pub fn tally(rows: &[Row]) -> (Vec<RuleTally>, MicroUsd) {
    let mut out: Vec<RuleTally> = Vec::new();
    let mut cost = 0u64;
    for row in rows {
        cost = cost.saturating_add(row.cost.map_or(0, MicroUsd::micros));
        let index = match out.iter().position(|t| t.tag == row.tag) {
            Some(i) => i,
            None => {
                out.push(RuleTally {
                    tag: row.tag.clone(), // allow:clone: the tally owns its key, outliving the borrowed rows
                    without: ArmTally::default(),
                    with: ArmTally::default(),
                });
                out.len() - 1
            }
        };
        let entry = &mut out[index];
        match row.arm {
            Arm::With => entry.with.add(row.outcome),
            Arm::Without => entry.without.add(row.outcome),
        }
    }
    (out, MicroUsd::from_micros(cost))
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

/// The pilot table, as markdown.
#[must_use]
pub fn render(tallies: &[RuleTally], cost: MicroUsd) -> String {
    let mut out = String::from(
        "| Rule | WITHOUT violated [95% Wilson] | WITH violated [95% Wilson] | \
         Reduction [95% Newcombe] | Unscored w/o (inc, cont, err) | \
         Unscored with (inc, cont, err) | Recommendation |\n\
         |---|---|---|---|---|---|---|\n",
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
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} |\n",
            t.tag.as_str(),
            arm_cell(t.without.count()),
            arm_cell(t.with.count()),
            diff,
            unscored(t.without),
            unscored(t.with),
            t.recommendation().as_str()
        ));
    }
    out.push_str(&format!("\nReported cost, summed: ${cost}\n"));
    out
}
