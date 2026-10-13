# FilmCraft Andrei

Personal macOS build based on FilmCraft 0.7.2, upstream commit
`71438c627b5cbcfb550526c3971f4d0037733c5c`. This branch changes identification and local
packaging, plus the personal features documented below.

## Preview transforms

Pause playback and use the Selection tool (V). Click a visible video or still in the
Program monitor, or select its clip in the timeline. A blue outline and four corner handles
appear. Drag inside it to move the clip; drag a corner to resize with its proportions preserved
and the opposite corner fixed. The picture follows the gesture, and one Undo restores the
entire move or resize. Position and Scale in the Properties panel reflect the same values.

The frontmost clip under the pointer wins. To reach the corner handles of an overlapping clip
behind it, select that clip in the timeline first. Text/shape and mask controls retain priority.
Locked tracks cannot be manipulated. Motion keyframes are edited at the current playhead.
If Motion has been disabled in the panel, enable it there before dragging its handles.
Snapping uses the existing Program guides/frame edges; hold Cmd to move freely.
This adds intrinsic Motion controls; rotation, anchor editing and separate Transform-effect
handles are still controlled through the panels. Automation uses `motion.set`,
`program.media.<clip>` and `program.media.<clip>.handle.<0..3>`.

Hold Option before dragging a text, image or video body in Program to duplicate it.
The copy goes on a new video track immediately above the original, at the same
timeline start and duration, retaining size, style and effects. Only its preview
position changes. When a graphic clip contains several internal layers, only the
grabbed text layer is copied. Media files are referenced, not copied. A gesture
is one Undo, including creation of the duplicate. Option on a resize handle still
resizes; use the element's body for duplication, with playback paused and Selection active.

Select several text clips in the timeline, including titles at different times.
Clicking an already-selected title in Program retains that selection. Drag the
visible title and all unlocked selected text clips take its new position live,
without changing their text, style or timing. This assigns the same frame position;
it does not preserve different offsets. Images/videos in a mixed selection are
not repositioned. Locked tracks are skipped, and animated positions follow the
existing edit-at-playhead behavior (inactive clips clamp to their own time range).

The desktop editor prepares at least ten video and ten audio tracks in every
sequence when opening a project or sequence, including the demo and recovered
projects. It preserves all existing tracks, settings and clips; empty new tracks
are appended and become part of the project on save. Projects with more tracks
retain them. New Sequence starts with ten of each. Tracks can require scrolling
in the timeline; all twenty need not fit on screen at once. Automation commands:
`preview.move` and `preview.ensureTracks`.

Shift-click additional visible texts, shapes, images or videos in Program to select
them together, including multiple internal layers of one Graphic clip. Each selected
element has a blue outline. Release Shift and drag the body of any selected element:
the group moves live, preserving relative offsets, sizes and timing, as one Undo.
The selection is temporary; no permanent group or nested sequence is created.
Click an unselected element without Shift to replace the selection, or empty Program
space to clear it. Locked/hidden elements are excluded. Group dragging uses Selection
with playback paused; hold Cmd to bypass snapping. Individual resize/anchor handles
are shown again when only one element is selected. Option-drag retains the existing
behavior of duplicating the grabbed element. Timeline title synchronization described
above remains separate: explicit preview groups translate relatively instead of
assigning all titles to one position. Automation: `preview.select`,
`preview.translate` and the `preview_elements` field returned by `state.inspect`.

The official `/Applications/FilmCraft.app` continues to receive ArtCraft Launcher updates.
Our app is `../Applications/FilmCraft Andrei.app`, with a different bundle identifier and
settings, models, logs and recovery in `~/Library/Application Support/FilmCraft Andrei`.
The previous `../local-data` retains its settings/autosaves/recovery as a local backup;
packaging moves the models once and copies preferences/workspaces without overwriting
existing Application Support settings. This avoids macOS blocking model reads from
Documents on ordinary Finder launches. Media and `.fcproj` projects can
be opened through File > Open without copying their media. Preserve a project backup before
opening it in a future release that might migrate its format.

## Automatic captions (Romanian and English)

Open the **Captions and Graphics** workspace, then **Text → Captions**. Choose
**Romanian**, **English** or **Auto-detect** and click **Generate auto captions**.
The personal desktop/CLI builds enable the existing pure-Rust Whisper backend.
The installed multilingual `whisper-base` model lives outside Git and the application
bundle in `~/Library/Application Support/FilmCraft Andrei/models/whisper-base/`
(about 293 MB, downloaded once and
checked against the source catalogue's pinned SHA-256 hashes). Inference is local;
there is no API account, audio upload or per-transcription fee.

Generation runs in a cancellable background job; the status bar shows its progress.
It decodes only selected audible timeline clips and their linked audio (respecting
mute/solo). With no selection, it uses sequence In/Out when both are set, otherwise
all audible timeline clips. It maps words to the edited sequence and creates a new
editable Subtitle track. The language menu closes after choosing; progress and errors
remain visible directly below Generate. Existing
tracks and cuts are retained. One Undo removes the generated transcript/captions;
SRT export and normal project save/reopen use the existing caption tools.
If the project is edited/replaced during recognition, the result is refused instead
of overwriting that work; run Generate again. Native inputs are capped at one hour
per selected audio span. Reverse/held clips and nested-audio limitations
remain outside this first implementation. Accuracy depends on the audio and model;
the text stays editable. `transcript.generate` remains the upstream synchronous
command; use the new button/`captions.autoGenerate` for background generation.

Automation: `captions.autoGenerate {language:"ro"|"en"|"auto",model?:"whisper-base"}`,
`jobs.list`, `jobs.cancel`, `captions.list`, `captions.export` and
`text.captions.auto.<language|generate>` UI widgets. Settings also includes Romanian
under Media Analysis & Transcription. Other catalogue models can be installed with
`transcript.downloadModel` when explicitly chosen.

## Automatically generated editable text

Select a timeline video clip (with its linked audio) or an audio clip, then click
the blue **T** after Toggle Proxies under the **Program** preview. It uses the
language chosen in **Text → Captions** and the same local speech model to create
individual Graphic text clips on a new **Generated text** video track above the
existing video tracks. Without selection it uses the marked In/Out or audible
sequence, as automatic captions do. Groups have up to two lines and approximately
four seconds of speech, using white text with a black outline by default.

Select any generated clip to edit its wording, font, style, size and appearance
in **Essential Graphics → Edit** or **Properties**. With playback paused and
Selection (V), drag the text in Program to reposition it. Existing caption tracks
are preserved. Generation is one Undo, runs in the background and refuses stale
results if the project changes. Save/reopen uses the existing Graphic format.
The button can be hidden/restored using the Program Button Editor (+).

Automation: `graphics.autoGenerateText {language:"auto"|"ro"|"en",model?:"whisper-base"}`,
`program.transport.graphics.autoGenerateText`, and the existing `graphics.*`
commands. The automatic-caption button continues to generate Subtitle tracks.
Verified with eight focused engine/monitor tests, formatting and Clippy, one final
product build, and native Romanian recognition, styling, Program dragging,
Undo/Redo and save/reopen in `../verification/auto-text-native/`.

## Visual text editing

With playback paused and Selection (V), drag a selected text's corner to change its
font size. For paragraph text, the middle handles resize its box and re-wrap words
at the existing font size. Double-click a text to edit, then click before or after
any letter to place the caret. Typing changes only that title's wording.

Select several text clips in the timeline to synchronize typography, appearance,
position and box size from the visible selected title. Selection is retained when
clicking or double-clicking that title. The panels' font family/style/size changes,
corner drags and paragraph box drags also reach titles at other times. Each edit is
one Undo; locked/disabled follower tracks are skipped and wording/timing stay local.
These controls apply to existing and automatically generated Graphic clips.
Timeline bulk edits and synchronized Program moves support up to 10,000 selected
clips. This is independent of the 1,024-element limit for simultaneous Shift-selected
preview groups; an extended speech sequence can contain more than 1,024 text clips.
Automation retains `graphics.set` and `program.layer.<clip>.<layer>.handle.<0..7>`;
corners are 0–3, middle sides 4–7 (right 5, left 7).
Verified with 21 graphics-engine and 17 focused UI tests, formatting/Clippy, one
final product build, and a native gesture/style/caret/Undo/save-reopen probe with
a representative screenshot in `../verification/text-edit-native/`.

## Text glow

Select a text clip, then open **Essential Graphics → Edit → Appearance → Glow**
(also available in **Properties**). Enable the checkbox and choose a halo colour,
Intensity (0–100%), Range (0–100 px), X-offset and Y-offset (±500 px).
The soft halo follows the letters and leaves their fill sharp; Shadow is independent.
Range and offsets follow the text's scale and rotation. The controls start at 40%
intensity, 45 px range and zero offsets, based on Andrei's CapCut screenshot;
this implements the soft halo rather than CapCut's commercial ray preset.

Selecting multiple text clips applies Glow edits to all selected text layers from
the visible title, including titles at other times, with one Undo per edit or drag.
Wording remains local. Existing and generated Graphic text clips support the effect;
old projects keep Glow off. Colour and numeric properties can be animated and survive
save/reopen. Preview and export share the rendering.
Automation: `graphics.set` with `glow`, `glow_color`, `glow_intensity`, `glow_range`,
`glow_offset_x`, `glow_offset_y`; UI ids are `graphics.prop.<parameter>`.
Verified with the model compatibility/animation test, two halo rendering tests,
the bulk-style engine test, five adjacent UI tests and formatting/Clippy for affected
crates, one final product build and a 12-second disposable native probe (checkbox,
bulk drag, Undo/Redo, persistence and identical reopened frame export).
Screenshot and exported frame proof: `../verification/text-glow-native/`.

## Export of rotated phone footage

The 0.7.2 integration uses the official isolated decode pool fix for rotated
phone footage and other decoding operations that start Rayon work under a GOP
cache mutex. It replaces our earlier narrow serial-rotation workaround.
Hardware encoding and GPU rendering remain opt-in export settings.

## Integration with 0.7.2

All personal features above remain enabled. Ordinary Selection clicks retain
our direct media move/proportional resize controls, Option duplication and
Shift group translation. Explicit Motion/Transform selection (or double-click
on media) uses the official eight-handle, rotation and anchor controls, with a
single gesture owner. Graphic text layers retain their own text handles.
The blue editable-text generation button also works in the official compact
transport's overflow menu. Background caption polling retains the official
export job diagnostics. Model, preferences and projects stay at existing paths.

## Build

The local Rust toolchain is in `../.tools`; upstream fonts are in `../craft-fonts`.
For a fresh checkout on Apple silicon, run `bash andrei/bootstrap.sh` first. Rust is pinned
to 1.99.0, and fonts to commit `3d9ab3a132ed0eff457e424226a435a901da30d3`.
`bash andrei/build.sh` builds the desktop app and CLI and verifies the local app signature.
Packaging keeps one preceding application as `FilmCraft Andrei.previous.app`; it replaces
older generated application backups instead of accumulating copies. User data stays outside it.
Routine edits follow [`WORKFLOW.md`](WORKFLOW.md): targeted tests, one final build
and one short native check. Use `bash andrei/check.sh --targeted filmcraft-CRATE TEST_ARGS...`
for focused checks. `bash andrei/check.sh --full` (or no arguments) runs upstream
quality gates for broad changes. `bash andrei/build.sh --no-package` builds once;
`bash andrei/build.sh --package-only` packages those binaries after the commit.
The source checkout contains no
personal footage, projects, settings, tokens or extracted commercial LUTs.
Checks retain release optimization but disable repeated LTO and use 16 codegen units
for the many test executables. Product builds retain the upstream thin-LTO profile.

## Updates

`upstream` points at `https://github.com/storytold/filmcraft.git` and `origin` at this fork.
The `andrei` branch includes the official 0.7.2 release; `main` tracks the upstream baseline.
Choose an upstream release, run `bash andrei/update.sh vX.Y.Z`, review/resolve the merge,
run checks, commit, rebuild and smoke-test before pushing. A conflict stops the merge;
personal commits are never reset or overwritten. Launcher updates do not update this build.

`target` is the regenerable build cache. Keeping it accelerates later changes. No media is
stored there except synthetic test fixtures. Toolchains and dependencies are in `../.tools`.

## Agent access

For a session that needs automation, launch the app executable with `--control 9876`.
`bash andrei/agent.sh` does this after the normal instance has been saved and closed.
The server listens on loopback only. The bundled `filmcraft-cli` can then use
`--bridge 127.0.0.1:9876`, including `mcp --bridge 127.0.0.1:9876`.
Normal launches keep the control server off. No global Codex settings are changed by this setup.
The launcher uses the personal Application Support directory; `FILMCRAFT_DATA_DIR`
can explicitly override it for a disposable agent session.
Keep the app inside this workspace for the build/update scripts.
Use a Finder alias or a Dock shortcut if you want easier access; do not move the bundle alone.

## Portable colour presets

The personal build saves the local LUT tables used by a Lumetri effect inside its
user preset. Import a version-2 preset through **Effects → Presets → right-click
an existing preset → Import Presets…**, then drag it onto a clip or adjustment
layer (or double-click with the target selected). The LUT is added to the target
project automatically; applying is one Undo step and repeated use reuses identical
tables. Exported presets carry the tables to another project or machine without
requiring the original LUT paths. Version-1 presets remain readable; older builds
cannot open version-2 preset files.

The owner's Old Money files and derived preset stay outside `source/` in the
workspace's `Andrei Presets/Old Money/`. They are local user materials and are not
published or bundled. The combined LUT chooses the Skin table for skin-like input
colours, the Background table for other colours, and blends smoothly between
these ranges. This is a colour heuristic, not face/skin segmentation; warm objects
can receive the same treatment. Initial Creative Intensity is 70%.
