+++
tag = "R:no-syscall-on-a-bounded-path"
title = "A syscall on a bounded path hands the deadline to the kernel"
error_class = "A system call on a path with a latency bound -- a clock read, a log write, a socket operation, a contended lock -- so the path's worst case includes a mode switch, whatever the kernel chooses to do inside it, and a cache and TLB partly evicted on return; and the cost is invisible in review because a syscall is spelled exactly like a function call"
home = { kind = "domain", name = "low-latency" }
applies_to = ["rust", "java"]
created = "2026-09-18"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-18 from the same review. The reason it is worth a rule rather than general knowledge is the clock: `clock_gettime` is a vDSO call costing tens of nanoseconds on one machine and a genuine syscall costing a microsecond on another, decided by a sysfs setting nobody in the code path has read, and the call looks identical in both."
+++

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
