+++
tag = "R:errors-name-what-failed"
title = "Error enums carry the values that identify the failure"
error_class = "An error modelled as a string, or as a variant with no fields, so the caller cannot match on what went wrong and the diagnostic omits the identifiers -- path, address, key, elapsed time -- needed to act on it"
home = { kind = "domain", name = "rust" }
created = "2026-08-24"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-24 from Pattern 7 of ~/.claude/CLAUDE.md, which states it in prose. Surfaced when the emitted Copilot artefact was audited against that pattern list and ten practices were found to have no rule in this library -- so they reached Claude through the always-on boot index and reached no other assistant at all. Its sibling, the ban on anyhow in library return types, is named here in prose rather than cited by tag on purpose: that rule has graduated to a hook, and a citation of a retired rule is a lint finding."
published_incident = "Codified from standing practice rather than mined from a failure. It surfaced in an audit of an emitted instruction artefact against the hand-written pattern list it was supposed to replace: ten practices were stated in that one hand-maintained layer and had no rule in the library, so they reached a single assistant and travelled to no other tool — which is the failure the library exists to prevent, arriving in the library's own contents. The practice: each distinct failure is a variant carrying the values that identify the instance, so a caller can match on it rather than parsing prose that changes the next time somebody edits a message, and one log line names the actual address or path instead of starting an investigation."

[[detector]]
name = "no-string-error"
engine = "line-regex"
pattern = 'Result<.*,[[:space:]]*(String|Box<dyn [a-z:]*Error[^>]*>)>[[:space:]]*(\{|;|=|where|$)|\b(Other|Unknown|Generic|Misc)\(String\)|map_err\(\|[a-z_]+\|[[:space:]]*[a-z_]+\.to_string\(\)\)'
comment = "//"
allow = "allow:string-error"
files = ["**/*.rs"]
exclude = ["**/main.rs", "**/*build.rs", "**/tests/**", "**/benches/**", "**/examples/**", "**/tests.rs", "**/*_test.rs", "**/*_tests.rs"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "This error is a string: a caller cannot match on it, and the message carries no value that identifies the failure. Return a thiserror enum with one variant per condition a caller could treat differently, each carrying the path, address, key or duration that names the instance."

[[detector.fixture]]
path = "src/config.rs"
input = 'pub fn load(path: &Path) -> Result<Config, String> {'
expect = "hit"

[[detector.fixture]]
path = "src/config.rs"
input = 'pub type Res<T> = Result<T, Box<dyn std::error::Error>>;'
expect = "hit"

[[detector.fixture]]
path = "src/error.rs"
input = '    Other(String),'
expect = "hit"

[[detector.fixture]]
path = "src/config.rs"
input = '    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;'
expect = "hit"

[[detector.fixture]]
path = "src/config.rs"
input = 'pub fn load(path: &Path) -> Result<Config, LoadError> {'
expect = "clean"

[[detector.fixture]]
path = "src/config.rs"
input = 'pub fn names(&self) -> Result<HashMap<u32, String>, LoadError> {'
expect = "clean"

[[detector.fixture]]
path = "src/config.rs"
input = 'pub fn name(&self) -> Result<String, LoadError> {'
expect = "clean"

[[detector.fixture]]
path = "src/main.rs"
input = 'fn main() -> Result<(), String> {'
expect = "clean"
+++

Every distinct failure is a variant of a `thiserror` enum, and every variant carries the values that identify the instance:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("TCP connect to {addr} failed")]
    Tcp { addr: SocketAddr, #[source] source: io::Error },
    #[error("timed out after {elapsed:?}")]
    Timeout { elapsed: Duration },
}
```

Two things follow that a string cannot give. The caller can `match` -- retry a `Timeout`, surface a `Tcp` -- rather than parsing prose that changes the next time someone edits a message. And the message names the actual address, so one log line is enough to act on rather than the start of an investigation.

Keep the cause in `#[source]` instead of interpolating it into the text. The chain then prints once, at the edge, without each layer restating the layer beneath it.

One variant per condition a caller could plausibly treat differently. Collapsing four causes into `Other(String)` re-creates the string error inside an enum: it reads as a type and behaves as prose, and it is the shape this rule exists to catch.
