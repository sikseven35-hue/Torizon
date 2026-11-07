"""Package only original source and programs; never include Minecraft files or .local/."""
from pathlib import Path
import argparse
import shutil

parser = argparse.ArgumentParser()
parser.add_argument('--game-bin', type=Path, required=True)
parser.add_argument('--launcher', type=Path)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parent.parent
out = args.output
runtime = out / 'runtime'
runtime.mkdir(parents=True, exist_ok=True)
(runtime / '.installed').write_text('Torizon 0.1.2\n', encoding='utf-8')
for name in ['horizon-client.exe', 'horizon-ctl.exe', 'horizon-prepare.exe', 'horizon-server.exe']:
    shutil.copy2(args.game_bin / name, runtime / name)
for name in ['research-lock.json', 'LICENSE', 'THIRD_PARTY_NOTICES.md']:
    shutil.copy2(root / name, runtime / name)
for name in ['tools', 'mods/horizon/src/test/java', 'crates/horizon-prepare']:
    shutil.copytree(root / name, runtime / name, dirs_exist_ok=True,
                    ignore=shutil.ignore_patterns('.local', '__pycache__', 'target', '*.pyc'))
for name in ['LICENSE', 'THIRD_PARTY_NOTICES.md', 'TORIZON.md']:
    shutil.copy2(root / name, out / name)
if args.launcher:
    shutil.copy2(args.launcher, out / 'Torizon.exe')
assert not any(p.name in {'.local', '.git', 'assets'} for p in runtime.rglob('*') if p.is_dir())
print(f'Packaged Torizon at {out}')
