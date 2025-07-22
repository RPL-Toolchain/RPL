#!/bin/python3

import os
import subprocess
import time
from typing import Iterable, TypeVar
import yaml
import re
import shutil

# inline-mir-threshold
# inline-mir-forwarder-threshold
# inline-mir-hint-threshold
# cross-crate-inline-threshold

compare: dict[int, dict]

if os.path.exists("compare.yaml"):
    with open("compare.yaml", "r") as f:
        compare = yaml.safe_load(f)
        # print(config)
else:
    compare = dict()

print("Warming up...")

# subprocess.run(
#     ["cargo", "uitest"],
#     env={**os.environ},
#     stdout=subprocess.DEVNULL,
#     stderr=subprocess.DEVNULL,
#     encoding="utf-8",
# )

subprocess.run(
    ["cargo", "uitest", "--release"],
    env={**os.environ},
    # stdout=subprocess.DEVNULL,
    stderr=subprocess.DEVNULL,
    encoding="utf-8",
)

if os.path.exists("docs/compare"):
    shutil.rmtree("docs/compare")

if not os.path.exists("docs/compare"):
    os.makedirs("docs/compare")

if os.path.exists("mir_dump"):
    shutil.rmtree("mir_dump")
# os.removedirs('mir_dump')

# Tests that we want to investigate which specific functions are inlined.
skip = [
    "tests/ui/cve_2020_35862/cve_2020_35862_manually_inlined.rs",
    "tests/ui/cve_2020_35862/cve_2020_35862.rs",
    "tests/ui/cve_2021_38190/cve_2021_38190.rs",
]

T = TypeVar("T")


def flat_map(l: Iterable[Iterable[T]]) -> list[T]:
    return [item for sublist in l for item in sublist]


skips = flat_map(["--skip", skip] for skip in skip)

re_result = re.compile(
    "test result: (FAIL|ok). ((?P<failed>\d+) failed; )?(?P<passed>\d+) passed; (?P<ignored>\d+) ignored;"
)

for cfg in [
    0,
    10,
    20,
    30,
    40,
    50,
    60,
    80,
    100,
    150,
    200,
    300,
    400,
    500,
    # 600,
    # 700,
    # 800,
    # 900,
    # 1000,
    # 2000,
    # 5000,
    # 10000,
    # 100000,
    # 1000000,
    # 10000000,
    # 100000000,
    # 1000000000,
    # 10000000000,
    # 100000000000,
    # 1000000000000,
    # 10000000000000,
    # 100000000000000,
    # 1000000000000000,
    # 10000000000000000,
    # 100000000000000000,
    # 1000000000000000000,
    # 10000000000000000000,
    # (1<<64)-1,
]:
    env = {
        **os.environ,
        "RPL_TEST_INLINE_MIR_THRESHOLD": str(cfg),
        "RPL_TEST_INLINE_MIR_FORWARDER_THRESHOLD": str(cfg),
        "RPL_TEST_INLINE_MIR_HINT_THRESHOLD": str(cfg),
        "RPL_TEST_CROSS_CRATE_INLINE_THRESHOLD": str(cfg),
    }
    print(f"Running with -Z inline-mir-threshold={cfg}")
    t1 = time.time()
    child = subprocess.run(
        ["cargo", "uitest", "--release", "--", *skips],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        encoding="utf-8",
        timeout=120,
    )
    t2 = time.time()
    print(f"Done with {cfg} in {t2-t1:.2f}s")

    matched = re.search(re_result, child.stdout)
    print(matched)
    with open(f"docs/compare/compare-{cfg}.txt", "w") as f:
        f.write(child.stdout)
    with open(f"docs/compare/compare-{cfg}-err.txt", "w") as f:
        f.write(child.stderr)
    assert "test result:" in child.stdout
    assert matched is not None, f"Error parsing output: {child.stdout}"
    groupdict = matched.groupdict()
    failed = (
        int(groupdict["failed"])
        if "failed" in groupdict and groupdict["failed"] is not None
        else 0
    )
    assert (child.returncode == 0) == (
        failed == 0
    ), f"Error running with {cfg}: {child.stderr}\n{child.stdout}"
    passed = int(groupdict["passed"])
    ignored = int(groupdict["ignored"])

    # child = subprocess.run(
    #     ["cargo", "uitest", "--release", "--", "--bless", *skip],
    #     env=env,
    #     stdout=subprocess.PIPE,
    #     stderr=subprocess.PIPE,
    #     encoding="utf-8",
    #     timeout=120,
    # )
    # with open(f"docs/compare/compare-{cfg}-debug.txt", "w") as f:
    #     f.write(child.stdout)
    # with open(f"docs/compare/compare-{cfg}-debug-err.txt", "w") as f:
    #     f.write(child.stderr)
    # assert "test result:" in child.stdout, child.stdout

    # shutil.copy('mir_dump/cve_2020_35862.{impl#14}-into_boxed_bitslice.-------.dump_mir..mir', f'docs/compare/compare-{cfg}-cve_2020_35862.mir')
    # shutil.copy('mir_dump/cve_2020_35862.{impl#14}-into_boxed_bitslice.-------.dump_mir..mir.cfg.dot', f'docs/compare/compare-{cfg}-cve_2020_35862.dot')

    # shutil.copy('mir_dump/cve_2020_35862_manually_inlined.{impl#14}-into_boxed_bitslice.-------.dump_mir..mir', f'docs/compare/compare-{cfg}-cve_2020_35862_manually_inlined.mir')
    # shutil.copy('mir_dump/cve_2020_35862_manually_inlined.{impl#14}-into_boxed_bitslice.-------.dump_mir..mir.cfg.dot', f'docs/compare/compare-{cfg}-cve_2020_35862_manually_inlined.dot')

    # shutil.copy('mir_dump/cve_2021_38190._#1-{impl#0}-deserialize.-------.dump_mir..mir', f'docs/compare/compare-{cfg}-cve_2021_38190.mir')
    # shutil.copy('mir_dump/cve_2021_38190._#1-{impl#0}-deserialize.-------.dump_mir..mir.cfg.dot', f'docs/compare/compare-{cfg}-cve_2021_38190.dot')

    compare[cfg] = {
        "time": t2 - t1,
        "passed": passed,
        "failed": failed,
        "ignored": ignored,
    }

print("Done with all tests")

print("Writing results to compare.yaml")

with open("docs/compare.yaml", "w") as f:
    yaml.safe_dump(compare, f)
print("Done writing results to compare.yaml")
