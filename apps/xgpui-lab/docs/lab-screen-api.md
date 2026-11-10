# LabScreen: proposed screen layout API

Status: **draft for review**. Nothing is implemented. Names and signatures are
proposals.

Related: [XGPUI-LAB-DESIGN.md](XGPUI-LAB-DESIGN.md) (the rules),
[lab-ui.md](lab-ui.md) (the current helpers and their history).

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
