+++
tag = "R:order-by-explicit-rank"
title = "Order by an explicit rank, not an incidental string sort"
error_class = "Deriving a semantic ordering from an incidental lexical sort (e.g. sorting home layers by their slug string), so the order is an alphabetical accident rather than a stated intent"
home = { kind = "project", path = "relearn" }
created = "2026-08-13"
origin = "mined"
status = { kind = "active" }
incident = "relearn (2026-08-13): the copilot and AGENTS.md emitters ordered rules by home slug string, which sorts domain-rust before global ('d' < 'g') -- putting the broadest layer in the middle by accident and leaving two ordering tests asserting opposite orders. Caught in integration review; fixed with an explicit home_rank (global -> domain -> project)."
+++

When output has a meaningful order, derive it from an explicit rank that states the intent (general to specific, most to least severe), not from an incidental lexical sort of some string field. An alphabetical accident is not an ordering decision, and it changes silently the day the underlying strings change.
