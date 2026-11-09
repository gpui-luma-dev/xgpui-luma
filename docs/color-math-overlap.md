# Color math overlap audit

Status: findings only (issue #143). Nothing has been changed yet.

`xgpui-color` (0.3.1) and `xgpui-gamut` (0.3.3) are published from
`gpui-luma-dev/xgpui`, so both sides of the overlap are under our control.

## What each crate uniquely owns

- **SDK `xgpui_luma::color`** (palette 0.7, f32): the source type `ColorValue`
  (tagged sRGB / linear sRGB / Display P3 / Oklch), serde storage, CSS parse and
  format, unclamped conversions, `is_in_gamut`, CSS local-MINDE and Clip mapping
  to sRGB and Display P3, the GPUI sRGB bridge and render cache.
- **`xgpui-luma-color`**: interactive controls (field, slider, ring, arc, swatch)
  and editing specs (`Hsv`, `Hsl`, `Lab`, Oklch).
- **`xgpui-color`** (f64, no palette): `RgbSpace` (sRGB, Display P3, Rec. 2020,
  Adobe RGB), sign-preserving transfer functions, `XyzD65`, Oklab/Oklch, ICC
  parsing (`moxcms`), Bradford adaptation, custom `RgbDefinition`s,
  `LinearGradient`, and `WGSL_COLOR_MATH` (the same math for shaders).
- **`xgpui-gamut`**: wide-gamut SDR producers (`WideColor`, `WideGradient`,
  `WidePainter`) that take **linear** Display P3. Rendering only.

## Overlap

| Capability | SDK (palette) | xgpui-luma-color | look-radix `custom_colors` | xgpui-color |
|---|---|---|---|---|
| sRGB / P3 matrices, transfer | palette | palette (`wheel_models` clamps before converting) | own Color.js 0.5.2 matrices, `linearize` / `encode` | `RgbSpace` + WGSL |
| Oklab / Oklch | palette | palette | own LMS → Oklab | `Oklab` / `Oklch` + WGSL |
| Gamut mapping | CSS local-MINDE, Clip | wheel: 12-step chroma bisection (`oklch_to_srgb_gamut_mapped`) | — | Clip, `ReduceChroma` (CPU + WGSL `gamut_map`) |
| Gradient interpolation | — | `interpolate_source` (encoded sRGB, HSL, D65 Lab) | `Color::mix` | `LinearGradient`: linear RGB, Oklab, Oklch with hue modes |
| Extended HSL | palette `Hsl` | `Hsl` spec via palette | — | — |

## Problems this causes

1. **Inconsistent results.** Three gamut-mapping algorithms mean the same color
   can map differently in the gradient builder, the color wheel and the
   chromaticity/gamut demos.
2. **No bridge.** There is no `ColorValue` ↔ `xgpui_color` adapter. The lab builds
   `EncodedRgb::new(RgbSpace::DisplayP3, [x as f64, ...])` by hand
   (`apps/xgpui-lab/src/gamut/labs.rs`, `gamut/reference.rs`,
   `color_demos/interactive_spaces.rs`), and nothing feeds a `ColorValue` into
   `xgpui-gamut`.
3. **Palette's unclamped HSL clips.** `Hsl::from_color_unclamped` clips negative
   channels (e.g. Oklch(0.65 0.4 145) → hue snapped to 120°, saturation 1.0).
   `xgpui-luma-color`'s `Hsl::from_color_value` uses it, contradicting the SDK
   docs that spec conversions do not gamut-map. The lab works around it with
   CSS Color 4 extended HSL (PR #142, `color_stop/hsl.rs`).
4. **Hand-written shader math.** The lab's P3 gradient shaders (`p3.rs`,
   `mesh_p3.rs`, `color_field/field.metal`) hand-roll sRGB decoding instead of
   using `xgpui_color::WGSL_COLOR_MATH`.
5. **A third private math library.** look-radix's Color.js port duplicates what
   `xgpui-color` provides (matrices, transfer, Oklab).

## Proposed direction (to evaluate)

- Make **`xgpui-color` the single home for color math**: spaces, transfer,
  Oklab/Oklch, gamut mapping (add CSS local-MINDE alongside `ReduceChroma`) and
  extended HSL.
- Keep **`ColorValue` in the SDK as the source type**, implemented on
  `xgpui-color` conversions. Palette becomes an implementation detail or is
  dropped.
- Add **one explicit adapter** in the SDK: `ColorValue` ↔
  `xgpui_color::EncodedRgb` / `LinearRgba` (and so to `xgpui-gamut`).
- Move **look-radix `custom_colors`** and the **color wheel** onto `xgpui-color`;
  retire the wheel's private bisection or express it as a mapping policy.
- Use `WGSL_COLOR_MATH` in lab shaders instead of hand-written decode.

Risks to check: f64 vs f32 results (SDK CSS MINDE reference tests, look-radix
Color.js fixture tolerance), palette types in public SDK APIs (`DisplayP3Color`,
`Srgba` in `ColorValue` variants), and published-crate versioning across the two
repositories.
