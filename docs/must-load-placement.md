# Must-load placement — a snapshot, 2026-10-31

**Analysis only.** Nothing here changes the rule format, a home, or an emitter. A snapshot
states its own numbers, and nothing else quotes them.

## The question

Some rules are expensive to miss. Either the error class has already recurred, or a
block-grade detector was drafted for it, which is a judgement that the class is worth
stopping at write time. In Claude Code, each of these rules reaches a session only as
its home's layer does. So a home that loads only on request, or only when a matching
file is read, can leave an expensive rule out of the very session that needed it.

For each such rule, this note asks which of four answers is right:

- keep;
- tighten the skill description, so the skill opens when it should;
- move the rule to another home;
- a rule-level load override.

The override is held to a stricter bar. It is recommended only where a decisions-log
entry shows that moving the rule is wrong, because an override is a second placement
and weighs against `[R:one-home-per-rule]`.

## Inputs

- **Expensive.** `relearn list --format tsv`: rules with `recurrences > 0`, plus any rule
  carrying a `[[detector]]` with `action = "block"`. Install grade (`yes` or `draft`)
  comes from the `Install` column of `hooks-pack/README.md`.
- **Load semantics.** `LoadSemantics::for_home` in `src/emit.rs`. `Project` is `Always`.
  `Domain` is `WhenReading` for a language with globs (`rust`, `java`) and `OnRequest`
  otherwise. **`Global` is `Always` in that function, but not on Claude Code.** The
  `claude-rules` target refuses `Global`, on the reasoning that it already has an
  always-resident home (`claude_rules::RulesLayer::admits`). The only place relearn
  emits it for Claude Code is the `global` skill, and a skill is `OnRequest`.
- **Load rate.** `relearn probe tally` over the 2026-10-31 pilot (Part 4): the share of
  judged WITH-arm sessions that opened the rule's own skill. Only rules with a probe
  have one. A `rules-layer` home has no Skill call to count, and `relearn loads` over an
  `InstructionsLoaded` log is now the instrument for it.
- **Budget.** `relearn budget` against this repository and the author's `~/.claude`
  measured 84,365 always-resident loader characters against a 120,000 budget, leaving
  35,635 of headroom. The candidate layers, measured the way the loader counts them
  (front matter and comments stripped) from a fresh `relearn build --targets claude`:
  `global` 59,716; `domain-low-latency` 49,734; `domain-disclosure` 10,874.

Making `global` or `domain-low-latency` always-resident would exceed the headroom on its
own, so moving either whole home is out. `domain-disclosure` would fit, but no home
type expresses "always-resident domain" without a format change, and that is out of
scope here.

## The table

| Rule | Home | Delivery on Claude Code | Why expensive | Pilot load rate | Recommendation |
|---|---|---|---|---|---|
| `R:no-silent-spend` | global | skill, OnRequest | recurred | 0/10 (`load-failure`) | **Tighten description** |
| `R:check-the-claim-you-inherit` | global | skill, OnRequest | recurred | 0/10 | **Tighten description** |
| `R:verify-through-production-path` | global | skill, OnRequest | recurred | 0/10, every session inconclusive | **Tighten description**; the probe task needs repair before its rate means anything |
| `R:doc-currency` | global | skill, OnRequest | recurred | 2/10 | **Tighten description** |
| `R:prefer-by-construction` | global | skill, OnRequest | recurred | 6/9; held 0/6 violated when loaded, 2/3 when not | **Keep**: loaded, the prose holds |
| `R:signal-needs-a-consequence` | global | skill, OnRequest | recurred | 8/10 | **Keep** |
| `R:repair-the-lying-artefact` | global | skill, OnRequest | recurred | 10/10 | **Keep** |
| `R:guarantee-needs-a-reader` | global | skill, OnRequest | recurred ×4 | no probe | **Keep**, partial with named controls; a probe is the next evidence |
| `R:verdict-survives-the-channel` | global | skill, OnRequest | recurred ×3 | no probe | **Keep**, partial; see the note on hand pins |
| `R:detector-excludes-own-definitions` | global | skill, OnRequest | recurred | no probe | **Keep**, partial; `relearn check` holds the mechanical half on every build |
| `R:case-collision` | global | skill, OnRequest | block detector, `yes` | no probe | **Keep**: graduated, held by an installed detector |
| `R:report-the-hit-not-the-match` | domain-disclosure | skill, OnRequest | recurred | no probe | **Tighten description** |
| `R:no-tool-attribution-in-public-artefacts` | domain-disclosure | skill, OnRequest | block detector, `draft` | no probe | **Keep**; the lever is installing the draft detector, which holds whatever loads |
| `R:repository-private-by-default` | domain-disclosure | skill, OnRequest | block detector, `draft` | no probe | **Keep**, same lever |
| `R:attack-the-design-in-a-second-pass` | domain-low-latency | skill, OnRequest | recurred | no probe | **Tighten description** |
| `R:no-unwrap-in-production` | domain-rust | rules file, WhenReading | block detectors, `yes` | n/a (rules layer) | **Keep** |
| `R:no-anyhow-in-libraries` | domain-rust | rules file, WhenReading | block detector, `yes` | n/a | **Keep** |
| `R:errors-name-what-failed` | domain-rust | rules file, WhenReading | block detector, `draft` | n/a | **Keep** |
| `R:no-let-underscore-on-a-local` | domain-rust | rules file, WhenReading | block detector, `draft` | n/a | **Keep** |
| `R:parse-dont-validate` | domain-rust | rules file, WhenReading | block detector, `draft` | n/a | **Keep** |
| `R:xplat-fixtures` | domain-rust | rules file, WhenReading | block detector, `draft` | n/a | **Keep** |
| `R:identity-is-not-equality-for-boxes` | domain-java | rules file, WhenReading | block detector, `draft` | n/a | **Keep** |
| `R:serializable-is-a-second-constructor` | domain-java | rules file, WhenReading | block detector, `draft` | n/a | **Keep** |

## Why each answer

**Tighten description, five global rules and two domain ones.** Each pilot WITH arm
had the skill installed and listed, and the open rate still ran from 0/10 to 10/10
depending on the task. That is the description deciding, not the rule. The descriptions
are generated as `Covers:` followed by every rule title in the home: 935 characters for
`global`, 612 for `domain-low-latency`. That lists topics, where a description should
name the moment the skill is needed. Tightening is an emitter change, not a format
change. The description is resident in every session, so a rewrite costs little
against the headroom, and the load log will show whether it worked.

**Keep, where the split already says so.** When `R:prefer-by-construction` loaded, it
held every time. When it did not load, it was violated twice in three sessions. That is
the prose working whenever it arrives. Its miss rate is a delivery question, and the
tightened description above covers it.

**Keep, where a detector holds the class.** A hook runs on every `Write` and `Edit`
whatever the session loaded, so a block-grade detector makes the rule's load
irrelevant to the expensive miss. For a `draft` detector the useful action is
installing it (`hooks-pack/README.md`, "Install"), not moving the prose.

**Keep, WhenReading domains.** `**/*.rs` attaches `domain-rust` on the same reads its
detectors watch the writes of. The remaining gap is a new file written without any
`.rs` file being read first, and that is exactly what the detectors catch. The
`InstructionsLoaded` half of the load log can now measure how often the layer
actually attaches.

**No move, and no override.** Moving a global rule to a project home narrows its
audience to one repository. Moving it to a domain home narrows it to one language.
Either loses the rule everywhere else, which costs more than the miss it prevents.
No decisions-log entry yet argues that a move is wrong *and* that the miss is expensive
enough to justify a second placement, so an override is not recommended.

## A note on hand pins

The author's `~/.claude/CLAUDE.md` already carries hand-written, always-resident text for
`R:no-silent-spend`, `R:report-the-hit-not-the-match`, `R:verdict-survives-the-channel`
and the verify-through-production-path practice. Their stated reason is that "the skill
is on-demand, this file is always loaded". That is a rule-level load override in
practice, made outside relearn, where neither `relearn budget` nor `[R:one-home-per-rule]`
can see it as such. It is also why the pilot's `R:no-silent-spend` `load-failure` does
not mean the rule is missing from the author's own sessions: the probe deliberately
excludes that layer. Whether to record those pins as a decision, or retire them once a
tightened description is measured to open the skill, is George's call. `TODO.md`
carries it.
