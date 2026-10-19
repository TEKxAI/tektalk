# TEKtalk Design System

`tokens.json` is the source of truth for semantic colors, spacing, radii, layout
dimensions and the Message → AI → Me navigation contract. Change tokens here,
then run `python3 scripts/generate-design-system.py` and commit its outputs.
CI rejects stale generated resources and text contrast below 4.5:1.

| Semantic role | Light | Dark |
| --- | --- | --- |
| Primary / brand | `#6558F5` | `#B1A6FF` |
| Text on primary | `#FFFFFF` | `#221A57` |
| Background | `#F5F6FB` | `#11121A` |
| Surface | `#FFFFFF` | `#191A24` |
| Surface variant | `#EEEFF7` | `#252632` |
| Main text | `#20202A` | `#F4F3FA` |
| Secondary text | `#616477` | `#B1B4C6` |
| Error | `#C63447` | `#FFB1BC` |
| Success | `#147B50` | `#73DBAE` |

Use semantic names in UI code. Decorative avatar gradients may vary; text,
navigation, chat bubbles, actions and status colors use semantic roles. Native
fonts, Dynamic Type, control behavior, keyboard shortcuts and focus remain
platform conventions. Do not introduce a custom font that defeats text scaling.

The spacing scale is 4 / 8 / 12 / 16 / 24 / 32 / 48 logical units. Control,
bubble and card radii are 12 / 18 / 24. Navigation targets are at least 48 units;
desktop sidebar width is 224. Native controls may apply their platform's own
metrics where necessary for accessibility.

## Platform adapters

- iOS/macOS: generated named color assets supply light/dark variants; SwiftUI
  `TekDesign` exposes the colors and dimensions. System appearance updates
  automatically. Compact iPhone uses bottom tabs; regular iPad and macOS use
  a left navigation panel and right content with the same three destinations.
- Android: generated light/dark Material 3 schemes and tab vector assets.
  Compact windows use bottom navigation; widths of 820 dp or more use a rail.
- Windows: generated WinUI theme resources and a native left NavigationView.
  No extra Settings destination is inserted. The content appears on the right.
- Web: generated CSS custom properties; native system font stack, keyboard
  focus, reduced motion and safe-area handling. A left panel appears above
  820 CSS pixels; compact layouts use bottom tabs. Theme cycles system / light /
  dark and follows system appearance changes while system mode is selected.

## Verification

```sh
python3 scripts/generate-design-system.py --check
python3 scripts/test-design-system.py
```

CI builds all five clients. Visual smoke runs registration, login, device
verification and bidirectional chat, and exports actual screen recordings.
The Web appearance checks cover both themes, desktop/mobile navigation and
English/Vietnamese labels. Validate native appearance on the target OS; passing
token generation alone does not prove that a control renders correctly.
