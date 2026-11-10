# LabScreen: proposed screen layout API

Status: **migration complete** (steps 1–6): Color Spaces, Chromaticity, Italy, Image Filters and Shell Lab render through `src/screen/`, and `lab_ui.rs` is removed. Other lab pages are out of scope until they are prepared.
Later sections are still proposals; *Implemented so far* below records where
the code differs from them.

Related: [XGPUI-LAB-DESIGN.md](XGPUI-LAB-DESIGN.md) (the rules),
[lab-ui.md](lab-ui.md) (the current helpers and their history).

## Implemented so far

`src/screen/` exists and Color Spaces renders through it. Headless layout
bounds of every Color Spaces section (crumb, header, field, caption, controls,
each card, both panels, note) are identical before and after at 1280 and 720 px.
During the migration `lab_ui.rs` forwarded to `screen/` for Italy and Shell Lab;
it was removed once they migrated.

Chromaticity (step 3) now renders through `LabScreen` too, and its drift is fixed:
1240 / p6 page, `crumb()` separator, `TextRole` title and type, standard card
and panel geometry, and the axis labels use `muted_text` instead of a literal.
Its legend coverage now comes from `chromaticity_model::coverage` (primaries
over the closed CIE locus): 34 / 45 / 45 / 63 %, where the old hard-coded values
were 34 / 46 / 45 / 64 %. The status tag is a look-radix `Badge`: `Soft` for
"In" and `Outline` for "Out", because the Radix tones are accent and gray only.
Gamut and marker colors live in `screen/data.rs`, and `chromaticity_model::COLORS`
re-exports them. The chart shader now takes its grid (`Border`), locus outline
(`MutedForeground`) and D65 marker (`Foreground` on a `Background` halo) from
the look. The gold source marker has a fixed dark halo (`MARKER_SUBJECT_HALO`),
so light mode reads correctly. Color Spaces screenshots are still byte-identical.

Image Filters (step 5) renders through `LabScreen` in `src/media_lab/filters_screen.rs`;
the other media tabs keep their own views. It added:

- `Hero::compare(ComparePane, ComparePane)`. A pane's title, subtitle and badge
  share one heading line, and the visual sits in a card surface. The comparison
  viewports fill their pane width (measured on the previous frame) at 3:2.
- `LabCard::icon(lucide Icon)` for status cards: a plain 32 px line icon (`card_icon`),
  with no box or border, so it doesn't read as a button.
- `LabScreen::action(..)`: buttons kept together at the right end of the control row.
- `ParametersPanel::action(..)`, a header control (*Reset filters*), and
  `Param::range(..)`, the accepted range shown at the right of the label.
  Parameters lay out in at most three columns.
- `ChainPanel` with `.step(name, detail, active)` and `.note(..)`. It renders
  numbered rows (inactive steps muted, details in mono) and generates the
  "N of M steps active" count.
- `FluentBuilder` on every builder (`ComparePane`, `Legend`, the panels).
- Value text fields for the six filter parameters, synced both ways with their sliders.

Where it differs from the mockup:

- The mockup's credits say filters run in linear light and that identity steps
  are skipped. `ImageFilter` composes all steps into one affine matrix in
  **encoded sRGB**, and steps are never skipped (an identity step just has no
  effect). The screen states what the code does.
- The atlas pane subtitle is "Baseline · single level", without "bilinear",
  which isn't verified here. The *Aliasing* and *Filtered* badges show only
  while the image is downscaled (scale < 1×), when they're true.
- *Animate adjustments* is left out: it drives the linear-adjustments path,
  not these filters. *Sweep zoom* is a switch.
- Badges sit right after the card name (the `LabCard` rule), not at the far right.
- Slider tracks are the plain Radix slider. The mockup's gradient tracks and
  identity ticks are not built yet.

Italy (step 4) and Shell Lab render through `LabScreen` too. They added:

- `Hero::fixed(visual, max_width)`. The visual is centered at up to `max_width`,
  the screen's caption sits under it in the same column, and the control row
  centers. Italy's 600 px 3:2 photo uses it.
- `LabScreen::section(element)` and `.section_panel(panel)`: full-width sections
  after the panels. Italy's photo cards stay Luma buttons, so they go in a
  section rather than through `LabCard`. Shell Lab's *Recent events* is a section panel.
- `PanelShell`: a panel with the standard heading, an optional action and footer,
  and a free body (`.child(..)`).
- `screen::properties(look, rows)`: a key column in `Label` with values in mono
  `Value`. Keys align with the first line of a wrapping value.
- `LabMetrics::photo_card_width`, `photo_thumbnail` and `property_key_width`.
- `screen::form_group(look, title, rows)` with `FormRow::new(label, control, set)`,
  and `form_groups(..)` to lay groups side by side: a settings form with aligned
  labels, muted until a row is set. `Choice::label()` exposes a selector's label for it.

Typography (xgpui-shader) joins `LabScreen` too, which answers open question 1
(the eyebrows, the display line and the 1040 px column are gone). It added:

- `Hero::stage(ComparePane)`: one full-width pane with a heading and badges over a surface.
- `screen::card_button(look, id, CardWidth, selected, content, cx, on_click)`: a
  selectable card that stays a Luma button (input, focus, accessibility), filled
  from the look's interaction colors. Italy's photo cards and Typography's sample
  cards both use it. `CardWidth::Fixed` is Italy's 200 px; `Flexible` fills a host.
- `LabScreen::card(element)`: hosts a card-like element in the card row, sized like a `LabCard`.
- `LabCard::contents(look)` (a card without its surface, for `card_button`) and
  `LabCard::body(element)` (free content between the heading and the values).

Spatial Canvas and Mandelbrot (xgpui-canvas) and the four xgpui-color tabs
(Conversions, Interpolation, Gamut Mapping, CPU Gradients) join `LabScreen` too.
They add no new `screen/` types beyond `data::OUT_OF_GAMUT`:

- **Canvas pages** use `Hero::full` with a fixed-height, interactive viewport that owns
  scroll and pinch. The map overlay floats inside the hero with look chrome. The scene and
  the fractal keep their own dark data colors in both themes (`canvas_lab/palette.rs`),
  because they are the subject, like a photo. Each page has its own navigation entry, so
  the old F page-swap key is gone.
- **Color pages** draw swatches and the CPU-sampled bar at the window's real output
  (`color_lab/display.rs`): linear Display P3 through xgpui-gamut's `WidePainter` when
  the window presents wide-gamut SDR, sRGB previews otherwise. The caption says which.
  Reference rows that map into sRGB stay sRGB. Gamut Mapping opens on Display P3 → sRGB
  and has a Saturation control (OkLCh chroma ×0–2). Whole gradients through both
  policies live on xgpui-gamut → Gamut Mapping, which draws them natively.

Differences from the proposal below:

- **Metrics.** `LabMetrics::STANDARD` is a const, not `LabLookExt::lab_metrics()`,
  because nothing in it depends on the look yet. Spacing is `Rems`, so it matches
  GPUI's `p_6` / `gap_5` exactly. Added `header_gap`, `inline_slider_width`,
  `param_min_width` and `wide_panel` / `narrow_panel` (`PanelWidth { grow, basis, min }`).
  Step 3 added `card_mark`, `hero_min_width`, `legend_width` and `legend_mark_width`.
- **Type.** `TextRole::apply(look, element)` and `screen::text(look, role, text)`
  replace `look.text(role) -> LumaTextStyle`, because today's output uses GPUI's
  rem-based `text_sm` / `text_lg` / `text_3xl`, and converting those to pixels
  would change rendering when rem size isn't 16. Roles: `Crumb`, `Title`, `Lede`,
  `Section`, `Caption` (was `Body`), `Name` (new, card name), `Label` (keys),
  `Detail` (small words), `Value` (mono values, replacing the proposed `Mono`)
  and `Fine` (credits). `screen::readout` renders key/value pairs,
  `screen::value_field_look` gives value text fields the same mono `code` style, and
  `screen::mono_family(look)` is the one mono family. Every role reads a look
  typography token (`role.h2`…`h4`, `role.p`, `label`, `code`, `caption`). The UI face / mono rule is in
  [XGPUI-LAB-DESIGN.md](XGPUI-LAB-DESIGN.md#ui-face-or-mono).
- **LabScreen.** `.control(element)` appends one item, instead of
  `controls(iter)`, because control rows mix element types. `screen::labeled_control`
  applies the switch label typography to a `Choice`. `.notice(..)` may be called
  more than once. Besides the slots, `{id}-header`, `{id}-content`,
  `{id}-panel-wide` and `{id}-panel-narrow` carry selectors. `credits` was
  added in step 3. Not built yet: `actions`, `section`.
- **Hero.** `Hero::full` and `Hero::with_legend`. The hero's `{id}-hero` selector sits on a wrapper
  `div`, because a GPUI element holds one debug selector and the visual keeps its own.
  `Legend` is built with `.item(mark, label, value)`, `.note(..)` and
  `.marker(mark, label)`. Its marks are `Mark::Line(Hsla)`,
  `Mark::Cross { color, halo }` (drawn like the diagram's cross; `None` means
  foreground / background) and `Mark::Hatch` (drawn as the `////` glyph).
  `compare` and `fixed` wait for Image Filters and Italy.
- **LabCard.** `.swatch(impl IntoElement)` takes an element, not an `Hsla`,
  because Color Spaces paints its swatches natively in the source space.
  `.values(..)` takes any iterator of label/value pairs. Added
  `.debug_selector(..)`. `LabCard` implements `FluentBuilder`, so
  `.when(disabled, |c| c.badge(..))` works. Step 3 added `.swatch_color(..)`,
  `.mark(..)`, and truncation of names and subtitles (the badge never shrinks).
  `icon`, `selected`, `on_click` and `body` wait for Italy and Image Filters.
- **Panels.** `Panel` has `Parameters` and `Preview` only. `Param::new(label, input, slider)`
  has no range yet. `ParametersPanel::footer(..)` holds presets. `PreviewPanel`
  takes `.swatch(element)` for one unlabeled swatch, or `.labeled(element, key, value)`
  / `.labeled_color(color, key, value)` for swatches side by side, then
  `.reference(key, value)` or `.reference_values(key, pairs)`. `action`, `ChainPanel` and `PanelShell` are not
  built yet. Both panel rows use the standard wide/narrow widths, so Chromaticity's
  old 500/300 bases are gone.
- **Behavior change (error path only).** "Color conversion unavailable" now goes
  through `.notice`, so it renders as a muted caption instead of body text.
  Normal output is unchanged.

## Problem

Four lab screens lay out the same page: breadcrumb, title, lede, hero, caption,
controls, cards, two panels, note and credits. Each one writes that layout by
hand, and each has drifted:

| | Color Spaces | Italy | Chromaticity | Typography |
|---|---|---|---|---|
| Uses `lab_ui` | yes, split into `render_*` methods | yes, one long chain | no | no |
| Max width / padding | 1240 / p6 | 1240 / p6 | 1280 / p4 | 1040 / p8 |
| Title | `lab_ui::header` | `lab_ui::header` | `typography.role.h2` | 42 px literal |
| Card swatch / min width | 28 / 200 | photo buttons | 34 / 205 | n/a |
| Literal chrome colors | none | none | status tag, marker | none |

A screen author has to know and repeat about 40 spacing, size and color
decisions. The page structure is a convention, not something the code enforces.

## Goal

A screen describes **what** goes on the page. One module decides **where**
it goes and **how** it looks. A screen's `render` should be a short list of
filled slots, and the rest of its file should be its model, its hero renderer
and small functions that build cards and panels.

Non-goals:

- general-purpose layout primitives (the `vstack!` / `hstack!` macros and GPUI already cover those);
- any change to look-radix, the Luma SDK or xgpui. Everything here is `pub(crate)` in `xgpui-lab`;
- moving models, color math, media loading or rendering paths. Those stay in each lab, as `lab-ui.md` already requires.

## Where it lives

`apps/xgpui-lab/src/screen/` replaces `lab_ui.rs`, which becomes a thin
re-export during migration and is then removed.

```
screen/
  mod.rs        LabScreen, slot order, page container
  metrics.rs    LabMetrics, TextRole: every size, gap and type role
  hero.rs       Hero, Legend, ComparePane
  card.rs       LabCard
  panel.rs      Panel, ParametersPanel, PreviewPanel, ChainPanel, Param
  data.rs       shared data colors (gamut colors, markers)
```

## API

### LabScreen

```rust
pub(crate) struct LabScreen { /* slots; all optional except id, look and title */ }

impl LabScreen {
    pub fn new(id: &'static str, look: Arc<Look>) -> Self;

    pub fn crumb(self, owner: &'static str, screen: &'static str) -> Self; // "xgpui-color", "Chromaticity"
    pub fn title(self, text: impl Into<SharedString>) -> Self;
    pub fn lede(self, text: impl Into<SharedString>) -> Self;
    pub fn hero(self, hero: Hero) -> Self;
    pub fn caption(self, text: impl Into<SharedString>) -> Self;          // live state line
    pub fn controls(self, items: impl IntoIterator<Item = impl IntoElement>) -> Self;
    pub fn actions(self, items: impl IntoIterator<Item = impl IntoElement>) -> Self; // right end of control row
    pub fn notice(self, text: Option<impl Into<SharedString>>) -> Self;   // errors and fallbacks, under the controls
    pub fn cards(self, cards: impl IntoIterator<Item = LabCard>) -> Self;
    pub fn panels(self, wide: impl Into<Panel>, narrow: impl Into<Panel>) -> Self;
    pub fn note(self, text: impl Into<SharedString>) -> Self;
    pub fn credits(self, text: impl Into<SharedString>) -> Self;

    /// Escape hatch: an extra full-width section after the panels.
    pub fn section(self, element: impl IntoElement) -> Self;
}

impl IntoElement for LabScreen { /* via RenderOnce */ }
```

Contract:

- **Order is fixed by the type, not by call order.** The rendered order is
  always crumb, title, lede, hero, caption, controls/actions, notice, cards,
  panels, sections, note, credits.
- **Empty slots render nothing**, and leave no gap behind.
- **The breadcrumb takes two parts**, so the ` / ` separator and its spacing are
  never typed by hand. (Today Color Spaces and Italy write `"xgpui-color  /  …"`
  with two spaces, and Chromaticity writes `xgpui-gamut`.)
- **Every slot gets a `debug_selector`** named `"{id}-{slot}"`, for tests.
- **Wrapping:** at most four cards per row, wrapping to two and then one at
  narrow widths. The panels stack below about 720 px. The control row wraps,
  and actions stay together.

### Hero

```rust
pub(crate) enum Hero {
    /// Full content width. Color Spaces field.
    Full(AnyElement),
    /// Square-ish visual with a legend column (220) to its right; legend drops below when narrow.
    WithLegend { visual: AnyElement, legend: Legend },
    /// Two panes side by side, each with its own header. Image Filters.
    Compare(ComparePane, ComparePane),
    /// Visual that owns its geometry, centered at a maximum size. Italy photo.
    Fixed { visual: AnyElement, max: Size<Pixels> },
}

pub(crate) struct Legend { items: Vec<LegendItem>, note: Option<SharedString>, markers: Vec<LegendItem> }
pub(crate) struct LegendItem { mark: Mark, label: SharedString, value: Option<SharedString> }
pub(crate) enum Mark { Line(Hsla), Cross(Hsla), Hatch }

pub(crate) struct ComparePane { title: SharedString, subtitle: SharedString, badge: Option<AnyElement>, visual: AnyElement }
```

Constructors such as `Hero::full(el)`, `Hero::with_legend(el, legend)`,
`Hero::compare(a, b)` and `Hero::fixed(el, size(px(600.), px(400.)))` keep call
sites short.

`Legend` values are data, so Chromaticity's coverage figures come from the
model instead of the current hard-coded `"34%", "46%", …`.

### LabCard

```rust
pub(crate) struct LabCard { /* … */ }

impl LabCard {
    pub fn new(name: impl Into<SharedString>) -> Self;
    pub fn swatch(self, color: impl Into<Hsla>) -> Self;     // data color
    pub fn icon(self, icon: lucide_svg_static::Icon) -> Self; // status cards instead of a swatch
    pub fn mark(self, color: Hsla) -> Self;                  // small series square before the name
    pub fn subtitle(self, text: impl Into<SharedString>) -> Self;
    pub fn badge(self, badge: impl IntoElement) -> Self;     // right of the name; any element
    pub fn values<const N: usize>(self, pairs: [(&'static str, String); N]) -> Self;
    pub fn selected(self, selected: bool) -> Self;
    pub fn disabled(self, disabled: bool) -> Self;           // Color Spaces' out-of-range state
    pub fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self;
    pub fn body(self, element: impl IntoElement) -> Self;    // escape hatch, e.g. Italy thumbnails
}
```

Rules the type enforces:

- **Fixed card styling.** Swatch size, title and detail text roles, a mono
  values row and the minimum width are fixed. A card cannot set its own font sizes.
- **Single-line names and subtitles.** They truncate rather than wrap.
- **The badge is the screen's choice.** `LabCard` only places it: right of
  the name, not shrinking, vertically centered. The screen passes whatever
  element it wants, normally a look-radix `Badge`. `LabScreen` doesn't define
  what a badge is or which states exist.
- **Fills come from the look.** Selected, hover and pressed fills come from
  `interaction_colors().background(selected, state)`. A clickable card stays a
  Luma button, so keyboard and accessibility behavior keep working, as Italy
  already does.

### Panels

```rust
pub(crate) enum Panel { Parameters(ParametersPanel), Preview(PreviewPanel), Chain(ChainPanel), Custom(PanelShell) }

pub(crate) struct ParametersPanel { /* title, subtitle, params, footer, action */ }
impl ParametersPanel {
    pub fn new(title: impl Into<SharedString>) -> Self;
    pub fn subtitle(self, text: impl Into<SharedString>) -> Self;
    pub fn params(self, params: impl IntoIterator<Item = Param>) -> Self;  // 3-column grid
    pub fn footer(self, element: impl IntoElement) -> Self;               // presets row
    pub fn action(self, element: impl IntoElement) -> Self;               // header button, e.g. Reset filters
}

pub(crate) struct Param { label: SharedString, range: Option<SharedString>, input: AnyElement, slider: AnyElement }

pub(crate) struct PreviewPanel { /* title, subtitle, swatches, reference */ }
impl PreviewPanel {
    pub fn new(title: impl Into<SharedString>) -> Self;
    pub fn subtitle(self, text: impl Into<SharedString>) -> Self;
    pub fn swatch(self, label: impl Into<SharedString>, color: impl Into<Hsla>, value: impl Into<SharedString>) -> Self; // 1 or 2
    pub fn reference(self, label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self;
}

pub(crate) struct ChainPanel { /* title, subtitle, steps */ }
impl ChainPanel {
    pub fn step(self, name: impl Into<SharedString>, detail: impl Into<SharedString>, active: bool) -> Self;
    // Renders numbered rows and the "N of M active" line itself.
}

pub(crate) struct PanelShell; // title + subtitle + free body, for anything else
```

`panels(wide, narrow)` sets the width ratio (wide grows 2 from a 440 basis,
narrow grows 1 from a 265 basis), so screens stop passing `flex_basis` and
`min_w`.

### LabMetrics and TextRole

```rust
pub(crate) struct LabMetrics {
    pub max_width: Pixels,      // 1240
    pub page_padding: Pixels,   // p6
    pub part_gap: Pixels,       // gap5, between crumb/header/content
    pub section_gap: Pixels,    // gap4, between content sections
    pub control_gap: Pixels,    // gap3
    pub card_min_width: Pixels, // 200
    pub card_swatch: Pixels,    // 28
    pub legend_width: Pixels,   // 220
    pub preview_height: Pixels, // 80
}

pub(crate) enum TextRole { Crumb, Title, Lede, Section, Body, Label, Detail, Mono, Fine }

trait LabLookExt { // added beside ColorVizLookExt in theme.rs
    fn lab_metrics(&self) -> LabMetrics;
    fn text(&self, role: TextRole) -> LumaTextStyle; // size, line height, weight; color is role-derived
}
```

- **Defaults come from today's Color Spaces values**, so migrating it causes no
  visual change.
- **The look decides type.** `TextRole::Title` maps to what `lab_ui::header`
  uses now, and `Mono` resolves one monospace family instead of four
  `font_family("monospace")` calls.
- **One table to edit.** The design doc's *Type* and *Spacing* tables become a
  description of this one, so a design change happens in one place.

### Data colors

`screen/data.rs` holds the colors a screen is about: `GAMUT_SRGB`,
`GAMUT_P3`, `GAMUT_ADOBE`, `GAMUT_REC2020`, `MARKER_REFERENCE` (the
`Foreground` role) and `MARKER_SUBJECT`. `chromaticity_model::COLORS` and the
literal `0xffdf62` move here. Nothing chrome-related may be added to this module.

## What screens look like after

**Chromaticity**

```rust
fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    LabScreen::new("chromaticity", self.look.clone())
        .crumb("xgpui-color", "Chromaticity")
        .title("Chromaticity & Gamut")
        .lede("Keep one pixel fixed and change how it is interpreted. Compare reference gamut boundaries, luminance, and mapping on the CIE 1931 diagram.")
        .hero(Hero::with_legend(self.chart(cx), self.legend()))
        .caption(self.caption_line())
        .controls(self.choices.iter().cloned().chain(self.switches.iter().cloned()))
        .notice(self.error.clone())
        .cards(model::SPACES.iter().map(|space| self.space_card(*space)))
        .panels(self.pixel_parameters(), self.pixel_preview())
        .note(INTERACTION_NOTE)
        .credits(CIE_LOCUS_CREDIT)
}

fn space_card(&self, space: Space) -> LabCard {
    let c = self.coordinates(space);
    LabCard::new(space.name())
        .swatch(c.preview).mark(data::gamut(space)).subtitle(space.notation(self.source))
        .badge(Badge::new(&self.look, if c.inside { "In" } else { "Out" }))
        .values([("R", c.fmt(0)), ("G", c.fmt(1)), ("B", c.fmt(2))])
        .selected(space == self.simulated)
}
```

The current render is about 250 lines, with roughly 170 of them layout. It
would become about 15 lines of slots plus four small builders.

**Color Spaces**: `Hero::full(self.render_field())`. `render_comparisons`
becomes four `LabCard`s, and `render_parameters` and `render_preview` become
`ParametersPanel` and `PreviewPanel`. Its private `card`, `panel` and `caption`
wrappers are deleted.

**Italy**:
- `Hero::fixed(photo, 600 × 400)`.
- Thumbnails become `LabCard::new(name).body(thumbnail).selected(..).on_click(..)`.
  If the card variant can't keep the photo-card geometry from `lab-ui.md`,
  thumbnails go through `.section(..)` instead.
- `.actions([presentation_selector])`.

**Image Filters**:
- `Hero::compare(atlas_pane, mip_pane)`.
- Four status cards built with `.icon(..)`.
- `ParametersPanel` with `.action(reset_filters)`.
- `ChainPanel` for the filter order.

**Typography** (xgpui-shader): undecided. See *Open questions*.

## Testing

- **One shared test, run on every screen:** the crumb, title, hero and caption
  selectors are present, nothing overflows horizontally at 720 px and 1280 px,
  and this holds in both light and dark mode.
- **Unit tests on `LabCard`:** no literal chrome colors (every color
  resolves from the look or `data.rs`), names truncate rather than wrap, and a
  badge never pushes the name onto a second line.
- **Existing tests keep working.** Screen interaction tests keep their own
  `debug_selector`s, such as `interactive-spaces-field-bounds`.

## Migration

Each step lands separately:

1. **Add `screen/`.** Write it and its tests, and re-export the old helpers
   from `lab_ui.rs`. No screen changes yet.
2. **Color Spaces.** Its rendering must stay the same: compare screenshots at
   the same window size in both modes.
3. **Chromaticity.** This also fixes the breadcrumb, the hard-coded legend
   values, the literal marker color, and the 1280 / p4 drift. Its hand-made
   status tag becomes a look-radix `Badge` passed to `.badge(..)`. Wait
   for the in-progress edits in `color_demos/chromaticity.rs` to land first.
4. **Italy.**
5. **Image Filters.**
6. **Remove `lab_ui.rs`.** Update `lab-ui.md` and `XGPUI-LAB-DESIGN.md` to point
   at `screen/`.

## Open questions

1. **Does Typography join?** Either migrate it, accepting that its uppercase
   eyebrow labels and display sizes change, or add a `LabScreen::showcase()`
   variant that keeps eyebrows and a narrower 1040 column.
2. **Does the hero card take part in selection?** Chromaticity's simulated gamut
   can be chosen by clicking a card or by its selector. Should `cards` accept
   `on_select` and own that sync?
3. **Builder or struct literal?** Builders read well and allow optional slots.
   A struct with `..Default::default()` makes every slot visible at the call
   site. This note assumes builders.
4. **How long does the escape hatch stay?** Should `.section(..)` exist after
   migration, or is any remaining use a sign that a slot type is missing?
