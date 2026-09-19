+++
tag = "R:a-wrapper-type-is-not-free-here"
title = "A wrapper type costs an object until escape analysis removes it"
error_class = "Carrying a Rust habit into Java unexamined -- wrapping every domain concept in its own type on the assumption that the wrapper compiles away -- so a decision that is genuinely zero-cost in one language quietly adds an allocation, an indirection and a cache miss per value in the other, on paths where that is the whole budget"
home = { kind = "domain", name = "java" }
created = "2026-09-19"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-19 with the rest of the Java set, and it exists because the corpus itself creates the hazard. `[R:newtype-liberally]` is stated for Rust with the argument that a newtype is ordinarily zero-cost -- which is true there and is not true here, and a reader holding both disciplines needs the difference stated rather than inferred."
+++

Give domain concepts their own types here too — an `OrderId` that cannot be passed where
an `AccountId` belongs is worth as much in Java as anywhere. But do not carry over the
*cost* argument with the *design* argument, because only one of them travels.

In Rust a newtype is a compile-time construct: `struct OrderId(u64)` is a `u64` at run
time and the compiler proves it. In Java, `record OrderId(long value)` is an **object** —
a header, a field, a reference to chase, and a separate cache line from whatever pointed
at it. The JIT can remove it: escape analysis plus scalar replacement will flatten a
wrapper that provably does not escape its compilation unit, which covers a great deal of
ordinary code. What it does not cover is the case you care about — a wrapper stored in a
field, put in a collection, handed across a method the JIT declined to inline, or in an
array, where `OrderId[]` is an array of pointers and `long[]` is an array of numbers.

So the discipline splits by where the value lives:

* **At an API boundary, and anywhere correctness is the concern**, use the wrapper. The
  swapped-argument bug it prevents is worth an allocation the JIT will usually remove
  anyway.
* **Inside a hot loop, in a large array, or in a per-message structure**, measure before
  assuming the wrapper vanished. This is `[R:verify-the-abstraction-compiled-away]`
  arriving in a language where the optimiser's decision is dynamic: `-XX:+PrintInlining`,
  a JFR allocation profile, or an escape-analysis dump says what actually happened, and
  the answer can differ between runs of the same binary.
* **Where it does not vanish and the budget is real**, keep the primitive and put the
  safety somewhere that costs nothing — a named parameter, a builder, a static factory
  whose signature cannot be called wrongly.

The version of this rule that is coming is worth knowing about rather than waiting for:
Project Valhalla's value classes are designed to make exactly this wrapper flatten by
specification instead of by the optimiser's discretion. Until the code runs on a JVM where
that is true, the cost is dynamic and must be measured rather than assumed.

Failure-mode check, before wrapping a primitive on a path with a budget: **does this
object still exist after the JIT has finished, and what told me?** "It is just a wrapper"
is the Rust answer, and it is the one that does not transfer.
