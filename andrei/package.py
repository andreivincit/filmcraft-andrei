"""Assemble a local macOS application, preserving the preceding verified build."""
from pathlib import Path
import plistlib
import filecmp
import shutil
import subprocess
import tempfile
from migrate_data import migrate

source = Path(__file__).resolve().parents[1]
workspace = source.parent
applications = workspace / "Applications"
applications.mkdir(exist_ok=True)
app = applications / "FilmCraft Andrei.app"
migrate()
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
    subprocess.run(["clang", "-O2", "-Wall", "-Wextra", "-Werror", str(source / "andrei/launcher.c"), "-o", str(wrapper)], check=True)
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
    same_engine = app.exists() and filecmp.cmp(app / "Contents/MacOS/FilmCraftCore", executable / "FilmCraftCore", shallow=False)
    rollback = Path(temporary) / "replaced.app" if same_engine else previous
    if not same_engine and previous.exists():
        shutil.rmtree(previous)
    if app.exists():
        app.rename(rollback)
    try:
        staged.rename(app)
    except OSError:
        if rollback.exists() and not app.exists():
            rollback.rename(app)
        raise
print(app)
