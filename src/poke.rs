//! `poke` — the federation's signal, surfaced inside `lint`.
//!
//! **One reactive trigger, four broadcast ones, and the difference is the
//! whole design.** The reactive poke arrives at the moment a developer has just
//! demonstrated they needed it — *you recorded a recurrence, and upstream
//! already has a rule for that class*. That is what makes a corpus a collective
//! memory rather than a mailing list. Everything else is broadcast, and
//! broadcast is how a notification channel teaches people to ignore it: Paper 3
//! §9 already documents the failure on exposed rows — a column is *"useful only
//! while it embarrasses someone"*, and twenty of them teach readers to skip it.
//! So broadcast pokes are **capped per run**, the cap is a number the operator
//! passes rather than a judgement buried here, and two of the four are off
//! until asked for.
//!
//! Three things this module must not do, each one named in the programme as
//! what goes wrong.
//!
//! * **It must not run during `build`.** `lint` reports; `build` emits. A build
//!   that talks is a build whose output depends on what a clone happened to
//!   contain, and `tests/poke.rs` reads the emitters' source to keep it that
//!   way.
//! * **It must not fail a run.** A poke is news, not a finding: it has no
//!   severity, `--deny` does not reach it, and the exit code is identical with
//!   and without it. A federated signal that can fail CI has made federation
//!   required, which invariant 3 forbids.
//! * **It must not require anything.** No clone, no poke — silently, with no
//!   warning, because the solo install is the product and a nag is a soft
//!   requirement.
//!
//! Everything here is a pure function of two libraries and an aggregate. The
//! clone is read by the caller; this module performs no I/O.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::aggregate::Aggregate;
use crate::library::{Library, Validated};
use crate::report::Bucket;
use crate::rule::{Authority, Date, Rule, RuleTag, ScopeTag, Status, Version};

/// Whether a poke is addressed to this install's own evidence, or to everyone.
///
/// The distinction the cap acts on, as a type rather than as a `bool` on each
/// variant: a reactive poke follows something that happened *here*, so its
/// volume is bounded by this install's own recurrences, and capping it would
/// withhold the only signal worth having. A broadcast poke follows something
/// that happened somewhere else and is bounded by the size of the corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reach {
    /// Follows evidence recorded on this machine.
    Reactive,
    /// Follows something upstream, addressed to everyone alike.
    Broadcast,
}

/// One of the five triggers: §6's four, plus §12.6's upstream retirement.
///
/// **Declaration order is the rank**, and the rank is what orders the output —
/// never an incidental sort of the rendered text (`[R:order-by-explicit-rank]`).
/// Reactive first because it is the one worth reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trigger {
    /// A rule fired here, and upstream already has one for that class.
    ClassCoveredUpstream,
    /// A rule this install caches has a newer revision upstream.
    CacheBehind,
    /// A rule this install caches has been retired upstream.
    CacheRetiredUpstream,
    /// A rule serving this install's audience was contributed, and this install
    /// does not hold it.
    ContributedInAudience,
    /// Many installs report a rule this one does not hold.
    HighRecurrenceUnheld,
}

/// A trigger spelling nothing answers to.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown poke trigger {got:?} (expected one of: {expected})")]
pub struct UnknownTrigger {
    /// What was asked for.
    got: String,
    /// The trigger names that exist, comma-separated.
    expected: String,
}

impl Trigger {
    /// Every trigger, in rank order.
    pub const ALL: [Trigger; 5] = [
        Trigger::ClassCoveredUpstream,
        Trigger::CacheBehind,
        Trigger::CacheRetiredUpstream,
        Trigger::ContributedInAudience,
        Trigger::HighRecurrenceUnheld,
    ];

    /// Whose evidence this trigger follows.
    #[must_use]
    pub fn reach(self) -> Reach {
        match self {
            Trigger::ClassCoveredUpstream => Reach::Reactive,
            Trigger::CacheBehind
            | Trigger::CacheRetiredUpstream
            | Trigger::ContributedInAudience
            | Trigger::HighRecurrenceUnheld => Reach::Broadcast,
        }
    }

    /// Whether this trigger is on when nobody has said which they want.
    ///
    /// **The design's table, in one exhaustive match and nowhere else**, so the
    /// defaults are a fact a reader can check rather than behaviour spread over
    /// call sites. Three are on: the reactive one, and the two about a copy
    /// this install already holds — the stale cache, which the design puts as
    /// "on, silent until `lint`", and the upstream retirement, which §12.6
    /// puts as a warning and which cannot fire unless you cache something. The
    /// two that are off fire about things nobody here has touched.
    #[must_use]
    pub fn on_by_default(self) -> bool {
        match self {
            Trigger::ClassCoveredUpstream
            | Trigger::CacheBehind
            | Trigger::CacheRetiredUpstream => true,
            Trigger::ContributedInAudience | Trigger::HighRecurrenceUnheld => false,
        }
    }

    /// The set that runs when nobody named one.
    #[must_use]
    pub fn defaults() -> Vec<Trigger> {
        Trigger::ALL
            .into_iter()
            .filter(|t| t.on_by_default())
            .collect()
    }

    /// The name this trigger is asked for by.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Trigger::ClassCoveredUpstream => "class-covered",
            Trigger::CacheBehind => "cache-behind",
            Trigger::CacheRetiredUpstream => "cache-retired",
            Trigger::ContributedInAudience => "contributed",
            Trigger::HighRecurrenceUnheld => "high-recurrence",
        }
    }

    /// Read a trigger name, or refuse and list what exists.
    pub fn parse(s: &str) -> Result<Self, UnknownTrigger> {
        Trigger::ALL
            .into_iter()
            .find(|t| t.as_str() == s.trim())
            .ok_or_else(|| UnknownTrigger {
                got: s.to_owned(),
                expected: Trigger::ALL
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            })
    }
}

/// How many broadcast pokes one run may print.
///
/// **A number the operator passes, never a judgement in the code.** Zero is a
/// real setting and not an error — it turns broadcast off entirely while
/// leaving the reactive trigger alone, which is the shape of the failure this
/// cap exists for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BroadcastCap(usize);

impl BroadcastCap {
    /// The cap when the operator has not said. Small on purpose: a poke block
    /// that has to be scrolled is one that gets skipped, and the reactive
    /// pokes it sits beside are the ones worth reading.
    pub const DEFAULT: BroadcastCap = BroadcastCap(3);

    /// A cap of `n` broadcast pokes per run.
    #[must_use]
    pub fn new(n: usize) -> Self {
        BroadcastCap(n)
    }

    /// The number.
    #[must_use]
    pub fn get(self) -> usize {
        self.0
    }
}

/// Something the corpus knows that this install might want to.
///
/// Deliberately **not** a [`lint::Finding`](crate::lint::Finding): a finding is
/// a fault in this library and carries a severity that `--deny` can act on. A
/// poke is news from elsewhere, and giving it a severity is the first step to a
/// build that fails because a stranger published something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Poke {
    /// A rule of this install's recurred, and upstream already carries a rule
    /// for the same error class. The reactive one.
    ///
    /// The class itself is **not** carried: a rule has exactly one, so the two
    /// tags name it exactly, and this corpus's classes are paragraphs. A poke
    /// nobody reads to the end is the failure the cap exists for, arriving one
    /// line at a time instead of one poke at a time.
    ClassCoveredUpstream {
        /// The local rule that fired.
        fired: RuleTag,
        /// How many recurrences it has recorded.
        times: usize,
        /// The upstream rule covering the same class.
        upstream: RuleTag,
    },
    /// A cached rule is behind the upstream revision.
    CacheBehind {
        /// The cached rule.
        tag: RuleTag,
        /// The revision this copy holds.
        held: Version,
        /// The revision upstream publishes.
        upstream: Version,
    },
    /// A rule this install holds a copy of has been retired upstream.
    ///
    /// **A warning, and structurally incapable of being anything more.**
    /// Deleting an instruction a team relies on because a stranger retired it
    /// is a correction lost with no reader — P1, and the exact failure this
    /// tool exists to prevent; the local install may hold evidence the upstream
    /// author does not. Only a **local** `Status` reaches
    /// `Status::emittability`, so nothing here can suppress emission, and
    /// `tests/poke.rs` asserts it over the real emitters rather than trusting
    /// the argument.
    CacheRetiredUpstream {
        /// The rule this install holds a copy of.
        tag: RuleTag,
        /// Upstream's own reason, which is what decides between the three human
        /// resolutions — and which is already public in the clone.
        reason: String,
        /// When upstream retired it.
        since: Date,
    },
    /// An upstream rule serves an audience this install declares, and this
    /// install does not hold it.
    ContributedInAudience {
        /// The upstream rule.
        tag: RuleTag,
        /// The audiences it declares, or empty when it serves everyone.
        serves: Vec<ScopeTag>,
    },
    /// Enough installs report recurrences of a rule this one does not hold.
    HighRecurrenceUnheld {
        /// The rule, as the aggregate names it.
        tag: String,
        /// How many distinct installs reported it.
        installs: usize,
    },
}

impl Poke {
    /// Which trigger produced it.
    #[must_use]
    pub fn trigger(&self) -> Trigger {
        match self {
            Poke::ClassCoveredUpstream { .. } => Trigger::ClassCoveredUpstream,
            Poke::CacheBehind { .. } => Trigger::CacheBehind,
            Poke::CacheRetiredUpstream { .. } => Trigger::CacheRetiredUpstream,
            Poke::ContributedInAudience { .. } => Trigger::ContributedInAudience,
            Poke::HighRecurrenceUnheld { .. } => Trigger::HighRecurrenceUnheld,
        }
    }

    /// Whose evidence it follows.
    #[must_use]
    pub fn reach(&self) -> Reach {
        self.trigger().reach()
    }

    /// The tag it sorts under, within its trigger.
    fn sort_key(&self) -> &str {
        match self {
            Poke::ClassCoveredUpstream { fired, .. } => fired.as_str(),
            Poke::CacheBehind { tag, .. }
            | Poke::CacheRetiredUpstream { tag, .. }
            | Poke::ContributedInAudience { tag, .. } => tag.as_str(),
            Poke::HighRecurrenceUnheld { tag, .. } => tag.as_str(),
        }
    }
}

impl fmt::Display for Poke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Poke::ClassCoveredUpstream {
                fired,
                times,
                upstream,
            } => write!(
                f,
                "{} has fired {times} time(s) here, and upstream already has {} for the same class — read theirs before writing more prose for yours",
                fired.as_str(),
                upstream.as_str()
            ),
            Poke::CacheBehind {
                tag,
                held,
                upstream,
            } => write!(
                f,
                "your cache of {} is at revision {held} and upstream publishes {upstream} — pull it, or `adopt` it if you have reasons to stay where you are",
                tag.as_str()
            ),
            // Three resolutions, named, because the poke is worth nothing
            // without them — and `adopt` is first among equals: a retirement
            // installs refuse is the population telling an author something no
            // single install can know.
            Poke::CacheRetiredUpstream { tag, reason, since } => write!(
                f,
                "upstream retired {} in {since} ({reason}) — your copy still emits, and will keep emitting: pull the retirement, `adopt` it if you hold evidence they do not, or drop it",
                tag.as_str()
            ),
            Poke::ContributedInAudience { tag, serves } => {
                let audience = if serves.is_empty() {
                    "every audience".to_owned()
                } else {
                    serves
                        .iter()
                        .map(ScopeTag::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                write!(
                    f,
                    "{} was contributed upstream and serves {audience} — you do not hold it",
                    tag.as_str()
                )
            }
            Poke::HighRecurrenceUnheld { tag, installs } => write!(
                f,
                "{installs} install(s) report recurrences of {tag}, which you do not hold — and that count measures diligence as much as frequency",
            ),
        }
    }
}

/// What a run has to say, and what it withheld saying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pokes {
    shown: Vec<Poke>,
    withheld: usize,
    cap: BroadcastCap,
}

impl Pokes {
    /// The pokes to print, reactive first, then broadcast in rank order.
    #[must_use]
    pub fn shown(&self) -> &[Poke] {
        &self.shown
    }

    /// How many broadcast pokes the cap held back.
    ///
    /// **Published as a number, like the aggregate's suppressed count**: a cap
    /// that silently swallows the rest reads as "that was all there was", and a
    /// reader cannot tell a quiet corpus from a throttled one.
    #[must_use]
    pub fn withheld(&self) -> usize {
        self.withheld
    }

    /// The cap that was in force.
    #[must_use]
    pub fn cap(&self) -> BroadcastCap {
        self.cap
    }

    /// Whether there is nothing to say.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shown.is_empty() && self.withheld == 0
    }
}

/// The bucket that makes a cross-install count worth a broadcast.
///
/// **A published boundary rather than a tuned threshold.** `Bucket` is the
/// vocabulary reports already speak, so "high recurrence" means *some install
/// put this rule in the top bucket* — a fact a reader can check against the
/// document — instead of an integer somebody chose and nobody can audit.
const HIGH: Bucket = Bucket::Many;

/// Everything the corpus has to say to this install.
///
/// Pure: the caller reads the clone, this decides what is worth saying. Every
/// trigger the caller did not enable produces nothing at all, and a trigger
/// with no data to read produces nothing either — an empty upstream corpus and
/// an empty aggregate are ordinary states, not failures.
#[must_use]
pub fn pokes(
    local: &Library<Validated>,
    upstream: &Library<Validated>,
    aggregate: &Aggregate,
    enabled: &[Trigger],
    cap: BroadcastCap,
) -> Pokes {
    let held: BTreeSet<&str> = local.rules().iter().map(|r| r.tag().as_str()).collect();

    let mut raised: Vec<Poke> = Vec::new();
    for trigger in Trigger::ALL {
        if !enabled.contains(&trigger) {
            continue;
        }
        match trigger {
            Trigger::ClassCoveredUpstream => {
                raised.extend(class_covered_upstream(local, upstream, &held));
            }
            Trigger::CacheBehind => raised.extend(cache_behind(local, upstream)),
            Trigger::CacheRetiredUpstream => {
                raised.extend(cache_retired_upstream(local, upstream));
            }
            Trigger::ContributedInAudience => {
                raised.extend(contributed_in_audience(local, upstream, &held));
            }
            Trigger::HighRecurrenceUnheld => {
                raised.extend(high_recurrence_unheld(aggregate, &held));
            }
        }
    }

    // Rank, then tag: an explicit order stating the intent, never an incidental
    // sort of rendered text (`[R:order-by-explicit-rank]`).
    raised.sort_by(|a, b| {
        a.trigger()
            .cmp(&b.trigger())
            .then_with(|| a.sort_key().cmp(b.sort_key()))
    });

    // The cap reaches broadcast only. A reactive poke follows evidence recorded
    // here, so withholding one would withhold the reader's own news.
    let mut shown = Vec::new();
    let mut broadcast = 0;
    let mut withheld = 0;
    for poke in raised {
        match poke.reach() {
            Reach::Reactive => shown.push(poke),
            Reach::Broadcast => {
                if broadcast < cap.get() {
                    broadcast += 1;
                    shown.push(poke);
                } else {
                    withheld += 1;
                }
            }
        }
    }

    Pokes {
        shown,
        withheld,
        cap,
    }
}

/// The audiences this install declares: every scope its own corpus names.
///
/// **The corpus is the only non-ambient answer.** There is no configuration
/// file to hold a scope list and there must not be one (invariant 3), so what
/// an install *is* is read from what its rules say they serve. An install whose
/// corpus declares no scope therefore has an empty audience, which
/// [`Rule::serves`] reads as "no filter" — so every upstream rule matches. That
/// is loud rather than wrong, and it is one of the reasons this trigger is off
/// until asked for and capped when it is not.
fn audience(local: &Library<Validated>) -> Vec<ScopeTag> {
    let scopes: BTreeSet<&ScopeTag> = local
        .rules()
        .iter()
        .flat_map(|rule| rule.applies_to().iter())
        .collect();
    scopes.into_iter().cloned().collect() // allow:clone: the caller owns an audience that outlives this borrow of the library, and a scope is a short kebab token
}

/// The reactive trigger: a rule fired here, and upstream covers that class.
///
/// Only rules that are **recurrence evidence** count — the same predicate the
/// local tally and the report flow read, so a mandate (never mined) cannot
/// produce a poke saying prose failed. An upstream rule whose tag this install
/// already holds is not news: holding it *is* the answer the poke would give.
fn class_covered_upstream<'a>(
    local: &'a Library<Validated>,
    upstream: &'a Library<Validated>,
    held: &BTreeSet<&str>,
) -> Vec<Poke> {
    let mut by_class: BTreeMap<String, Vec<&'a Rule>> = BTreeMap::new();
    for rule in upstream.rules() {
        if held.contains(rule.tag().as_str()) {
            continue;
        }
        by_class
            .entry(rule.error_class().match_key())
            .or_default()
            .push(rule);
    }

    let mut out = Vec::new();
    for rule in local.rules() {
        if !(rule.counts_toward_recurrence_statistics() && rule.has_recurred()) {
            continue;
        }
        let Some(covering) = by_class.get(&rule.error_class().match_key()) else {
            continue;
        };
        for other in covering {
            out.push(Poke::ClassCoveredUpstream {
                fired: rule.tag().clone(), // allow:clone: the poke owns its tags and outlives the library borrow it was read from
                times: rule.recurrences().len(),
                upstream: other.tag().clone(), // allow:clone: as above
            });
        }
    }
    out
}

/// The stale-cache trigger: a cached or adopted rule whose upstream revision
/// has moved past the one this copy holds.
///
/// An upstream rule with **no** revision recorded produces nothing: there is no
/// comparison point, and inventing one would tell every cache it is stale
/// forever. That is why a published rule states its revision — see
/// [`Authority::Local`].
fn cache_behind(local: &Library<Validated>, upstream: &Library<Validated>) -> Vec<Poke> {
    let published: BTreeMap<&str, Version> = upstream
        .rules()
        .iter()
        .filter_map(|rule| Some((rule.tag().as_str(), rule.authority().version()?)))
        .collect();

    local
        .rules()
        .iter()
        .filter_map(|rule| {
            let held = match rule.authority() {
                Authority::Local { .. } => return None,
                other => other.version()?,
            };
            let upstream = *published.get(rule.tag().as_str())?;
            rule.authority()
                .is_behind(upstream)
                .then(|| Poke::CacheBehind {
                    tag: rule.tag().clone(), // allow:clone: the poke owns its tag and outlives the library borrow
                    held,
                    upstream,
                })
        })
        .collect()
}

/// The retirement trigger: a rule this install holds a copy of, retired
/// upstream.
///
/// **Only a copy, and only an attic.** A rule this install *owns* has no
/// upstream to be retired by — an upstream rule sharing its tag is the same
/// rule, published, not a retirement of anything held here; the same predicate
/// as [`cache_behind`], so the two cannot disagree about what "holds a copy"
/// means. And a *graduated* upstream rule is not retired: it is still emitted,
/// annotated with the stronger control that also holds it. Only tag-level death
/// is an attic, which is why §12.6 needed no new `Status` variant.
///
/// What this trigger deliberately **cannot** do is act. It is news, and the
/// resolution is a person's: pull, adopt, or drop.
fn cache_retired_upstream(local: &Library<Validated>, upstream: &Library<Validated>) -> Vec<Poke> {
    let retired: BTreeMap<&str, (&str, Date)> = upstream
        .rules()
        .iter()
        .filter_map(|rule| match rule.status() {
            Status::Attic { reason, date } => Some((rule.tag().as_str(), (reason.as_str(), *date))),
            Status::Active | Status::Partial { .. } | Status::Graduated { .. } => None,
        })
        .collect();

    local
        .rules()
        .iter()
        .filter(|rule| !matches!(rule.authority(), Authority::Local { .. }))
        .filter_map(|rule| {
            let (reason, since) = retired.get(rule.tag().as_str())?;
            Some(Poke::CacheRetiredUpstream {
                tag: rule.tag().clone(), // allow:clone: the poke owns its tag and outlives the library borrow
                reason: (*reason).to_owned(),
                since: *since,
            })
        })
        .collect()
}

/// The contribution trigger: an upstream rule serving this install's audience
/// that this install does not hold.
///
/// **"New" means "absent here", and nothing else.** There is no record of what
/// this install has already been shown — keeping one would be per-machine state
/// — so the honest reading of *new to you* is *you do not have it*. A rule
/// dismissed today therefore returns tomorrow, which is the cost of holding no
/// state and the second reason this trigger is off by default.
fn contributed_in_audience(
    local: &Library<Validated>,
    upstream: &Library<Validated>,
    held: &BTreeSet<&str>,
) -> Vec<Poke> {
    let audience = audience(local);
    upstream
        .rules()
        .iter()
        .filter(|rule| !held.contains(rule.tag().as_str()))
        .filter(|rule| rule.serves(&audience))
        .map(|rule| Poke::ContributedInAudience {
            tag: rule.tag().clone(), // allow:clone: the poke owns its tag and outlives the library borrow
            serves: rule.applies_to().to_vec(), // allow:clone: same — the poke renders the audience after the borrow ends
        })
        .collect()
}

/// The population trigger: a rule enough installs report, that this one does
/// not hold.
///
/// Reads the aggregate's rows, which means it reads only what survived the
/// k-floor — so the floor protects reporters here exactly as it does in the
/// published document, with no second policy to keep in step.
fn high_recurrence_unheld(aggregate: &Aggregate, held: &BTreeSet<&str>) -> Vec<Poke> {
    aggregate
        .rows()
        .iter()
        .filter(|row| !held.contains(row.tag()))
        .filter(|row| row.in_bucket(HIGH.as_str()) > 0)
        .map(|row| Poke::HighRecurrenceUnheld {
            tag: row.tag().to_owned(),
            installs: row.installs(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The design's table, asserted rather than described. Three on, two off,
    /// the reactive one among those on — and the two that are on beside it are
    /// both about a copy this install already holds, so neither can fire
    /// against an install that caches nothing.
    #[test]
    fn the_default_set_is_the_designs_table() {
        assert_eq!(
            Trigger::defaults(),
            vec![
                Trigger::ClassCoveredUpstream,
                Trigger::CacheBehind,
                Trigger::CacheRetiredUpstream
            ]
        );
        assert!(Trigger::ClassCoveredUpstream.on_by_default());
        assert!(!Trigger::ContributedInAudience.on_by_default());
        assert!(!Trigger::HighRecurrenceUnheld.on_by_default());
    }

    #[test]
    fn exactly_one_trigger_is_reactive() {
        let reactive: Vec<Trigger> = Trigger::ALL
            .into_iter()
            .filter(|t| t.reach() == Reach::Reactive)
            .collect();
        assert_eq!(reactive, vec![Trigger::ClassCoveredUpstream]);
    }

    #[test]
    fn every_trigger_name_round_trips() {
        for trigger in Trigger::ALL {
            assert_eq!(Trigger::parse(trigger.as_str()), Ok(trigger));
        }
        assert!(Trigger::parse("reactive").is_err());
    }

    /// The error names what does exist, so a typo is one read away from fixed
    /// rather than one search.
    #[test]
    fn an_unknown_trigger_lists_the_ones_that_exist() {
        let error = Trigger::parse("cache_behind").expect_err("underscores are not the spelling");
        let text = error.to_string();
        for trigger in Trigger::ALL {
            assert!(text.contains(trigger.as_str()), "{text}");
        }
    }
}
