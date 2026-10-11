"""Assemble a local macOS application, preserving the preceding verified build."""
from pathlib import Path
import plistlib
import shutil
import subprocess
import tempfile

source = Path(__file__).resolve().parents[1]
workspace = source.parent
applications = workspace / "Applications"
applications.mkdir(exist_ok=True)
app = applications / "FilmCraft Andrei.app"
(workspace / "local-data").mkdir(exist_ok=True)
version = next(line.split('"')[1] for line in (source / "Cargo.toml").read_text().splitlines() if line.startswith("version ="))
commit = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()

with tempfile.TemporaryDirectory(prefix="package-", dir=applications) as temporary:
    staged = Path(temporary) / app.name
    executable = staged / "Contents/MacOS"
    resources = staged / "Contents/Resources"
    executable.mkdir(parents=True)
    resources.mkdir(parents=True)
    for binary in ("filmcraft", "filmcraft-cli"):
        destination = "FilmCraftCore" if binary == "filmcraft" else binary
        shutil.copy2(source / "target/release" / binary, executable / destination)
    shutil.copy2(source / "assets/app-icon/filmcraft.icns", resources / "FilmCraft.icns")
    for name in ("LICENSE-MIT", "LICENSE-APACHE", "NOTICE", "ATTRIBUTION.md"):
        shutil.copy2(source / name, resources / name)
    for license_file in (workspace / "craft-fonts/fonts").glob("*/OFL.txt"):
        shutil.copy2(license_file, resources / f"OFL-{license_file.parent.name}.txt")
    wrapper = executable / "FilmCraftAndrei"
    wrapper.write_text('''#!/bin/bash
set -euo pipefail
bundle_bin="$(cd "$(dirname "$0")" && pwd)"
workspace="$(cd "$bundle_bin/../../../.." && pwd)"
export FILMCRAFT_DATA_DIR="$workspace/local-data"
exec "$bundle_bin/FilmCraftCore" "$@"
''')
    wrapper.chmod(0o755)
    info = {
        "CFBundleName": "FilmCraft Andrei",
        "CFBundleDisplayName": "FilmCraft Andrei",
        "CFBundleIdentifier": "dev.andreivincit.filmcraft",
        "CFBundleExecutable": "FilmCraftAndrei",
        "CFBundleIconFile": "FilmCraft",
        "CFBundlePackageType": "APPL",
        "CFBundleVersion": version,
        "CFBundleShortVersionString": version,
        "FilmCraftBuildCommit": commit,
        "LSMinimumSystemVersion": "11.0",
        "LSApplicationCategoryType": "public.app-category.video",
        "NSHighResolutionCapable": True,
        "NSSupportsAutomaticGraphicsSwitching": True,
        "NSRequiresAquaSystemAppearance": False,
        "NSMicrophoneUsageDescription": "FilmCraft Andrei uses the microphone for voice-over recording.",
        "NSHumanReadableCopyright": "Based on FilmCraft. MIT OR Apache-2.0; see bundled notices.",
    }
    with (staged / "Contents/Info.plist").open("wb") as handle:
        plistlib.dump(info, handle)
    for name in ("FilmCraftCore", "filmcraft-cli"):
        subprocess.run(["codesign", "--force", "--sign", "-", "--timestamp=none", "--options", "runtime", "--entitlements", str(source / "packaging/macos/entitlements.plist"), str(executable / name)], check=True)
    subprocess.run(["codesign", "--force", "--sign", "-", "--timestamp=none", str(staged)], check=True)
    subprocess.run(["codesign", "--verify", "--strict", "--deep", str(staged)], check=True)
    previous = applications / "FilmCraft Andrei.previous.app"
    if previous.is_symlink():
        raise SystemExit(f"Refusing to replace a symlink at backup path: {previous}")
    if previous.exists():
        shutil.rmtree(previous)
    if app.exists():
        app.rename(previous)
    try:
        staged.rename(app)
    except OSError:
        if previous.exists() and not app.exists():
            previous.rename(app)
        raise
print(app)
