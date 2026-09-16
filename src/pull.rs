//! `pull --all` — what a bulk pull would do, worked out before it does any of
//! it.
//!
//! **A plan is a report before it is an action.** Printed with no `--confirm`,
//! it is the status check to run before starting work: what this install would
//! take, what it would refresh, what it would drop, and — the part that matters
//! most — **everything it would leave alone, named, with the reason**. An
//! omission nobody is told about is the failure a plan exists to prevent, and a
//! bulk operation is exactly where one hides.
//!
//! Three things this module holds, each of which could have been a convenience
//! and is not.
//!
//! * **Asking for all of them is still asking.** §10 forbids automatic sync in
//!   either direction; `--all` is a person naming a whole corpus instead of one
//!   tag, and it still prints first and writes only on confirmation. Nothing
//!   here runs on a schedule, and nothing decides on its own that a rule has
//!   become relevant.
//! * **An audience bounds it.** §10's answer to a corpus too large to compile is
//!   scope, not retrieval — so `--scope` narrows what a bulk pull takes, through
//!   the same `Rule::serves` every other narrowing uses. An unscoped upstream
//!   rule is taken regardless, which is A1's safety default: narrowing can never
//!   withhold a rule that declared no audience.
//! * **Dropping is opt-in and reaches only caches.** Removal is the one
//!   operation here that deletes a file. It is safe for exactly one reason — a
//!   cache is regenerable — and `DroppableCache` is what keeps it from ever
//!   reaching source. `Prune::Keep` is the default, and an unwanted cache is
//!   still *reported* under it, so the status check tells you what a prune would
//!   take without taking it.
//!
//! Pure: two libraries in, a plan out. The caller reads the drive and, if asked,
//! acts on what this decided.

use std::collections::BTreeMap;

use crate::library::{Library, Validated};
use crate::rule::{
    Date, DroppableCache, NotDroppable, NotPullable, PulledRule, Rule, RuleTag, ScopeTag, SourceId,
    Version,
};

/// Whether a bulk pull may remove caches it finds unwanted.
///
/// An enum rather than a `bool` because the two values are decisions with
/// names, and because a call site reads `Prune::Drop` rather than `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prune {
    /// Leave every cache in place; report the unwanted ones and do nothing.
    Keep,
    /// Remove the caches that are gone or retired upstream.
    Drop,
}

/// Why a rule was left alone.
///
/// **Every rule the plan does not act on appears here**, so the report accounts
/// for the whole of both corpora. A bulk operation listing only what it touched
/// would leave the reader to diff two libraries by hand, which is the thing
/// they ran the command to avoid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skipped {
    /// The cache already holds upstream's revision.
    AlreadyCurrent,
    /// This install owns the rule; upstream's copy is not authoritative here.
    YoursToKeep,
    /// A fork taken deliberately, which a pull would discard.
    ADeliberateFork,
    /// Upstream declares no revision, so no cache of it could be told it is
    /// stale.
    NoUpstreamVersion,
    /// A home that never leaves a machine never arrives on one.
    HomeIsWithheld,
    /// The rule serves audiences this install did not ask for.
    NotInYourAudience,
    /// A cache that is gone or retired upstream, under [`Prune::Keep`].
    UnwantedButKept,
}

impl Skipped {
    /// What to tell the reader.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Skipped::AlreadyCurrent => "already at upstream's revision",
            Skipped::YoursToKeep => "yours, not a cache",
            Skipped::ADeliberateFork => "a fork you took deliberately",
            Skipped::NoUpstreamVersion => "upstream declares no revision",
            Skipped::HomeIsWithheld => "a home that never leaves a machine",
            Skipped::NotInYourAudience => "serves an audience you did not ask for",
            Skipped::UnwantedButKept => "unwanted, and kept — pass --prune to drop it",
        }
    }
}

/// Everything a bulk pull would do, and everything it would not.
#[derive(Debug)]
pub struct Plan {
    take: Vec<PulledRule>,
    refresh: Vec<PulledRule>,
    drop: Vec<DroppableCache>,
    skipped: Vec<(RuleTag, Skipped)>,
}

impl Plan {
    /// Work out what a bulk pull would do.
    ///
    /// `audience` narrows by scope and behaves exactly as it does for `build`:
    /// empty means no filter, and an upstream rule declaring no `applies_to` is
    /// taken whatever is asked for.
    #[must_use]
    pub fn of(
        local: &Library<Validated>,
        upstream: &Library<Validated>,
        from: SourceId,
        on: Date,
        audience: &[ScopeTag],
        prune: Prune,
    ) -> Self {
        let held: BTreeMap<&str, &Rule> = local
            .rules()
            .iter()
            .map(|rule| (rule.tag().as_str(), rule))
            .collect();
        let published: BTreeMap<&str, &Rule> = upstream
            .rules()
            .iter()
            .map(|rule| (rule.tag().as_str(), rule))
            .collect();

        let mut plan = Plan {
            take: Vec::new(),
            refresh: Vec::new(),
            drop: Vec::new(),
            skipped: Vec::new(),
        };

        for rule in upstream.rules() {
            let held = held.get(rule.tag().as_str()).copied();
            plan.consider_upstream(rule, held, &from, on, audience);
        }
        for rule in local.rules() {
            let published = published.get(rule.tag().as_str()).copied();
            plan.consider_held(rule, published, prune);
        }
        plan
    }

    /// One upstream rule: take it, refresh it, or say why not.
    fn consider_upstream(
        &mut self,
        rule: &Rule,
        held: Option<&Rule>,
        from: &SourceId,
        on: Date,
        audience: &[ScopeTag],
    ) {
        // Audience first, because it is the narrowing the *operator* chose:
        // being told "not in your audience" is more useful than being told
        // something else about a rule they deliberately narrowed away.
        if !rule.serves(audience) {
            self.skip(rule, Skipped::NotInYourAudience);
            return;
        }
        // A retired upstream rule is never taken or refreshed: caching an
        // instruction its own author has withdrawn is the one thing an attic
        // unambiguously means. **Whether it is already held changes nothing
        // here** — that case is decided entirely on the held side, as "drop it"
        // or "unwanted and kept", and reporting it from both sides said the
        // same rule was current *and* unwanted in one run.
        if rule.status().is_withdrawn() {
            return;
        }
        let source = from.clone(); // allow:clone: each pulled rule owns the source it names, and the plan may mint many from one borrowed id
        match PulledRule::of(rule, held, source, on) {
            Ok(pulled) => match held {
                None => self.take.push(pulled),
                Some(held) if held.authority().is_behind(version_of(rule)) => {
                    self.refresh.push(pulled);
                }
                Some(_) => self.skip(rule, Skipped::AlreadyCurrent),
            },
            Err(why) => self.skip(rule, skipped_for(why)),
        }
    }

    /// One rule this install holds: drop it, or say why it stays.
    fn consider_held(&mut self, rule: &Rule, published: Option<&Rule>, prune: Prune) {
        match DroppableCache::of(rule, published) {
            Ok(unwanted) => match prune {
                Prune::Drop => self.drop.push(unwanted),
                Prune::Keep => self.skip(rule, Skipped::UnwantedButKept),
            },
            // A rule upstream still carries was already accounted for from the
            // upstream side; saying it twice would make the report longer
            // without making it more complete.
            Err(_) if published.is_some() => {}
            Err(NotDroppable::YoursToKeep) => self.skip(rule, Skipped::YoursToKeep),
            Err(NotDroppable::ADeliberateFork) => self.skip(rule, Skipped::ADeliberateFork),
            Err(NotDroppable::StillWanted) => {}
        }
    }

    fn skip(&mut self, rule: &Rule, why: Skipped) {
        let tag = rule.tag().clone(); // allow:clone: the plan owns its report, which outlives the library borrow it was read from
        self.skipped.push((tag, why));
    }

    /// Rules to take that this install does not hold.
    #[must_use]
    pub fn take(&self) -> &[PulledRule] {
        &self.take
    }

    /// Caches to replace with a newer copy.
    #[must_use]
    pub fn refresh(&self) -> &[PulledRule] {
        &self.refresh
    }

    /// Caches to remove. Empty under [`Prune::Keep`].
    #[must_use]
    pub fn drop(&self) -> &[DroppableCache] {
        &self.drop
    }

    /// Everything left alone, with the reason.
    #[must_use]
    pub fn skipped(&self) -> &[(RuleTag, Skipped)] {
        &self.skipped
    }

    /// Whether the plan would change anything at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.take.is_empty() && self.refresh.is_empty() && self.drop.is_empty()
    }
}

/// The revision an upstream rule declares.
///
/// Only reached for a rule [`PulledRule::of`] has already accepted, and that
/// acceptance is what proves the revision is there.
fn version_of(rule: &Rule) -> Version {
    rule.authority()
        .version()
        .expect("the witness refuses an upstream rule with no revision, so this one has one")
}

/// The reason a refusal gives, as the plan words it.
fn skipped_for(why: NotPullable) -> Skipped {
    match why {
        NotPullable::NoUpstreamVersion => Skipped::NoUpstreamVersion,
        NotPullable::HomeIsWithheld => Skipped::HomeIsWithheld,
        NotPullable::WouldClobberLocal => Skipped::YoursToKeep,
        NotPullable::WouldClobberFork => Skipped::ADeliberateFork,
    }
}
