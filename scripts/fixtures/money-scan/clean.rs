//! Fixture: nothing below may be caught by `no-float-money.sh`.
//!
//! Every line here is either a measured quantity that is legitimately a float,
//! a monetary quantity in a correct representation, or a deliberate near miss
//! that a substring matcher would flag and a concept matcher must not. A
//! detector that goes red on this file is over-eager, gets muted, and then
//! holds nothing — which is the failure this fixture exists to prevent.

use rust_decimal::Decimal;

/// Measured, not counted. Latency is a real number and belongs in a float.
pub struct Timing {
    elapsed_secs: f64,
    jitter_us: f32,
}

pub struct Sensor(f64);

pub type Ratio = f64;

/// The correct representations. The scan must stay quiet on the fix, or it
/// punishes the change it exists to cause.
pub struct Position {
    price: Decimal,
    amount_minor: i64,
    unrealised_pnl: Decimal,
    fees: Vec<Decimal>,
}

/// A genuinely measured quantity that happens to carry a monetary word. The
/// escape is read before comments are stripped, which is why it can live here.
pub struct Book {
    spread_ratio: f64, // money-scan: measured
}

/// Near misses. A substring matcher flags all three; a concept matcher must
/// not, because none of these words IS a monetary word.
pub struct NearMiss {
    costume_weight: f64,
    valuation_model_id: u32,
    totality: f64,
}

/// A path, not a declaration.
pub const CEILING: f64 = std::f64::MAX;

// price: f64 — named only in a comment, and comments are stripped.
