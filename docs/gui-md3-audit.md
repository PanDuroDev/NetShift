# Material 3 Conformance Audit

Date: 2026-10-31

This audit compares the desktop prototype with:

1. The project design specification in issue #7.
2. The official Material Design 3 guidance at <https://m3.material.io/>.
3. The implementation that existed before the conformance branch.

The result is a focused correction, not a claim of complete Material 3 Expressive
compliance. Expressive shape morphing, spring motion, and a production component
library are still out of scope for this prototype.

## Color interpretation

The requested palette is a monochrome pair: `#111111` and `#eaeaea`, with the
light theme inverted. In Material 3, color roles must contrast with the surface
they sit on. A literal `primary = #111111` on a `surface = #111111` dark theme
would be invisible and would fail the contrast requirement.

The implementation therefore treats `#111111` as the neutral seed and dark
surface, and `#eaeaea` as its inverse tone:

| Role | Dark theme | Light theme |
| --- | --- | --- |
| `surface` | `#111111` | `#eaeaea` |
| `on-surface` | `#eaeaea` | `#111111` |
| `primary` | `#eaeaea` | `#111111` |
| `on-primary` | `#111111` | `#eaeaea` |
| `primary-container` | `#2b2b2b` | `#d6d6d6` |
| `secondary-container` | `#2b2b2b` | `#e0e0e0` |
| `outline` | `#8a8a8a` | `#6f6f6f` |
| `outline-variant` | `#3a3a3a` | `#b8b8b8` |

Tonal surface containers step from `#0c0c0c` to `#2e2e2e` in dark mode and from
`#ffffff` to `#dedede` in light mode. Success, warning, and error keep semantic
container/on-container pairs. This preserves the requested base pair while
keeping every role usable.

## Findings

| Area | Project rule | Official MD3 guidance | Before | Resolution |
| --- | --- | --- | --- | --- |
| Color roles | Centralized light/dark tokens, semantic error/success/warning | Primary, on-primary, containers, surface containers, outline, inverse roles | Cyan accent with ad-hoc `surface`, `surface-raised`, `surface-high` names | Replaced with MD3 role tokens; monochrome `#111111`/`#eaeaea` pair; no cyan |
| Typography | 12 px minimum; display 32, screen 28, section 22, card 18, body 14, label 12 | Display/headline/title/body/label roles with fixed line heights | Many 8-11 px labels, 25-32 px fluid headings, inconsistent weights | Tokenized type scale; all essential text at least 12 px; fixed 28/36 screen title and 22/28 section title |
| Spacing | 4 px base, documented scale | 4 px grid | Mixed 5, 7, 9, 11, 13, 15, 17, 18, 19, 21, 22, 23, 25, 26, 28, 34 px values | Normalized to 4/8/12/16/24/32 px |
| Shape | 8 small, 12 medium, 16 large, 24 extra-large, pills | Shape scale; buttons full; cards 12; dialogs 28 | 10/14/18/22/26 px radii, 7 px chips, 99 px pills everywhere | Shape tokens: 4/8/12/16/28/full; cards 12; fields 8; dialogs 28; buttons full or 20 |
| Elevation | Tonal surfaces first, subtle shadows | Tonal surface roles replace elevation for most surfaces | Arbitrary `0 2px 6px` cards and `0 18px 48px` dialog | Removed card shadows; one level-3 dialog shadow; tonal containers |
| State layers | Hover, focus, pressed, disabled defined | Hover 8%, focus 10%, pressed 10%, dragged 16% | Hover background swap, pressed `scale(0.98)`, disabled opacity/filter | Current-color overlays at 8/10/10%; disabled 38%; no scale |
| Motion | 100-150 ms feedback, 180-250 ms larger transitions | Short/medium duration tokens and standard/emphasized easing | 130-140 ms `ease`; no tokens | Duration and easing tokens; 100/150/300/400 ms; reduced-motion override |
| Buttons | Primary 48/24/min 96; secondary and outlined 40/16/min 80 | Default 40 dp, full shape; 24 dp icon; state layers | 36-40 px pills, 11 px labels, no primary size distinction | Primary and hero 48/min 96/pill; secondary, tonal, outlined 40/min 80/20 radius; 14 px labels |
| Icon buttons | 40 px minimum, 24 px icon, accessible name and tooltip | Default 40 dp; 48 dp preferred target; 24 dp icon | 27-38 px targets with 14-18 px icons | 40 px targets, 20-24 px icons, 500 ms tooltips |
| Chips | Not specified; use MD3 | 32 dp height, 8 dp radius, 18 dp icon, 8 dp padding with icon | 23 px status pills; 7 px capability chips; 12 px text | 32 px chips, 8 px radius, 18 px icons, 8 px padding and gap |
| Cards | 16 radius, 20 padding | 12 radius, 16 padding, tonal variants | 18 radius, 16-21 padding, arbitrary shadow | 12 radius, 16 padding, outlined tonal surface |
| Dialogs | 480 default, max 560, 24 padding, 24 radius | 28 radius, 280-560 width, 24 padding, 16 title gap, 24 action gap | 440 width, 22 radius, 25 padding | 560 max, 28 radius, 24 padding, 16/24 gaps |
| Navigation | 240 expanded, 80 compact, 48 items, tonal selected | Rail 80; active indicator 56x32; drawer item 56; tonal indicator | 240/80/72 sidebar; 3 px left active bar; cyan active text | 240/80/72 shell; 56 px items; full tonal active container; no left bar |
| App bar | 64 high; 40 icon buttons; 8 gaps | 64 dp; surface/on-surface; 40 dp actions | 64 high, 38 search and icons, translucent blur | 64 high, 44 search, 40 icons, 8 gap, opaque surface |
| Tooltips | 500 ms; not the only discovery path | 500 ms delay; hover and focus | Native `title` attributes only | `data-tooltip` with 500 ms delay and focus support; labels remain visible when expanded |
| Focus | 2 px outline, 2 px offset | 2 px focus indicator | 2 px outline with 3 px offset | 2 px outline, 2 px offset |
| Responsive | 1440/1280/1024/960; no horizontal scroll | Adaptive layout | Breakpoints present but many fixed tiny sizes | Same breakpoints; 4/2/1 column collapse; compact rail tooltips; verified at four sizes |

## Intentional project-specific choices

These are deliberate desktop adaptations. They are not presented as exact
Material 3 component measurements:

- The expanded sidebar is 240 px, not the 360 px navigation drawer. This is a
  desktop utility shell, and the narrower rail leaves more room for network data.
- Primary and hero buttons are 48 px high to match the project's emphasis rule;
  standard MD3 buttons are 40 px.
- Display and title weights are 600 instead of the 400/500 baseline weights, to
  match the project's typography specification.
- Cards use the official 12 px radius and 16 px padding rather than the earlier
  project proposal of 16 px and 20 px.
- Status tags are compact badges, not interactive chips; capability tags follow
  the MD3 chip geometry.
- The prototype has no production component library, so Expressive shape morph,
  spring motion, and predictive back are not implemented.

## Verification

Automated checks cover:

- TypeScript compilation and Vite production build.
- English-only LTR declarations and representative fixture values.
- Responsive breakpoints and the 960 x 640 minimum window.
- MD3 token presence, monochrome palette, state-layer opacities, shape tokens,
  motion tokens, 12 px minimum essential text, and tooltip attributes.
- No Tauri invoke, fetch, or XHR calls.

Manual validation still covers native window rendering on Windows and Linux,
native clipboard round trips, screen-reader behavior, keyboard focus order,
dialog focus trapping, contrast inspection, and the four reference viewports.

## Sources

- Material 3 color roles: <https://m3.material.io/styles/color/roles>
- Material 3 type scale: <https://m3.material.io/styles/typography/type-scale-tokens>
- Material 3 motion tokens: <https://m3.material.io/styles/motion/easing-and-duration/tokens-specs>
- Material 3 state layers: <https://m3.material.io/foundations/interaction/states/applying-states>
- Buttons: <https://m3.material.io/components/buttons/specs>
- Icon buttons: <https://m3.material.io/components/icon-buttons/specs>
- Chips: <https://m3.material.io/components/chips/specs>
- Cards: <https://m3.material.io/components/cards/specs>
- Dialogs: <https://m3.material.io/components/dialogs/specs>
- Navigation rail: <https://m3.material.io/components/navigation-rail/specs>
- Navigation drawer: <https://m3.material.io/components/navigation-drawer/specs>
- App bars: <https://m3.material.io/components/app-bars/specs>
- Project specification: issue #7, "GUI Stage 1: Research the Desktop UI Stack
  and Define a Material 3 Expressive Interface"
