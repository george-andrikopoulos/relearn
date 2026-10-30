use crate::store::Ledger;

pub fn record_refund(ledger: &mut Ledger, amount_cents: u64) {
    ledger.record(&format!("refund {amount_cents}"));
}
