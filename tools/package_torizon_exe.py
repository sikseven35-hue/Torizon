"""Append our runtime to a Windows launcher so the single EXE installs offline."""
import argparse
from pathlib import Path
import shutil
import zipfile


def package(launcher: Path, runtime: Path, output: Path) -> None:
    for name in ("horizon-client.exe", "horizon-prepare.exe", "research-lock.json"):
        if not (runtime / name).is_file():
            raise ValueError(f"Missing runtime file: {name}")
    if launcher.resolve() == output.resolve():
        raise ValueError("Output must differ from the original launcher")
    files = sorted(p for p in runtime.rglob("*") if p.is_file())
    for path in files:
        relative = path.relative_to(runtime)
        if any(part in {".local", ".git", "assets", "target"} for part in relative.parts):
            raise ValueError(f"Private or build files in runtime: {relative}")
        if path.suffix.lower() in {".jar", ".mca"}:
            raise ValueError(f"Minecraft data in runtime: {relative}")
    output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(launcher, output)
    with zipfile.ZipFile(output, "a", zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
        for path in files:
            archive.write(path, path.relative_to(runtime).as_posix())


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--launcher", type=Path, required=True)
    parser.add_argument("--runtime", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    package(args.launcher, args.runtime, args.output)
