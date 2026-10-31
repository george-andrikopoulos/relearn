//! Violation rates with their uncertainty, and the recommendation they support.
//!
//! **Wilson's score interval** for one arm, because a pilot's arms are small
//! and often at zero: the normal approximation gives a zero-width interval at
//! 0/10, which would read as certainty about ten samples. **Newcombe's hybrid
//! score interval** (1998, method 10) for the difference, built from the two
//! Wilson intervals, for the same reason. Both at 95%.

/// The two-sided 95% normal quantile.
const Z: f64 = 1.959_963_984_540_054;

/// Violations out of scored samples. Inconclusive samples are not scored:
/// they are reported beside the count, never folded into either side of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Count {
    violations: u32,
    scored: u32,
}

impl Count {
    /// `None` when there were no scored samples, or more violations than
    /// samples. A rate over nothing is not a rate of zero.
    #[must_use]
    pub fn new(violations: u32, scored: u32) -> Option<Self> {
        (scored > 0 && violations <= scored).then_some(Count { violations, scored })
    }

    #[must_use]
    pub fn violations(self) -> u32 {
        self.violations
    }

    #[must_use]
    pub fn scored(self) -> u32 {
        self.scored
    }

    /// The observed violation rate.
    #[must_use]
    pub fn rate(self) -> f64 {
        f64::from(self.violations) / f64::from(self.scored)
    }
}

/// A closed interval.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    lo: f64,
    hi: f64,
}

impl Interval {
    #[must_use]
    pub fn lo(self) -> f64 {
        self.lo
    }

    #[must_use]
    pub fn hi(self) -> f64 {
        self.hi
    }
}

/// Wilson's 95% score interval for a violation rate.
#[must_use]
pub fn wilson(count: Count) -> Interval {
    let n = f64::from(count.scored);
    let p = count.rate();
    let z2 = Z * Z;
    let denom = 1.0 + z2 / n;
    let centre = (p + z2 / (2.0 * n)) / denom;
    let half = Z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / denom;
    // Clamped against rounding only: the exact interval lies inside [0, 1] and
    // contains `p`; the float form can miss either by an ulp.
    Interval {
        lo: (centre - half).clamp(0.0, p),
        hi: (centre + half).clamp(p, 1.0),
    }
}

/// Newcombe's 95% interval for `without.rate() - with.rate()`: how much the
/// layer lowers the violation rate. Positive throughout means it measurably
/// helps.
#[must_use]
pub fn newcombe(without: Count, with: Count) -> Interval {
    let (p1, p2) = (without.rate(), with.rate());
    let (a, b) = (wilson(without), wilson(with));
    let d = p1 - p2;
    let lo = d - ((p1 - a.lo).powi(2) + (b.hi - p2).powi(2)).sqrt();
    let hi = d + ((a.hi - p1).powi(2) + (p2 - b.lo).powi(2)).sqrt();
    Interval {
        lo: lo.clamp(-1.0, d),
        hi: hi.clamp(d, 1.0),
    }
}

/// Below this upper Wilson bound, a load rate is *measurably* low: most WITH
/// sessions did not open the skill, and the sample says so with 95%
/// confidence rather than by a point estimate on ten sessions.
pub const LOW_LOAD_CEILING: f64 = 0.5;

/// The share of a WITH arm's judged sessions that opened the rule's own skill.
///
/// A separate type from [`Count`] because the two count different things —
/// a load is not a violation — and a function taking one must not be handed
/// the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadRate(Count);

impl LoadRate {
    /// `None` when there was no session, or more loads than sessions.
    #[must_use]
    pub fn new(loaded: u32, sessions: u32) -> Option<Self> {
        Count::new(loaded, sessions).map(LoadRate)
    }

    #[must_use]
    pub fn loaded(self) -> u32 {
        self.0.violations
    }

    #[must_use]
    pub fn sessions(self) -> u32 {
        self.0.scored
    }

    #[must_use]
    pub fn rate(self) -> f64 {
        self.0.rate()
    }

    /// Wilson's 95% interval for the load rate.
    #[must_use]
    pub fn interval(self) -> Interval {
        wilson(self.0)
    }

    /// Measurably low: the whole interval lies below [`LOW_LOAD_CEILING`].
    #[must_use]
    pub fn is_low(self) -> bool {
        self.interval().hi < LOW_LOAD_CEILING
    }
}

/// What the WITH arm says about delivery, for a rule delivered as a skill:
/// how often the skill was opened, and the violations among the sessions
/// that opened it (`None` when none of them was scored).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadEvidence {
    rate: LoadRate,
    when_loaded: Option<Count>,
}

impl LoadEvidence {
    #[must_use]
    pub fn new(rate: LoadRate, when_loaded: Option<Count>) -> Self {
        LoadEvidence { rate, when_loaded }
    }

    #[must_use]
    pub fn rate(self) -> LoadRate {
        self.rate
    }

    #[must_use]
    pub fn when_loaded(self) -> Option<Count> {
        self.when_loaded
    }

    /// The skill was measurably left unopened, and the prose did not fail in
    /// the sessions it reached: none of them violated, or they did measurably
    /// better than WITHOUT, or none was delivered at all — in which case the
    /// prose has not been measured, and a verdict on it would rest on no
    /// session that read it.
    fn delivery_is_the_failure(self, without: Count) -> bool {
        self.rate.is_low()
            && match self.when_loaded {
                None => true,
                Some(loaded) => loaded.violations == 0 || newcombe(without, loaded).lo > 0.0,
            }
    }
}

/// What a pilot's numbers support, for one rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recommendation {
    /// The layer measurably lowers the violation rate: prose holds it.
    Keep,
    /// The WITH arm still violates and the layer is not measurably better:
    /// prose does not hold it, so promote it to the control the triage names.
    GraduateNow,
    /// The WITH arm never violated, but the difference cannot be told from
    /// zero at this sample size. Neither a finding for the prose nor against.
    Underpowered,
    /// The WITHOUT arm never violated, or an arm has no scored sample: the
    /// probe cannot measure the rule, so rewrite it. Never "the rule holds"
    /// (`[R:evidence-must-discriminate]`).
    NonDiscriminating,
    /// The rule's skill was measurably left unopened, and where it was opened
    /// the prose held (or it was never opened at all). The remedy is placement
    /// or the skill's description — not a stronger control, and not firmer
    /// prose, because the prose is not what failed.
    LoadFailure,
}

impl Recommendation {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Recommendation::Keep => "keep",
            Recommendation::GraduateNow => "graduate-now",
            Recommendation::Underpowered => "underpowered",
            Recommendation::NonDiscriminating => "non-discriminating",
            Recommendation::LoadFailure => "load-failure",
        }
    }
}

/// Decide from the two arms, and from delivery where the rule is a skill.
/// `None` is an arm with no scored sample, or a rule whose layer loads with
/// no `Skill` call (a `.claude/rules/` file), where there is no delivery to
/// measure.
///
/// Order: an arm that cannot discriminate decides first, then a measurable
/// reduction (the prose holds as delivered, however it was delivered), then
/// delivery, and only then the prose.
#[must_use]
pub fn recommend(
    without: Option<Count>,
    with: Option<Count>,
    load: Option<LoadEvidence>,
) -> Recommendation {
    let (Some(without), Some(with)) = (without, with) else {
        return Recommendation::NonDiscriminating;
    };
    if without.violations == 0 {
        return Recommendation::NonDiscriminating;
    }
    if newcombe(without, with).lo > 0.0 {
        return Recommendation::Keep;
    }
    if load.is_some_and(|l| l.delivery_is_the_failure(without)) {
        return Recommendation::LoadFailure;
    }
    if with.violations > 0 {
        Recommendation::GraduateNow
    } else {
        Recommendation::Underpowered
    }
}
