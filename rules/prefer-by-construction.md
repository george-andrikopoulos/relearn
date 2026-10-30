+++
tag = "R:prefer-by-construction"
title = "Prefer by-construction impossibility over after-the-fact controls"
error_class = "Reaching for a runtime control (a check, a guard, a review step) to catch a mistake after it occurs when the design could have made that mistake impossible to express in the first place"
home = { kind = "global" }
created = "2026-08-13"
origin = "mined"
status = { kind = "active" }
incident = "A standing principle in the operating model, tagged [R:prefer-by-construction] there. Its concrete realizations recur: R:make-illegal-states-unrepresentable is the type-level form, and the repo-in-place design (editing a file IS editing the live system, so there is no deploy step to forget) removes a class of staleness by construction rather than guarding against it. The weaker the layer a guarantee lives at, the more it must be remembered; by-construction is the layer that cannot be forgotten."
published_incident = "The recurring observation behind a family of rules rather than a single failure: the weaker the layer a guarantee lives at, the more it must be remembered. Making an illegal state unrepresentable is the type-level form; removing a deploy step so there is nothing to forget is the process form. A guarantee held by construction is the only one that survives the person who knew about it leaving, and every instance where one was held by prose instead has eventually been paid for."

[[recurrence]]
date = "2026-09-26"
incident = "fix-gateway. The FIX session core advanced its outbound counter in `sent()`; the reactor numbered outbound actions by their position in the action list. Correctness needed exactly one `sent()` before every outbound `push`, and twenty call sites had one. The twenty-first, in the acceptor's case-E10 branch, did not, so every acceptor reconnect following a heartbeat reused a sequence number and ended its own session. The comment above the coupling called it true by construction. It was a convention followed twenty times, and a property test that could not reach the twenty-first site was cited as its guarantee. Retired by one function, `send()`, that consumes the number and records the action together, with `push` refusing outbound actions -- the missing call became unwritable rather than caught."

[probe]
task = "Add a function `record_refund(ledger: &mut Ledger, amount_cents: u64)` in a new module src/refunds.rs that writes a refund entry to the ledger, export the module from src/lib.rs, and make sure the crate builds."
workspace = "fixture"

[probe.checker]
kind = "run"
adversary = "The next call site: a program outside the crate that performs one half of the append-and-bump pair alone, then reads whether the sequence still equals the entry count."
+++

When a class of mistake can be designed out, design it out, rather than adding a control that catches it after the fact. A control that catches a mistake still admits the mistake; a design that cannot express the mistake retires the whole class. Rank the options by how little must be remembered for them to hold: a type the compiler enforces beats a test that samples beats a review step that relies on attention. R:make-illegal-states-unrepresentable is this rule in the type system.

**A convention followed everywhere is not construction.** When correctness needs two actions paired at every call site -- advance a counter then emit, acquire then release, validate then store -- and a new call site could do one without the other and still compile, the pairing is held by memory, however consistently it has been followed so far. Fold the pair into one function that does both, and make the unpaired half unreachable from outside it. Test: *could the next call site I write omit half of this and still compile?* If it could, the words "by construction" do not apply, and a comment using them is a guarantee with no reader (R:guarantee-needs-a-reader).
