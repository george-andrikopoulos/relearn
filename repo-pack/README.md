# repo-pack — the corpus's repository gates as scripts

Some rules can only be checked against a **whole repository**, not a single write: whether a
script's line ending is pinned in `.gitattributes`, which documents sit at the root, whether
a lint is switched on anywhere in the manifest. A rule that carries a gate `[[detector]]`
(event `repository-gate`) is compiled by `relearn` into a bash script here. A repository's
own gate (a `pre-push` hook, a `check.sh` stage, a CI job) calls it.

```
repo-pack/
└── gates/
    ├── <name>.sh        one per gate
    └── test-gates.sh    every fixture through the scripts above
```

| Script | Rule | Engine | Action | Install |
|---|---|---|---|---|
| `executable-text-eol-pinned.sh` | `[R:pin-eol-for-executable-text]` | eol-attribute | block | draft |
| `five-root-documents.sh` | `[R:five-files-no-more]` | root-documents | warn | draft |
| `pass-by-value-lint-on.sh` | `[R:borrow-in-signatures]` | lint-enabled | block | draft |

That table is hand-written. `tests/repo_gates.rs` reads it back against the scripts. It
fails when a script is missing from the table, a row names a script that does not exist,
a row's rule is not the one the script was generated for, or an `Install` cell disagrees
with the rule's status.

**A `draft` row is not for installing yet.** Its rule is still `active`. The gate's
fixtures pass, but whether it covers the rule is the graduation review's call, and the
fixtures cannot answer that. A row reads `yes` once the rule names `gate:<name>` in its
status.

## Running a gate

```bash
bash gates/<name>.sh <repository root>   # 0 clean, 1 finding, 2 could not check
bash gates/<name>.sh --judge < records   # the judge alone, records on stdin
```

Exit 2 is **not a pass**. It means the root is not a git checkout, `git` is missing, the
collect stage failed, or the judge met a record it could not read. A `warn` gate prints
its findings and exits 0. A `block` gate exits 1.

Each script has two stages. **Collect** asks `git` about the checkout: `ls-files`,
`check-attr`, and the files themselves. It prints one tab-separated record per fact. **Judge** is `awk`
over those records. `relearn check` runs every fixture through the same judge, written in
Rust, when the rule parses. `test-gates.sh` runs them through the generated `awk`, so the
two cannot drift apart unseen.

`eol-attribute` reads only the repository's own attributes. It switches off the machine's
system and global git config and its per-user attributes file, because a pin that lives on one
machine is not a pin. `.git/info/attributes` is per clone and is still read. This needs
git 2.32 or later.

## Install

relearn installs nothing. Copy `gates/<name>.sh` for each `yes` row into the consuming
repository, keeping the file and its generated marker intact, and call it from that
repository's gate. `test-gates.sh` is not a gate and is never copied.

## Requirements

`bash`, `git`, `awk`, `grep`, `sed`, `tr`, `xargs`, `mktemp`. No `jq`, no network. Exercised
on Windows (Git Bash). Linux has not been run yet, and `TODO.md` carries that.

## Never edit a script

Every script ends in a `# <!-- relearn:generated ... -->` marker holding a hash of its body.
`relearn verify --targets repo --out repo-pack` reports a hand edit. Change the rule file,
then rebuild:

```bash
relearn build --targets repo --out repo-pack
bash repo-pack/gates/test-gates.sh --trees tests/gates
```

`--trees` builds a throwaway checkout per gate from `tests/gates/<name>/hit` and `clean`,
then runs the whole script, collect stage included. A dotfile is stored there as
`dot-<name>`, so this repository never reads a fixture's `.gitattributes` as its own.
