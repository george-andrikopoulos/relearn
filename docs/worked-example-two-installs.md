# Worked example: one rule, two installs, a shared directory

The [single-install walk-through](worked-example.md) follows one correction from an incident
to five vendor instruction files. This one follows the same rule **between two engineers**:
published by one, taken by the other, revised, refreshed, and finally forked when they
disagree. Every block below is **actual command output**, not a mock-up — see
[Reproducing this](#reproducing-this).

Nothing here is required. A single engineer needs none of it, and `tests/solo_mode.rs` fails
the build on a socket, a spawned process, an ambient read or a networking crate anywhere in
the tree — so none of it can quietly become a requirement for the solo path, which is the
product.

What the two installs share is a **directory both can see**: `<drive>` below is a common
drive, a mounted share, a git checkout. There is no server, no account, and no sync.

## 1. Alice publishes

Alice holds `R:no-sentinel-values` and wants to share it. She cannot simply hand over the
file: its `incident` is a verbatim quotation from a private session, naming a colleague, a
date and an internal tool. What travels instead is `published_incident`, a separate field she
writes by hand — the tool derives nothing, because a scrubber leaks what it does not
recognise and destroys context it does not understand.

```sh
relearn contribute --rules alice/rules --tag R:no-sentinel-values \
    --terms ~/terms.sha256 --version 1 --out <drive>/rules --confirm
```

```
written to <drive>/rules/no-sentinel-values.md — nothing has been transmitted;
publishing it is a pull request you open yourself
```

What lands on the drive:

```toml
+++
tag = "R:no-sentinel-values"
title = "No sentinel values: absent states are enum variants"
error_class = "Encoding a distinct state as a magic value of an existing type (0, -1, \"\", T::zero()) that downstream logic must remember to special-case"
home = { kind = "global" }
created = "2026-07-16"
origin = "mined"
status = { kind = "active" }
authority = { kind = "local", version = 1 }
incident = "A distinct state -- a service being stopped -- was encoded as the zero value of an existing numeric type. A downstream gate compared that zero against a minimum and read it as too young, suppressing forever the exact recovery the tool existed to perform. Every per-task review passed it."
+++
```

Two things to notice. The **published** incident stands where `incident` goes, and the
quotation Alice's own file carries is nowhere in it — the projection that renders this never
borrows the raw field, so no renderer can leak it, forget to strip it, or strip it wrongly.
And `authority` records **which revision this is**. That number is required, and it is the
thing the rest of this example turns on.

## 2. Bob checks what the shared corpus would change

Before starting work, Bob asks what is there. Without `--confirm` this writes nothing: it is
a report.

```sh
relearn pull --rules bob/rules --upstream <drive> --all --from shared --on 2026-09-13
```

```
take     R:no-sentinel-values
keep     R:doc-currency (yours, not a cache)

1 to take, 0 to refresh, 0 to drop, 1 left alone.
Nothing is removed without --prune.
Nothing written. Re-run with --confirm to apply this.
```

The `keep` line is the half that matters. Bob's own `R:doc-currency` is named and accounted
for, with the reason — a bulk command that listed only what it touched would leave him to
diff two corpora by hand, which is what he ran it to avoid.

## 3. Bob takes it

```sh
relearn pull --rules bob/rules --upstream <drive> --all --from shared --on 2026-09-13 --confirm
```

```
wrote   bob/rules/no-sentinel-values.md
```

```toml
authority = { kind = "cached", from = "shared", version = 1, pulled = "2026-09-13" }
```

The rule is now a **cache**: what it is a copy of, at which revision, and when. It keeps the
home it arrived with — `global` — which is the whole point of taking it.

## 4. The cache compiles exactly like Bob's own rule

```sh
relearn build --rules bob/rules --out bob/out --targets claude
```

```
wrote 1 file(s) under bob/out
```

```
bob/out/skills/global/SKILL.md
  ## Update the doc in the same change as the thing it describes [R:doc-currency]
  ## No sentinel values: absent states are enum variants [R:no-sentinel-values]
```

One skill, two rules, no distinction between the one Bob wrote and the one he took. `Home`
decides which layer a rule lands in; `Authority` decides who maintains it. They are
independent, and that independence is what makes caching worth doing.

What Bob may *not* do is edit the cache. The write path refuses it — an edited cache is a
silent fork, with nothing recording that the copy and its source have diverged.

## 5. Alice revises, and cannot go backwards

Alice improves the title and publishes revision 2.

```sh
relearn contribute ... --version 2 --out <drive>/rules --confirm     # written
relearn contribute ... --version 1 --out <drive>/rules --confirm     # refused:
```

```
error: revision 1 does not supersede the 2 already published: a republication must go
forwards, or every cache of this rule reads as newer than upstream and nothing notices,
because staleness is decided by comparing these two numbers
```

That refusal is not fussiness. Bob's cache says `version = 1`; staleness is decided by
comparing it against what the drive publishes. Republishing at or below 1 would make his
copy read as *current* forever, and the check that would have caught it is exactly the
comparison that had been corrupted.

## 6. Bob's lint tells him

He does not have to remember to look. `lint` already runs on every change, and with
`--upstream` it carries the federation's news:

```sh
relearn lint --rules bob/rules --upstream <drive>
```

```
poke [broadcast]: your cache of R:no-sentinel-values is at revision 1 and upstream
publishes 2 — pull it, or `adopt` it if you have reasons to stay where you are
Pokes are news, not findings: none of them changed the exit code above.
```

A poke has no severity and cannot fail a run. A signal from other people that could break
Bob's build would have made the sharing mandatory, which is the one thing it must never be.

## 7. Bob refreshes

```sh
relearn pull --rules bob/rules --upstream <drive> --all --from shared --on 2026-09-14
```

```
refresh  R:no-sentinel-values → revision 2
keep     R:doc-currency (yours, not a cache)

0 to take, 1 to refresh, 0 to drop, 1 left alone.
```

With `--confirm`:

```toml
authority = { kind = "cached", from = "shared", version = 2, pulled = "2026-09-14" }
```

## 8. Bob disagrees, and forks deliberately

Suppose Bob has evidence Alice does not, and wants his own wording. He does not edit the
cache — he adopts it.

```sh
relearn adopt --rules bob/rules --tag R:no-sentinel-values --on 2026-09-15
```

```
adopted R:no-sentinel-values — a local fork now, recorded in bob/rules/no-sentinel-values.md
```

```toml
authority = { kind = "adopted", from = "shared", version = 2, pulled = "2026-09-14", adopted = "2026-09-15" }
```

The fork **remembers what it came from**: the source, the revision it was forked at, when
that copy was pulled, and when the fork was taken. A cache that simply "became local" would
be indistinguishable from a rule Bob wrote himself.

And every later pull leaves it alone:

```
keep     R:no-sentinel-values (a fork you took deliberately)
keep     R:doc-currency (yours, not a cache)
0 to take, 0 to refresh, 0 to drop, 2 left alone.
```

If Bob later wants his change to be everyone's, he contributes the fork — which is a change
he made, and is exactly what the design tells him to do instead of editing a cache.

## What the shape guarantees

Three refusals appear above, and each is the point rather than an inconvenience:

- **A home that never leaves a machine never arrives on one.** A project home names a
  filesystem path and an org layer is an organisation's own — withheld in both directions,
  by one exhaustive match that cannot be added to without deciding the question.
- **A republication goes forwards**, or the staleness signal silently inverts for everyone
  holding a copy.
- **Only a cache may be deleted.** `pull --prune` drops copies that are gone or retired
  upstream, and nothing else: a cache is regenerable, where a rule you own and a fork you
  took are source that nothing restores.

And one absence: **recurrence counts travel separately, and anonymously.** `relearn report`
publishes upstream tags, bucketed counts and month-level dates — no title, no incident, no
body, no path, no name — because recording that a rule of yours failed is something nobody
does with their name on it. `relearn aggregate` publishes nothing below a five-install floor
and prints its own confounds beside every number.

## Reproducing this

Every block above is real output. From a checkout, with `D` a scratch directory:

```sh
mkdir -p "$D"/{drive/rules,alice/rules,bob/rules}

# Alice's copy of the rule, with a published incident authored for it. (No rule
# in this repository carries one: publishing is a deliberate act of writing the
# account that may travel, and the raw quotation never does.)
cp rules/no-sentinel-values.md "$D/alice/rules/"
$EDITOR "$D/alice/rules/no-sentinel-values.md"   # add a published_incident = "..." line

# A term list: required, because a scrub that can run disarmed is not a scrub.
printf 'salt = abcdef\n7 %s\n' "$(printf 'f%.0s' {1..64})" > "$D/terms"

# Bob starts with one rule of his own, so the plan has something to leave alone.
cp rules/doc-currency.md "$D/bob/rules/"

relearn contribute --rules "$D/alice/rules" --tag R:no-sentinel-values \
    --terms "$D/terms" --version 1 --out "$D/drive/rules" --confirm
relearn pull --rules "$D/bob/rules" --upstream "$D/drive" --all --from shared --on 2026-09-13
relearn pull --rules "$D/bob/rules" --upstream "$D/drive" --all --from shared --on 2026-09-13 --confirm
relearn build --rules "$D/bob/rules" --out "$D/bob/out" --targets claude

# Alice revises and republishes; the backwards attempt is refused.
relearn contribute --rules "$D/alice/rules" --tag R:no-sentinel-values \
    --terms "$D/terms" --version 2 --out "$D/drive/rules" --confirm
relearn contribute --rules "$D/alice/rules" --tag R:no-sentinel-values \
    --terms "$D/terms" --version 1 --out "$D/drive/rules" --confirm   # refused

relearn lint --rules "$D/bob/rules" --upstream "$D/drive"
relearn pull --rules "$D/bob/rules" --upstream "$D/drive" --all --from shared --on 2026-09-14 --confirm
relearn adopt --rules "$D/bob/rules" --tag R:no-sentinel-values --on 2026-09-15
relearn pull --rules "$D/bob/rules" --upstream "$D/drive" --all --from shared --on 2026-09-16
```

Paths in the output above are shortened for reading; everything else is verbatim.
