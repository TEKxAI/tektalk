#!/usr/bin/env python3
"""Validate accessible semantic color pairs in both appearance modes."""
import json
from pathlib import Path
import re

tokens = json.loads((Path(__file__).resolve().parents[1] / 'design-system/tokens.json').read_text())


def luminance(color):
    channels = [int(color[index:index + 2], 16) / 255 for index in (1, 3, 5)]
    channels = [c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in channels]
    return sum(c * weight for c, weight in zip(channels, (0.2126, 0.7152, 0.0722)))


pairs = [('text', 'surface'), ('text', 'background'), ('muted', 'surface'),
         ('muted', 'surfaceVariant'), ('onPrimary', 'primary'),
         ('onPrimaryContainer', 'primaryContainer'), ('error', 'surface'), ('success', 'surface')]
assert set(tokens['colors']) == {'light', 'dark'}
assert set(tokens['colors']['light']) == set(tokens['colors']['dark'])
for mode, colors in tokens['colors'].items():
    assert all(re.fullmatch(r'#[0-9A-F]{6}', color) for color in colors.values())
    for foreground, background in pairs:
        low, high = sorted([luminance(colors[foreground]), luminance(colors[background])])
        contrast = (high + 0.05) / (low + 0.05)
        assert contrast >= 4.5, f'{mode}: {foreground}/{background} contrast {contrast:.2f} fails 4.5:1'
        print(f'{mode}: {foreground}/{background} = {contrast:.2f}:1')
assert [tab['id'] for tab in tokens['navigation']] == ['message', 'ai', 'me']
assert tokens['layout']['touchTarget'] >= 44
print('Light/dark text contrast and shared navigation contract passed.')
