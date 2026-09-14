+++
tag = "R:measure-cost-per-task"
title = "Measure cost-per-completed-task; never choose by price tier"
error_class = "Selecting a mechanism or model by its reputation or price tier rather than its measured cost to complete the task"
home = { kind = "global" }
created = "2026-07-21"
origin = "mined"
status = { kind = "active" }
incident = "Measured 2026-07-21: on George's workload Fable ran 2-3x the tokens per task versus Opus 4.8, making Opus both cheaper per completed task and higher quality -- so the mechanical rust-* agents carry no cheaper-tier override. A model's price tier (Haiku is the cheap one) is not its cost per completed task; a verbose model that triples tokens or needs a retry is dearer."
published_incident = "A model was chosen by its price tier rather than by what it consumed on the work. Measured against the task actually being run, the cheaper-per-token model used two to three times the tokens to reach the same result, making the more expensive one both cheaper per completed task and better — so the assignment that looked like a saving was a loss in both directions. A verbose model that triples token use, or fails and needs a retry, is dearer than a flagship that lands it once."
+++

Do not pick a mechanism or model by reputation or sticker price. State what it actually costs to complete the task -- tokens consumed times price, including retries -- and choose on that measured cost. After choosing, run one failure-mode check: under what configuration does this cause the exact harm it was chosen to prevent? Then bound that configuration.
