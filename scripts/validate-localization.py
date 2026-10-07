#!/usr/bin/env python3
"""Validate that English is canonical and translations map the same keys."""

from pathlib import Path
import re
import sys
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
APPLE_LINE = re.compile(r'^\s*"([^"]+)"\s*=\s*"(?:[^"\\]|\\.)*"\s*;\s*$')


def apple_keys(path: Path) -> set[str]:
    keys: set[str] = set()
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip() or line.lstrip().startswith("/*"):
            continue
        match = APPLE_LINE.match(line)
        if not match:
            raise ValueError(f"{path}:{number}: invalid .strings entry")
        key = match.group(1)
        if key in keys:
            raise ValueError(f"{path}:{number}: duplicate key {key}")
        keys.add(key)
    return keys


def android_keys(path: Path) -> set[str]:
    root = ET.parse(path).getroot()
    keys = [item.attrib["name"] for item in root.findall("string")]
    if len(keys) != len(set(keys)):
        raise ValueError(f"{path}: duplicate string name")
    return set(keys)


def compare(canonical: Path, translation: Path, parser) -> list[str]:
    base = parser(canonical)
    translated = parser(translation)
    errors: list[str] = []
    missing = sorted(base - translated)
    extra = sorted(translated - base)
    if missing:
        errors.append(f"{translation}: missing keys: {', '.join(missing)}")
    if extra:
        errors.append(f"{translation}: unknown keys: {', '.join(extra)}")
    return errors


def main() -> int:
    pairs = [
        (ROOT / "clients/ios/TEKtalk/en.lproj/Localizable.strings", ROOT / "clients/ios/TEKtalk/vi.lproj/Localizable.strings", apple_keys),
        (ROOT / "clients/macos/TEKtalkMac/en.lproj/Localizable.strings", ROOT / "clients/macos/TEKtalkMac/vi.lproj/Localizable.strings", apple_keys),
        (ROOT / "clients/android/app/src/main/res/values/strings.xml", ROOT / "clients/android/app/src/main/res/values-vi/strings.xml", android_keys),
    ]
    errors: list[str] = []
    try:
        for canonical, translation, parser in pairs:
            canonical_keys = parser(canonical)
            non_ascii = sorted(key for key in canonical_keys if not key.isascii())
            if non_ascii:
                errors.append(f"{canonical}: canonical keys must be ASCII: {', '.join(non_ascii)}")
            errors.extend(compare(canonical, translation, parser))
    except (OSError, ValueError, ET.ParseError) as error:
        errors.append(str(error))
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("Localization catalogs are aligned; English is canonical.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

