//! Fixture: every declaration below MUST be caught by `no-float-money.sh`.
//!
//! This file is never compiled. It is input to the detector, and
//! `caught.expected` is the exact output the detector must produce for it —
//! so a matcher that silently stops seeing one of these shapes fails the
//! self-test instead of reporting a clean tree.

/// The tuple newtype the rule names by name. `[R:newtype-liberally]` is
/// satisfied here and the defect is still present.
pub struct Price(f64);

pub struct Balance(pub f32);

/// The alias, which hides the representation from every use site.
pub type Notional = f64;

pub struct Order {
    price: f64,
    pub filled_quantity: f32,
    commission: Option<f64>,
    fees: Vec<f64>,
    unrealised_pnl: f64,
}

/// A parameter is a holder too, and this is how a bad representation crosses
/// a module boundary after the field has been fixed.
pub fn charge(amount: f64) -> f64 {
    let total_cost: f64 = amount * 1.2;
    total_cost
}

/// Camel case reads the same as snake case.
pub struct Ledger {
    pub NotionalAmount: f64,
}
