# TEKtalk Design System

`tokens.json` is the source of truth for semantic colors, spacing, radii, layout
dimensions and the Message → AI → Me navigation contract. Change tokens here,
then run `python3 scripts/generate-design-system.py` and commit its outputs.
CI rejects stale generated resources and text contrast below 4.5:1.

| Semantic role | Light | Dark |
| --- | --- | --- |
| Primary / brand | `#0089FF` | `#63B5FF` |
| Text on primary | `#001B2B` | `#001B2B` |
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
desktop sidebar width is 320. Desktop hosts use exactly two panes. The left
pane shows chat history for Message, or the corresponding AI / Me section.
The three Mobile tabs are pinned in one horizontal row at the bottom of that
pane; selected conversation / destination details fill the right pane.
Changing tabs preserves chat history and unsent drafts. Native controls may apply their platform's own metrics
where necessary for accessibility.

## Platform adapters

- iOS/macOS: generated named color assets supply light/dark variants; SwiftUI
  `TekDesign` exposes the colors and dimensions. System appearance updates
  automatically. Compact iPhone uses bottom tabs; regular iPad and macOS use
  a left panel. macOS pins its horizontal tab row below the destination sidebar
  content. Primary buttons explicitly use the same primary/onPrimary pair as
  Android, rather than the OS default button foreground.
- Android: generated light/dark Material 3 schemes and tab vector assets.
  Compact windows use bottom navigation; widths of 820 dp or more use a rail.
- Windows: generated WinUI theme resources and a left panel with chat history
  above a pinned bottom tab row. The content appears on the right and survives
  tab switches.
- Web: generated CSS custom properties; native system font stack, keyboard
  focus, reduced motion and safe-area handling. A left panel appears above
  820 CSS pixels, with history and bottom tabs in that same pane (no third rail);
  compact layouts use full-width bottom tabs. Theme cycles system / light /
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
