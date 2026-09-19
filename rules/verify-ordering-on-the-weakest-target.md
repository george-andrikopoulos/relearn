+++
tag = "R:verify-ordering-on-the-weakest-target"
title = "Memory ordering is verified on the weakest architecture it ships to"
error_class = "An atomic ordering chosen or omitted, then exercised only on a strongly-ordered processor where the wrong ordering and the right one are indistinguishable at runtime, so the test suite certifies a program that reorders on the machine it was actually written for"
home = { kind = "domain", name = "low-latency" }
applies_to = ["rust", "java"]
created = "2026-09-18"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-18 from the same review, and it is the rule that generalises furthest beyond the queue that prompted it. x86 gives release-acquire semantics to ordinary loads and stores at no cost, so on a developer workstation an incorrect ordering produces the correct answer every time it is run -- the defect is not rare there, it is absent there, and appears in full on the first weakly-ordered deployment target."
+++

State the weakest memory model the binary will run under, and verify the ordering there --
not on the machine the code was written on.

x86 is total-store-order: loads are not reordered with loads, stores are not reordered
with stores, and an ordinary access already carries most of what release-acquire asks for.
So a missing acquire on the consumer side of a queue, or a relaxed store where a release
was needed, is not an intermittent bug on that hardware -- the processor will not reorder
what the code got wrong. Every run passes, the stress test passes, the loop runs overnight
and passes. On aarch64 -- a laptop, a cloud instance, an embedded target -- the same source
reorders and the queue hands out a node whose contents are not yet visible.

**There are two reorderers, and only one of them is the processor.** A relaxed access
where an acquire was needed licenses the *compiler* to move other accesses across it, and
the compiler is the same compiler on every target. So x86 is not a safe platform for a
wrong ordering; it is a platform on which one of the two mechanisms cannot expose it. The
hardware half of the defect is absent there and the software half is not, which is why the
failure on x86 is rare and confusing rather than impossible -- and why "we have never seen
it in production" is evidence about the optimiser's mood on a particular build, not about
the ordering. The correct ordering is what makes both reorderers behave; the architecture
only decides which one catches you.

This is the exact shape `[R:verify-through-production-path]` warns about, with the
production path being a processor rather than a wiring channel: the test exercised a
stand-in whose hardware guarantees are strictly stronger than the real one, so a pass
carries little information about the case that matters. And the usual defence against an
untestable property -- run it more -- buys almost nothing here, because repetition on TSO
samples the same hardware guarantee every time and varies only the compiler's choices,
which are fixed for a given build.

So the ordering is held by an argument and a tool, never by a green test on one machine:

* Name the pairing at the point of use. Every release has a named acquire that reads it
  and the comment says which -- an ordering with no partner is either a mistake or a
  relaxed access that has not admitted it.
* Run the model checker or the race detector rather than the program. `loom` in Rust,
  `jcstress` on the JVM, TSAN under the C++ model: these explore the orderings the
  hardware is permitted to produce rather than the one it happened to produce.
* Where the target is known, build and run the test suite on it. An aarch64 CI runner is
  ordinary infrastructure now, and it is the only cheap thing on this list that tests the
  actual machine.

Failure-mode check, before accepting any atomic that is not sequentially consistent:
**which architecture would show me this was wrong, and have I run it there?** If the
answer is that it passes locally, the ordering is unverified -- `[R:guarantee-needs-a-reader]`,
with the unread state being the hardware's.
