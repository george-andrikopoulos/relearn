+++
tag = "R:publish-safely-or-not-at-all"
title = "An object handed to another thread is published safely, or it arrives half-built"
error_class = "Making a reference visible to another thread without a happens-before edge -- a plain field write, a non-final field read after construction, a collection shared without synchronisation -- so the reader can observe a non-null reference to an object whose fields are still at their default values"
home = { kind = "domain", name = "java" }
created = "2026-09-19"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-19 with the rest of the Java set. It is the Java Memory Model's own subject and belongs here rather than in the latency domain by the homing test that decided this home: safe publication is a correctness requirement in any concurrent Java program, and the JMM is what makes it exist at all."
+++

Handing a reference to another thread is not the same as handing it the object. Name the
edge that makes the fields visible, or the reader may see the reference before the
contents.

The failure is specific and not intuitive: a thread can observe a **non-null** reference to
an object whose final assembly it cannot yet see, because the writes that filled the fields
and the write that published the pointer are not ordered with respect to each other. The
reader gets a real object with zeros and nulls in it. Nothing throws at the moment of
publication; the NullPointerException happens later, in code that did check for null, on a
field that was definitely assigned.

The mechanisms that establish the edge, cheapest first:

* **`final` fields.** A field assigned in the constructor and never after is guaranteed
  visible to any thread that sees the reference, without synchronisation — provided `this`
  did not escape during construction, which is exactly what
  `[R:no-reference-to-internals-escapes]` forbids and is why the two rules are not
  separable in practice. An immutable object with all-final fields is safe to hand to
  anyone, and this is the reason to prefer one.
* **`volatile`**, for the mutable case: a write to a volatile field happens-before every
  subsequent read of it, which publishes everything written before it.
* **A lock**, an `AtomicReference`, a `ConcurrentHashMap`, or a `BlockingQueue` — each
  carries the edge as part of its contract. This is why handing an object through a proper
  queue is safe and handing it through a plain field is not.
* **`VarHandle`** where the ordering must be stated precisely rather than taken at
  `volatile`'s strength.

What does **not** establish it: a `synchronized` block the reader does not also enter, a
`HashMap` shared between threads, a field that merely happens to be assigned before the
thread starts but is not final, and "it works every time we run it" — which is
`[R:verify-ordering-on-the-weakest-target]` arriving in Java, since x86 hides most of this
and aarch64 does not.

Failure-mode check, for every object crossing a thread boundary: **what happens-before
edge makes its fields visible, and can I name it?** If the answer is that the reference is
assigned before the reader starts, that is an argument about time, and the JMM makes no
promises about time.
