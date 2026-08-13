//! `emit::claude::emit` requires `&Library<Validated>`. Passing a freshly
//! constructed `Library<Unvalidated>` must NOT compile: validation is the only
//! path to emission, and the typestate makes "emit an unchecked library" a
//! compile error rather than a runtime guard.
//!
//! Every emitter shares this signature, so proving it for one proves the gate.

use relearn::emit;
use relearn::library::Library;

fn main() {
    let unvalidated = Library::new(); // Library<Unvalidated>
    let _ = emit::claude::emit(&unvalidated);
}
