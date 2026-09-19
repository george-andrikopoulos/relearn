---
name: project-relearn
description: "Engineering discipline for the relearn repository. Covers: generate guards unversioned; order by explicit rank"
---

# project: relearn rules

## A generator never overwrites content it did not generate [R:generate-guards-unversioned]

A generator that writes into a directory shared with hand-authored files must never overwrite a file it did not itself generate. Stamp every generated file with a marker the generator can recognise on the next run, refuse to overwrite any target lacking it, and make the check all-or-nothing: abort the whole write before touching disk if any target is unversioned, rather than leave a half-generated tree. A dropped rule is a lost correction; a clobbered human file is a lost correction the tool itself destroyed.

## Order by an explicit rank, not an incidental string sort [R:order-by-explicit-rank]

When output has a meaningful order, derive it from an explicit rank that states the intent (general to specific, most to least severe), not from an incidental lexical sort of some string field. An alphabetical accident is not an ordering decision, and it changes silently the day the underlying strings change.

<!-- relearn:generated v0.1.0 sha256=1063d860e854de705f0af2062126e8f1cc0ee1b9fb04b49ed06e084fc243c7c7 rules=R:generate-guards-unversioned,R:order-by-explicit-rank -- DO NOT EDIT; regenerate with `relearn build` -->
