//! `probe` — scoring a probe run: one session's transcript into events, and a
//! pilot's verdicts into rates, intervals and a recommendation.
//!
//! Pure, like every module but `fsio`: a transcript arrives as text the caller
//! read, and nothing here starts a model or a process. The runner that does
//! (`scripts/probe-run.sh`) is outside the binary, so `tests/solo_mode.rs`
//! still holds (ARCHITECTURE decisions log, 2026-10-30).
//!
//! **Must NOT:** perform I/O, or be reachable from `build` — a build that
//! behaved differently for having seen a probe result would make the
//! instruction layer depend on what a results directory happened to contain.

mod results;
mod stats;
mod transcript;

pub use results::{
    Arm, ArmTally, Outcome, ResultsError, Row, RuleTally, parse_results, render, tally,
};
pub use stats::{Count, Interval, Recommendation, newcombe, recommend, wilson};
pub use transcript::{MicroUsd, Session, TranscriptError, session_from_stream_json};
