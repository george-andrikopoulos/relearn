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
}

impl Recommendation {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Recommendation::Keep => "keep",
            Recommendation::GraduateNow => "graduate-now",
            Recommendation::Underpowered => "underpowered",
            Recommendation::NonDiscriminating => "non-discriminating",
        }
    }
}

/// Decide from the two arms. `None` is an arm with no scored sample.
#[must_use]
pub fn recommend(without: Option<Count>, with: Option<Count>) -> Recommendation {
    let (Some(without), Some(with)) = (without, with) else {
        return Recommendation::NonDiscriminating;
    };
    if without.violations == 0 {
        return Recommendation::NonDiscriminating;
    }
    if newcombe(without, with).lo > 0.0 {
        return Recommendation::Keep;
    }
    if with.violations > 0 {
        Recommendation::GraduateNow
    } else {
        Recommendation::Underpowered
    }
}
