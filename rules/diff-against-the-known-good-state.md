+++
tag = "R:diff-against-the-known-good-state"
title = "When something used to work, test what changed since then first"
error_class = "A regression diagnosed by tuning parts that did not change since the known-good state, while the actual difference goes untested; each half-helping tweak looks like progress and lengthens the search"
home = { kind = "global" }
created = "2026-10-13"
origin = "mined"
status = { kind = "active" }
incident = "Home Assistant heating, 2026-10-11 to 2026-10-13. The combi short-cycled (~23 starts/h of ~22 s burns). Its owner said early on that under the previous control (an Ideal Halo thermostat, no TRVs) it never short-cycled. Over two days Claude tuned the side that had not been shown to differ: raised the minimum flow temperature, set a 30 % modulation cap, attempted a 60 °C trial, switched the controller to a new anti-cycling algorithm, and ruled out pressure and the bypass. The owner then pointed at the valves. Setting all six TRVs fully open — recreating the known-good state — produced one continuous 21-minute burn at minimum modulation within 20 minutes of trying. The difference between the working and the broken configuration had been named on day one and was the last thing tested."
+++

When something used to work, find what changed since then and test that first.

A regression has a known-good state behind it, and that state is the most valuable fact
in the diagnosis: everything that did not change between it and now is, for this
symptom, already exonerated. Tuning the parts that stayed the same spends effort where
the evidence says the fault is not, and each plausible tweak that half-helps makes the
search longer, because it looks like progress.

So before adjusting anything, write down the differences between the working
configuration and the failing one — hardware, settings, versions, controllers, people,
weather — and rank them by how cheaply each can be neutralised. Then run the cheapest
experiment that recreates the known-good state for one difference at a time: revert it,
bypass it, or force it to the old behaviour. One such experiment can retire a whole
family of theories that tuning would only ever nibble at.

If the owner volunteers a known-good state ("it worked with the old thermostat", "it was
fine before the upgrade"), treat it as the starting map, not as background.

Failure-mode check: **what is different from when it worked, and have I tested
neutralising that difference?** If the honest answer is "I have been adjusting things
that were the same then", stop and run the diff experiment first.
