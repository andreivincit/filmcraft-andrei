# Personal changes

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
