# FilmCraft interface

The egui frontend draws panels and dialogs, and dispatches project changes through the engine.
Interactive controls keep stable automation ids regardless of the interface language.

## Program media transforms (personal fork)

Selection-tool clicks pick video/still/subclip/nested-sequence clips in visible track order.
Selected clips show their source bounds transformed by the renderer's Motion matrix, including
rotation, pixel aspect ratio and Scale to Frame Size. Body drags change Position live; corner
drags preserve the existing aspect and pin the opposite corner. Each gesture dispatches merged
atomic `motion.set` edits, so one Undo restores Position and both scale axes. Animated parameters
are changed at source time derived from the captured playhead; body drags do not insert Scale keys.

Graphics and masks keep their existing controls. Handles are disabled during playback and on
locked tracks or with Motion disabled. Automation IDs are `program.media.<clip>` and `.handle.<0..3>` (TL/TR/BR/BL).
`media_transform_ui` covers imported PNGs, video overlays, rotated/nonuniform scale, stacking,
mask priority, locked tracks, undo/redo and base-clip isolation. Rotation/anchor handles and the
separate Transform effect are outside this change. No project-format or render/export change.

## Transition drop previews

Dragging a video or audio transition over a compatible clip shows its actual time span, the
selected cut and an In/Out label. Shared cuts center the span; isolated edges use a one-sided
span. The preview uses the engine's read-only transition planner, including Timeline duration
preferences, and release dispatches `effects.apply` with the same edge. Locked tracks and
incompatible clip types show no valid transition preview. Ordinary effects retain their clip
outline. `timeline.transitionDropPreview` exposes the span and edge to automation.

`transition_drop_ui` covers both edges of short video/audio clips, hover without project edits,
the committed span and undo. Set `FILMCRAFT_UI_SNAPSHOT_DIR` to render screenshots with wgpu.

## Automatic editable text (personal fork)

The Program transport's blue T (`program.transport.graphics.autoGenerateText`)
dispatches `graphics.autoGenerateText` with the Text panel's caption language.
It creates normal Graphic clips above the existing video tracks and opens
Essential Graphics Edit. Existing preview selection, dragging and text styling
apply to each generated clip. Both caption and graphic generation share progress,
cancellation, stale-project protection and one Undo. The Button Editor controls
the new action independently of the Source monitor.

Text corners change numeric font size;
paragraph middle handles resize the wrapping box. Timeline text selections synchronize
style and geometry from the visible title, retaining each clip's wording. Double-click
to edit and single-click beside a letter to place the caret there.

Text Appearance also includes Glow: enable, colour, Intensity, Range and X/Y offsets.
These use the same `graphics.set` bulk/Undo path as typography; the numeric drag is
one edit and the halo remains independent of Shadow.

## Localisation

Edit > Language offers English, Japanese, Spanish, Brazilian Portuguese, Ukrainian, Simplified
Chinese, German, Russian and French. The language is stored in the engine's `general.interfaceLanguage` preference,
restored on startup, and also available in Settings > General.
Its default, System Language (`system`), follows the operating system: the first of the user's
preferred languages that the interface has (the host supplies them through
`HostHooks::system_languages`: `sys-locale` on the desktop, `navigator.languages` on the web),
otherwise English.
The `app.language.*` UI commands and `prefs.set` reach it through the control channel.
Japanese requires a craft-fonts build or a suitable installed font.

English source strings are lookup keys in `src/i18n/<code>.tsv`. `tl!` translates literals, `tlf!`
fills translated templates, and `i18n::t` translates names from registries. Placeholder values
(including user filenames containing braces) are inserted literally. Catalog translations are
original work using ordinary language, without proprietary localisation resources.

Spanish, Japanese, Simplified Chinese and French cover menus, panels, dialogs, settings and registry labels. Searches accept both the
translated label and its English source, including Unicode capitals. Project content, command ids
and preference values retain their original values. Engine errors, CLI and MCP messages remain
English. Brazilian Portuguese and Ukrainian currently cover the core menus, German every menu; they
fall back to English elsewhere.

Verification: `cargo test -p filmcraft-ui-egui` checks catalog syntax, duplicate keys, placeholders,
literal/menu/registry coverage and UI behaviour; `cargo xtask ci` runs the workspace gates. Visual
checks use the control channel to switch language and capture the resulting panels.

## Source playback

`source_playback.rs` auditions media independently of Program playback. It reuses MediaSource
decoding, output mapping, the device clock and the desktop play-ahead buffer. Only one monitor
owns the device at a time. Source frame jobs use the existing frame server and Source resolution.
The live project, undo stack and timeline playhead are unchanged. See [monitor documentation](../../docs/monitors.md)
for behavior, commands, limitations and regression tests. Source playback tests use synthetic
demo media and fake audio outputs; no third-party assets are introduced.

Source drag controls snapshot the selected span and dispatch `timeline.place` for video, audio,
or linked video/audio. Full-clip marks are implicit until set by the user. The engine owns edits,
range validation and undo; the UI owns the drag gesture. See `source_drag_ui` and the engine's
`source_placement_tests` for regression coverage.

Source range handles use transient gesture previews and the engine's integer-frame
`source_monitor::adjust_range`; release dispatches one mark edit. Monitor command routing
keeps Source navigation, markers and marked-range playback separate from Program.
