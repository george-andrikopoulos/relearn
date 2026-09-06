+++
tag = "R:verify-the-glyph-exists"
title = "A character the UI draws must have a glyph in the faces the app actually loads"
error_class = "Choosing a character for a user interface from how it renders in the document where it was written, rather than from the font stack the application loads at runtime -- so it draws as an empty box. The instruction file that prescribes the substitute is itself unverified, and is believed because it is written down"
home = { kind = "project", path = "design-architecture-tool" }
created = "2026-08-16"
origin = "mined"
status = { kind = "graduated", to = "test:client/src/app/font_tests.rs + gate:verify.sh emoji_ban", date = "2026-08-16" }
incident = "2026-08-16. CLAUDE.md's rule 1 -- the rule whose entire subject was preventing tofu boxes -- prescribed a fullwidth plus as the add symbol. That codepoint is a CJK width variant that none of the four faces egui 0.29 loads (Ubuntu-Light, Hack, NotoEmoji-Regular, emoji-icon-font) covers, so the NEW PROJECT and ADD SERVICE buttons drew as empty boxes for the entire life of the project. Two emoji had also slipped past the rule, in a window title and on a Topology button. The rule was violated three times while it was prose, and the third time the prose was the source of the defect."
+++

Ask the font stack, not the document. A character that renders in an editor, a terminal
or a markdown preview proves nothing about the four faces egui loads at runtime, and the
failure is silent -- an empty box, not an error.

Prose cannot hold this, and this repository has the proof: the rule existed, was read,
and prescribed a codepoint that produced the exact defect it forbade. When the
instruction itself is the defect, re-reading it does not help, which is the argument for
pushing a rule down a layer rather than restating it more firmly.

Two controls now hold it, and they cover different halves:

* `client/src/app/font_tests.rs` asks the REAL `FontDefinitions::default()` -- the very
  stack the app builds -- whether each non-ASCII character the UI draws has a glyph, in
  the proportional and monospace families both. This is the positive half: what the table
  in CLAUDE.md claims is now asserted against the thing it claims about.
* `scripts/verify.sh` `emoji_ban()` blocks two whole codepoint blocks outright in
  non-comment lines under `client/src` -- astral emoji (U+1F000-U+1FAFF) and fullwidth
  forms (U+FF00-U+FFEF). The BLOCK is banned rather than the individual codepoint,
  because every member of it is equally uncovered and banning one at a time is how the
  second one gets in.

Failure-mode check, before any non-ASCII character reaches a widget: **which of the four
loaded faces has this glyph, and what asserted that?** If the answer is that it looked
fine where it was typed, nothing has been verified.
