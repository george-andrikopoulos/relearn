+++
tag = "R:no-false-sharing-on-a-hot-line"
title = "Two hot values on one cache line contend without sharing anything"
error_class = "Independent values written by different cores placed within one cache line, so the coherence protocol serialises writes the program logic never made dependent -- contention with no shared datum, no lock to find, and nothing in the source that looks wrong"
home = { kind = "domain", name = "low-latency" }
applies_to = ["rust", "java"]
created = "2026-09-18"
origin = "codified"
source = "Ulrich Drepper, \"What Every Programmer Should Know About Memory\" (2007), §3.3 on cache coherency and §6.4 on multi-threaded optimisation"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-18 from the same review. It belongs beside the queue rules rather than in general performance advice because the canonical instance is a queue: a producer writing the head and a consumer writing the tail, declared as adjacent fields, so the structure designed to let two threads avoid each other puts their two hottest writes on one cache line."
+++

Know which cache line each hot field lands on, and who writes each line. Sharing a line is sharing,
whatever the program thinks it is doing.

Coherence is maintained per line, not per variable -- 64 bytes on x86-64 and most ARM, 128 on some
Apple parts. Two independent counters declared next to each other occupy one line, so a write by
core A invalidates core B's copy and B's next write must fetch it back. The threads never touch a
common value and never take a lock; they simply pass one line back and forth at the cost of a
cross-core transfer each time. Nothing in the source says "shared", which is why this survives
review: the code is correct, the design is correct, and the layout is the defect.

The instance worth memorising is the one in the structure this domain keeps reaching for. A ring
buffer or queue has a head written only by producers and a tail written only by consumers. Declare
them as two adjacent fields and every enqueue invalidates the line the consumer is reading its
tail from. The queue was chosen so that two threads would not contend, and the field order gave
the contention back.

The other reliable sites: an array of per-thread or per-core counters indexed by id, a lock placed
immediately before the data it protects, a flag beside the buffer it guards, and any struct where
a hot mutable field sits next to a hot read-only one -- the readers are invalidated by every write
even though they never write.

Separate what different cores write. `#[repr(align(64))]` on the field's wrapper type in Rust,
padding either side in C, `@Contended` on the JVM -- which does nothing unless
`-XX:-RestrictContended` is set, a detail that turns the fix into a comment if it is missed. Then
**verify the separation survived**: alignment attributes can be dropped by an allocator that
returns a less-aligned block, by a `Vec` of padded elements whose base is not aligned, or by a
field the compiler reordered. `[R:verify-the-abstraction-compiled-away]` applies exactly -- the
padding is a claim about the emitted layout, so read the layout rather than the source.

Pad what is measurably contended and nothing else. Every padded field is cache the rest of the
program does not get, and a struct padded everywhere on principle trades a contention problem for
a capacity problem, which is the same mistake with better intentions.

Failure-mode check: **which line does each of these hot fields land on, and which cores write
that line?** If the answer needs a diagram of the struct, draw it -- that diagram is the design.
