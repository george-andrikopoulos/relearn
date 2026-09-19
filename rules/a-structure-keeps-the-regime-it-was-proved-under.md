+++
tag = "R:a-structure-keeps-the-regime-it-was-proved-under"
title = "A structure is correct only under the concurrency regime its proof assumed"
error_class = "Using a data structure outside the producer/consumer regime its correctness argument was written for -- a second consumer on an MPSC queue, a second writer on an SPSC ring, a non-reentrant structure entered from a signal handler -- where it does not fail loudly but silently duplicates, loses or corrupts under a race that only interleaves under load"
home = { kind = "domain", name = "low-latency" }
applies_to = ["rust", "java"]
created = "2026-09-18"
origin = "codified"
source = "Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores -- where the regime is stated in the algorithm's name and nowhere in its code"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-18 from the same review. The prompt was noticing that the regime of the queue under discussion is carried entirely by four letters in its name: MPSC states the precondition, the implementation contains nothing that enforces it, and a second consumer compiles, links, runs and is wrong."
+++

Read the regime out of the structure's correctness argument, then name what in the code prevents
violating it. "The documentation says single-consumer" is not a mechanism.

MPSC means multi-producer, **single**-consumer, and the single is a precondition rather than a
description. Vyukov's `mpscq_pop` reads `self->tail`, walks from it, and writes it back -- a plain
load and a plain store, unsynchronised, because exactly one thread was assumed to be executing
them. Two consumers interleave there and both can return the same node, or one can lose a node
entirely. Neither outcome raises anything. There is no assertion, no lock to contend, no error
path: the queue returns plausible values and the program continues with a duplicated or vanished
item.

That silence is the whole problem, and it is compounded by load. Two consumers running a hundred
operations may never interleave in the window that matters; the test suite passes, the staging
soak passes, and the interleaving arrives with production traffic. A defect that only appears
under the conditions you cannot reproduce is indistinguishable, from the outside, from a defect
somewhere else entirely.

So put the regime in the type, where the language allows it. Hand out exactly one consumer handle
at construction; make it neither `Clone` nor `Sync`; take `&mut self` on `pop` so the borrow
checker enforces exclusivity. Then "two consumers" is not a bug to find but a program that does
not compile -- `[R:make-illegal-states-unrepresentable]`, and `[R:typestate-for-protocols]` where
the regime changes across a structure's lifetime. This is the highest-leverage instance of that
discipline in the whole domain, because the alternative control is a sentence in a header
comment and the failure it prevents is unattributable.

Where the language will not carry it -- Java has no such handle -- the control has to be
something that actually runs: record the owning thread on first use and assert it on every
subsequent call in a debug build, or route all consumption through a single object whose
construction is the only place the invariant is stated. A comment is not a control
(`[R:guarantee-needs-a-reader]`).

The class is wider than queues, and the other members fail the same way. An SPSC ring with two
producers. A structure documented as not reentrant, entered from a signal handler or a callback.
A `HashMap` reached from two threads because the field that held it became shared three
refactorings later. An iterator held across a mutation. In each case the regime was a hypothesis
of the correctness argument and nothing in the artefact records it.

Failure-mode check: **what regime does this structure's correctness argument assume, and what
would fail if I violated it right now?** If the honest answer is "the name says so" or "we are
careful", the structure is unguarded and the next person to reach for it will not know.
