+++
tag = "R:profiler-samples-where-it-can-stop"
title = "A sampling profiler reports where it was allowed to stop, not where the time went"
error_class = "Reading a sampling profiler's output as a map of where time was spent, when the sampler can only take a sample where the runtime permits one -- a JVM safepoint, an instruction boundary a signal can be delivered at -- so time spent in code containing no such point is attributed to the next one, and the hottest loop in the program can be entirely absent from its own profile"
home = { kind = "domain", name = "low-latency" }
applies_to = ["rust", "java"]
created = "2026-09-18"
origin = "codified"
source = "The Java safepoint-bias literature (Nitsan Wakart, Psy-Lob-Saw); async-profiler's rationale for AsyncGetCallTrace over JVMTI stack walks"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-18 from the same review as the coordinated-omission rule, and paired with it deliberately. Both are instrument defects that produce a confident number rather than an obviously broken one, and in both cases every downstream step of an ordinary process is reading the corrupted output and agreeing with it."
+++

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
