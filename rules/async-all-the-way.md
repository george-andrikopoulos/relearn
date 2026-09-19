+++
tag = "R:async-all-the-way"
title = "Async is async all the way down"
error_class = "A blocking call inside an async context -- block_on, a synchronous file or socket read, a std Mutex guard held across an await -- which parks a runtime worker thread and stalls every unrelated task scheduled on it"
home = { kind = "domain", name = "rust" }
created = "2026-08-24"
origin = "codified"
status = { kind = "partial", by = "hook:no-block-on-in-async", uncovered = "every other way a runtime thread is blocked inside async -- synchronous file and network I/O, a std::sync lock held across an await, a long CPU section never handed to spawn_blocking -- and any Rust file outside a `src/` tree", date = "2026-09-19" }
incident = "Codification-dated, not single-incident: ported 2026-08-24 from the General Coding Rules of ~/.claude/CLAUDE.md, which state it in prose. Surfaced when the emitted Copilot artefact was audited against that pattern list and ten practices were found to have no rule in this library -- so they reached Claude through the always-on boot index and reached no other assistant at all. A practice that lives in one vendor's instruction layer only is the vendor lock this tool exists to remove."
published_incident = "Codified from standing practice rather than mined from a failure. It surfaced in an audit of an emitted instruction artefact against the hand-written pattern list it was supposed to replace: ten practices were stated in that one hand-maintained layer and had no rule in the library, so they reached a single assistant and travelled to no other tool — which is the failure the library exists to prevent, arriving in the library's own contents. The practice: a blocking call inside asynchronous code does not delay one task, it removes a worker from the pool and delays every task that would have run there. The victims are unrelated to the code that blocked, so the symptom is unexplained tail latency somewhere else entirely and is close to unattributable afterwards."
+++

No `block_on` inside async code. An async runtime multiplexes many tasks onto few threads, so a blocking call does not delay one task -- it removes a worker from the pool for the duration and delays every task that would have run there. The victims are unrelated to the code that blocked, which is why the symptom is unexplained tail latency somewhere else entirely, and why it is close to unattributable after the fact.

`block_on` is correct only at the boundary where synchronous code enters async: `main`, a test, a callback from a C library. Once inside, stay inside -- async I/O, an async-aware lock wherever a guard must survive an `await`, and `spawn_blocking` for work that genuinely blocks, such as CPU-bound compute or a synchronous third-party client.

The same applies to a `std::sync::Mutex` guard held across an `await`. Nothing flags it as blocking, but it holds a lock while the task is descheduled, so the contending task blocks its own worker thread and the failure presents as a deadlock rather than as a lock.

Nested `block_on` on a current-thread runtime does not degrade -- it deadlocks outright. That is the honest failure; the multi-threaded case merely hides the same mistake behind a thread count, until load removes the hiding place.
