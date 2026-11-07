import importlib.util
from pathlib import Path
import tempfile
import unittest
import zipfile

spec = importlib.util.spec_from_file_location(
    "package_torizon_exe", Path(__file__).resolve().parents[1] / "tools/package_torizon_exe.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class StandalonePackageTests(unittest.TestCase):
    def test_overlay_preserves_executable_and_all_runtime_files(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            launcher = root / "launcher.exe"
            launcher.write_bytes(b"MZ\x00original executable")
            runtime = root / "runtime"
            runtime.mkdir()
            files = {"horizon-client.exe": b"client", "horizon-prepare.exe": b"prepare",
                     "research-lock.json": b"{}", "tools/nested/source.rs": b"Rust tool"}
            for name, contents in files.items():
                path = runtime / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(contents)
            output = root / "Torizon.exe"
            module.package(launcher, runtime, output)
            self.assertTrue(output.read_bytes().startswith(launcher.read_bytes()))
            with zipfile.ZipFile(output) as archive:
                self.assertEqual(set(archive.namelist()), set(files))
                for name, contents in files.items():
                    self.assertEqual(archive.read(name), contents)
            (runtime / "forbidden.jar").write_bytes(b"private")
            with self.assertRaises(ValueError):
                module.package(launcher, runtime, output)
