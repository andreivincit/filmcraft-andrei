# Local baseline verification

Verified on Apple silicon macOS on 2026-10-11, based on upstream FilmCraft
0.6.0 (`2ff3f5e266838700c292a35aaf762e3d4dcef980`). Rust 1.99.0;
craft-fonts `3d9ab3a132ed0eff457e424226a435a901da30d3`.

- `bash andrei/check.sh` completed successfully: formatting, warning-free workspace
  Clippy, workspace tests, dependency layers, attribution of 61 assets, and WebAssembly
  checking of 44 crates. Checks used release optimization with LTO off and 16 codegen units.
- Workspace tests: 2,771 passed, zero failed, 41 upstream-ignored tests across
  245 groups. Optional models/features excluded by the upstream default configuration
  were not exercised.
- Native desktop and CLI release builds succeeded with the upstream product profile.
- The separately identified macOS app passed strict deep signature verification and
  opened its demo timeline in the native UI. Data went to the separate local directory.
- The CLI saved and reopened a synthetic `.fcproj`, rendered a frame, and exported
  a one-second H.264 video. ffprobe verified 480×270 and 24 frames; ffmpeg decoded it.
- Headless MCP initialized and exposed 17 tools.
- A separate desktop control session accepted project inspection, a timeline razor,
  undo, and UI inspection. The test session then closed successfully.

These checks establish the editable baseline and automation connection. They do not
add new editing features or establish compatibility with every real media/project file.
Synthetic outputs and personal material remain outside this public source checkout.

## Program preview media transforms

Verified on 2026-10-11 after adding image/video selection, body dragging and
proportional corner resizing. `bash andrei/check.sh` passed formatting, Clippy,
all workspace tests (2,778 passed, zero failed, 41 upstream ignored across 246
groups), dependency layers, 61 asset attributions and 44 WebAssembly crates.
Seven new tests cover atomic motion edits, validation, animated parameters,
image/video gestures, rotated nonuniform media and graphics/mask/lock priority.

The native desktop and CLI product release builds succeeded. A disposable native
control session using synthetic media verified video dragging, still-image corner
resizing with the opposite corner fixed, one history entry per gesture, undo/redo
and saving a project. The Program screenshot was visually inspected. Test files
are under the local workspace's `verification/transform-native/`, outside Git.

## Option drag, synchronized text and minimum tracks

Verified on 2026-10-11 with the personal targeted workflow. Formatting and Clippy
for engine/UI passed. Twelve relevant tests passed: three new engine regressions,
six media-transform UI tests (including two new acceptance tests), and three
existing graphics UI regressions. The full workspace suite was not repeated.
Tests cover preserved timing/style/originals, one-step duplicate-and-move Undo,
live synchronization of nonconcurrent selected titles, hostile command input,
locked tracks, minimum-track idempotence and the existing resize/mask behavior.

One desktop/CLI release build completed. A disposable native control session
verified Option-drag duplication of text/video/still, synchronized titles,
Undo/Redo, save/reopen, and opening the prior three-track synthetic project with
at least ten video/audio tracks. The Program screenshot was visually inspected.
Local proof files are in `verification/preview-native/`, outside Git. No original
user projects were modified during verification.

## Shift-click preview selection and relative group movement

Verified on 2026-10-12 using targeted checks, without repeating workspace CI.
Formatting and engine/UI Clippy passed. Fifteen relevant tests passed: four
preview engine regressions, eight media-transform UI tests and three existing
graphics UI regressions. New coverage includes internal layers in one Graphic,
separate graphic clips, mixed text/image/video groups, preserved spacing, hostile
parameters, selection invalidation and one-step Undo. The two new acceptance tests
were repeated after fixes to click-modifier handling and rapid Shift double-clicks.

Desktop/CLI release builds succeeded; the native rapid-click finding required one
additional product build. The final native proof used fresh temporary data to avoid
a recovery dialog left by an earlier interrupted test session. It verified Shift
selection of internal/separate titles and a mixed text/video/image group, relative
movement, one-step Undo/Redo and saving a synthetic project. Both selected-title
and moved-mixed-group screenshots were visually inspected. Local proof files are
in `verification/group-native/`, outside Git. Original user projects were untouched.

## Local Romanian and English automatic captions

Verified on 2026-10-12 with Whisper enabled in the personal desktop/CLI builds.
Formatting and warning-free engine/UI Clippy passed with speech features enabled.
Nine targeted transcript tests passed (seven engine, two UI), including background
generation, language selection, cancellation, changed-project protection and one-step
Undo/Redo. The optional tokenizer's former panic shortcut was replaced with an error.
The full workspace suite was not repeated.

One final desktop/CLI release build completed. Native recognition of locally generated
six-second Romanian and five-second English voices completed in approximately 1.64 s
and 1.24 s respectively on this laptop. These short synthetic timings do not establish
performance on long recordings. English text matched the spoken example; Romanian
included recognition errors ("subtitorile automat" for "subtitrările automate"), so
the installed fast base model's output still needs proofreading.

The new Captions button, responsive UI during recognition, editable subtitle tracks,
single Undo/Redo, SRT export and saving/reopening both synthetic projects were verified.
The Romanian Program screenshot was visually inspected. Proof files are under
`verification/captions-native/`, outside Git; no user projects were changed.
The multilingual whisper-base model's four catalogue files were size/SHA256 verified
at the pinned source revision and installed in the separate local data directory.
Models and recordings are not bundled or published in the source repository.
