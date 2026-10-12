"""Move local models to macOS Application Support; preserve legacy user data."""
from pathlib import Path
import hashlib
import shutil

workspace = Path(__file__).resolve().parents[2]
legacy = workspace / "local-data"
data = Path.home() / "Library/Application Support/FilmCraft Andrei"


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def migrate():
    if data.is_symlink():
        raise SystemExit(f"Refusing a symlink at {data}")
    data.mkdir(parents=True, exist_ok=True)
    models = legacy / "models"
    destination = data / "models"
    if models.is_symlink() or destination.is_symlink():
        raise SystemExit("Refusing a symlink model store")
    if models.exists() and not destination.exists():
        files = {p.relative_to(models): digest(p) for p in models.rglob("*") if p.is_file()}
        # Both stores are on this user's local volume: rename without another model copy.
        models.rename(destination)
        if any(digest(destination / name) != value for name, value in files.items()):
            raise SystemExit("Model verification failed after migration; preserve both data directories")
    for name in ("preferences.json", "workspaces.json"):
        src, dst = legacy / name, data / name
        if src.is_file() and not dst.exists():
            shutil.copy2(src, dst)
    print(data)


if __name__ == "__main__":
    migrate()
