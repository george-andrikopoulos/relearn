---
name: project-relearn
description: "Engineering discipline for the relearn repository. Covers: A generator never overwrites content it did not generate; Order by an explicit rank, not an incidental string sort"
---

# project: relearn rules

## A generator never overwrites content it did not generate [R:generate-guards-unversioned]

A generator that writes into a directory shared with hand-authored files must never overwrite a file it did not itself generate. Stamp every generated file with a marker the generator can recognise on the next run, refuse to overwrite any target lacking it, and make the check all-or-nothing: abort the whole write before touching disk if any target is unversioned, rather than leave a half-generated tree. A dropped rule is a lost correction; a clobbered human file is a lost correction the tool itself destroyed.

## Order by an explicit rank, not an incidental string sort [R:order-by-explicit-rank]

When output has a meaningful order, derive it from an explicit rank that states the intent (general to specific, most to least severe), not from an incidental lexical sort of some string field. An alphabetical accident is not an ordering decision, and it changes silently the day the underlying strings change.

<!-- relearn:generated v0.1.0 sha256=56ef6e6ddc26d51977410f250a436340369761a09b6ff4877c91bd99b358ca2d rules=R:generate-guards-unversioned,R:order-by-explicit-rank -- DO NOT EDIT; regenerate with `relearn build` -->
