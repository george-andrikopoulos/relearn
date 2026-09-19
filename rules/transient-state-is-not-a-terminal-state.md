+++
tag = "R:transient-state-is-not-a-terminal-state"
title = "A window between two stores is not a state the reader may act on"
error_class = "A reader of a lock-free structure observing the gap between a writer's two stores and treating what it sees as a settled answer -- reading a not-yet-linked successor as an empty queue and parking, so an item that was already published sits undelivered until something unrelated wakes the reader"
home = { kind = "domain", name = "low-latency" }
applies_to = ["rust", "java"]
created = "2026-09-18"
origin = "codified"
source = "Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores; and grivet/mpsc-queue, a C11 implementation whose poll() splits the window out as MPSC_QUEUE_RETRY"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-18 alongside the producer-side rule, from the same review that found the low-latency domain holding two reasoning rules and no mechanism vocabulary. This is the half of the Vyukov MPSC queue that its own reputation hides: the algorithm is cited for a wait-free producer, and the consumer-side obligation that pays for it is left in the reader's hands without being named."
+++

When a writer publishes in two steps, the reader can see the middle. Decide what the
middle means before writing the read path, and never let it share a return value with a
settled state.

The Vyukov MPSC enqueue is an exchange followed by a store: the new node becomes the head
first, and only then does the previous node learn to point at it. Between those two
instructions the queue is fully populated and partly unreachable -- a consumer walking
from the tail finds a successor link that is still null. That null does not mean empty. It
means *not yet linked*, and the producer that owns the gap is a handful of cycles from
closing it.

A consumer that returns "empty" there has not merely returned early; it has converted a
few nanoseconds of writer-side latency into however long it takes for something else to
wake it up. If the consumer parks on an empty result, the item sits in a queue that
already contains it, and the symptom is an occasional multi-millisecond delivery in an
otherwise microsecond system. It reproduces under contention and vanishes under a
debugger, which is what makes it expensive to find later and cheap to prevent now.

**The exchange rate is what makes this expensive rather than untidy.** Waking a parked
thread is a futex into the kernel, a scheduler decision that need not be immediate, and a
restart with cache, TLB and branch predictors belonging to whatever ran in the meantime:
single-digit microseconds at best, tens to hundreds under load, against an enqueue costing
tens of nanoseconds. So misreading the window does not cost the window's width; it costs
three orders of magnitude more than the entire queue.

Which means the consumer's behaviour on an empty queue is a design decision in its own
right, and it has to be made rather than defaulted. Pure spinning gives the lowest and most
predictable delivery latency and costs a whole core -- and on a hyperthreaded sibling it
steals issue slots, so an idle spinner can slow the producer it is waiting for, and
spinning on a core the producer also runs on is worse than parking outright. Pure parking
costs nothing while idle and pays the wake on every item. Between them sit spin-then-yield,
and spin-briefly-then-park with the spin window sized to the observed inter-arrival gap,
which is usually the right answer for bursty arrivals: the burst is served at spin latency
and the quiet period costs nothing. Whether spinning is affordable at all is a question
about the machine's core budget and isolation, not about the consumer's code --
`[R:answer-the-requirement-at-its-layer]`.

A consumer that distinguishes the two states can take the cheap option for the one and the
expensive option for the other: spin through "not yet", which resolves in nanoseconds, and
park only on "drained", which is the state that might last.

Distinguish the two cases explicitly. If the successor is null *and* the tail is the
head, the queue is genuinely drained. If the successor is null and the tail is not the
head, a producer is mid-push: spin briefly, return a distinct "not yet" answer, or both --
but do not return the same value you return for drained, because the caller's correct
response is different in each case and a shared return value makes choosing impossible.
That is `[R:no-sentinel-values]` in a concurrent setting: "empty" and "a writer is in the
window" are two states, and collapsing them into one null is exactly the sentinel the rule
forbids.

**This rule is deliberately stronger than the reference implementation, and the evidence
is that the reference was corrected downstream.** Vyukov's own `mpscq_pop` reaches exactly
this fork -- `mpscq_node_t* head = self->head; if (tail != head) return 0;` -- and returns
the same `0` it returns for drained. He documents the consequence rather than fixing it,
as the algorithm's single listed disadvantage: *"Push function is blocking wrt consumer.
If producer blocked in (*), then consumer is blocked too."* The null is load-bearing and
uninformative at the same time. A later C11 implementation of the same algorithm
(`grivet/mpsc-queue`) splits the return into `MPSC_QUEUE_ITEM`, `MPSC_QUEUE_EMPTY` and
`MPSC_QUEUE_RETRY` -- the third value is this window, given a name. Do that.

The precise vocabulary is worth keeping, because it is what makes the window findable in a
specification rather than in a debugger: the queue is **serializable but not
linearizable**. Insertions are consistently ordered across producers, but an insertion is
two memory transactions, so the state can be found inconsistent *within* the series. The
consumer is correspondingly **obstruction-free, not wait-free** -- its progress depends on
another thread's, which is the asymmetry the headline "wait-free producers" conceals.

Failure-mode check, for any structure whose publication is more than one store: **what
does a reader see between them, and which of my return values is it currently sharing?**

The general form is a reader acting on a state the writer never intended to publish. It
arises anywhere a commit is not a single instruction -- a two-phase update, a length
written after a payload, a version counter bumped after the data it protects -- and the
remedy is always the same: make the intermediate state nameable, then refuse to let it
alias a terminal one.
