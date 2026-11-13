---
name: xgpui-lab-design
description: Layout and look rules for xgpui-lab demo screens (Color Spaces, Chromaticity & Gamut, Image Filters). Colors, type and controls come from the xgpui-luma look-radix Look; this document says which roles, steps and builders to use and how a lab page is laid out. Read before building or restyling a lab screen.
applies_to: apps/xgpui-lab/src/**
status: draft — screens are being implemented against this document
---

# XGPUI Lab Design

This document describes how a lab screen in `xgpui-luma/apps/xgpui-lab` is
laid out and styled. It is written so that a person can skim it and an LLM
agent can follow it step by step.

**The rule everything else rests on:** the lab is a look-radix app. Every
chrome color, type style and control comes from the shared `Look` that
`theme::default_look()` builds. This document picks roles, scale steps and
builders. It never defines a color of its own. The only literal colors on a
lab page are *data*: the colors a screen exists to show.

> **Keep the published copy in sync.** `docs/xgpui-lab-screen-design.html` (the
> docs site's *Lab Screen Design* page) summarizes this document by hand, and
> `docs/xgpui-lab.html` repeats the launch options from [launching.md](launching.md).
> When you change a rule, a value in `src/screen/metrics.rs` or a launch option,
> update those pages in the same change, or they drift.
>
> The *Spacing and shape* table below still names older helpers
> (`panel()`, `comparison_card()`); the current values are in
> `LabMetrics::STANDARD` and the docs page.

When this document and a reference image disagree, the document wins. The
mockup images were drawn before this rule was applied, so their hex values are
illustrations only.

## When to use this

Use it when you:

- add a demo screen to `xgpui-lab`;
- restyle a screen that still uses the older "buttons on top, status text below" layout;
- review a screen for consistency.

Don't use it for benchmark harnesses, headless tools, or the shell chrome.

## Reference screens

Paths are relative to `apps/xgpui-lab/`. Screens are built with `LabScreen` in `src/screen/` ([lab-screen-api.md](lab-screen-api.md)); [lab-ui.md](lab-ui.md) records the history of the earlier `lab_ui` helpers. This document is the rule set for new and restyled screens.

| Screen | Breadcrumb | Reference image | Implementation | State |
| --- | --- | --- | --- | --- |
| Color Spaces | `xgpui-color / Color Spaces` | [mockups/color-spaces-reference.webp](mockups/color-spaces-reference.webp) | [`src/color_demos/interactive_spaces.rs`](../src/color_demos/interactive_spaces.rs) | **Source of the design.** Copy it when in doubt. |
| Chromaticity & Gamut | `xgpui-gamut / Chromaticity` | [mockups/chromaticity-mockup.png](mockups/chromaticity-mockup.png) | [`src/color_demos/chromaticity.rs`](../src/color_demos/chromaticity.rs), hosted by [`src/color_gamut_page.rs`](../src/color_gamut_page.rs) | follows the design (`LabScreen`) |
| Conversions, Interpolation, Gamut Mapping, CPU Gradients | `xgpui-color / …` | — | [`src/color_lab/views.rs`](../src/color_lab/views.rs) (state and controls in [`src/color_lab.rs`](../src/color_lab.rs)) | follows the design (`LabScreen`). One page, four tabs: `Hero::compare` swatches (Conversions, Gamut Mapping) or `Hero::full` gradient bars (Interpolation, CPU Gradients); a strip under each bar marks samples outside sRGB (`OUT_OF_GAMUT`). On a wide-gamut window, swatches and the sampled bar are drawn as linear Display P3 through xgpui-gamut's `WidePainter` ([`src/color_lab/display.rs`](../src/color_lab/display.rs)), otherwise as sRGB previews; the caption says which. Reference rows that target sRGB stay sRGB. Gamut Mapping opens on Display P3 → sRGB and adds a Saturation parameter (OkLCh chroma ×0–2) that pushes the input out of gamut |
| Wide Color Fills | `xgpui-gamut / Wide Color Fills` | — | [`src/gamut/fills.rs`](../src/gamut/fills.rs) | follows the design (`LabScreen`). Paired native P3 and clipped sRGB sample columns; both show clipped sRGB in fallback mode. Window negotiation stays in a diagnostics panel |
| Wide Gradients, Gamut Mapping, Overlays | `xgpui-gamut / …` | — | [`src/gamut/labs.rs`](../src/gamut/labs.rs) | follows the design (`LabScreen`). Native gradient/reference heroes or an ordered compositing scene, retained SDK controls and shared presentation diagnostics. Scene data colors stay fixed in both themes |
| Italy Photo Comparison | `xgpui-color / Italy Photo Comparison` | — | [`src/color_demos/italy.rs`](../src/color_demos/italy.rs) | follows the design (`LabScreen`). Photo variant: `Hero::fixed` 3:2 photo up to 600 px with its caption under it and the control row centered; 200 px selectable photo cards in a section (see [lab-ui.md](lab-ui.md#supported-variations)) |
| Shell Lab, Desktop Preferences | `xgpui-shell / …` | — | [`src/shell_lab.rs`](../src/shell_lab.rs) | follows the design (`LabScreen`). Diagnostics variant: `PanelShell` panels and `properties` lists |
| Typography | `xgpui-shader / Typography` | [mockups/typography-mockup.html](mockups/typography-mockup.html) | [`src/shader/typography.rs`](../src/shader/typography.rs) | follows the design (`LabScreen`): stage hero, selectable sample cards, Motion and Logical layer panels. Sample colors follow the look's mode |
| Images, Textures & Mipmaps | `xgpui-media / …` | — | [`src/media_lab/image_screens.rs`](../src/media_lab/image_screens.rs) | follows the design (`LabScreen`). Retained image stage or atlas/media comparison, shared camera controls, upload status and sampling diagnostics. Visual snapshot review pending |
| Linear Adjustments, Float / P3 | `xgpui-media / …` | — | [`src/media_lab/adjustment_screens.rs`](../src/media_lab/adjustment_screens.rs) | `LabScreen` comparison heroes, synchronized numeric controls and a live linear-light chain. Float / P3 preserves native output with a labeled sRGB fallback. Visual snapshot review pending |
| Resource Limits, Video Frames | `xgpui-media / …` | — | [`src/media_lab/resource_screen.rs`](../src/media_lab/resource_screen.rs), [`src/media_lab/video_screen.rs`](../src/media_lab/video_screen.rs) | `LabScreen` heroes, retained SDK controls and diagnostics panels. Resource Limits restores the original media-lab per-frame upload count/byte policy, twelve persistent status chips and final previews. Video preserves immutable frame comparisons and bounded SDR blur. Visual snapshot review pending |
| Tiled Canvas, Photo Viewer | `xgpui-media / …` | — | [`src/media_lab/tiled.rs`](../src/media_lab/tiled.rs), [`src/photo_viewport.rs`](../src/photo_viewport.rs) | `LabScreen` canvas heroes that own scroll/pinch or drag, with retained source/cache diagnostics. Tiled blur stays bounded; Photo Viewer preserves physical-pixel 1:1 and native P3 negotiation. Visual snapshot review pending |
| Image Filters | `xgpui-media / Image Filters` | [mockups/image-filters-mockup.png](mockups/image-filters-mockup.png) | [`src/media_lab/filters_screen.rs`](../src/media_lab/filters_screen.rs) (state in [`src/media_lab.rs`](../src/media_lab.rs)) | follows the design (`LabScreen`). The mockup's "linear light" and "skipped steps" text is wrong for `ImageFilter`; the screen says what the code does |
| Spatial Canvas, Mandelbrot | `xgpui-canvas / …` | — | [`src/canvas_lab/screen.rs`](../src/canvas_lab/screen.rs) (one `Lab` in [`src/canvas_lab.rs`](../src/canvas_lab.rs) hosts both) | follows the design (`LabScreen`). Canvas variant: `Hero::full` interactive viewport, 560 px tall, that owns scroll and pinch; the map or preview floats in its bottom-right corner on `panel_background`. The scene and fractal keep their own dark data colors in both themes ([`src/canvas_lab/palette.rs`](../src/canvas_lab/palette.rs)) |

Shared building blocks already exist. Use them before you write new ones:

| File | Provides |
| --- | --- |
| [`src/theme.rs`](../src/theme.rs) | `default_look()`, `ColorVizLookExt` (`chrome()`, `interaction_colors()`, `typography_scale()`), `TypographyExt`, `icon_button()` |
| [`src/screen/`](../src/screen/) | `LabScreen` (slots), `Hero`, `LabCard`, panels (`ParametersPanel`, `PreviewPanel`, `ChainPanel`, `PanelShell`), `TextRole`, `readout`, `properties`, `LabMetrics`, `data` colors |
| [`src/lab_controls.rs`](../src/lab_controls.rs) | `Choice`, a labeled Radix `Selector` with stable numeric IDs |

## Page anatomy

Every lab screen is one `LabScreen`: a vertical scroll column with the parts in
this order. The type fixes the order, whatever order the slots are filled in. Leave out a part only if
the screen has nothing to put in it. Don't reorder them.

```
┌──────────────────────────────────────────────────────────────┐
│ 1  Breadcrumb        crate / Screen name                     │
│ 2  Title             Screen name                             │
│ 3  Lede              one sentence: what you compare or explore│
│ 4  Hero visual       the thing being studied, full width     │
│ 5  Caption line      live state of the hero in one line      │
│ 6  Control row       selectors · switches · (actions right)  │
│ 7  Card row          up to 4 equal cards: variant or status  │
│ 8  Panels            [ Parameters  wide ][ Preview  narrow ] │
│ 9  Note              how the controls relate (1–2 sentences) │
│ 10 Fine print        licenses, limits, what isn't measured   │
└──────────────────────────────────────────────────────────────┘
```

1. **Breadcrumb**: the crate that owns the feature, then ` / `, then the screen
   name. This tells a reader which extension the demo tests.
2. **Title**: the screen name only (`.title(..)`).
3. **Lede**: one sentence in the form "Compare X…" or "Explore X…" (`.lede(..)`).
4. **Hero visual**: the canvas, field, diagram or image comparison. It is the
   largest element. Two-way comparisons go side by side, each with its own
   header. A square diagram can have a narrow legend column to its right.
5. **Caption line**: one line of `caption` text separated by ` · `, restating
   what the hero shows with live values. It replaces loose status lines such as
   `Sweep false · Adjustment animation false`.
6. **Control row** (`.control(..)`, `.action(..)`): selectors first, then switches, then
   actions such as *Open image…*, *Reload* and *Reset view* pushed to the right.
7. **Card row**: equal-width cards (`LabCard`). Each is a variant being
   compared (sRGB / Display P3 / OKLCH / Rec2020) or a status (Image mode /
   Upload / Sample / Tint). If you have fewer than four, use the count you have.
8. **Panels** (`.panels(wide, narrow)`): a wide *Parameters* panel with editable values
   and a narrow *Preview* (or *Chain*, *Details*) panel that shows their result.
9. **Note**: plain `caption` text on how the controls and the hero relate.
10. **Fine print**: data sources and licenses (for example the CIE locus credit),
    and what the screen does **not** measure.

## Color

### Where colors come from

The lab's `Look` is Radix with a custom palette (`theme::default_look()`):

- accent palette `Gray` and gray palette `Sand`, both re-seeded from one custom
  neutral (`#5A6169`), so accent and gray are the same cool neutral;
- background seed black in dark mode and `#F0F0F0` in light mode;
- dark mode by default, with light mode supported. **Every screen must work in both.**

The palette is deliberately neutral, so a lab screen has no brand accent. The
subject (swatches, gradients, the hero) supplies the color.

### Chrome roles

Read chrome colors through `look.chrome()` or `look.resolve_role(..)`:

| Use | API | Radix step |
| --- | --- | --- |
| Page background | `chrome().content_background` | `Background`, gray 1 |
| Cards, panels, tab strip | `chrome().panel_background` | `Surface`, gray 2 |
| Card, panel and divider borders | `chrome().border` | `Border`, gray 6 |
| Titles, body text, values | `chrome().body_text` / `title_text` | `Foreground`, gray 12 |
| Lede, captions, subtitles, labels | `chrome().muted_text` | `MutedForeground`, gray 11 |
| Selected card, active row | `interaction_colors().selected` | gray 3 |
| Hovered / focused card | `interaction_colors().hovered` | gray 4 |
| Pressed card | `interaction_colors().pressed` | gray 5 |
| Focus ring | `resolve_role(SemanticRole::Focus)` | color 8 |

For anything else, take a gray step with
`look.resolve_step(ScaleFamily::Gray, n)` and use it the way the Radix
scale defines:

| Steps | Use |
| --- | --- |
| 1–2 | backgrounds |
| 3–5 | component fills: normal, hover, pressed |
| 6–8 | borders: subtle, normal, strong or focus |
| 9–10 | solid fills, scrollbar thumb (7 / 9 / 10 is already used by `theme.rs`) |
| 11 | low-contrast text: grid labels, axis ticks, fine print |
| 12 | high-contrast text |

Rules:

- No `rgb(0x…)` or `hsla(..)` for chrome. If a screen needs a color that isn't
  a role or a step, add the role to look-radix (`SemanticRole`) or to
  `ColorVizLookExt` in `theme.rs`, and document it here.
- Use the same role for the same job on every screen. For example, every card
  border is `chrome().border`.
- Use Radix alpha steps where something sits over the hero, such as a legend
  over a canvas, so it reads on any content.

### Badges

The lab doesn't standardize badges. A card or comparison pane has a badge
slot, and the screen passes whatever element suits it, usually a look-radix
`Badge` with whatever variant and tone it chooses. The layout only places it.

Two rules still apply:

- keep badge text short so it never wraps or pushes the name onto a second line;
- the text carries the meaning, so a badge must still make sense to someone
  who can't tell its colors apart.

### Data colors (the one exception)

Colors that are the *subject* of a screen aren't chrome, and they don't come
from the Look:

- swatches, gradient slider tracks, the field and diagram fill: computed from
  the color math (`xgpui-color`) for the current values;
- **gamut outlines**: one fixed color per gamut, the same on every screen that
  plots gamuts. Define them once (for example `chromaticity.rs` `COLORS`) and
  reuse them. Check that each still reads on both gray 1 backgrounds;
- **markers**: a 2 px cross over a 4 px halo, drawn the same in the diagram and
  its legend. The reference point (D65) uses `Foreground` with a `Background`
  halo. The subject point uses one fixed marker color (`MARKER_SUBJECT`) with a
  fixed dark halo (`MARKER_SUBJECT_HALO`), so the light gold reads on light pages
  and bright fills. Diagram grid and outline strokes are chrome: `Border` and
  `MutedForeground`.
- **out-of-gamut strips**: the 4 px strip under a gradient bar marks samples outside
  the target gamut in `OUT_OF_GAMUT` (amber); samples inside use the chrome `Border`.

Keep data colors in one module per screen family, named for what they mean
(`GAMUT_SRGB`, `MARKER_SOURCE`), not for how they look.

## Type

Type comes from the Look's typography tokens
(`look.common_stylesheet().tokens.typography`), through the `TextRole`s in
`src/screen/metrics.rs`. A screen picks a role; it never sets a size, weight or
font family itself. Change type in the look, or change a role's token here,
and every screen follows.

| Role | Use | Look token (Luma default) | Color |
| --- | --- | --- | --- |
| `Title` | screen title | `role.h2` (28 / 36, semibold) | `Foreground` |
| `Section` | panel title | `role.h3` (20 / 28, semibold) | `Foreground` |
| `Name` | card name, compare-pane title | `role.h4` (16 / 22, medium) | `Foreground` |
| `Crumb`, `Lede`, `Caption` | breadcrumb, lede, live state, notices, notes, panel subtitles | `role.p` (14 / 20) | `MutedForeground` |
| `Field` | parameter label above its text field (`Red`, `Contrast`) | `role.p` (14 / 20), the panel subtitle's size | `Foreground` |
| `Label` | keys and small labels: `R`, `x`, `u′`, `Y`, `Clip`, `Source`, legend notes | `label` (12.5 / 18, medium) | `MutedForeground` |
| `Detail` | small words: legend names, chain steps | `label` (12.5 / 18, medium) | `Foreground` |
| `Value` | measured or edited values | `code` (13 / 18) in `font.mono` | `Foreground` |
| `Fine` | credits and provenance | `caption` (11 / 16, medium) | `MutedForeground` |
| Control labels | `screen::labeled_control`, which applies the Radix switch label typography | control | control |

### UI face or mono

Lab screens are technical presentations, and the face says what a piece of text is:

- **UI face for words**: names, labels, keys, sentences and control text.
- **Mono for values**: channel values, coordinates, hex, percentages in a
  legend or table, and diagram axis ticks. Mono digits all have the same width,
  so values don't shift sideways while they change, and they line up across rows.
- **A value has a key**: the key in `Label` (UI, muted) and the value in
  `Value` (mono, foreground), e.g. `R 0.797`, `x 0.2650`, `Clip #00FF00`.
- **One size for values**: the look's `code` style everywhere, whether a value
  is shown on a card, in a legend, in a preview or in a text field. The one
  exception is diagram axis ticks, at 10 px (`LabMetrics::axis_tick`) mono in
  `MutedForeground`, because they annotate the plot rather than report a value.
- **Numbers inside a sentence stay in the UI face**, e.g. the live caption
  `fill Y 0.18 · … 67.1%`. They are part of the sentence, not a readout.
- **One mono family**: the look's `font.mono`, through `screen::mono_family(look)`.
  Never write a family name in a screen. GPUI passes family names to the OS
  as-is, so a name that isn't installed (including the CSS generic
  `"monospace"`) silently falls back to the UI font. Luma's default `Monaco`
  exists only on macOS, so the lab bundles **IBM Plex Mono** (OFL) in
  `assets/fonts/ibm-plex-mono/`, registers it at startup
  (`theme::register_fonts`), and sets it as the look's `font.mono` in
  `theme::default_look()`.
- **Key/value rows** go through `screen::readout(look, pairs)`, which is what
  cards, preview captions and reference lines use.
- Text fields that edit a value use the same mono size as the value they edit
  (`screen::value_field_look`).

Show hierarchy with size and the `Foreground` / `MutedForeground` split. Use
heavier weight only where the roles already do (title, section, name).

- Fix decimals per quantity: encoded channels `0.000`, filter factors `0.00`,
  chromaticity `0.0000`, angles `0.0°`, hex in upper case (`#DB5294`).
- Put units on values: `16 px`, `8.00 MiB`, `1536×1024`.

## Spacing and shape

### Desktop density

The lab is for desktop computers, which almost always have landscape monitors.
Vertical space is the scarce dimension, so avoid vertical scrolling that isn't needed:

- **A heading and its subtitle are one unit.** Stack a title and its subtitle with
  no gap between them; their line heights give enough separation. This applies to
  panel headings (`ParametersPanel`, `PreviewPanel`) and card name/subtitle pairs.
  The panel's `gap_3` separates the heading from the panel body, not the two heading lines.
- **Spend width before height.** Prefer rows that wrap (cards, panels, parameters,
  swatch pairs) over stacking things vertically.
- **Don't add vertical space for decoration.** Use spacers and gaps only where
  they separate different groups.

Use the values the shared helpers already set rather than new numbers:

| Thing | Value |
| --- | --- |
| Content column | `container()`: max width 1240, centered, `p_6`, `gap_5` between parts |
| Control row | `control_row()`: wraps, `gap_3` |
| Panel | `panel()`: `p_4`, `gap_3`, `rounded_lg`, 1 px `border` |
| Card | `comparison_card()`: `p_3`, `gap_3`, `rounded_lg`, 1 px `border` |
| Card row / panel row gaps | `gap_4` / `gap_5` |
| Control radius, heights, focus ring | the Look's metrics (`look.metrics()`), via the builders below |

## Controls

Use the look-radix builders, passing `.look(look)`. Don't draw controls by hand.

| Need | Builder | Notes |
| --- | --- | --- |
| Pick one of several | `lab_controls::Choice` (Radix `Selector`) | Label to the left, in the control row |
| Boolean that changes the hero | Radix `Switch` | Never show a boolean as text like `Sweep false` |
| Action | Radix `Button` (`icon_button` for icon-only) | Right end of the control row, or in a panel header when it acts on that panel only (*Reset filters*) |
| Numeric value | Radix `TextField` + `Slider` | Kept in sync. Typing a value outside the range doesn't move the thumb |
| Color channel | `xgpui_luma_color::color_slider` (`ColorSliderBuilder`, `GradientDelegate`) | The track shows the channel's real color sweep, as in Color Spaces |
| Badge | any element, usually a Radix `Badge` | See *Badges* |
| Sub-pages | SDK `Tabs` with `studio_tabs::template`, as in `color_gamut_page.rs` | Tab strip on `panel_background` with a bottom `border` |

## Components

### Variant / status card

```
┌───────────────────────────────────────┐
│ ██  Name                       [Tag]  │  swatch or icon · name · Badge
│ ██  Subtitle                          │  muted_text: standard, transfer, role
│                                       │
│ R 0.797  G 0.356  B 0.573             │  mono values; keys in muted_text
└───────────────────────────────────────┘
```

- Start from `comparison_card(look)` or `comparison_surface(button_row, look)`.
- The swatch shows the actual color (a data color). For a status card, use a
  plain 32 px line icon (`LabCard::icon`, `card_icon`) in `Foreground`, with no
  box, fill or border, so it never reads as a button.
- Put a small square of the gamut's data color before the name when the card
  stands for a gamut that also appears in the hero.
- Name and subtitle stay on one line. Truncate rather than wrap.
- Selected, hovered and pressed fills come from `interaction_colors().background(selected, state)`.

### Settings form

For many selectors that each override a default (Shell Lab's *Preview overrides*),
don't wrap label/selector pairs into one long row. Use `screen::form_groups` of
`screen::form_group`s:

- **Group by what the settings affect**, as titled columns (*Accessibility*,
  *Text*, *Titlebar actions*), side by side, wrapping when narrow.
- **Align**: labels in one column (`form_label_width`), selectors in the next.
  Labels stay on one line, so prefer short platform wording (*Reduce motion*).
- **Show state**: a row still on its default has a muted label, and an
  overridden row has a foreground label. The panel subtitle counts the overrides
  ("2 of 8 overridden · the rest follow the desktop").
- **Reset** sits in the panel heading as an outline button, disabled when nothing
  differs from the defaults.

### Parameter

A label in `Foreground`, an optional range in `MutedForeground`, a `TextField`,
then a slider. For a color channel, the color slider track shows the real
sweep. For a filter factor, the track shows the effect where that is cheap
to draw (black→white for brightness, the hue wheel for hue rotation).
Otherwise use the plain Radix slider. When a parameter has a neutral value
(1.0 for factors, 0 for rotation), mark it so that changed values stand out.
Lay parameters out in a 3-column grid inside the panel.

### Preview panel

One large result swatch, or a pair of them (for example *Clip* | *Reduce
chroma*), with a `caption` under each and the value in mono. A reference line
ends the panel (`sRGB reference #DB5294`, or the source xy / u′v′ / Y).

### Chain panel

For an ordered pipeline (filters, transforms), list its steps as numbered
rows, with the name on the left and the formula or value on the right in mono.
Active rows use `Foreground` on `interaction_colors().selected`, and rows left
at their neutral value use `MutedForeground`. Separate rows with `border`. End
with a count line (`2 of 7 active`).

### Comparison panes (hero)

Two panes side by side, each with a header row
`Name · muted subtitle · Badge (right)`. Each image sits on a gray 3 stage, so
that rounded corners and transparency show in both modes. Both panes draw the
same source at the same size, and the caption line states that size and scale.

### Diagram

- Grid lines on gray 4–6, axis ticks and axis names in gray 11.
- The outline of the locus or plotted shape is gray 11–12.
- Series (gamuts) use their data colors. The selected series gets a heavier
  stroke than the others, so don't change its color.
- The legend goes to the right of a square diagram, or under it in a narrow
  window. Explain markers in the legend, not only in the fine print.

## Writing rules

- Name things the way the reader recognizes them: *Upload*, *Sample*, *Tint*,
  not internal type names.
- Status reads as `Thing · value · value`, not `Thing: value, Flag false`.
- The fine print states limits plainly: what is a preview, what is simulated,
  and what is not measured.
- Don't use emoji, and don't put exclamation marks in UI text.

## Keeping it in the lab

These helpers and data colors belong in `xgpui-lab`. A missing color role or
control variant belongs in look-radix, not in GPUI core. Treat any addition to
a published crate's public API (look-radix, the Luma SDK, xgpui) as an API
change that needs its own review.

## Review checklist

Before you call a screen done, check that:

- [ ] the screen is a `LabScreen`, so the parts follow the order in *Page anatomy*;
- [ ] no chrome color is a literal: everything is `chrome()`, `interaction_colors()`, `resolve_role` or `resolve_step`;
- [ ] data colors live in one named module, and gamut colors match the other screens;
- [ ] the screen looks right in **both** light and dark mode;
- [ ] snapshots before and after the change compare as expected (`scripts/lab_snapshots.py`, see [launching.md](launching.md));
- [ ] controls are look-radix builders: selectors, switches (no text booleans), buttons, text fields, sliders, badges;
- [ ] the hero is the largest element, and the caption line under it shows live values;
- [ ] cards are equal width, and no name, subtitle or badge wraps;
- [ ] numbers use the fixed decimals and units, in mono or tabular figures;
- [ ] the fine print credits external data and names what is not measured;
- [ ] the reference image in `docs/mockups/` matches the shipped screen, or has been updated.
