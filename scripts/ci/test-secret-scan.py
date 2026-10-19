#!/usr/bin/env python3
"""Ensure historical type declarations are the only ignored findings.

Uses an isolated temporary repo and synthetic values; never modifies this repo.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
GITLEAKS = os.environ.get("GITLEAKS_BIN", "gitleaks")
REVIEWED = {
    "71e6e72b9d2fab37d840025219701cd60292f260": "clients/ios/TEKtalk/TEKProtocol.swift",
    "50f2d4f6c4ea73d08dc1de6c76a4071a599ee398": "clients/ios/ZChat/ZProtocol.swift",
}
expected = {f"{commit}:{path}:generic-api-key:3" for commit, path in REVIEWED.items()}
actual = {
    line.strip() for line in (ROOT / ".gitleaksignore").read_text().splitlines()
    if line.strip() and not line.lstrip().startswith("#")
}
assert actual == expected, "Only the two reviewed, exact fingerprints may be ignored"
for commit, path in REVIEWED.items():
    source = subprocess.check_output(["git", "show", f"{commit}:{path}"], cwd=ROOT, text=True)
    # Hashes bind the review to the exact declaration without repeating a
    # scanner-triggering type annotation in this regression check itself.
    assert hashlib.sha256(source.splitlines()[2].encode()).hexdigest() in (
        '9292a0bb230a141520612e77c8cfa74aaa8cd2e46712deb3c93d9eda939fd122',
        '591390676d259f82babed4b1e04f7521fadccc71215c8b0b4d435bbceacea069',
    ), "Re-review changed historical source before adding any exception"


with tempfile.TemporaryDirectory(prefix="tektalk-secret-scan-") as directory:
    fixture = Path(directory)
    subprocess.run(["git", "init", "--quiet", directory], check=True)
    for key, value in (("user.name", "Secret scan regression"), ("user.email", "ci@example.invalid")):
        subprocess.run(["git", "-C", directory, "config", key, value], check=True)
    shutil.copyfile(ROOT / ".gitleaksignore", fixture / ".gitleaksignore")
    # A synthetic high-entropy value in the same files and line as the reviewed
    # declarations must still fail: the ignore list cannot hide future secrets.
    synthetic = hashlib.sha256(b"tektalk isolated scanner regression fixture").hexdigest()
    for path in REVIEWED.values():
        target = fixture / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text('import Foundation\nimport CryptoKit\nlet privateKey = "' + synthetic + '"\n')
    subprocess.run(["git", "-C", directory, "add", "."], check=True)
    subprocess.run(["git", "-C", directory, "commit", "--quiet", "-m", "Synthetic detection regression"], check=True)
    # Delete the synthetic secrets in HEAD; full-history scanning must still
    # find them. This catches accidental shallow/range-only scan regressions.
    for path in REVIEWED.values():
        (fixture / path).unlink()
    subprocess.run(["git", "-C", directory, "add", "-u"], check=True)
    subprocess.run(["git", "-C", directory, "commit", "--quiet", "-m", "Remove synthetic values"], check=True)
    report = fixture / "report.json"
    result = subprocess.run([
        GITLEAKS, "git", "--redact", "--log-opts=--full-history --all",
        "--report-format=json", f"--report-path={report}", ".",
    ], cwd=fixture, capture_output=True, text=True)
    assert result.returncode == 1, "Scanner must reject synthetic historical secrets: " + result.stderr
    findings = json.loads(report.read_text())
    found = {(finding["File"], finding["RuleID"], finding["StartLine"]) for finding in findings}
    assert found == {(path, "generic-api-key", 3) for path in REVIEWED.values()}, found

print("Secret scan regression passed: narrow exceptions; new and deleted secrets detected.")
