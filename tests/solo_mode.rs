//! Solo mode is the product: `relearn` compiles one person's rules to their own
//! assistants, on one machine, with nothing else present.
//!
//! **Why this file exists (2026-09-13).** `docs/federated-relearn.md` designs a
//! corpus that installs contribute to, cache from, and report recurrences into.
//! Every federated feature in that design is a reason for the tool to acquire an
//! account, a network client, a config file, an ambient state directory — and
//! the moment any of those becomes *required*, the thing a single engineer
//! downloads to emit their own rules to Copilot and Claude has stopped existing.
//! The design says federation is opt-in and that publication is a human opening
//! a pull request. That sentence is prose, and prose is the layer this
//! repository trusts least.
//!
//! **What this proves.** `relearn`'s own code opens no socket, spawns no
//! process, and consults no ambient state: its entire input is the path given on
//! the command line, and its entire output is the path given on the command
//! line. No network stack is in the dependency tree. A federated build that
//! keeps this green is one where the tool still never speaks to anyone — which
//! is exactly what the design asks for, since contribution and reporting are a
//! person running `git`, not a client calling a server.
//!
//! **What it cannot prove.** Not that emission is unchanged — `relearn verify`
//! holds that, and it is the artifact to run beside this one. Not that a
//! dependency lacks a socket internally; it proves only that nothing here calls
//! one, which is what matters, since an uncalled socket cannot open. And the
//! crate deny-list below cannot enumerate every networking crate that will ever
//! exist: it names the ones a Rust programmer actually reaches for, and the
//! other half is `tests/dependencies.rs`, which refuses any new dependency that
//! is not priced in the decisions log. A crate too obscure for the list still
//! has to be argued for in writing.
//!
//! **Comments are stripped before matching**, because this file's own subject
//! matter is the marker strings, and a doc comment in `src/` that *names*
//! `std::net` while explaining that we do not use it would otherwise turn the
//! gate permanently red — `[R:detector-excludes-own-definitions]`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Markers for network I/O. Reaching the network from the standard library
/// means one of these appears in the source.
const NETWORK_MARKERS: &[&str] = &[
    "std::net",
    "TcpStream",
    "TcpListener",
    "UdpSocket",
    "SocketAddr",
    "ToSocketAddrs",
];

/// Markers for spawning a child process. Shelling out to `git` or `curl` is the
/// back door to both the network and to ambient state, and it is how a tool that
/// "makes no network calls" ends up making network calls.
const PROCESS_MARKERS: &[&str] = &["std::process::Command", "Command::new"];

/// Markers for ambient state — anything read from the machine rather than from
/// the paths the caller named. A config file in the home directory is how a tool
/// starts behaving differently on two machines for reasons the command line does
/// not show.
/// Markers for ambient state — anything read from the machine rather than from
/// the paths and values the caller named.
///
/// **The clock is on this list, and that is a decision rather than an
/// oversight** (2026-09-13, B1). `adopt` records when a fork was taken, and
/// reading `SystemTime::now()` for it would have been the obvious
/// implementation: it is also ambient state, it makes the command
/// unreproducible, and it makes the output untestable without freezing time.
/// The date is passed on the command line instead, and this list is what keeps
/// that decision from quietly eroding the first time somebody finds the
/// argument inconvenient.
const AMBIENT_MARKERS: &[&str] = &[
    "home_dir",
    "dirs::",
    "XDG_",
    "env::var",
    "SystemTime::now",
    "Instant::now",
    "UNIX_EPOCH",
];

/// Networking, TLS and async-runtime crates. Not exhaustive, and the module doc
/// says so; `tests/dependencies.rs` is the half that catches the rest by
/// refusing an unpriced dependency.
const NETWORK_CRATES: &[&str] = &[
    "async-std",
    "attohttpc",
    "curl",
    "git2",
    "gix",
    "h2",
    "hickory-resolver",
    "hyper",
    "isahc",
    "mio",
    "native-tls",
    "octocrab",
    "openssl",
    "openssl-sys",
    "quinn",
    "reqwest",
    "rustls",
    "smol",
    "socket2",
    "surf",
    "tokio",
    "trust-dns-resolver",
    "tungstenite",
    "ureq",
];

/// The repository root, resolved from the manifest dir so the test does not
/// depend on the working directory it is run from.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Every `.rs` file under `src/` — the library and the binary, which is the code
/// that runs when someone types `relearn build`.
fn library_sources() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect_sources(&repo_root().join("src"), &mut found);
    found.sort();
    assert!(
        !found.is_empty(),
        "no sources found under src/ — the gate would pass by measuring nothing"
    );
    found
}

fn collect_sources(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_sources(&path, found);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            found.push(path);
        }
    }
}

/// Drop everything from `//` to end of line. Crude — it also truncates a `//`
/// inside a string literal, such as a URL — and that is acceptable here, because
/// the result is only ever searched for the markers above and a truncated URL
/// cannot hide one.
fn without_comments(text: &str) -> String {
    text.lines()
        .map(|line| match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every `(file, line, marker)` hit for a marker set, so a failure names the
/// site rather than only the count.
fn hits(markers: &[&str]) -> Vec<String> {
    let root = repo_root();
    let mut found = Vec::new();
    for path in library_sources() {
        let text = fs::read_to_string(&path).expect("a readable source file");
        let code = without_comments(&text);
        for (index, line) in code.lines().enumerate() {
            for marker in markers {
                if line.contains(marker) {
                    let shown = path.strip_prefix(&root).unwrap_or(&path);
                    found.push(format!("{}:{}: {marker}", shown.display(), index + 1));
                }
            }
        }
    }
    found
}

#[test]
fn the_tool_opens_no_socket() {
    let found = hits(NETWORK_MARKERS);
    assert!(
        found.is_empty(),
        "relearn performs no network I/O: contribution and reporting are a person \
         running `git`, never a client calling a server. Found:\n  {}",
        found.join("\n  ")
    );
}

#[test]
fn the_tool_spawns_no_process() {
    let found = hits(PROCESS_MARKERS);
    assert!(
        found.is_empty(),
        "relearn spawns no child process: shelling out is the back door to the \
         network and to ambient state. Found:\n  {}",
        found.join("\n  ")
    );
}

#[test]
fn the_tool_reads_no_ambient_state() {
    let found = hits(AMBIENT_MARKERS);
    assert!(
        found.is_empty(),
        "relearn's input is the path it was given and nothing else: no home \
         directory, no config file, no environment. A rule library is compiled \
         the same way on every machine. Found:\n  {}",
        found.join("\n  ")
    );
}

#[test]
fn no_network_crate_is_in_the_dependency_tree() {
    let lock = fs::read_to_string(repo_root().join("Cargo.lock"))
        .expect("a Cargo.lock — the gate must not pass by failing to read it");
    let present: BTreeSet<String> = lock
        .lines()
        .filter_map(|line| line.strip_prefix("name = "))
        .map(|name| name.trim().trim_matches('"').to_owned())
        .collect();
    assert!(
        !present.is_empty(),
        "no crate names parsed out of Cargo.lock — the gate would pass by measuring nothing"
    );

    let banned: Vec<&str> = NETWORK_CRATES
        .iter()
        .copied()
        .filter(|crate_name| present.contains(*crate_name))
        .collect();
    assert!(
        banned.is_empty(),
        "a networking, TLS or async-runtime crate reached the dependency tree: {}. \
         Federation is git pull requests moved by a human; if the tool itself ever \
         needs a client, it belongs behind an optional feature that solo mode never \
         compiles — and this list needs the argument, not the exemption.",
        banned.join(", ")
    );
}
