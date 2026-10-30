/// The ledger. Invariant: `seq() == len()` between writes.
pub struct Ledger {
    entries: Vec<String>,
    seq: u64,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger { entries: Vec::new(), seq: 0 }
    }

    /// Append an entry and advance the sequence: the only way to write.
    pub fn record(&mut self, entry: &str) {
        self.entries.push(entry.to_owned());
        self.seq += 1;
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}
