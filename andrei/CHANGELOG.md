# Personal changes

## 2026-10-12 — normal macOS launch can read speech models

The installed Finder-launched app reported `config.json: Operation not permitted`
for the model stored under Documents. Move personal runtime data to
`~/Library/Application Support/FilmCraft Andrei` through packaging and the launcher.
Migrate the model by rename with hash verification, copy initial preferences/workspaces,
preserve old autosaves/recovery, and retain an explicit data-directory override.
No Rust changes or compilation are required.

## 2026-10-12 — caption button feedback and selected audio

Close the language popup immediately after choosing and use a standard Generate
button. Keep job progress and actionable errors visible directly in Captions.
Decode selected clips and their linked audio rather than the full source file;
without selection, honor In/Out or the audible timeline. Preserve transcript words
outside the recognized source spans and retain the single-Undo/background safeguards.

## 2026-10-12 — local automatic captions in Romanian and English

Enable the existing Whisper backend in personal desktop/CLI builds. Text → Captions
now offers Romanian, English and automatic language detection, with background
generation, progress and cancellation. Results add an editable subtitle track and
transcripts as one Undo; concurrent project edits are protected. The multilingual
base model is stored separately from the app/repository. Romanian is also available
in transcription preferences. See [usage](README.md#automatic-captions-romanian-and-english).

## 2026-10-12 — Shift-click and relative preview groups

Shift-click visible Program elements to extend the selection, including internal
layers within one Graphic clip and mixed text/image/video selections. Drag a selected
body to translate the selection live while preserving spacing, size and timing,
with one Undo. Timeline title-position synchronization and Option-drag duplication
retain their existing behavior. See [usage](README.md#preview-transforms).

## 2026-10-11 — Option drag, synchronized titles and ten tracks

Option-drag a text, image or video body in Program to create a same-time,
same-duration copy on a new track immediately above it. Dragging a title from a
timeline multi-selection assigns its position live to the selected text clips,
including titles outside the current frame. Both gestures are one Undo. The
desktop editor prepares at least ten video and ten audio tracks per sequence
on opening, without changing existing clips. See [usage](README.md#preview-transforms).

## 2026-10-11 — preview transforms

Video and image clips can be selected, moved and resized directly in the Program monitor.
Corner drags preserve proportions and the opposite corner. Live edits use intrinsic Motion
values and one undo entry per gesture. The new `motion.set` command changes Position and Scale
atomically for UI, CLI, MCP and native control clients. See [usage](README.md#preview-transforms).
