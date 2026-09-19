---
paths:
  - "**/*.java"
---

# Rules for domain: java

## A path optimised on yesterday's inputs deoptimises when today's arrive [R:a-speculated-path-deoptimises-when-the-input-changes]

> Written down from Aleksey Shipilev, "JVM Anatomy Quarks", on profile pollution, inlining and deoptimisation; HotSpot's uncommon-trap mechanism.

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

## A collection view is a window onto someone else's data [R:a-view-is-not-a-copy]

Know whether you are holding data or a window onto data. The static type is `List<T>`
either way and will never tell you.

The library returns views far more often than most code assumes, and each has its own way
of surprising:

* **`Arrays.asList(a)`** is a fixed-size wrapper *over the array*. `set` writes through to
  the array; `add` and `remove` throw `UnsupportedOperationException`. `List.of(...)` is
  immutable and independent, and is usually what was meant.
* **`list.subList(from, to)`** is a live window. Writes go through, and any structural
  change to the parent makes the sublist throw `ConcurrentModificationException` on its
  next use — including the common idiom of taking a sublist and then clearing the parent.
* **`map.keySet()`, `values()`, `entrySet()`** are views. `keySet().remove(k)` removes the
  entry from the map. That is occasionally what you want and always worth being deliberate
  about.
* **`Collections.unmodifiableList(l)`** wraps without copying: the wrapper refuses writes
  and the original still works, so it defends against the caller and not against you. It
  is not a defensive copy, which is why `[R:no-reference-to-internals-escapes]` reaches for
  `List.copyOf` instead.
* **`Stream.toList()`** is unmodifiable; **`collect(Collectors.toList())`** gives no such
  guarantee and may or may not be. Do not rely on either being mutable.

So state which you want at the point you create it. `List.copyOf` when the receiver must be
independent, the view when sharing is the intent and the lifetime is short. A view stored
in a field is the shape to look at hardest: it outlives the expression that made it, and
the backing collection can change under it at any point afterwards.

The same distinction governs iteration. Modifying a collection while iterating it throws
`ConcurrentModificationException` even single-threaded, because the iterator is a view too
— use `removeIf`, or iterate a copy, or collect the removals and apply them after.

Failure-mode check, for any collection received or returned: **if somebody writes to this,
what else changes?** If you cannot answer without opening the factory method's Javadoc,
copy it.

## A wrapper type costs an object until escape analysis removes it [R:a-wrapper-type-is-not-free-here]

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

## A resource is closed by the construct that opened it, on every path [R:close-what-you-open]

Open a resource in a `try`-with-resources header and nowhere else. The construct closes on
every exit, in reverse order, and — the part a hand-written `finally` cannot do — it keeps
the original exception when the close also fails.

```java
try (var conn = pool.take(); var stmt = conn.prepare(sql)) {
    return stmt.execute();
}
```

The failure that a `finally` block introduces is worth stating plainly because it looks
correct. If the body throws and `close()` also throws, the `finally`'s exception replaces
the body's: the program reports a failure to close a connection and says nothing about the
query that failed first. Try-with-resources instead **suppresses** the close failure and
attaches it to the original, so both survive and the cause is the one you wanted. A leak
costs you a handle; a lost cause costs you the investigation.

Three cases where the construct does not apply and the discipline still does:

* **A resource whose lifetime is a field**, not a block — a pool, a client, an executor.
  The owning object becomes `AutoCloseable` and the discipline moves up a level; the thing
  to refuse is a field that is opened and never closed by anyone in particular.
* **An `ExecutorService`** is not closed by `shutdown()` alone. `shutdown()` then
  `awaitTermination()` then `shutdownNow()` is the sequence, and skipping it leaves
  non-daemon threads holding the JVM up.
* **A `Stream` over a file** (`Files.lines`, `Files.walk`) holds a handle and must be
  closed. It is the one stream that leaks, and it looks exactly like the ones that do not.

Do not reach for a finalizer or `Cleaner` as the primary mechanism. They run at an
unspecified time or never, and they exist to catch the case where the discipline already
failed.

Failure-mode check, for every resource: **which construct closes this if the next line
throws?** If the answer is a `close()` call further down the method, nothing does.

## A class is final, or its inheritance is designed and documented [R:design-for-inheritance-or-forbid-it]

Make the class `final` unless you have designed for a subclass, and if you have, say
exactly what a subclass may override and what it must not.

Inheritance is not the reuse mechanism it looks like. A subclass depends on which of the
superclass's methods call which others — `addAll` calling `add` is the canonical example
— and that is an implementation detail the superclass is entitled to change in a patch
release. So a working subclass breaks when the parent is refactored, without either author
doing anything wrong. Composition does not have this property: a wrapper depends on the
public contract and nothing else.

The constructor case is worse than fragile, it is broken from the start. A constructor
that calls an overridable method runs the subclass's override **before the subclass's own
fields are assigned**, so the override sees `null` and `0` in fields it declared `final`
and initialised. Nothing warns. This is the same publication hazard
`[R:no-reference-to-internals-escapes]` names as `this` escaping, reached by a route that
looks like ordinary method dispatch — and `[R:publish-safely-or-not-at-all]` is why it
matters even when there is only one thread.

So:

* **`final` by default** on classes, and on any method a subclass has no business
  replacing. Sealing (`[R:seal-the-alternatives]`) is the stronger form where the set of
  subtypes is closed and known.
* **Never call an overridable method from a constructor**, an initialiser, or `clone()` or
  `readObject()`, which are constructors wearing other names.
* **If a class is designed for inheritance**, document the self-use: which methods call
  which, which are safe to override, what a subclass must call. That documentation is part
  of the contract, and the cost of it is the honest reason most classes should be `final`
  instead.
* **Prefer composition.** A wrapper that holds the instance and forwards is longer to
  write, immune to the parent's internals, and the thing to reach for when the motivation
  was reuse rather than substitutability.

Failure-mode check, for every non-final class: **which of my methods call each other, and
would a subclass overriding one of them still be correct after I reorder them?** If that
cannot be answered, the class was extensible by accident.

## equals and hashCode are one decision, and a mutable key breaks both [R:equality-is-one-contract]

Decide equality once, implement both halves in the same change, and derive them from
fields that cannot change while the object is reachable from a collection.

The contract is mechanical: equal objects must have equal hash codes. Break it and a
`HashMap` looks in the wrong bucket, so `get` returns null for a key it contains and
`remove` cannot find an entry to remove -- a leak whose size grows with traffic. Nothing
detects this. The collection is behaving exactly as specified; the specification was
handed an object that lied.

The mutable-key half is the one that survives review, because the class is correct when
it is written and becomes wrong when someone adds a setter. An object whose `hashCode`
depends on a field that is later mutated has moved bucket without the map being told, and
is now unreachable through the very key it is filed under. **So a type used as a key is
immutable, or it is not used as a key.** A `record` gives you both halves generated from
the components and final fields by construction, which is why it is the default shape for
a value type and why hand-writing these two methods should feel like a decision rather
than boilerplate.

Two more that follow from the same contract and are worth stating because each has its
own way of going wrong:

* **`compareTo` must agree with `equals`** wherever the type reaches a `TreeMap` or a
  sorted set, which use ordering and never call `equals` at all. A comparator saying two
  objects are equivalent while `equals` says they differ produces a set that silently
  holds one of them.
* **Inheritance and equality do not compose.** An `equals` written with `instanceof`
  accepts a subclass and is then asymmetric; one written with `getClass()` is symmetric
  and rejects every subclass. There is no third option, which is one of the reasons
  `[R:design-for-inheritance-or-forbid-it]` prefers `final`.

Failure-mode check, before a type is used as a key or put in a set: **which fields does
its equality read, and can any of them change while it is in there?**

This is `[R:make-illegal-states-unrepresentable]` where the language will not help: Java
cannot stop you writing an inconsistent contract, so the remedy is to generate it
(`record`) rather than to write it carefully.

## Catch what you can answer; never catch Exception [R:exceptions-name-what-failed]

A `catch` clause is a claim that you can do something about what you caught. Catch the
type you can answer and let the rest go up.

`catch (Exception e)` claims you can answer everything, which is never true: it absorbs
the `IllegalStateException` from a bug three frames down and the
`InterruptedException` that was asking the thread to stop, and it treats both the same as
the `IOException` the author was thinking about. The narrower the caught type, the more
the code says. Multi-catch (`catch (IOException | TimeoutException e)`) is how you handle
two without widening to their common supertype.

Three refusals that carry most of the value:

* **Never swallow.** An empty catch block, or one that logs and continues, converts a
  failure into a wrong answer computed quickly. If there is genuinely nothing to do,
  rethrow; if the method cannot throw, that is the design problem to fix rather than to
  hide.
* **Never lose the cause.** `throw new ServiceException("failed")` discards the stack that
  says why. Wrapping is fine and often right — `throw new ServiceException("loading " + id, e)`
  — as long as the cause travels. A chain printed once at the edge is worth more than
  every layer restating what the layer beneath it already said.
* **Never catch `InterruptedException` without restoring the flag.** Either propagate it or
  `Thread.currentThread().interrupt()`; absorbing it silently is how a shutdown request
  disappears and a thread pool refuses to stop.

On checked exceptions, the choice is about the caller, not about taste: a condition the
caller can plausibly recover from is checked, and a programming error — a broken
precondition, an impossible state — is unchecked. The failure mode to avoid is declaring
`throws Exception`, which is checked in form and unanswerable in practice, and the one to
avoid next is making everything unchecked so the signature stops mentioning failure at
all. That is the Rust library's ban on erasing a typed error contract, arriving in a
language where the erasure is free and needs no crate. (Named in prose rather than by
tag: that rule has graduated to a hook, and citing a retired rule is a `lint` finding —
the same precedent `[R:doc-currency]` set.)

Exception types are types. One per condition a caller could treat differently, each
carrying the values that identify the instance — the id, the path, the elapsed time — so
one log line is enough to act on rather than the start of an investigation. A message
built by string concatenation with no fields is prose wearing a class name.

Failure-mode check, at every `catch`: **what will this clause do with a failure I have not
thought of?** If the answer is "the same thing", the caught type is too wide.

## Comparing boxed numbers with == works until the value exceeds 127 [R:identity-is-not-equality-for-boxes]

`==` on a reference asks whether these are the same object. On a boxed number or a string
that is almost never the question, and the language will not stop you asking it.

What makes this a rule rather than a known gotcha is *where* it is wrong. `Integer` caches
boxes for −128 to 127, so `==` returns the right answer for every small value and the
wrong one above it:

```java
Integer a = 127, b = 127;   a == b   // true
Integer a = 128, b = 128;   a == b   // false
```

A unit test written with an id of `1` or a quantity of `10` passes forever. Production
arrives with an order id of 5000 and the comparison silently starts returning false — and
it returns false *correctly*, in the sense that the two really are different objects, so
there is nothing to find in a debugger except two values that look identical.

String literals behave the same way for the same reason: literals are interned, so `==`
works for them and fails for any string that was built, read from a socket, parsed, or
concatenated at run time.

So:

* **Use `equals`** — or `Objects.equals(a, b)`, which also handles a null on either side
  and is the right default at a boundary where either may be absent
  (`[R:null-is-not-a-value]`).
* **Use primitives where the value cannot be absent.** `int` rather than `Integer` removes
  the question entirely, along with the allocation and the possible
  `NullPointerException` on unboxing. A boxed type in a field or a signature should be
  there because absence is meaningful, not by default.
* **Compare enums with `==`** — that one is correct, deliberate, and null-safe, which is
  part of why enums are the right shape for a closed set
  (`[R:seal-the-alternatives]`).
* **Never unbox in a comparison chain.** `Integer x = null; if (x == 1)` throws; the
  unboxing is invisible in the source.

Failure-mode check, at every `==` between two references: **is this asking about identity?**
If the answer is that it is asking whether the values are the same, it is the wrong
operator and it will pass the tests.

## A class that hands out its own mutable state has no invariant [R:no-reference-to-internals-escapes]

`private` is about who may *name* the field, not about who may change what it points at.
A getter returning the field hands out the field.

```java
private final List<Leg> legs;

public List<Leg> legs() { return legs; }               // callers can add to your order
public List<Leg> legs() { return List.copyOf(legs); }  // they cannot
```

Same access modifier, same `final`, opposite guarantees. `final` on a reference field
freezes the reference and says nothing about the object at the end of it, which is the
distinction that makes this worth a rule: a class can be entirely `private final` and
entirely mutable from outside.

Copy on the way in as well as on the way out. A constructor that stores the caller's list
has given the caller a handle to its own state, so the object can be mutated after
construction by code that has nothing to do with it — and the defect surfaces far from
both. `List.copyOf`, `Map.copyOf` and `Set.copyOf` do the copy and return something
unmodifiable in one step; `Collections.unmodifiableList` wraps without copying, so the
original still works as a back door and the wrapper is a view rather than a defence
(`[R:a-view-is-not-a-copy]`).

Two Java-specific escapes that are easy to miss:

* **A record's components are shallow.** A record with a `List` component is not immutable;
  it has a final reference to a mutable list, and the generated accessor returns it. Copy in
  the compact constructor, or the record is a value type in name only.
* **`this` escaping during construction.** Registering a listener, starting a thread, or
  calling an overridable method from a constructor publishes a half-built object — the
  fields assigned after that point are not yet visible, and a subclass's override runs
  before its own fields are initialised. The object exists before it is finished; do not let
  anyone else learn about it until it is.

Failure-mode check, for every method returning a reference and every constructor taking
one: **can the caller change what this points at, and would the class survive it?**

`[R:private-fields-only]` is the same argument in Rust and the two are deliberately
separate: satisfying that one is nearly automatic there and is only the first half here.

## Never return null to mean absent [R:null-is-not-a-value]

A method that can return nothing says so in its type. `null` does not say it; it only
does it.

The contract is the whole problem. `Order find(String id)` and `Order get(String id)` have
the same signature, and one of them returns `null` on a miss while the other throws --
the caller cannot tell which without reading the body, so it either checks everywhere or
checks nowhere, and both are wrong. `Optional<Order>` states the answer in the type: the
caller cannot reach the value without deciding what absence means.

Three shapes, and the right answer differs:

* **A method that may legitimately find nothing** returns `Optional<T>`. Use it at the
  return position and nowhere else -- an `Optional` field costs a second allocation and a
  second dereference on every read, and an `Optional` parameter forces every caller to wrap,
  which is three states (`null`, `empty`, `present`) where the signature promised two.
* **A method returning a collection** returns an empty one, never `null`. `Collections.emptyList()`
  allocates nothing and removes the check entirely; a null collection makes the caller write
  a guard before a loop that would have run zero times by itself.
* **A method that cannot meaningfully continue** throws. Absence and failure are different
  answers and must not share a return value.

Accepting `null` is a separate decision from returning it. At an API boundary, reject it
loudly -- `Objects.requireNonNull(x, "x")` in the constructor -- so the failure lands at
the boundary that was handed the bad value rather than at the first dereference three
layers in. That is `[R:parse-dont-validate]` in a language whose type system will not
carry the witness: the check happens once, at the perimeter, and the field is trusted
afterwards because nothing else can write it (`[R:private-fields-only]`).

Where the codebase tolerates nullable references, annotate them and turn the analysis on.
`@Nullable`/`@NonNull` with a checker in the build is the difference between a convention
and a control; unenforced annotations are comments that look like types
(`[R:guarantee-needs-a-reader]`).

Failure-mode check, for any reference-returning method: **what does the caller do on a
miss, and what in the signature told them?** If the answer is a Javadoc line, nothing told
them.

This is `[R:no-sentinel-values]` in the language where the sentinel is built into every
reference type, which is why it needs its own rule rather than an instance of that one:
you cannot stop `null` existing, only stop it meaning something.

## An object handed to another thread is published safely, or it arrives half-built [R:publish-safely-or-not-at-all]

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

## A closed set of alternatives is a sealed hierarchy, matched exhaustively [R:seal-the-alternatives]

When the set of cases is fixed, say so in the type and let the compiler find every place
that must decide.

```java
public sealed interface Order permits Limit, Market, Stop {}

// no default: adding a permitted type stops this compiling
String describe(Order o) {
    return switch (o) {
        case Limit l  -> "limit at " + l.price();
        case Market m -> "market for " + m.quantity();
        case Stop s   -> "stop at " + s.trigger();
    };
}
```

The value is entirely in what it refuses. An open interface can gain an implementation in
another module, so no switch over it can ever be complete and every one needs a `default`
— and a `default` is a decision taken in advance on behalf of a case that did not exist
yet, which is the definition of a decision nobody made. Sealing closes the set, and an
exhaustive `switch` over a sealed type is then checked: add a permitted subtype and every
incomplete switch **fails to compile**, which is the report you want and the one no test
gives you.

So the shape to reach for is a sealed interface whose permitted types are records: the
cases are closed, each carries exactly its own data, and none carries a field belonging to
another. The shape to stop writing is the one it replaces — a class with a `type` field
and eight nullable columns, seven of which are null for any given instance, where the
combinations that cannot occur are representable and guarded by convention.

Two disciplines follow:

* **Never write `default` over a sealed type**, and do not add one to silence a warning.
  It converts a compile error into a runtime branch, which is the whole guarantee,
  discarded for a line. Where a genuine catch-all is needed, name the remaining cases.
* **An `enum` is the degenerate case and gets the same treatment.** Java checks
  exhaustiveness on a switch expression over an enum with no default, so the same
  compile-time report is available and a `default` throws it away.

Failure-mode check, for any type with a set of alternatives: **what happens when somebody
adds a case tomorrow, and where does the compiler tell them what to update?** If nothing
tells them, the set was not closed.

`[R:make-illegal-states-unrepresentable]` is the general form and this is how Java spells
it; `[R:seal-closed-trait-sets]` is the same decision in Rust, where the mechanism is a
private supertrait rather than a `permits` clause.

## Implementing Serializable adds a constructor that checks nothing [R:serializable-is-a-second-constructor]

`implements Serializable` is not a marker. It is a second, invisible, public constructor
that takes a byte array and performs no validation.

Deserialization does not call your constructor. It allocates the object and writes the
fields directly from the stream, so every check the constructor makes — the non-null, the
range, the "these two fields must agree" — is skipped, and an attacker or a corrupted file
produces an instance the class's own author believes cannot exist. That is
`[R:private-fields-only]` and `[R:parse-dont-validate]` defeated by a language feature
rather than by anyone's code: the perimeter was built, and this walks around it.

Two consequences beyond the invariant:

* **The field layout becomes a published API.** Once instances are serialized anywhere
  durable, renaming or removing a private field is a compatibility break. A `serialVersionUID`
  controls only whether the break is detected, not whether it happened.
* **Deserializing untrusted data is remote code execution**, not a theoretical risk. The
  stream chooses which classes to instantiate, and a gadget chain assembled from whatever
  is on the classpath does the rest. The JDK's own filtering
  (`ObjectInputFilter`) exists because the mechanism cannot be made safe by being careful
  with it.

So the default is: **do not implement it.** Where an object must cross a process boundary
or reach a disk, use an explicit format with an explicit parser — JSON, a schema, a binary
codec you wrote — so the reconstruction runs through a constructor and the perimeter
holds. The serialization format is then a decision with a version, rather than a shadow of
the class's private fields.

Where it cannot be avoided, the obligations are real and none of them is optional: a
`readObject` that validates exactly what the constructor validates, `readResolve` for a
type that must be a singleton — an enum is `Serializable` correctly and for free, which is
one more reason to prefer one — and a declared `serialVersionUID` so the break is at least
visible. Treat every one of those as evidence that the type should not have been
serializable.

Failure-mode check, before adding the interface: **what does this class's constructor check,
and am I content for a byte stream to skip it?**

<!-- relearn:generated v0.1.0 sha256=5054178399ee50a993b8b8db594b22324f69cef86fdfa4e6695e84d2d864e4b1 rules=R:a-speculated-path-deoptimises-when-the-input-changes,R:a-view-is-not-a-copy,R:a-wrapper-type-is-not-free-here,R:close-what-you-open,R:design-for-inheritance-or-forbid-it,R:equality-is-one-contract,R:exceptions-name-what-failed,R:identity-is-not-equality-for-boxes,R:no-reference-to-internals-escapes,R:null-is-not-a-value,R:publish-safely-or-not-at-all,R:seal-the-alternatives,R:serializable-is-a-second-constructor -- DO NOT EDIT; regenerate with `relearn build` -->
