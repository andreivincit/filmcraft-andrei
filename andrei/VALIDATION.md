# Local baseline verification

## FilmCraft 0.7.2 integration

Verified on 2026-10-13 after integrating official v0.7.2. Formatting and workspace
Clippy passed. The complete workspace test coverage passed: 3,280 tests, zero final
failures, 44 ignored across 294 programs. The initial full run found integration
failures in UI expectations/localization; after fixes, affected and remaining
targets were resumed without repeating already-passing core suites. Dependency
layers, attribution of 61 assets and WebAssembly checking of 49 crates passed.
Optional features outside the standard test configuration remain excluded.

One final desktop/CLI release build with Whisper completed in 4m55s. A disposable
native session opened a saved copy of the owner's actual 21.733-second project,
saved/reopened it without changing clips/settings/markers and exported all 652
frames in 9.57 seconds with H.264 hardware encoding and GPU rendering enabled.
ffprobe verified 1080×1920/30 fps and AAC stereo; ffmpeg decoded the complete
output without errors. Two extracted frames were visually inspected.

The native personal desktop expanded a one-track sequence to ten tracks of each
kind. The blue Program text button generated editable Graphic text locally from
Romanian audio, with one Undo/Redo and no new subtitle track. Wording, font, size
and glow survived save/reopen; the Program screenshot was visually inspected.
Gesture regressions cover personal Selection controls and exclusive ownership by
the official Motion controls. The original project copy remained byte-identical.
Proof files are in `../verification/update-0.7.2/`, outside the public repository.
These measured results cover this project and short synthetic recognition, not
all media formats or long transcriptions.

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

## Selected-audio captions and visible feedback

Updated on 2026-10-12 after the owner reported an apparently inactive Generate button.
The language popup now explicitly closes after a choice; Generate uses an ordinary
egui button and shows persistent progress/error text immediately underneath.
Recognition reads selected audible timeline clips/linked audio, or In/Out with no
selection, rather than decoding every source file in full. Existing transcript words
outside decoded spans are preserved.

Engine/UI library Clippy passed; one final desktop/CLI build completed in 3m25s.
No test suite was run, at the owner's request to avoid repeated tests. A native
operation on a separate copy of the user's project generated three captions for
the selected approximately 19-second `ai use.mp4` audio clip in about 3 seconds.
Automatic detection chose English. The first script assertion expected Romanian,
but inspection showed the scripted menu interaction had retained Auto-detect;
the follow-up captured the actual chosen language and successful output.
The original project/media were untouched. Local evidence and the inspected screenshot
are in `verification/captions-native/selected-fix.*`, outside Git.

## Bulk text sequences beyond 1,024 clips

Fixed on 2026-10-13 after the owner's sequence of 1,109 titles hit the old
`graphics.set` and `preview.move` limits. Both timeline operations now share a
10,000-selected-clip bound, separate from simultaneous preview-group limits.
Formatting and engine library Clippy passed. All 22 graphics tests and four
preview tests passed, including a new 1,109-title regression for typography,
Glow, geometry, preserved wording, atomic Undo/Redo and oversized-input refusal.
No full workspace suite was run.

One final desktop/CLI build completed in 4m48s. A 15-second native probe on a
separate copy of the owner's project verified Program selection retention,
font/style/bold, corner resizing, one-step Undo/Redo, synchronized Program
positioning and the actual panel Glow checkbox across all 1,109 titles.
Wording and timing were unchanged, the screenshot was visually inspected, and
the original project's SHA-256 stayed identical. Local evidence is under
`../verification/bulk-text-fix/`; no old source or project backup was restored.
