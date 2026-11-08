# SDK color values and GPUI previews

`xgpui_luma::color::ColorValue` retains tagged Palette values: encoded sRGB,
linear sRGB, encoded Display P3, or Oklch. Display P3 support requires Palette
0.7.7; SDK dependencies use the workspace version.

## Source contract

Components use `f32`. RGB values can be negative or greater than one, and Oklch
lightness is not bounded in source storage. Chroma must be nonnegative. Hue is in
degrees, including at zero chroma; source hue is not rewritten during preview
conversion. Alpha is straight (not premultiplied), finite, and in `0..=1`.
`with_alpha` replaces alpha rather than multiplying opacity.

Constructors retain their arguments. `validate`, rendering, serialization, and
deserialization reject nonfinite components and invalid alpha/chroma. Equality
compares the tagged Palette values, not perceptual equivalence across spaces.
Cache keys use the original component bits and source-space tag; NaN is rejected
before caching.

Persistence uses `{ space, components }`, where components are RGB/alpha or
lightness/chroma/hue-degrees/alpha. Tags are `srgb`, `linear-srgb`, `display-p3`,
and `oklch`. This format preserves extended-range source values. CSS source formatting is a
separate API; neither persistence path performs gamut mapping.

Stylesheet palette and shadow defaults use readable `"hsla(h, s%, l%, a)"`
strings; their shared build/runtime loader converts these into source colors.
The bundled SDK UI palette is monochrome; authored look/application colors and
wide-gamut source persistence remain supported. Chart palettes can be maintained
independently of UI tokens. See [stylesheet authoring](shared-stylesheet.md).

## Preview bridge

```rust
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};

let source = ColorValue::display_p3(1.0, 0.0, 0.0, 0.5);
let preview = gpui_bridge::to_rgba(source, GamutMapping::CssLocalMinde)?;
// Keep source in the model; preview is only for the current GPUI backend.
```

`to_srgba_unclamped` converts without gamut mapping. `is_in_gamut` distinguishes
sRGB from Display P3. `to_srgba_fallback` maps a copy to bounded encoded sRGB.
The current GPUI bridge uses this fallback, with straight alpha; raster paths
requiring premultiplication must perform it separately. `preview_rgba` and
`preview_hsla` adapt already prepared backend colors without source mapping.
HSL editor views use `from_palette_hsla` / `to_palette_hsla` to isolate degrees
versus turns while retaining authored achromatic hue and the 360° endpoint.

Mapping is explicit: `Clip` clips RGB channels; `CssLocalMinde` implements the
binary-search/local-MINDE UI policy from the
[30 September 2026 CSS Color 4 draft, section 14.2.2](https://www.w3.org/TR/2026/CRD-css-color-4-20260930/#binsearch).
Reference-sample tests allow `0.001` RGB error for f32 conversions. This policy
does not define an image-document rendering intent.

`cached_rgba` shares a bounded, 256-entry sRGB cache in the GPUI App. Exact source
components, source-space tag, and mapping policy identify entries. Source/policy
changes resolve different entries. At capacity the cache clears rather than
growing without bound. This cache targets a fixed encoded-sRGB backend: future
P3/profile-aware output must include output configuration in its cache identity.

## Swatches

`ColorSwatch::new` and `ColorSwatchData.color` now take `ColorValue`, retaining
the original color and resolving a cached preview before canvas painting. Swatches
use `CssLocalMinde` by default; `ColorSwatch::mapping` can select clipping. Invalid
source colors produce a transparent preview, with errors available via the
strict bridge/validation APIs.

App integrations still producing GPUI theme colors call `gpui_bridge::from_hsla`
explicitly. There are no implicit conversions or compatibility constructors on
the swatch API. This work prepares retained source data for future backend support. GPUI output
continues to use sRGB previews.


## CSS and Shadcn theme sources

`ColorValue::parse_css` accepts concrete hex (3/4/6/8 digits), HSL/HSLA,
Oklch, `transparent`, and `color(srgb ...)`, `color(srgb-linear ...)`, or
`color(display-p3 ...)`. It supports decimal/percentage alpha and angle units;
Oklch percentages use the CSS scales (100% lightness = 1, 100% chroma = 0.4).
This is a documented subset of [CSS Color 4](https://www.w3.org/TR/css-color-4/),
not a complete CSS parser. Named colors, RGB functions, `none`, relative syntax,
and expressions are unsupported. Source RGB and Oklch lightness remain extended
range; invalid alpha/chroma are rejected. Extended HSL percentages in theme
exports convert to extended sRGB through Palette rather than being clipped.
Bare alpha `50` is invalid: use `50%` or `0.5`.

`to_css` writes the original space/components with round-trip f32 precision.
HSL inputs become encoded sRGB through Palette, without gamut mapping. Source
CSS formatting does not implement CSSOM serialization or normalize Oklch hue.
`adjust_ui_lightness` derives Oklch lightness into 0–1, preserving chroma, hue,
and alpha; it never clips RGB or mutates the original.

Shadcn's `CssTokenMap::source_color` and `LookResolver::resolve_source_decl`
retain source values through alpha/state resolution. `ResolvedSourceColor`
provides a separate `srgb_preview` projection. Existing paint/inspect palette
fields remain GPUI previews; they must not be used to reconstruct source colors.
Theme snapshots retain source state arrays alongside precomputed sRGB previews,
so `ShadcnLook::source_color` / `resolve_source_color_state` retrieve source
values without recomputing gamut mapping. Rebuilding the theme after overrides
rebuilds both arrays.

Shadcn color override APIs now take `HashMap<String, ColorValue>`; copying via
`with_color_overrides` returns `Result` and rejects invalid colors. Override
storage retains the space and precision instead of rounded HSL. GPUI callers
must explicitly convert at ingress if they only have a preview. Studio retains
source overrides and unchanged CSS colors. HSL adjustments convert an unclamped
source through Palette; the HS palette generator retains its derived Oklch values.

GPUI-valued look/inspect fields describe the current rendering preview. Retrieve
source values through the source APIs when editing, persisting, or deriving colors.


## Core SDK theme palette

`LumaPalette` and its semantic groups now use `ColorValue` by default and support
source serialization. `LumaThemeMode` contains the editable source palette.
`LumaPalette<Hsla>` is the current backend projection, distinguished by its
component type.

`ThemeTokens` is a prepared theme snapshot, retaining the source palette and a
precomputed preview in `SrgbPalette`. Field access on `tokens.palette` reads the
immutable preview; `tokens.palette.source()` reads source colors. Source and
preview are shared when tokens are cloned. No mutable dereference is provided.
Theme construction validates every color, reports the failing field, and uses
the bridge cache to avoid repeated mapping of identical colors. Built-in light
and dark palette snapshots are shared and need no per-frame gamut mapping.

```rust
use xgpui_luma::color::{ColorValue, GamutMapping};
use xgpui_luma::theme::{LumaThemeMode, ThemeTokens};

let mut source = LumaThemeMode::light();
source.palette.state.selected.background = ColorValue::display_p3(1.0, 0.0, 0.0, 1.0);
let tokens = ThemeTokens::from_source(source, GamutMapping::CssLocalMinde)?;
// Pass tokens to DefaultSliderTheme::new, DefaultButtonFamilyTheme::new, etc.
```

`tokens.source()` retrieves an editable source copy with current metrics,
typography, and elevation. To change colors or mapping policy, prepare a new
snapshot from that source. Replacing `tokens.palette` with another prepared
palette replaces source and preview together. Control look fields remain current-backend representations. Source elevation
colors are retained alongside their prepared previews.


## Radix source scales and custom seeds

`ColorScale::new` / `named` take `[ColorValue; 12]` and an explicit mapping policy,
returning `Result`. Each scale retains immutable source steps and precomputed
sRGB previews. `source_step` and `resolved_source` use the same clamped 1-based
indexing as the preview APIs. `Look::resolve_step_source` and
`resolve_role_source` expose source values with SDK provenance.

`CustomColors` now contains `ColorValue` seeds. `generate_colors` returns `Result`
and requires opaque, validated seeds. The original Radix generator still uses
its upstream f64 perceptual calculations and P3 reference scales. Its output
now stores derived Oklch colors and keeps the original tagged accent at step 9
when the solid-selection policy chooses it. Generated values are not rounded to
8-bit RGB. The SDK bridge handles gamut mapping exclusively for previews.
Upstream JavaScript fixture comparisons retain their one-channel-level tolerance.

`GeneratedColors` includes original inputs and a retained contrast source;
`Look::custom_inputs` retains mode-specific seeds even when the algorithm picks
a derived solid. `set_accent_seed` and `set_custom_colors` return `Result` and
validate generation before updating state. Forks copy sources and previews;
selecting named palettes clears custom inputs and contrast.

Named Radix catalog data remains sRGB. `parse_source_color` and
`family_source_steps` retain its source channels; the RGBA catalog syntax also
retains fractional channel values. Malformed source values return errors rather
than risking string-slicing panics. Radix Studio stores source values in editor models and emits `ColorValue` edits.
The three Radix seed fields and their picker fields display six-digit RGB hex.
Input parsing and source-space preservation stay in the backend; formatting the
readout never replaces the retained source.

GPUI's RGB-to-HSL calculation can overshoot saturation by a few f32 ULPs.
`to_hsla` and `SrgbRenderCache::resolve_hsla` bound that final preview saturation;
source colors and unclamped conversion APIs remain unaffected.


## Shadows and editor operations

`LumaElevation`, `LumaShadow`, and `LumaShadowLayer` use `ColorValue` by default.
`ThemeTokens.elevation` is an immutable `SrgbElevation` snapshot: source colors
remain available through `.source()`, and GPUI box shadows use prepared previews.
`ThemeTokens::from_source` validates shadow colors/geometry and resolves them once.
Each layer also stores its explicit `inset` flag; `LumaShadowLayer::new` and
`LumaShadow::single` accept that flag as their final argument. Inset layers paint
inside the surface and do not contribute to its outer projection extent.

Shadcn `ShadowTokenParts` also retains source colors. `to_css_value` and
`shadow_ladder_overrides` return `Result` and format source colors/geometry without
rounding. Ladder opacity scales a copy and bounds derived alpha to one. Standard
shadow previews are cached with each theme snapshot; rebuilding overrides refreshes
them. `ShadcnLook::parse_shadow_source_token` retrieves editable source layers.
Malformed geometry/colors fail validation. Bundled Fallout/Jarvis/Optimus shadow
exports were corrected to valid concrete colors and bounded alpha.

Studio pickers retain a `ColorValue` separately from their HSV interaction view.
Loading, opening, synchronizing, or changing alpha preserves the original color
space/components. Editing hue/saturation/value intentionally authors an sRGB color
from the displayed HSV plane. Luma Studio retains its established compact HSL/HSLA
readouts, shadow fields, and picker clipboard format. Source serialization syntax
is internal and does not appear in those UI values.

`ColorSpecification::to_color_value` exports source values; `from_color_value`
converts spaces without gamut mapping. `to_hsla` describes a rendering preview.
Oklch imports retain raw lightness/chroma/hue/alpha, and channel editing does not
implicitly force the model into sRGB gamut. Gamut indicators remain useful for
previewing the current display boundary. Lab editor conversion uses Palette with
an explicit D65 white point, exporting extended linear sRGB; this preserves color
information without mislabeling D65 data as CSS `lab()` (which uses D50).

Gradient builders now accept `Vec<ColorValue>` and return `Result`. A
`GradientDelegate` retains immutable source stops plus precomputed sRGB paint
stops. `source_at_position` and `interpolate_source` sample original colors in the
selected encoded-sRGB, HSL, or D65 Lab space before preview mapping. These use
straight alpha; they are UI gradient operations, not an image compositing policy.
Mapped stop colors are never used to reconstruct the original gradient sources.
Paint sampling continues to interpolate the prepared preview stops to preserve
existing gradient appearance. HSL interpolation uses the shortest hue arc and
retains the existing half-turn tie (0° ↔ 180° passes through 90°).

HSV and wheel color calculations use Palette. Editor channel bounds and the
Oklch wheel's existing boundary-chroma reduction remain explicit UI policies.
Raster fields, rings, and arcs convert prepared previews through the GPUI bridge;
their encoded-sRGB sampling, coverage, byte order, and premultiplication remain
backend details. Those 8-bit buffers are never used for source persistence.

This work preserves SDK data for when GPUI gains wider-gamut output. Native P3
rendering, display profiles, image-buffer color management, and photographic
rendering intents are outside this implementation.
