# ledger

An append-only ledger. Readers trust that the sequence number equals the number of
entries; a ledger where they differ is torn, and readers refuse it.

Every writer appends an entry and then bumps the sequence, as `src/bin/import.rs` does.
