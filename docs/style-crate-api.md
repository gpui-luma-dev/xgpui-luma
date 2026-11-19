# `xgpui-luma-style` — public API

Status: implemented on `issue-285/style-lang` (2026-11-19), except where marked *later*. This document records the API decisions (topics 1–8) and the names as they are in the code. Syntax, name resolution and the cascade are in [style-language-design.md](style-language-design.md).

## 1. Purpose and boundary

One crate that **styles** Luma controls and lets tools **introspect** that styling. It is built only for Luma, but it knows nothing about Luma: everything Luma-specific comes in through a small, deliberate API. **No UI** lives in the crate, and the crate never writes files.

| Inbound (consumer supplies) | Outbound (crate returns) |
|---|---|
| `Config`: layers, states, exclusive state group, attributes, snapping | `Resolved` / `PartView`: values for one control instance |
| Style files (`Source`) by layer | `Explanation` / `Inspection`: where every value came from |
| `TokenSource`: native token roots and look functions | `Diagnostics`: problems with code, location, range and snippet |
| `Overrides`: values set on one instance | Edited and exported `.lss` text |

Controls, parts and control-specific properties are declared **in the style files**. Rust declares only `Config`. Dependencies: `pest`, `pest_derive`, `thiserror`.

## 2. Configuration

Built once by the SDK (`xgpui_luma::style::config()`), shared by every stylesheet.

```rust
let config = Arc::new(Config::builder()
    .layers(&["sdk", "look", "app"])
    .states(&["hover", "pressed", "focused", "disabled", "checked", "selected",
              "active", "expanded", "open", "invalid", "high_contrast"])
    .exclusive(&["disabled", "pressed", "hover"])     // only the strongest active one counts
    .attributes(&["variant", "size", "tone"])         // at most 8
    .snap(SnapPolicy::Lengths)
    .build()?);

config.state("checked")      // Option<StateId>
config.attribute("variant")  // Option<AttrId>
```

The SDK has no light/dark mode; looks provide modes. An SDK-only app switches themes by compiling different token files.

## 3. Setup: files, compile, bind (topic 3)

```rust
let sheet = Stylesheet::builder(config)
    .layer("sdk",  xgpui_luma::style::sources())   // SDK tokens + one .lss beside each control
    .layer("look", shadcn::style::sources())       // look tokens + overrides
    .layer("app",  app_sources)                    // optional
    .compile()?;                                   // Result<Stylesheet, Diagnostics>

let theme = Theme::bind(&sheet, shadcn::style::tokens(&catalog), "light")?;
let sdk_only = Theme::bind(&sdk_sheet, Arc::new(NoNativeTokens), "")?;
```

- `Source::new(name, text)` for embedded files (`include_str!`); `Source::from_path(path)` to read from disk (import, hot reload, the lab).
- Each crate exports its own `sources()`; a test in the SDK fails if a `.lss` file under `crates/sdk/src` is missing from the list (*later*, with the SDK migration).
- `sheet.modes()` lists modes found in token files (empty for the SDK alone). Binding an unknown mode is an error.
- A look can use the SDK styles unchanged and tweak them: a token file alone maps the SDK palette onto the look; control files override only what differs; `:=` replaces a property in every state. A look cannot add parts or declare properties on SDK controls; it can declare its own controls.

**Compile** checks syntax, declarations (only in the declaring layer), unknown controls/parts/properties/states/attributes, kinds, ambiguous names, built-in function arity, `extends`, and token kind consistency. **Bind** checks native token roots, look functions and the mode.

## 4. Keys and handles (topic 2)

Handles are looked up once and stay valid across rebinds and swaps: controls, parts, attribute values and properties are interned by name; states and attributes are `Config` indices.

```rust
// Once, at SDK startup (e.g. in a OnceLock):
let checkbox  = sheet.control("checkbox")?;                    // ControlId
let indicator = sheet.part(checkbox, "indicator")?;            // PartId
let size      = sheet.prop::<f32>(checkbox, "indicator_size")?; // Prop<f32>; errors if missing or another kind
let checked   = config.state("checked")?;                      // StateId
let variant   = config.attribute("variant")?;                  // AttrId
let soft      = AttrValue::named("soft");                      // interned when a builder sets .variant("soft")

// Per render: no strings, no allocation.
let key = StyleKey::new(checkbox)
    .state_if(checked, model.checked)
    .states(interaction_states)          // StateSet, a u32 bitset
    .attr_opt(variant, model.variant)
    .scale(window.scale_factor());
```

- `ControlId::new("checkbox")`, `PartId::new("indicator")`, `Prop::new(…)` are `const` for names known at compile time; `PartId::ROOT` is the control itself.
- `StyleKey` is small, `Copy` and `Hash`; it is the cache key.
- `theme.key_named(control, attrs, states)` builds a key from names, for tools and tests.

## 5. Reading styles: the generic view (topic 1)

```rust
let style = theme.resolve(&key);                 // Arc<Resolved>, cached per key
let part  = style.part(indicator);               // PartView
part.get(props::BACKGROUND)                      // Option<Color>
part.get(size)                                   // Option<f32>, declared property
part.background(); part.radius(); part.transition();   // shortcuts for core properties
part.value("indicator_size")                     // raw Value by name, for tools
Resolved::lerp(&off, &on, t)                     // animation between two resolved styles
```

- `props::*` holds handles for every core property (`BACKGROUND`, `RADIUS`, `SHADOW`, `TRANSITION`, …).
- `FromValue` types: `f32` (length, number, weight), `Color`, `Motion`, `Arc<[ShadowLayer]>`, `Arc<str>` (font, icon), `TextStyle`.
- Lengths are snapped to device pixels on output (unless `exact(…)`); properties that read other properties see unsnapped values.
- Cost (release build, Radix checkbox fixtures): about 15–17 µs for the first resolve of a key, then a cache lookup. Resolving names at compile time and comparing interned names by pointer are the next steps if profiling the SDK shows a need.
- Motion: for a change from key A to key B, the animation runtime uses `resolve(B).part(p).transition()`.
- `.apply()` onto gpui elements is *later*, as a `gpui` feature built on this view.

## 6. Instance overrides (topic 5)

```rust
let overrides = Overrides::new()
    .set(indicator, props::BACKGROUND, Color::oklch(0.6, 0.2, 145.0, 1.0))
    .set(PartId::ROOT, props::TRANSITION, Motion::Instant)          // replaces today's .motion(|m| …)
    .set_token(indicator, props::BORDER, "palette.focus.ring");      // follows mode and theme changes
let style = theme.resolve_with(&key, &overrides);
```

- An override replaces its property **in every state** (like `:=` above all layers), including the disabled look.
- Overrides take part in the cascade: a rule `border = background` follows an overridden background, and `explain` reports `ValueSource::InstanceOverride`.
- The type checks values (`Prop<Color>` takes a `Color`). Override sets with equal contents share cached results (in any `set` order), so building one per frame costs nothing extra; the cache holds at most 4,096 entries per theme and is cleared when full.
- An override for a part or property the control does not have is ignored and reported (with a suggestion). Override problems carry no file location, since they come from Rust.
- Overrides apply to one instance; styling every instance belongs in the app layer.
- Per-state overrides are *later* (non-breaking).

## 7. Rebind, refresh, swap (topic 4)

A `Theme` never changes; every change returns a new one that shares the compiled stylesheet. Failures return `Diagnostics` and leave the current theme as it is.

```rust
theme.with_mode("dark")?                                   // same files and tokens, other mode
theme.with_tokens(Arc::new(new_catalog))?                  // new native token source
theme.with_token_values(&[("palette.app.background", v)])? // live edits above every layer, no recompile
sheet.with_source("app", Source::new("app/tweak.lss", text))? // recompile with one file replaced or added
theme.with_stylesheet(&sheet2)?                            // same tokens and mode, new files
theme.generation()                                         // changes with every new theme
theme.token_values()                                       // live edits in effect
```

Holding the current theme and refreshing windows is the SDK's job.

## 8. Tokens

```rust
pub trait TokenSource: Send + Sync {
    fn roots(&self) -> &[&str];                                          // e.g. ["css"], ["gray", "accent", "tone"]
    fn token(&self, path: &str, cx: &TokenCx<'_>) -> Option<Value>;      // None = absent (see first())
    fn index(&self, root: &str, index: i64, cx: &TokenCx<'_>) -> Option<Value> { None }  // gray[7]
    fn function(&self, name: &str) -> Option<FunctionSig> { None }       // look functions
    fn describe(&self, path: &str, cx: &TokenCx<'_>) -> Option<String> { None }  // for the inspector
    fn call(&self, name: &str, args: &[Value], cx: &TokenCx<'_>) -> Result<Value, String>;
}
```

- `TokenCx` gives the mode and the instance's attributes (`cx.attr("tone")`).
- File tokens (`tokens { … }`) and native tokens share one namespace. Look functions receive an absent token as `Value::None`.
- Built-in functions: `min`, `max`, `first`, `exact`, `alpha`, `mix`, `gray`, `srgb`, `hsla`, `oklch`, `oklab`, `color(space, …)`, `shadow`, `box_shadow`, `inset_shadow`, `text`, `timed`, `spring`. Built-in keywords: `none`, `transparent`, `black`, `white`, `instant`, easing names. Which further functions are built in or belong to a look is decided per function: built in if it needs no look data.

## 9. Values and colors

`Value` is `Number`, `Color`, `Shadow`, `Text`, `Str`, `Icon`, `Motion`, `Symbol`, `List` or `None`. Numbers have no units; the property's kind gives the meaning (lengths are logical pixels, durations milliseconds). `value.to_lss()` prints a value as `.lss`.

Colors follow CSS Color Level 4: `oklch(…)`, `oklab(…)`, `color(srgb | srgb_linear | display_p3 | a98_rgb | prophoto_rgb | rec2020, r, g, b[, a])`, hex, `srgb()`, `hsla()`, `gray()`. They are stored as OKLCH (unclamped), mixed and interpolated in OKLab, and written out as sRGB with CSS gamut mapping (`to_srgb()`, `to_hsla()`).

## 10. Introspection (topic 6)

```rust
theme.explain(&key, part, "background")            // Explanation for one property
theme.explain_with(&key, &overrides, part, prop)
theme.inspect(&key, Some(&overrides))              // Inspection: every part and row
sheet.axes(checkbox)                               // Axes: attribute values, states, modes for pickers
```

- `Explanation`: value, `ValueSource` (`Rule`, `InheritedFromRoot`, `InstanceOverride`, `Unset`), winning `RuleInfo` (source, layer, line, column, scope, states, expression text, `via` for `extends`), overridden rules (best first), the `:=` owner, and `references`.
- `Reference`: what an expression read (property, token, constant or native token), its value, where it is defined, the token source's `describe` text, and what *it* read, recursively — e.g. `shadcn.outline_layer.default` → `css.card`. Recorded only when explaining.
- `Inspection` rows: every property any rule sets (directly or through `extends`), every declared property of the control (even when unset), and text properties inherited from root; each with kind, `Category` (Colors, Layout, Typography, Elevation, Motion, Other), doc and explanation. Each part has a `BoxModel`.
- `Axes` come from the files (scopes, `extends` aliases, rule states, mode blocks), replacing per-control inspector specs.

## 11. Editing and export (topic 7)

```rust
let edit = Edit::replace(&row.explanation.winner?, "tone[11]");     // new value text, in place
let edit = Edit::insert("app", "app/overrides.lss",
    &RuleSpec::new("checkbox", "indicator", "background", "black").scope("variant", "brand").state("checked"));
let (sheet2, changed) = sheet.apply(&[edit])?;                      // recompiled + changed files (to save)
sheet.export_scope("checkbox", "variant", "brand", &session_rules) // a complete .lss with the rules under one scope
export_tokens(&theme.token_values())                                 // a tokens { … } file for live edits
```

- Replacements splice the original text, so comments and layout survive; inserted rules are printed in one standard layout.
- A `RuleInfo` records a fingerprint of the file text it came from. A replacement against a different version of the file is rejected (`Code::StaleEdit`); explain the rule again and retry.
- Ranges include parentheses: a value written `(2 + 3) * 4` is shown and replaced whole.
- Exports contain only the differences and round-trip: the tests compile exported text and compare resolved values.
- `Source::from_path` plus `with_source` is import.

## 12. Diagnostics (topic 8)

- Compile and bind fail with `Diagnostics`; **resolving never fails**: a problem leaves the property unset and is recorded once.
- `theme.diagnostics()` and `theme.diagnostics_since(mark)` (returns new problems and the next mark) for polling from the lab or Studio.
- `theme.check(control)` resolves every combination of the control's axes and returns the problems — one line per control in SDK and look tests. With more than 10 states it checks combinations of up to three and says so (`Code::PartialCheck`); modes that fail to bind are reported, not skipped.
- Every `Diagnostic` has a `Code` (`UnknownPart`, `KindMismatch`, `AmbiguousName`, `Cycle`, `AbsentToken`, …), source, line, column, byte `range`, message and snippet. Unknown names get a suggestion: `unknown part "indicatr" for checkbox (parts: …) — did you mean "indicator"?`
- Load-time lints (unused declarations, rules that can never win) are *later*.

## 13. Who owns what

| Piece | Lives in |
|---|---|
| Parser, compiler, cascade, values, resolve, cache, lerp, overrides, inspection, edit/export, diagnostics | `xgpui-luma-style` |
| `Config`, SDK `.lss` files and `sources()`, templates reading `PartView`, the animation runtime reading `transition`, holding the current `Theme` | SDK |
| Look `.lss` files and `sources()`, look `TokenSource`s, variant/size builder helpers | `look-shadcn`, `look-radix` |
| Inspector UI (from Studio's generic display) | SDK feature or a small `style-inspector` crate |

## 14. Decisions

| Topic | Decision |
|---|---|
| Crate | `xgpui-luma-style` in `crates/style`; files are `.lss` |
| 1 Reading styles | Generic view with typed property handles; `.apply()` later as a gpui feature |
| 2 Keys | Handles: interned names for controls, parts, attribute values and properties; `Config` indices for states and attributes |
| 3 Setup | Layered builder; files beside controls with a completeness test; embedded and from-disk sources; `modes()` |
| 4 Rebind | Themes never change; `with_mode`, `with_tokens`, `with_token_values`, `with_source`/`with_stylesheet`; failures keep the old theme; `generation()` |
| 5 Overrides | One override set per instance, typed, literals or tokens, inside the cascade, all states; cached by contents; unknown parts and properties reported |
| 6 Inspection | `explain` and `inspect` with references, declared-but-unset rows, axes from files, box model, categories |
| 7 Export | Edits in place; inserts; `export_scope`; `export_tokens`; round-trip tests; no file writing |
| 8 Diagnostics | Load fails, resolve never fails; `diagnostics_since`; `check`; codes, ranges and suggestions; lints later |
| Colors | CSS Color 4, stored as OKLCH, OKLab math, sRGB gamut mapping |
| Functions | Built in if they need no look data; decided per function |
