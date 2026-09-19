+++
tag = "R:seal-the-alternatives"
title = "A closed set of alternatives is a sealed hierarchy, matched exhaustively"
error_class = "Modelling a fixed set of cases as an open interface, a type field beside a bag of nullable columns, or an enum with a default branch -- so adding a case compiles everywhere, is handled nowhere, and takes whichever arm the wildcard happened to point at"
home = { kind = "domain", name = "java" }
created = "2026-09-19"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-19 with the rest of the Java set, and it is the language's version of a defect this repository has already paid for twice -- the 2026-09-16 sweep found nine decisions over four enums taken by `matches!` at a call site, five of which would have answered permissively for a variant nobody had added yet. Java's `default:` arm is the same hole with different syntax and, until sealed types, no alternative."
+++

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
