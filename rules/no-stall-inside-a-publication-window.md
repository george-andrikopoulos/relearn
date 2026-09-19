+++
tag = "R:no-stall-inside-a-publication-window"
title = "A writer that can be stopped mid-publication makes the reader's latency the scheduler's"
error_class = "A publication that spans two stores performed by a thread that can be cancelled, preempted, paused at a safepoint or faulted inside the window, so a reader blocked on that window waits for whatever stopped the writer -- and the wait is bounded by the scheduler, the collector or the page cache rather than by anything in the algorithm"
home = { kind = "domain", name = "low-latency" }
applies_to = ["rust", "java"]
created = "2026-09-18"
origin = "codified"
source = "Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores (sites.google.com/site/1024cores)"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-18 after reading Vyukov's original pages, and the reason it exists is the reason it is worth recording. Four rules had already been drafted that day from the same material discussed second-hand, and all four took the reader's side of the two-store window -- how a consumer must interpret what it sees. The primary source states the writer's side as the algorithm's single listed disadvantage, `Push function is blocking wrt consumer. If producer blocked in (*), then consumer is blocked too`, and a later C11 implementation of it spells out the consequence: producers cannot be cancelled mid-insertion without risking consumer deadlock. Reading the source produced an error class the derived discussion had not contained."
+++

Before adopting a structure whose publication is two stores, name everything that can stop
the writer between them -- and treat the reader's worst case as however long that lasts.

The window is the same one the reader-side rule describes, seen from the other end. A
producer in Vyukov's MPSC queue performs the exchange, and at the point his source marks
`(*)` it has become the head while the previous node does not yet point to it. If the
producer is stopped there, the consumer cannot reach an item that is already in the queue.
His own summary is blunt about it: *"If producer blocked in (*), then consumer is blocked
too."*

So the consumer's latency bound is not a property of the queue. It is a property of
whatever can suspend a producer:

* **Cancellation.** A thread cancelled inside the window never closes it. The item is
  unreachable permanently, not slowly -- this is a deadlock, not a delay.
* **Preemption.** The producer loses its timeslice, or a higher-priority thread takes the
  core. The consumer waits for the producer to be scheduled again.
* **A safepoint or a collection pause.** On a managed runtime the producer is stopped by
  the runtime rather than by anything in the code, and the window is held for the whole
  pause.
* **A page fault** on the node's own memory, which is why `[R:no-allocation-on-the-hot-path]`
  and its pre-faulting paragraph are load-bearing here rather than merely adjacent.
* **Migration, steal time, a debugger, a signal handler that runs long.**

The sharp case is priority inversion, and it is sharp because the usual remedy is
unavailable. A low-priority producer stopped in the window holds up a high-priority
consumer -- the textbook shape -- but there is no lock, so there is no priority-inheritance
protocol to enable, and nothing in the structure to hand a priority to. The mitigation has
to be arranged outside the algorithm or it does not exist: producers at no lower priority
than the consumer, or cooperative scheduling, or both. A "lock-free" label reliably reads
as "immune to inversion", and here it is the opposite -- it removes the mechanism that
would have solved it.

The remedies are all systems-layer, which is the point:

* Do not enqueue from a thread that can be cancelled, and keep insertions outside
  cancellable sections -- the C11 implementation of this algorithm says exactly this, and
  recommends a cooperative threading model instead.
* Do not enqueue from a signal handler, a finalizer, or any context the runtime may stop.
* Give producers a priority and an affinity that bound how long they can be off-core, and
  state the bound.
* Where the reader has a hard deadline, prefer a structure whose publication is a **single**
  release store -- a bounded slot array with a sequence counter, the shape of Vyukov's own
  bounded MPMC queue -- and pay for it in capacity. An unbounded queue with a two-store
  publication trades a memory bound for a latency bound, and that trade should be made on
  purpose.

Failure-mode check, before a two-store publication goes on a path with a deadline: **what
can stop this thread between the two stores, and what is the reader's latency while it is
stopped?** If the answer names the scheduler or the collector, the deadline is held by
neither.

`[R:answer-the-requirement-at-its-layer]` is why none of the remedies above is a library
choice: every one of them is a priority, an affinity, a threading model or a page
residency decision, and swapping the queue implementation answers none of them.
`[R:no-retry-loop-on-a-contended-path]` is what the window bought -- a wait-free producer --
and this rule is the invoice. `[R:transient-state-is-not-a-terminal-state]` is the reader's
obligation inside the same window; satisfying it is necessary and does nothing whatever
about a writer that has stopped.
