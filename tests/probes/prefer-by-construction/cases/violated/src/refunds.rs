use crate::store::Ledger;

pub fn record_refund(ledger: &mut Ledger, amount_cents: u64) {
    ledger.append_entry(&format!("refund {amount_cents}"));
    ledger.bump_seq();
}
