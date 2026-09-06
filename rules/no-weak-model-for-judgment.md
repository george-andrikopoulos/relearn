+++
tag = "R:no-weak-model-for-judgment"
title = "Never route judgment work to a weak model, and never embed a sub-tier local LLM"
error_class = "Wiring a meaningfully less capable model into a tool for work that needs judgment, on convenience or API-key-free grounds, so the tool is degraded wherever that model runs"
home = { kind = "global" }
created = "2026-08-22"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated: folded into ~/.claude/skills/rust-typedd/SKILL.md on 2026-07-22 from a standalone memory (the skill's own revision note records the fold and the removal of the atticked copy), then ported here 2026-08-22. Distinct from R:measure-cost-per-task, which it cites: that rule governs how to choose between capable models by measured cost, whereas this one is a capability floor that holds even when the cheaper option is genuinely cheaper per task."
+++

Work that needs judgment goes to a capable model. Do not embed a sub-tier local LLM (a 7B/13B behind Ollama, llama.cpp or similar) in a tool as a convenient, API-key-free fallback: a meaningfully dumber model degrades the tool it is wedged into, everywhere and silently, and the output looks like ordinary tool output rather than like a downgrade. When a tool needs intelligence, delegate to the capable model through the existing subscription -- the MCP server is the abstraction boundary and clients are peers. Note that the cost argument usually offered for the local model is the price-tier fallacy R:measure-cost-per-task names; but this rule is not an economic one and does not dissolve if the sums come out favourably. Mechanical, tool-restricted passes are a different matter and may be scoped tightly; the floor applies to work where the answer is a judgement.
