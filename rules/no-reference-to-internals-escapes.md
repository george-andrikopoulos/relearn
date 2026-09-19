+++
tag = "R:no-reference-to-internals-escapes"
title = "A class that hands out its own mutable state has no invariant"
error_class = "Returning or storing a reference to a mutable field -- an array, a collection, a date, a builder's backing list -- so a caller can change the object's state without going through the method that maintains it, and the invariant the constructor established is enforceable by nobody"
home = { kind = "domain", name = "java" }
created = "2026-09-19"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-19 with the rest of the Java set. In Rust the borrow checker makes most of this unrepresentable, which is why `[R:private-fields-only]` there needs only to say `no pub fields`; in Java a private field and a public getter returning it directly are the same escape with more ceremony, and the language will not object."
+++

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
