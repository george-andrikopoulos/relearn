---
name: domain-low-latency
description: "Engineering discipline for working in low-latency. Covers: attack the design in a second pass; a measurement matches the regime it reports; a queue without a bound has no overload behaviour; a structure keeps the regime it was proved under; allocated is not resident; answer the requirement at its layer; no allocation on the hot path; no coordinated omission; no false sharing on a hot line; no retry loop on a contended path; no stall inside a publication window; no syscall on a bounded path; profiler samples where it can stop; transient state is not a terminal state; verify ordering on the weakest target"
---

# domain: low-latency rules

## A measurement states whether it ran warm or cold, and matches production [R:a-measurement-matches-the-regime-it-reports]

> Written for the rust and java audiences.

> Written down from JMH and its samples (Aleksey Shipilev), whose existence is the standing claim that a hand-rolled microbenchmark on the JVM is wrong by default.

State which regime the measurement ran in, and check that it is the regime production runs in.
Two errors live here and only one of them has a reputation.

**Measuring cold when production is warm.** A first execution runs interpreted or from cold
instruction cache, with an untrained branch predictor, unpopulated page tables and, on a managed
runtime, no compiled code at all. The figure can be one to two orders of magnitude worse than the
steady state, and it describes the warm-up rather than the work. On the JVM the escalation is
staged -- interpreter, then C1, then C2, with on-stack replacement for long-running loops -- and
takes thousands of iterations, which is why a hand-rolled timing loop is wrong by default and why
a harness exists.

**Measuring warm when production is cold**, which is the half that gets no attention because it
looks like rigour. A path that runs once a minute is never warm. A market-open burst arrives into
cold code. An error handler runs for the first time that day at the worst possible moment. For
every one of those, the warm figure is a number no user will ever see, and the warm-up that was
carefully discarded *is* the production behaviour. Discarding it is not hygiene there; it is
deleting the measurement.

So the rule is a scope match rather than a procedure: say which regime was measured, say which
regime production is in, and show they are the same. Where production is genuinely both -- a
steady stream with cold bursts -- that is two measurements and two reported numbers, not one
averaged into meaninglessness.

This is `[R:measure-the-claim-not-a-subset]` applied to time rather than to a set: the warm-up
samples are a subset, discarding them is a scope decision, and a scope decision taken by reflex
is the one that manufactures a wrong number with the authority of having been measured.

Failure-mode check: **is the production path warm or cold when it matters, and which did I
measure?** If the benchmark discarded the first N iterations, name what production has that
corresponds to those iterations. If nothing does, the discarded part was the answer.

Two neighbours, deliberately separate. `[R:no-coordinated-omission]` is the instrument failing to
take samples during the events that matter; this is the instrument taking them in the wrong
regime. And `[R:verify-the-abstraction-compiled-away]` holds the case where the optimiser removed
the work the benchmark was timing -- a warm measurement of nothing at all.

## An unbounded queue converts overload into latency instead of refusal [R:a-queue-without-a-bound-has-no-overload-behaviour]

> Written for the rust and java audiences.

> Written down from Little's Law (J. D. C. Little, 1961): L = lambda W, so for a queue that never refuses, wait grows without bound once arrival rate exceeds service rate.

Decide what the queue does when consumption falls behind production. Growing is not a decision.

Queue depth *is* latency -- that is Little's Law and not a heuristic: with arrivals at rate lambda
and mean queue length L, the mean wait is L/lambda, so an unbounded L is an unbounded wait. A
queue with no capacity limit therefore has no latency limit, and a latency requirement stated
anywhere upstream of it is not held by anything.

The failure has a characteristic shape that makes it late to diagnose. Producers never block, so
nothing upstream reports a problem; throughput looks correct, because every item is eventually
processed; and the only symptom is that latency climbs monotonically. By the time it is visible
the backlog is doing work nobody wants -- every item consumed is already stale, so capacity is
being spent delivering answers to questions whose moment has passed, which is why the system does
not recover when load returns to normal. It has to drain first, at the same rate that fell behind.

So bound the queue, and choose the behaviour at the bound. Four are legitimate and the choice is
a domain decision, not a technical one:

* **Block the producer.** Real backpressure -- the slowest consumer sets the rate for everyone,
  and the pressure propagates to whoever can actually shed it. Wrong where the producer must not
  be stopped, which is the regime `[R:no-stall-inside-a-publication-window]` is about.
* **Drop the oldest.** Correct for market data, sensor readings, telemetry: a stale tick has
  negative value and the newest is the only one worth having.
* **Drop the newest.** Correct where the backlog is a fair queue and latecomers have not been
  promised anything.
* **Reject, and say so.** Correct for order entry and anything a caller must know the fate of. A
  silently dropped order is worse than a refused one by the whole width of the trust the caller
  placed in the call.

Not choosing selects a fifth behaviour that nobody would choose deliberately: absorb everything,
degrade every item's latency equally, and fail eventually by exhausting memory -- or, on a managed
runtime, by a collection pause caused by the backlog itself, which slows the consumer further and
is the closest thing in production to a feedback loop with the wrong sign.

The unbounded structure remains the right choice where the bound genuinely lives elsewhere -- a
queue whose producers are rate-limited upstream, or whose consumer is provably faster than any
possible arrival rate. Say which, in the code, next to the queue. An unbounded queue with a stated
reason is a decision; one without is the absence of a decision wearing the same shape.

Failure-mode check: **if consumption falls behind production for ten seconds, what does this queue
do and who finds out?** If the answer is "it grows" and "nobody", there is no overload behaviour
here, only an overload symptom waiting.

## A structure is correct only under the concurrency regime its proof assumed [R:a-structure-keeps-the-regime-it-was-proved-under]

> Written for the rust and java audiences.

> Written down from Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores -- where the regime is stated in the algorithm's name and nowhere in its code.

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

## Memory that has never been touched is not yet memory [R:allocated-is-not-resident]

> Written for the rust and java audiences.

> Written down from Ulrich Drepper, "What Every Programmer Should Know About Memory" (2007), on demand paging and the cost of a first touch.

Pre-allocating is not preparing. A successful allocation buys address space; the page arrives on
first touch, and the touch is what costs.

`malloc` and `mmap` hand back virtual addresses. Physical pages are supplied lazily: the first
write to each page traps into the kernel, which finds a frame, zeroes it -- because the previous
tenant's data must not leak -- and maps it. So a one-gigabyte buffer allocated at startup and
first written on the hot path contains roughly a quarter of a million page faults, arriving one
per page, spread across the first pass over it. Every allocation-free assertion on that path is
satisfied and every one of them is measuring the wrong thing: nothing called the allocator.

Residency is also not permanent once achieved. Pages can be reclaimed under memory pressure or
swapped, so a long-lived process that touched its buffers at startup and then left them idle can
fault again on a path that has been fault-free for a week. That is why the remedy has two halves
and why only doing the first is a common and expensive mistake.

So, for anything on a path with a deadline: write to every page the path will touch, at startup,
before any real work arrives -- the touch has to be a write, since a read of an untouched page may
be served by the shared zero page and prove nothing. Then pin it: `mlockall(MCL_CURRENT |
MCL_FUTURE)`, and no swap on the box. On the JVM, `-XX:+AlwaysPreTouch` does the first half for
the heap and is worth the slower startup precisely because the cost is moved off the path that
has a bound.

Huge pages change the arithmetic in both directions and deserve their own decision rather than a
default. Fewer, larger pages means fewer faults and less TLB pressure; each fault is bigger, and
transparent huge pages bring a defragmentation path that can stall a thread for milliseconds at
exactly the wrong moment. Explicit huge pages reserved at boot avoid that; leaving THP on
`always` and hoping is the shape this domain keeps having to name.

**Touch from the thread that will own the memory, not from the thread doing the setup.** On a
multi-socket machine the first write also decides *placement*: a page is allocated on the node
of whoever faults it, so the obvious way to satisfy this rule -- one loop in the initialisation
thread walking every buffer -- puts every page on that thread's node, and a worker pinned to the
other socket then reads all of it across the interconnect at roughly twice the local latency.
The path is now fault-free and entirely remote: a slower steady state bought with a faster first
message. Pin the owning threads first, then let each of them touch what it will use.

Failure-mode check: **has every page this path will touch been written to, by the thread that
will read it, and what stops it being reclaimed?** Three questions, because answering only the
first leaves a system that is fast until it goes idle, and answering only the first two leaves
one that is fast on a single socket.

`[R:no-allocation-on-the-hot-path]` is the sibling and neither subsumes the other -- that one is
about entering the allocator, this one about the memory it already returned.
`[R:answer-the-requirement-at-its-layer]` is why the remedies are all a memory policy, a lock, or
a boot parameter: no choice of container or crate reaches this.
`[R:no-stall-inside-a-publication-window]` is where it bites hardest, because a page fault taken
between a writer's two stores holds the window open for its whole duration.

## Answer a requirement at the layer it lives at [R:answer-the-requirement-at-its-layer]

> Written for the rust and java audiences.

State which layer a requirement lives at before choosing anything, and satisfy it there.

A requirement about the machine -- do not perturb a neighbouring isolated core, do not
fault a page on the send path, do not take a lock a real-time thread contends for -- is
not answered by picking a faster crate, a better-reputed transport, or an asynchronous
API. Those are library-level decisions. They may be correct and they may even help, but
they do not *hold* a systems-level constraint, and treating them as though they do
converts an open question into a closed one with nothing behind it.

The failure is not the wrong choice; it is the wrong ledger entry. A constraint marked
satisfied stops being examined. An open constraint is still visible to the next person,
which makes leaving it open strictly better than closing it at a layer that cannot keep
it.

So name the layer explicitly, then name what holds the requirement there: a CPU affinity
mask, an isolated core list, a memory policy, a pre-faulted and locked mapping, a thread
priority, a cgroup. If the answer to "what holds this?" is the name of a library, the
requirement is unheld.

Two rules meet here and neither subsumes this one. `[R:measure-cost-per-task]` governs
*how to choose* a mechanism -- by what it actually costs and touches, never by
reputation -- and its failure-mode check asks under what configuration the mechanism
causes the exact harm it was chosen to prevent. That check is necessary and it is not
sufficient: a mechanism can pass it and still be the wrong layer to have asked at.
`[R:guarantee-needs-a-reader]` is the general form of the ledger failure -- a claim with
nothing reading the state it asserts -- and this is the shape that claim takes when the
state in question is a property of the machine rather than of the code.

Then run the pass the sibling rule requires: `[R:attack-the-design-in-a-second-pass]`
exists because the person who chose the mechanism is the last person able to see that
they answered at the wrong layer.

## Attack the design in a separate pass, and build nothing in it [R:attack-the-design-in-a-second-pass]

> Written for the rust and java audiences.

> Has recurred 1 time(s) since it was written; most recently 2026-09-14.

When a component is finished, do a second pass whose only job is to attack it. Build
nothing in that pass.

The pass that produced a design cannot review it. It carries every assumption that went
into the design, including the ones nobody stated, and it will read the output as
confirmation because that is what it was optimising for. A review folded into the build
is not a weak review; it is the build, wearing a different label.

So separate it in time and in intent, and give it something to do:

- **Justify the choice against the simpler alternative that was not taken.** Not "is this
  good" but "why not the obvious thing" -- and if the answer is that the obvious thing was
  never considered, that is the finding.
- **Walk the failure modes out loud.** Under what configuration does this do the thing it
  was chosen to prevent? What does it do under saturation, on a cold cache, on the first
  message after an idle period, when the peer is slow rather than absent?
- **Prove the stated constraint, do not restate it.** "It does not perturb the isolated
  cores" is a claim; the affinity mask, the thread inventory and the measurement are the
  proof. `[R:answer-the-requirement-at-its-layer]` is what this pass is usually checking.
- **Ask what the evidence would look like if the opposite were true.** An artefact that
  exists, a command that exited zero and a test that passed are all compatible with the
  work never having happened -- which is `[R:verdict-survives-the-channel]`, and it is the
  single most common thing this pass catches.

**Mandatory for anything latency-sensitive.** In that domain the failure does not announce
itself: a design that perturbs a neighbouring core, allocates on a send path, or faults a
page under load still passes every functional test, and the cost appears as unexplained
tail latency somewhere else entirely -- attributable to nothing, days later, by somebody
who was not there.

## A latency-bounded path does not enter the allocator [R:no-allocation-on-the-hot-path]

> Written for the rust and java audiences.

A path with a latency bound owns its memory before the path starts.

The bound you can state about an algorithm is a bound on the algorithm. Allocation is how
it stops being a bound on the program: the fast path of a modern allocator is a few
nanoseconds and a handful of instructions, and its slow path takes a lock, asks the kernel
for pages, faults them in on first touch, or -- on a managed runtime -- stops the
application. Those are not rare events that can be amortised into an average. They are the
tail, and the tail is the number the requirement was written about.

A concurrent collector does not remove this; it changes what you pay and when. ZGC and
Shenandoah move the bulk of the work off the stop-the-world pause, which is why their
pause figures are quoted in sub-milliseconds and why those figures are then read as
"allocation is free here". They are not the same claim. The collector now runs
*alongside* the application, competing for cores and memory bandwidth with the path that
has the deadline, and when the allocation rate outruns what it can reclaim concurrently
the allocating thread is stalled until it can -- an allocation stall, which does not
appear in the pause metric at all. So the number that governs a bounded path is the
allocation **rate** the path sustains, not the collector's advertised pause. Quoting the
pause is the managed-runtime instance of `[R:measure-the-claim-not-a-subset]`: a real
measurement of one thing, offered against a claim about another.

This is why a lock-free queue worth using is *intrusive*: the node is a field inside the
caller's message, already allocated, already resident, so enqueue stores pointers and
never asks anyone for memory. Wrapping that queue in an API that boxes each message
undoes its entire argument while leaving every benchmark of the queue itself intact --
`[R:answer-the-requirement-at-its-layer]`, with the requirement held one layer above where
it was measured.

Allocation is easy to reintroduce without writing the word. A vector that grows, a hash
map that rehashes, a formatted log line, a boxed closure, a trait object returned by
value, a collected iterator, an error type that carries a `String` -- and in Java, every
one of those plus autoboxing and varargs. So the control cannot be "remember not to
allocate": pre-size every collection and assert its capacity never changes, take the
buffer as a parameter instead of returning an owned one, and where the platform allows it,
install an allocator that aborts when called from a thread marked hot. A path that cannot
allocate without failing loudly is `[R:prefer-by-construction]`; a path that merely should
not is prose.

Failure-mode check, for every call on a bounded path: **what does this do when the
allocator takes its slow path, and what proved this cannot reach the allocator at all?**

Pre-faulting and locking the pages is the same argument one level down, and it has to be
made separately: memory that is allocated but never touched is not resident, so the first
write on the send path is a page fault whether or not anything called the allocator.

## A load generator that waits for the system deletes the system's worst latencies [R:no-coordinated-omission]

> Written for the rust and java audiences.

> Written down from Gil Tene, "How NOT to Measure Latency"; and HdrHistogram's recordValueWithExpectedInterval, which exists to correct it.

Ask, before reporting any latency figure: **when the system under test stalls, does my load
generator stall with it?** If it does, the figure is not a percentile of the system's behaviour.

A closed-loop generator sends a request, waits for the response, then sends the next. That is the
natural way to write one and it is measuring the wrong thing. Suppose the intended rate is one
request every 10ms and the system stalls for one second. A closed-loop generator issues **one**
request during that second and records **one** sample of 1000ms. An open-loop generator, or a real
user population, would have issued a hundred: the first waits 1000ms, the next 990ms, the next
980ms, down to the last. The true contribution of that stall is a hundred samples averaging about
500ms. The closed-loop generator recorded one.

The consequence is not a small error and not a noisy one. Ninety-nine of the hundred worst samples
were never taken, so they cannot appear at any percentile, and the tail -- the only part of the
distribution anyone asked about -- is computed over a sample set from which the bad events were
systematically removed. This is how "the 99.9th percentile is 2ms" and "the 99.9th percentile is
800ms" can both be measurements of the same system on the same afternoon.

Three properties make it worse than ordinary measurement error, and they are the reason this needs
a rule rather than care:

* **It is systematic, not random.** Running longer, repeating the experiment, or averaging across
  runs does not reduce it. Every run omits the same events for the same reason.
* **It always flatters.** The bias has one direction. A number that is wrong in an unknown
  direction invites suspicion; a number that is always optimistic gets believed.
* **It scales with the badness it hides.** The worse the stall, the more samples the generator
  fails to take, so the measurement degrades fastest exactly where the system does.

So: drive load open-loop at a stated rate, and record each request's latency from the time it
**should have been sent**, not from the time the generator got round to sending it. Where an
existing closed-loop harness cannot be replaced, correct the recording --
`recordValueWithExpectedInterval` exists for precisely this and is not an optional refinement.
And report the intended rate beside the percentile: a tail latency with no load attached to it is
not a measurement of anything, which is the same point `[R:no-retry-loop-on-a-contended-path]`
makes about producer counts.

Failure-mode check: **what stops my instrument from taking a sample, and would it stop at the
moments I care about?**

`[R:measure-the-claim-not-a-subset]` is the near neighbour and the two are deliberately separate.
There, an analyst measures a narrower set than the claim covers and manufactures a discrepancy;
the remedy is to state the scope. Here the *instrument* silently deletes samples while the analyst
does everything right, the scope looks identical, and the remedy is in the harness rather than in
the reasoning. Same family -- a subset wearing the whole -- different mechanism and different fix.

## Two hot values on one cache line contend without sharing anything [R:no-false-sharing-on-a-hot-line]

> Written for the rust and java audiences.

> Written down from Ulrich Drepper, "What Every Programmer Should Know About Memory" (2007), §3.3 on cache coherency and §6.4 on multi-threaded optimisation.

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

## A contended path completes in a fixed number of steps, or its worst case is unbounded [R:no-retry-loop-on-a-contended-path]

> Written for the rust and java audiences.

> Written down from Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores (sites.google.com/site/1024cores).

On a path many writers contend for, prefer a structure whose writer side is a single
unconditional atomic operation over one that retries until it wins.

The Vyukov MPSC enqueue is the shape to reach for: swap the new node into the head with
one atomic exchange, then store the back-link. The exchange cannot fail, so it cannot be
retried, so the number of attempts is one whether one producer is pushing or sixty.

**That is a bound on steps, not on time, and the difference is the whole of what this rule
does and does not promise.** An exchange under contention still has to take the cache line
exclusively, and the line can only be in one core's cache at a time, so sixty producers
hammering it queue for it: the *time* for one enqueue grows with the number of contenders
even though the *step count* does not. What wait-freedom buys is that the growth is linear
and every attempt makes progress. A retry loop has no such floor -- an unlucky writer can
lose repeatedly, and the arrival of more contenders both lengthens each round and makes
losing more likely, so the tail grows faster than linearly and has no bound the code can
state. Choose the exchange because its worst case is a queue you can reason about, not
because it is free.

A Treiber-style CAS loop is the shape to justify before using. Read the head, build the
node, compare-and-swap, and go round again if somebody else got there first. Every
individual operation is cheap and the structure is lock-free by the textbook definition,
which guarantees that *the system* makes progress -- it guarantees nothing whatever about
*this writer*, which may lose arbitrarily many times. Lock-freedom is a liveness property
of the ensemble; latency is a property of the individual, and the two are routinely
confused because the word "free" is doing work it was never asked to do.

Read the progress guarantee per role, never for the structure as a whole. Vyukov's own
summary is "wait-free and fast producers", and it is exact -- about producers. The
consumer of the same queue is **obstruction-free**: it can be held up by a producer that
was descheduled mid-push, which is the listed disadvantage of the algorithm rather than a
subtlety of it. A structure described by the guarantee of its best-served role will be
adopted for the role it serves worst.

The measurement trap is what makes this worth a rule rather than a preference. A retry
loop's mean is almost unaffected by contention, because most attempts succeed first
time. What moves is the tail, and it moves superlinearly, so a benchmark at two producers
and a production deployment at thirty-two are not the same experiment. Do not report a
producer-side latency without reporting the producer count it was taken at.

Failure-mode check, before adopting any lock-free structure on a contended path: **how
many times can one writer go round before it succeeds, and what bounds that number?** If
the answer is "in practice, not many", the path is unbounded and nothing holds it.

`[R:measure-cost-per-task]` is the general form -- choose by what a mechanism actually
costs, never by its reputation -- and "lock-free" is exactly the kind of reputation it
warns about. `[R:answer-the-requirement-at-its-layer]` is the sibling that catches the
other half: a bounded producer path is a property of the algorithm, and it does not
survive being wrapped in something that allocates or takes a lock.

## A writer that can be stopped mid-publication makes the reader's latency the scheduler's [R:no-stall-inside-a-publication-window]

> Written for the rust and java audiences.

> Written down from Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores (sites.google.com/site/1024cores).

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

## A syscall on a bounded path hands the deadline to the kernel [R:no-syscall-on-a-bounded-path]

> Written for the rust and java audiences.

Know which calls on the path enter the kernel. The answer is not readable from the source, because
nothing about the spelling distinguishes a syscall from a function call.

A mode switch is the floor, not the cost. On top of it sits whatever the kernel does inside the
call, plus what it evicts: the return comes back to a cache and TLB partly filled with kernel
working set. The floor itself moved within living memory -- the Spectre and Meltdown mitigations
(page-table isolation, indirect-branch controls) multiplied the entry and exit cost several times
over, so a great deal of latency-sensitive code was measured and written in a world that no longer
exists and has never been re-measured in this one.

The calls that reach the kernel without announcing it are the ones to look for:

* **The clock.** `clock_gettime` is served from the vDSO -- no syscall, tens of nanoseconds --
  only while the clocksource is TSC. With the clocksource at HPET or `acpi_pm`, the same call
  traps, and costs a microsecond or more. Read
  `/sys/devices/system/clocksource/*/current_clocksource` on the actual machine rather than
  assuming. A timestamp taken per message is the commonest way this lands on a hot path.
* **Logging.** A log call that formats and writes is a write syscall, and a blocking one if the
  sink is a pipe or a full buffer. Format and hand off; never write from the path.
* **Allocation.** The allocator's fast path is userspace; its slow path is `mmap` or `brk`. This
  is the kernel-side half of `[R:no-allocation-on-the-hot-path]`, and `[R:allocated-is-not-resident]`
  is the same trap reached without any call at all -- a page fault is a trap into the kernel that
  no audit of call sites will find.
* **Locks.** An uncontended futex-based mutex stays in userspace; a contended one enters the
  kernel to sleep. So the lock's cost is a function of the contention, which is a function of
  load, which is why it is absent from every unloaded measurement.
* **Sockets.** Every send and receive, unless the path is kernel bypass or busy-poll.

Remedies are per-call and mostly mean moving the work off the path rather than making it cheaper:
batch and defer logging to another thread, take the timestamp once and pass it, pre-allocate and
pre-fault, keep locks uncontended or remove them, and use bypass or busy-polling where the I/O
itself is the bound.

Failure-mode check: **which of the calls on this path enters the kernel, and what did I read to
know that?** If the answer is that it looks like a library call, nothing has been established --
and the clock is the one that will be wrong.

## A sampling profiler reports where it was allowed to stop, not where the time went [R:profiler-samples-where-it-can-stop]

> Written for the rust and java audiences.

> Written down from The Java safepoint-bias literature (Nitsan Wakart, Psy-Lob-Saw); async-profiler's rationale for AsyncGetCallTrace over JVMTI stack walks.

Before acting on a profile, ask what the profiler was **able** to sample -- then assume everything
it could not sample is missing from the answer.

A sampling profiler does not interrupt a thread wherever it likes. It stops it where stopping is
legal, and on a managed runtime that means a safepoint: a poll the JIT inserted at a method return
or a back-edge it could not prove bounded. A counted `int` loop is exactly the shape the JIT
proves bounded, so it may contain **no safepoint poll at all** -- and a thread spinning inside one
cannot be sampled while it is there. The sampler waits, the thread leaves the loop, the sample is
taken at the next legal point, and the time is attributed to whatever runs next.

The result is not a blurred picture. It is a picture of a different program. The method consuming
most of the wall clock can appear nowhere in its own profile, while the small method after it
appears to dominate. The remedy people reach for -- sample more often, run longer -- does nothing,
because the bias is deterministic: the same code is unsamplable on every run, so more samples
means more samples of the same lie. This is the shape `[R:verify-ordering-on-the-weakest-target]`
names in a different domain: the defect is not *rare* in the instrument, it is *absent* from it,
and repetition cannot find what the instrument cannot represent.

So choose the instrument by where it can sample, and say which one was used beside any profile
that decides something:

* On the JVM, a sampler built on `AsyncGetCallTrace` (async-profiler) takes its sample from a
  signal handler rather than at a safepoint, which is the whole reason it exists. Hardware PMU
  sampling via `perf` with a JIT symbol map is the same argument one layer down.
* Anything that walks stacks through JVMTI or `Thread.getStackTrace` -- which includes most
  IDE-bundled and APM profilers -- is safepoint-biased by construction. It is not useless; it is
  answering a different question than the one being asked of it.
* Outside a managed runtime the same structure applies with different names: a signal-based
  sampler cannot sample where signals are blocked or deferred, and an instrumenting profiler
  changes the code it measures, so inlining decisions differ between the profiled build and the
  shipped one.

Failure-mode check, before believing any profile: **where can this profiler not take a sample, and
what would code living there look like in its output?** The answer is that it would look like the
code that runs immediately afterwards -- which is indistinguishable from a real finding.

`[R:measure-cost-per-task]` is the general form, measure rather than assume, and this is the case
where measuring is not sufficient because the measurement itself carries the assumption. The
sibling in this domain is `[R:no-coordinated-omission]`: there the instrument fails to take
samples during the events that matter, here it fails to take them in the code that matters.

## A window between two stores is not a state the reader may act on [R:transient-state-is-not-a-terminal-state]

> Written for the rust and java audiences.

> Written down from Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores; and grivet/mpsc-queue, a C11 implementation whose poll() splits the window out as MPSC_QUEUE_RETRY.

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

## Memory ordering is verified on the weakest architecture it ships to [R:verify-ordering-on-the-weakest-target]

> Written for the rust and java audiences.

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

<!-- relearn:generated v0.1.0 sha256=74dd8d78ee3a3dc28852c988f4737ab89fbb9f8c2eaccab212f40d34f376ed89 rules=R:a-measurement-matches-the-regime-it-reports,R:a-queue-without-a-bound-has-no-overload-behaviour,R:a-structure-keeps-the-regime-it-was-proved-under,R:allocated-is-not-resident,R:answer-the-requirement-at-its-layer,R:attack-the-design-in-a-second-pass,R:no-allocation-on-the-hot-path,R:no-coordinated-omission,R:no-false-sharing-on-a-hot-line,R:no-retry-loop-on-a-contended-path,R:no-stall-inside-a-publication-window,R:no-syscall-on-a-bounded-path,R:profiler-samples-where-it-can-stop,R:transient-state-is-not-a-terminal-state,R:verify-ordering-on-the-weakest-target -- DO NOT EDIT; regenerate with `relearn build` -->
