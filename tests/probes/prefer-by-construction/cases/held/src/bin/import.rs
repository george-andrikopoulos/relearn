use ledger::store::Ledger;

fn main() {
    let mut ledger = Ledger::new();
    for line in ["opening balance", "deposit 100"] {
        ledger.record(line);
    }
    println!("imported {} entries, seq {}", ledger.len(), ledger.seq());
}
