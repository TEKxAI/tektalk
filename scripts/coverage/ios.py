#!/usr/bin/env python3
"""Check coverage of testable iOS production logic from an xccov JSON report."""

import json
import os
import sys

minimum = float(os.environ.get("IOS_COVERAGE_MINIMUM", "0.60"))
excluded_suffixes = (
    "/ContentView.swift",       # declarative UI shell
    "/PluginContract.swift",    # protocol declarations have no executable logic
)

report = json.load(sys.stdin)
files = []
for target in report.get("targets", []):
    name = target.get("name", "")
    if name not in ("TEKtalk", "TEKtalk.app"):
        continue
    for item in target.get("files", []):
        path = item.get("path", "")
        if not path.endswith(excluded_suffixes):
            files.append(item)

executable = sum(int(item.get("executableLines", 0)) for item in files)
covered = sum(int(item.get("coveredLines", 0)) for item in files)
if executable == 0:
    print("No executable iOS production lines were found in the coverage report.", file=sys.stderr)
    sys.exit(2)

ratio = covered / executable
print(f"iOS testable logic line coverage: {ratio:.2%} ({covered}/{executable}); required: {minimum:.2%}")
if ratio + 1e-12 < minimum:
    sys.exit(1)
