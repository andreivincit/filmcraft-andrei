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
