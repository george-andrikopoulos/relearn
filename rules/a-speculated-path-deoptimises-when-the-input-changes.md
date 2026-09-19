+++
tag = "R:a-speculated-path-deoptimises-when-the-input-changes"
title = "A path optimised on yesterday's inputs deoptimises when today's arrive"
error_class = "A JIT-compiled hot path whose speculative assumptions -- one receiver type at a call site, a branch never taken, a field never null, a class not yet loaded -- are invalidated at run time, so the path traps back to the interpreter, re-profiles and recompiles, and delivers its worst latency at the exact moment the input stopped being ordinary"
home = { kind = "domain", name = "java" }
created = "2026-09-18"
origin = "codified"
source = "Aleksey Shipilev, \"JVM Anatomy Quarks\", on profile pollution, inlining and deoptimisation; HotSpot's uncommon-trap mechanism"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-18 from a review of low-latency practice, and rehomed 2026-09-19 when the Java homing was examined. It was first written as low-latency doctrine carrying `applies_to = [\"java\"]`, on the reasoning that the error class is about a speculating runtime rather than a language's design discipline. That was wrong twice over. The test that settles it is whether the error class requires a latency requirement to exist or requires the JVM to exist: deoptimisation costs throughput as readily as latency and is invisible without a JIT, so the JIT is constitutive and the deadline is not. And the audience tag was doing no work the home does not already do -- `applies_to` decides whether a rule is emitted under a filter, never where it lands, so tagging a rule `java` creates no Java layer and never can. The practical difference is the whole point: `low-latency` is not a language, gets no globs, and is `OnRequest`, where `domain-java` attaches on reading any `.java` file."
+++

A JIT does not compile your code. It compiles a bet about your code, and the bet is settled at the
moment the bet stops being true.

HotSpot compiles on profile: this call site has only ever seen one receiver type, so inline it and
guard; this branch has never been taken, so do not emit it; this field has never been null, so
skip the check; this class has never been loaded, so assume no subclass exists. Each assumption
gets a guard, and the compiled path is fast precisely because the guards are cheap and the bodies
are absent. When a guard fails the frame is deoptimised: an uncommon trap, interpreter frames
rebuilt from the compiled ones, execution continuing interpreted, and a wait to be recompiled with
the new profile.

The timing is what makes this a latency rule rather than a throughput one. The assumptions break
when the input changes -- the unusual order type, the first exception of the day, a new venue's
message format, the second implementation of an interface loaded at hour six. So the worst latency
the path will ever produce is delivered on the least ordinary event, which is generally the event
that mattered most. Load testing with representative-on-average traffic reproduces none of it.

It is also silent by default. Nothing is logged, no metric moves, and the only trace is a latency
outlier indistinguishable from a collection pause or a scheduling delay. `-XX:+PrintCompilation`
and `-XX:+TraceDeoptimization` say what happened; JFR carries the events with less overhead and is
the one worth leaving on.

So warm the path with the **unusual** cases as well as the common ones -- which is where this
meets `[R:a-measurement-matches-the-regime-it-reports]`: a warm-up built only from typical traffic
produces a profile that typical traffic confirms and atypical traffic destroys. Keep hot call
sites deliberately monomorphic; a megamorphic interface call on the hot path is a permanent
inlining barrier rather than a one-off trap. Load the classes the path will need before the path
is live. And treat a lambda or a dynamic call site added to a hot path as a new profile that has
to be earned again.

Failure-mode check: **what has the JIT assumed about this path, and what happens the first time
one of those assumptions is false?** If the honest answer is that nobody knows what was assumed,
that is the finding -- the assumptions are readable, and reading them is the work.
