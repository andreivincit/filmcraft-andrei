# FilmCraft Andrei

Personal macOS build based on FilmCraft 0.6.0, upstream commit
`2ff3f5e266838700c292a35aaf762e3d4dcef980`. This branch changes identification and local
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
The `andrei` branch starts at the installed 0.6.0 release; `main` tracks the upstream baseline.
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
