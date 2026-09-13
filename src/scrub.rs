//! `scrub` — the banned-terms matcher, over one authored paragraph.
//!
//! **Why this exists in Rust at all.** `scripts/no-banned-names.sh` holds the
//! same matcher and scans this repository's *files*; a contribution is one
//! string, so the file-walking half does not transfer and the matching half
//! does (§12.2 of `docs/federated-relearn.md` says exactly that). Sharing the
//! implementation would mean spawning the script, and `tests/solo_mode.rs`
//! forbids spawning processes — for good reasons that this phase is not the
//! place to relitigate. So there are two implementations of one matcher, and
//! **the term-list format is the contract between them**, stated here and in
//! the script's header. That duplication is recorded in the decisions log
//! rather than pretended away.
//!
//! Two properties carried over from the script, both load-bearing:
//!
//! * **A finding never names what it found** (`[R:report-the-hit-not-the-match]`).
//!   [`Hit`] carries a location and a length. Printing the matched text would
//!   move the exposure into a terminal, a CI log or a session transcript rather
//!   than closing it — which is the failure this whole gate exists to prevent.
//! * **Disarmed is never clean.** The script exits 2 when it has no list. Here
//!   [`TermList::parse`] is the only constructor, it returns a `Result`, and
//!   callers take a `&TermList` — so "ran with no list" is not a case to
//!   remember but a value that cannot be built.
//!
//! **What it cannot do.** It finds the names somebody wrote down. A published
//! incident can identify a person, a customer or a repository with none of
//! them, and no matcher reads for that. §12.2 puts the scrub on the
//! contributor, and this is the mechanical half of it, not a replacement.

use std::fmt;

use sha2::{Digest, Sha256};

/// Why a term list could not arm the matcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ScrubError {
    /// The list carried no salt, or no terms. Refused rather than read as an
    /// empty list: a matcher that reports clean because it has nothing to match
    /// is the disarmed gate reporting a pass.
    #[error(
        "the term list has no salt or no terms — a disarmed matcher is not a clean one, \
         so this is a refusal rather than an empty result"
    )]
    Disarmed,
    /// The salt line was present but was not hexadecimal.
    #[error("the term list's salt is not hexadecimal")]
    MalformedSalt,
}

/// One banned term, as the list stores it: the length of its normalised form
/// and the salted digest of that form. The term itself is not here, and cannot
/// be — that is the point of the format.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Digested {
    length: usize,
    digest: String,
}

/// A term list: the salt, and the digests of every protected name.
///
/// Constructible only by [`TermList::parse`], which refuses a list that cannot
/// match anything. Holding one *is* the proof that the matcher is armed.
#[derive(Debug, Clone)]
pub struct TermList {
    salt: Vec<u8>,
    terms: Vec<Digested>,
    lengths: Vec<usize>,
}

/// A banned term was found. Carries **where** and **how long**, never what.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    at: usize,
    length: usize,
}

impl Hit {
    /// The character offset in the scanned text where the match begins.
    #[must_use]
    pub fn at(self) -> usize {
        self.at
    }

    /// The length of the matched term in its normalised form.
    #[must_use]
    pub fn length(self) -> usize {
        self.length
    }
}

impl fmt::Display for Hit {
    /// Location and length. **Never the match, and never its context** — a
    /// surrounding snippet is the match with extra steps, and a path whose last
    /// component *is* the name defeats a redaction that prints the parent.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a protected name appears at character {} (length {})",
            self.at, self.length
        )
    }
}

impl TermList {
    /// Parse a term list in the shared format:
    ///
    /// ```text
    /// salt = <hex>              one line, prepended to every term before hashing
    /// <len> <64 hex chars>      one per term: length of the normalised form,
    ///                           then sha256(salt_bytes || normalised_term)
    /// ```
    ///
    /// Blank lines and `#` comments are ignored. A list with no salt or no terms
    /// is [`ScrubError::Disarmed`] rather than an empty matcher.
    pub fn parse(text: &str) -> Result<Self, ScrubError> {
        let mut salt: Option<Vec<u8>> = None;
        let mut terms: Vec<Digested> = Vec::new();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(hex) = line.strip_prefix("salt = ") {
                salt = Some(decode_hex(hex.trim()).ok_or(ScrubError::MalformedSalt)?);
                continue;
            }
            if let Some((length, digest)) = line.split_once(char::is_whitespace)
                && let Ok(length) = length.trim().parse::<usize>()
            {
                let digest = digest.trim().to_ascii_lowercase();
                if length > 0 && digest.len() == 64 {
                    terms.push(Digested { length, digest });
                }
            }
        }

        let salt = salt.ok_or(ScrubError::Disarmed)?;
        if terms.is_empty() {
            return Err(ScrubError::Disarmed);
        }
        let mut lengths: Vec<usize> = terms.iter().map(|t| t.length).collect();
        lengths.sort_unstable();
        lengths.dedup();

        Ok(TermList {
            salt,
            terms,
            lengths,
        })
    }

    /// How many terms the list holds. For a caller that wants to say the matcher
    /// is armed without saying what it is armed with.
    #[must_use]
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Whether the list is empty — never true, since [`TermList::parse`] refuses
    /// an empty list. Present because clippy asks for it beside `len`, and it
    /// documents the invariant rather than contradicting it.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// The first banned term in `text`, or `None`.
    ///
    /// Two passes, because neither covers the other — the same two the script
    /// runs:
    ///
    /// * **Joining** runs of one to three consecutive tokens, so `ProductName`,
    ///   `product-name` and `Product Name` all reduce to one candidate.
    /// * **Sliding** every stored length inside each token, so a name fused to a
    ///   neighbour (`ProductNameConfig`) is still found.
    ///
    /// Returns the first hit rather than all of them: one is enough to refuse,
    /// and a list of locations is a longer description of the same secret.
    #[must_use]
    pub fn find(&self, text: &str) -> Option<Hit> {
        let tokens = tokenise(text);

        // Pass 1: join runs of up to three tokens.
        for start in 0..tokens.len() {
            let mut joined = String::new();
            for token in tokens.iter().skip(start).take(3) {
                joined.push_str(&token.normalised);
                if self.contains(&joined) {
                    return Some(Hit {
                        at: tokens[start].at,
                        length: joined.chars().count(),
                    });
                }
            }
        }

        // Pass 2: slide every stored length inside each token.
        for token in &tokens {
            let chars: Vec<char> = token.normalised.chars().collect();
            for &length in &self.lengths {
                if length > chars.len() {
                    continue;
                }
                for offset in 0..=(chars.len() - length) {
                    let window: String = chars[offset..offset + length].iter().collect();
                    if self.contains(&window) {
                        return Some(Hit {
                            at: token.at + offset,
                            length,
                        });
                    }
                }
            }
        }

        None
    }

    /// Whether `candidate` (already normalised) is a banned term.
    fn contains(&self, candidate: &str) -> bool {
        let length = candidate.chars().count();
        if !self.lengths.contains(&length) {
            return false;
        }
        let mut hasher = Sha256::new();
        hasher.update(&self.salt);
        hasher.update(candidate.as_bytes());
        let digest = format!("{:x}", hasher.finalize());
        self.terms
            .iter()
            .any(|t| t.length == length && t.digest == digest)
    }
}

/// One run of alphanumerics, normalised, with the character offset it started
/// at in the original text so a [`Hit`] can name a location.
struct Token {
    normalised: String,
    at: usize,
}

/// Split `text` into runs of ASCII alphanumerics, lowercased. Everything else
/// is a separator and is dropped — the same normalisation the list's digests
/// were built from.
fn tokenise(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut start = 0;
    for (index, ch) in text.chars().enumerate() {
        if ch.is_ascii_alphanumeric() {
            if current.is_empty() {
                start = index;
            }
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            tokens.push(Token {
                normalised: std::mem::take(&mut current),
                at: start,
            });
        }
    }
    if !current.is_empty() {
        tokens.push(Token {
            normalised: current,
            at: start,
        });
    }
    tokens
}

/// Decode an even-length hex string.
fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if s.is_empty() || !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenising_drops_separators_and_lowercases() {
        let tokens = tokenise("Product-Name_v2!");
        let words: Vec<&str> = tokens.iter().map(|t| t.normalised.as_str()).collect();
        assert_eq!(words, vec!["product", "name", "v2"]);
    }

    #[test]
    fn hex_decoding_rejects_odd_and_non_hex() {
        assert!(decode_hex("00ff").is_some());
        assert!(decode_hex("0").is_none());
        assert!(decode_hex("zz").is_none());
        assert!(decode_hex("").is_none());
    }

    // The format's own shape: a list that cannot match anything is refused.
    #[test]
    fn a_list_without_terms_is_disarmed() {
        assert_eq!(
            TermList::parse("salt = 00ff\n").expect_err("a salt with no terms is disarmed"),
            ScrubError::Disarmed
        );
    }
}
