# Personal changes

## 2026-10-13 — trim selected timeline layers together

Dragging the In or Out edge of a selected clip with the Selection tool trims all
selected clips, including other tracks, by the same time delta. Preview every
unlocked target during the gesture, preserve selection, and commit one Undo step.
The tightest source, neighbor or minimum-duration limit stops the whole group;
locked tracks stay unchanged. Ripple, Roll and Rate Stretch retain their behavior.
Agent callers can opt in with `timeline.trim {clip, edge, delta, selection:true}`.

## 2026-10-13 — Program review playback speed

Add a speed dropdown immediately before Program playback resolution, also in the
maximized monitor: 0.5×, 1×, 1.5×, 2×, 2.5× and 3×. The selected review rate survives
Pause/Play during the session and can change while playing. Reuse the existing
shuttle audio/clock path; no clip speed, timing, project data or export changes.
Agent command: `playback.setReviewSpeed {speed:number}`.

## 2026-10-13 — native macOS launcher

Replace the Bash application entry point with a signed native launcher which
sets the same personal data path and waits for FilmCraftCore. macOS no longer
identifies the main application's host as com.apple.bash. Preserve the preceding
engine backup when only the launcher/packaging changes; no product recompilation.

## 2026-10-13 — FilmCraft 0.7.2 integration

Merge official v0.7.2 (`71438c627b5cbcfb550526c3971f4d0037733c5c`) into
the personal branch, preserving automatic captions/editable text, typography and
glow, Option duplication, Shift preview groups and the desktop's ten-track policy.
Use the official isolated decode pool instead of the earlier serial-rotation
workaround. Ordinary Selection keeps the personal media controls; explicit
Motion/Transform editing uses official rotation/anchor/eight-handle controls.
Retain the blue text button in the compact transport and its overflow menu,
personal background polling and official export diagnostics. Generic UI clients
keep their explicit track counts; the native personal desktop opts into ten tracks.

Full quality gates and native project/export/text probes passed; see
[verification](VALIDATION.md#filmcraft-072-integration).

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
