# Personal fork: fast editing workflow

Andrei requested targeted checks, fewer compilations and faster delivery on
2026-10-11. On the personal `andrei` branch this policy replaces the upstream
requirement to run the entire workspace CI before every commit. Upstream safety,
licensing, command architecture and preservation of user projects still apply.

## Routine changes

1. Inspect the relevant code and make the change. Add regression tests only where
   they verify meaningful behavior or risk; use existing coverage when sufficient.
2. Run formatting, Clippy for the affected crates, and the relevant test targets
   or filters. Include adjacent behavior that can actually be affected, such as
   Undo, selection and mask/text priority for preview interactions. Run once;
   repeat only after relevant edits or a failure. Do not subsequently rerun the
   whole workspace as a routine final step.
3. For application code, build the final desktop and CLI once with
   `bash andrei/build.sh --no-package`. Use these binaries for one short native
   smoke check. For UI changes, inspect one representative screenshot. Exercise
   save/reopen when serialization changes and export when rendering changes.
   Native probes use a fresh temporary `--data-dir` each run: reusing an unsaved
   test session can open a recovery dialog that blocks synthetic clicks. Launch
   and drive the disposable window in one operation; close only that test instance.
4. Commit and push, then `bash andrei/build.sh --package-only`. Packaging the
   already-verified binaries records the new commit without another build.
   If code changes after the build, rebuild the affected product before packaging.
5. Report what passed and any material limit. Never claim a full-suite pass when
   only targeted checks ran. Do not promise an unmeasured completion time.

Examples (choose the actual affected tests, not these automatically):

```sh
bash andrei/check.sh --targeted filmcraft-engine --lib motion::tests
bash andrei/check.sh --targeted filmcraft-ui-egui --test media_transform_ui
bash andrei/build.sh --no-package
# Short native proof, then commit.
bash andrei/build.sh --package-only
```

`--targeted` still compiles dependencies required by the selected crate; it avoids
unrelated workspace test executables. Multiple affected crates may need separate
targeted commands. Do not launch simultaneous Cargo jobs against the same cache.
Product optimization remains the upstream release profile; targeted tests use the
existing no-LTO check profile. Cache both profiles and do not alternate profiles
for the same check or clear the build cache after routine changes.

## Broader changes

Use `bash andrei/check.sh --full` once for an upstream merge, dependency/toolchain
upgrade, broad architecture change, or a change whose effects cannot be bounded
to a few crates (for example shared project formats or codec infrastructure).
Check assets, dependency layers, fonts and WebAssembly individually when touched.
Expand checks when an actual failure or unresolved concern justifies it.

Documentation-only changes need diff/link review. Shell script changes need
`bash -n` and a lightweight relevant invocation, without product compilation.
Do not run app tests or rebuild/install the app for workflow-only edits.

Keep `target/` for reuse. Clean unusually large generated test artifacts only when
space requires it or after broad checks; do not repeatedly delete and rebuild
the focused tests during ordinary development. Do not accumulate app copies;
packaging retains only the preceding build.
