"""Benchmark the renderer's actual page grouping module, optionally against an old copy.

This measures CPU list preparation only, not game FPS. No game assets are needed.
"""
import argparse
import json
from pathlib import Path
import statistics
import subprocess
import tempfile


SOURCE = r'''
#[path = "MODULE"] mod page_runs;
use std::{hint::black_box, time::Instant};
fn main() {
    let mut group = page_runs::PageRuns::default();
    let n = RUNS;
    let mut draws = vec![[0u32; 5]; n * 4];
    let mut pages = vec![(0, None); n * 4];
    // Warm capacities before timing.
    for round in 0..=5 {
        let now = Instant::now();
        for _ in 0..ITERATIONS {
            group.clear();
            for i in 0..n {
                let page = ((i * 17 + i / 13) % 6) as u32;
                let distance = 16.0 + (i as f64) * 0.06;
                for j in 0..4 {
                    draws[i * 4 + j] = [i as u32, j as u32, 0, 0, 0];
                    pages[i * 4 + j] = (page, None);
                }
                group.push((distance * distance).to_bits(), page, i * 4, 4, [i as u32, 0, 0]);
            }
            black_box(group.regroup(&mut draws, &mut pages, 0));
        }
        if round > 0 {
            println!("{}", now.elapsed().as_secs_f64() * 1e6 / ITERATIONS as f64);
        }
    }
    black_box(draws);
}
'''


def run(module: Path, folder: Path, args) -> list[float]:
    source = folder / "bench.rs"
    source.write_text(SOURCE.replace("MODULE", module.resolve().as_posix())
                      .replace("RUNS", str(args.runs) + "usize")
                      .replace("ITERATIONS", str(args.iterations)), encoding="utf-8")
    exe = folder / "bench.exe"
    subprocess.run(["rustc", "--edition", "2021", "-O", "--crate-name", "page_runs_bench",
                    str(source), "-o", str(exe)], check=True)
    return [float(line) for line in subprocess.check_output([str(exe)], text=True).splitlines()]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--runs", type=int, default=12000)
    parser.add_argument("--iterations", type=int, default=500)
    parser.add_argument("--test-only", action="store_true", help="Run the actual module's unit tests without game dependencies")
    args = parser.parse_args()
    if args.runs < 1 or args.iterations < 1:
        parser.error("runs and iterations must be positive")
    current = Path(__file__).resolve().parents[1] / "crates/horizon-client-render/src/gfx/page_runs.rs"
    if args.test_only:
        with tempfile.TemporaryDirectory(prefix="torizon-page-runs-tests-") as folder:
            source = Path(folder) / "tests.rs"
            source.write_text(f'#[path = "{current.as_posix()}"] mod page_runs;', encoding="utf-8")
            exe = Path(folder) / "tests.exe"
            subprocess.run(["rustc", "--edition", "2021", "--test", "--crate-name", "page_runs_tests",
                            str(source), "-o", str(exe)], check=True)
            subprocess.run([str(exe)], check=True)
        raise SystemExit(0)
    result = {"runs": args.runs, "draws_per_run": 4, "unit": "microseconds per frame"}
    with tempfile.TemporaryDirectory(prefix="torizon-page-runs-") as folder:
        for label, module in [("before", args.baseline), ("after", current)]:
            if module:
                samples = run(module, Path(folder), args)
                result[label] = {"median": statistics.median(samples), "samples": samples}
    if args.baseline:
        result["reduction_percent"] = 100 * (1 - result["after"]["median"] / result["before"]["median"])
    print(json.dumps(result, indent=2))
