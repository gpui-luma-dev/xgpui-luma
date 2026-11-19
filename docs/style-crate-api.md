# `xgpui-luma-style` — public API spec (draft for review)

Status: draft, 2026-11-19, revised to match [style-language-design.md](style-language-design.md). Nothing implemented. All open questions in §14 are decided or deferred.
The language document defines syntax, name resolution and the cascade; this document defines the Rust API around them. Supersedes the #94 (HCL) and #282 (`#[derive(Style)]`) approaches.

## 1. Purpose and boundary

One crate that **styles** Luma controls and lets tools **introspect** that styling. It is built only for Luma, but it knows nothing about Luma: everything Luma-specific comes in through a small, deliberate API. **No UI** lives in the crate.

```
                   ┌──────────────────────────── xgpui-luma-style ────────────────────────────┐
  Config       ──▶ │                                                                          │
  Style files  ──▶ │  parse ─▶ compile (declarations, validation) ─▶ bind (tokens, mode)      │
  TokenSource  ──▶ │                                       └─▶ resolve ─▶ Resolved / PartStyle │ ──▶ templates (.apply), animation runtime
                   │                                       └─▶ inspect ─▶ Inspection          │ ──▶ inspector UI (outside the crate)
                   └──────────────────────────────────────────────────────────────────────────┘
```

| Inbound (consumer supplies) | Outbound (crate returns) |
|---|---|
| `Config`: states, exclusive state group, attributes, layers, snapping | `Resolved` / `PartStyle`: values for one control instance |
| Style files (text) with their layer | `Inspection`: the data behind every value |
| `TokenSource`: native token roots (`css.*`, `gray[n]`, `tone[n]`) and look functions | `Diagnostics`: errors with file, line and column |

**Not in this crate:** UI, Luma types (`ThemeTokens`, `InteractionState`, looks), control behaviour, layout structure, the animation runtime.

**Where the schema lives:** controls, parts and control-specific properties are declared **in the style files** (language §4). Rust declares only the global configuration below. There is no per-control Rust.

Dependencies: `pest`, `pest_derive`, `thiserror`. Feature `gpui` (default on) adds conversions and `.apply()`.

## 2. Concepts

| Term | Meaning |
|---|---|
| **Control** | A named styled thing (`checkbox`), declared by the first layer whose files mention it. |
| **Part** | A named piece of a control (`indicator`). The control block itself is part `root`. |
| **Core property** | A property the crate defines, with a fixed kind (§8): `background`, `padding_x`, `transition`, … |
| **Declared property** | A control-specific property declared in a style file: `indicator_size: length = …`. |
| **State** | A boolean flag from the consumer's set (`hover`, `checked`, `open`, …). |
| **Exclusive group** | States of which only the strongest active one counts (Luma: `disabled > pressed > hover`). |
| **Attribute** | A named axis set on an instance (`variant`, `size`); a scope in the language. |
| **Layer** | An ordered tier for style files (Luma: `sdk < look < app`). |
| **Mode** | A token variant chosen at bind time (`light`, `dark`). |
| **StyleKey** | Control + attributes + states + scale factor: what one render asks for. |

## 3. Configuration

Built once by the SDK at startup. It contains only what is global; everything per control comes from the files.

```rust
pub struct Config { /* opaque */ }

impl Config {
    pub fn builder() -> ConfigBuilder;
}

impl ConfigBuilder {
    pub fn layers(self, ordered: &[&'static str]) -> Self;            // lowest first
    pub fn states(self, states: &[StateDecl]) -> Self;                // at most 32
    pub fn exclusive(self, strongest_first: &[&'static str]) -> Self; // e.g. ["disabled", "pressed", "hover"]
    pub fn attributes(self, names: &[&'static str]) -> Self;          // e.g. ["variant", "size"]
    pub fn snap(self, policy: SnapPolicy) -> Self;
    pub fn build(self) -> Result<Config, ConfigError>;
}

pub struct StateDecl { pub name: &'static str, pub label: &'static str, pub doc: &'static str }
pub enum SnapPolicy  { None, Lengths }   // Lengths: every length snaps unless wrapped in exact()
```

Luma's configuration (in the SDK):

```rust
Config::builder()
    .layers(&["sdk", "look", "app"])
    .states(&[st("hover"), st("pressed"), st("focused"), st("disabled"), st("checked"), st("selected"),
              st("active"), st("expanded"), st("open"), st("invalid"), st("high_contrast")])
    .exclusive(&["disabled", "pressed", "hover"])
    .attributes(&["variant", "size"])
    .snap(SnapPolicy::Lengths)
    .build()?
```

## 4. Style files

Syntax, name resolution, cascade and motion are specified in the language document. The API:

```rust
pub struct Source { pub layer: &'static str, pub name: Arc<str>, pub text: Arc<str> }   // name: "radix/checkbox.lss"

pub struct Sheet { /* parsed, not yet validated */ }
impl Sheet {
    pub fn parse(source: Source) -> Result<Sheet, Diagnostics>;
}
```

Files are normally embedded with `include_str!` and parsed at startup.

## 5. Compile and bind

```rust
/// Validated rules, declarations and file-defined tokens for one set of files
/// (the SDK plus one look plus app files). Independent of native tokens and mode.
pub struct Stylesheet { /* Arc inside; cheap Clone */ }

impl Stylesheet {
    pub fn compile(config: Arc<Config>, sheets: impl IntoIterator<Item = Sheet>) -> Result<Stylesheet, Diagnostics>;
    pub fn controls(&self) -> impl Iterator<Item = ControlInfo<'_>>;      // for tools
    pub fn control(&self, name: &str) -> Option<ControlId>;
    pub fn part(&self, control: ControlId, name: &str) -> Option<PartId>;
    pub fn state(&self, name: &str) -> Option<StateId>;
    /// Typed handle to a declared property; errors if missing or of another kind.
    pub fn prop<T: FromValue>(&self, control: ControlId, name: &str) -> Result<Prop<T>, Diagnostic>;
}

/// A stylesheet bound to native tokens and a mode. Owns the resolve cache.
pub struct Theme { /* Arc inside; cheap Clone */ }

impl Theme {
    pub fn bind(stylesheet: Stylesheet, tokens: Arc<dyn TokenSource>, mode: &str) -> Result<Theme, Diagnostics>;
    pub fn resolve(&self, key: &StyleKey, overrides: Option<&Overrides>) -> Arc<Resolved>;
    pub fn inspect(&self, key: &StyleKey, overrides: Option<&Overrides>) -> Inspection;
    pub fn diagnostics(&self) -> &Diagnostics;   // evaluation problems found while resolving
}
```

**Compile** checks everything that does not need native tokens: syntax already passed; unknown control, part, attribute, state or property; a declaration outside the declaring layer; kind mismatches; ambiguous names; unknown built-in functions and arity; `extends` targets and cycles; reference cycles; token kind consistency across layers and modes.
**Bind** checks native token references and look functions, and selects the mode's token values.
A light/dark switch or a native token change (a new Shadcn CSS theme) is a new `bind` from the same `Stylesheet`.

## 6. Tokens

Two sources, one namespace:

- **File tokens**, from `tokens { … }` blocks (SDK `palette`, `metrics`, `typography`, `elevation`; Shadcn `shadcn.*`; Radix `radix.*`). Compiled into the `Stylesheet`; `mode` blocks are selected at bind.
- **Native tokens and functions**, from the consumer's `TokenSource`.

```rust
pub trait TokenSource: Send + Sync {
    /// Native roots this source provides, e.g. ["css"], ["gray", "accent", "tone", "destructive"].
    fn roots(&self) -> &[&'static str];
    /// `css.primary_foreground`. `None` means absent (see `first()`).
    fn token(&self, path: &str, mode: &str) -> Option<Value>;
    /// `gray[7]`, `tone[10]`.
    fn index(&self, root: &str, index: i32, mode: &str, key: &KeyView<'_>) -> Option<Value>;
    /// Look functions: `layer`, `shift`, `css_shadow`, `resting_shadow`, `lucide`, ….
    fn functions(&self) -> &[FunctionSig];
    fn call(&self, name: &str, args: &[Value], mode: &str, key: &KeyView<'_>) -> Result<Value, String>;
    /// Provenance for one native token, one step per line ("--primary in theme.css").
    fn trace(&self, path: &str, mode: &str) -> Vec<TokenStep>;
}

pub struct FunctionSig { pub name: &'static str, pub params: &'static [Kind], pub variadic: bool, pub returns: Kind }
pub struct KeyView<'a> { /* read-only attributes and states, e.g. for tone[n] */ }
pub struct TokenStep { pub label: Arc<str>, pub value: Option<Value> }
```

**Built-in functions** (crate, not consumer): `min`, `max`, `first`, `exact`, `alpha`, `mix`, `gray(lightness, alpha)`, `srgb`, `hsla`, `oklch`, `oklab`, `color(space, …)`, `shadow`, `text`, `timed`, `spring`.
**Built-in keywords:** `none`, `transparent`, `black`, `white`, `instant`, and the easing names (`linear`, `out_cubic`, …).

Luma implements `TokenSource` three times: SDK (empty or near-empty: its tokens are in files), Shadcn (`css.*` catalog plus `layer`, `first_layer`, `shift`, `px`, `css_shadow`, `css_font`), Radix (scales via `gray[n]` etc. plus the shadow helpers).

## 7. Values

The crate's own types, no gpui dependency:

```rust
pub enum Kind { Length, Number, Color, Shadow, Text, Font, Weight, Icon, Motion }

pub enum Value {
    Number(f32),                      // lengths are logical px; no units in files
    Color(Color),
    Shadow(Arc<[ShadowLayer]>),       // [] = no shadow
    Text(TextStyle),                  // size, line_height, weight; fields readable by path
    Font(Arc<str>),
    Weight(u16),
    Icon(Arc<str>),
    Motion(Motion),
    None,                             // `none`: not painted
}

pub struct Color       { pub l: f32, pub c: f32, pub h: f32, pub alpha: f32 }  // OKLCH, unclamped (§7.1)
pub struct ShadowLayer { pub x: f32, pub y: f32, pub blur: f32, pub spread: f32, pub color: Color, pub inset: bool }
pub struct TextStyle   { pub size: f32, pub line_height: f32, pub weight: u16 }
pub enum   Motion      { Instant, Timed { duration_ms: u32, easing: Easing }, Spring { stiffness: f32, damping: f32 } }
pub enum   Easing      { Linear, InCubic, OutCubic, InOutCubic, /* … SDK easings */ }
```

`Length` and `Number` share `Value::Number`; the property's kind gives the meaning (language §6, decisions 5 and 6).

### 7.1 Colors

Follows CSS Color Level 4.

| Authoring form | Example |
|---|---|
| OKLCH, OKLab | `oklch(0.62, 0.19, 259.8)`, `oklab(0.62, -0.03, -0.19)` |
| `color()` with a predefined space | `color(srgb, 0.41587, 0.50367, 0.36664)`, `color(display_p3, …)`, `color(a98_rgb, …)`, `color(prophoto_rgb, …)`, `color(rec2020, …)` |
| sRGB shorthands | `#1e293b`, `srgb(r, g, b)`, `hsla(h, s, l, a)`, `gray(lightness, alpha)` |

An optional last argument is alpha. Space names use `_` because `-` is subtraction (`display_p3` for CSS `display-p3`). Native tokens such as Shadcn `css.*` are parsed by the look with CSS syntax and arrive as `Value::Color`.

- **Stored as OKLCH**, unclamped, so wide-gamut colors (P3, Rec. 2020, ProPhoto) keep their full value through the cascade, the inspector and export.
- **Math in OKLab**: `mix`, `lerp` and `alpha` work in OKLab (the CSS default for `color-mix`), which avoids hue jumps and gives even transitions.
- **Output to gpui is sRGB**, converted with the CSS Color 4 gamut-mapping algorithm (reduce OKLCH chroma until the color fits). This matters for Shadcn themes from tools such as tweakcn, which often use OKLCH values outside sRGB.
- **Inspection** shows the authored form (from the source text), the stored OKLCH value, and whether the sRGB output was gamut-mapped.
- Conversion matrices are the published CSS Color 4 ones. The workspace already depends on `palette` 0.7 (OKLCH/OKLab); the other spaces need small hand-written conversions if `palette` lacks them.

## 8. Core properties

Defined by the crate. Every other property is declared in a style file.

| Kind | Properties | `.apply()` |
|---|---|---|
| color | `background`, `foreground`, `border`, `focus_border` | `background`, `foreground`, `border` (focus is drawn by the SDK's focus adorner) |
| length | `border_width`, `radius`, `padding`, `padding_x`, `padding_y`, `margin`, `gap`, `width`, `height`, `min_width`, `max_width`, `min_height`, `max_height`, `font_size`, `line_height` | yes |
| number | `opacity` | yes |
| shadow | `shadow` | yes |
| font / weight | `font_family`, `font_weight` | yes |
| motion | `transition` | no: read by the SDK animation runtime |

Text-related properties (`foreground`, `font_*`, `line_height`) inherit from `root` when a part does not set them.

## 9. Resolution

```rust
pub struct StyleKey {
    pub control: ControlId,
    pub attrs: Attrs,       // small sorted map: attribute → interned value
    pub states: StateSet,   // u32 bitset; normalized for the exclusive group by the crate
    pub scale: f32,
}

pub struct Resolved { /* every part of one control for one key */ }

impl Resolved {
    pub fn root(&self) -> &PartStyle;
    pub fn part(&self, part: PartId) -> &PartStyle;
    /// Interpolate two resolved styles (e.g. unchecked → checked). Colors, numbers and shadows
    /// lerp; other values switch at t = 0.5.
    pub fn lerp(a: &Resolved, b: &Resolved, t: f32) -> Resolved;
}

pub struct PartStyle {
    pub background: Option<Color>,
    pub foreground: Option<Color>,
    pub border: Option<Color>,
    pub focus_border: Option<Color>,
    pub border_width: Option<f32>,
    pub radius: Option<f32>,
    pub padding: Edges<Option<f32>>,
    pub margin: Edges<Option<f32>>,
    pub gap: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub opacity: Option<f32>,
    pub shadow: Option<Arc<[ShadowLayer]>>,
    pub font_family: Option<Arc<str>>,
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub font_weight: Option<u16>,
    pub transition: Option<Motion>,
    declared: DeclaredValues,
}

impl PartStyle {
    pub fn get<T: FromValue>(&self, prop: Prop<T>) -> Option<T>;   // declared properties
}

/// Typed handle to a declared property, obtained once from `Stylesheet::prop`.
pub struct Prop<T> { /* PropertyId + PhantomData<T> */ }
```

- **Motion:** for a change from key A to key B the animation runtime uses `resolve(B).part(p).transition` (destination state, language §16). Presence: enter uses the key with `open`, exit the key without it.
- **Instance overrides** replace builder-level edits such as today's `.motion(|m| …)`:

```rust
/// Per-instance values, ranked above every layer and owning their property in all states (like `:=`).
pub struct Overrides { /* small, Arc'd */ }
impl Overrides {
    pub fn new() -> Self;
    pub fn set(self, part: PartId, property: PropertyId, value: Value) -> Self;
}
```

- `resolve` never fails or panics. Evaluation problems leave the property unset and are recorded in `Theme::diagnostics`. Results are cached per `(StyleKey, Overrides)`.

### `gpui` feature

```rust
impl From<Color> for gpui::Hsla;          // gamut-mapped to sRGB (§7.1)
impl From<&ShadowLayer> for gpui::BoxShadow;

pub trait ApplyStyle: gpui::Styled + Sized {
    /// Applies the set core properties that map to gpui (§8). Never applies declared properties
    /// or `transition`.
    fn apply(self, style: &PartStyle) -> Self;
}
impl<E: gpui::Styled> ApplyStyle for E {}
```

## 10. Introspection

Read-only in phase 1. The crate returns data; the inspector UI lives above it.

```rust
pub struct Inspection {
    pub key: StyleKey,
    pub control: ControlInfo<'static>,
    pub axes: Axes,
    pub parts: Vec<PartInspection>,
}

pub struct ControlInfo<'a> {
    pub name: &'a str,
    pub doc: &'a str,                        // from `///` in the declaring file
    pub parts: Vec<PartInfo<'a>>,            // name, doc
    pub declared: Vec<PropertyInfo<'a>>,     // name, kind, doc, declared_at
    pub extends: Option<&'a str>,
}

pub struct Axes {
    pub attributes: Vec<(Arc<str>, Vec<Arc<str>>)>,   // values found in scopes: ("variant", ["classic","soft"])
    pub states: Vec<StateId>,                         // states the control's rules use
}

pub struct PartInspection {
    pub part: PartId,
    pub properties: Vec<PropertyInspection>,
    pub box_model: BoxModel,
}

pub struct PropertyInspection {
    pub property: PropertyId,
    pub kind: Kind,
    pub category: Category,
    pub doc: Option<Arc<str>>,
    pub value: Option<Value>,
    pub inherited_from_root: bool,
    pub winner: Option<RuleRef>,
    pub overridden: Vec<RuleRef>,         // lower-ranked candidates, highest first
    pub owned_by: Option<RuleRef>,        // the `:=` that removed less specific rules, if any
    pub expression: Arc<str>,             // source text of the winning value
    pub references: Vec<Reference>,       // what the expression read, with values and traces
}

pub struct RuleRef {
    pub source: Arc<str>,                 // "radix/checkbox.lss"
    pub layer: &'static str,              // or "inherited" via extends
    pub line: u32,
    pub column: u32,
    pub scope: Arc<str>,                  // "variant soft"
    pub states: Arc<str>,                 // ":checked:hover"
    pub via_extends: Option<Arc<str>>,    // "button.root"
    pub instance_override: bool,
}

pub enum Reference {
    Token { path: Arc<str>, value: Option<Value>, defined_at: Option<RuleRef>, native: Vec<TokenStep> },
    Property { part: PartId, property: PropertyId, value: Option<Value> },
    Const { name: Arc<str>, value: Option<Value>, defined_at: RuleRef },
}

pub struct BoxModel { pub size: (Option<f32>, Option<f32>), pub padding: Edges<f32>, pub border: f32, pub margin: Edges<f32> }
pub enum Category { Colors, Layout, Typography, Elevation, Motion, Other }
```

This one shape covers what Studio's per-control inspectors show today (value and source rows, box model, elevation, typography, motion) for every control in every look.

### Phase 2 (designed for, not built)
- `Stylesheet::edit(&RuleRef, PropertyId, new_source: &str) -> Result<Stylesheet, Diagnostics>`: live edits. Edits splice the source text by span, so comments survive.
- `Stylesheet::export(layer, attrs) -> String`: the lab's "export style" for a variant.

## 11. Diagnostics

```rust
pub struct Diagnostics(pub Vec<Diagnostic>);
pub struct Diagnostic { pub severity: Severity, pub source: Option<Arc<str>>, pub line: u32, pub column: u32, pub message: String }
pub enum Severity { Error, Warning }
```

`Diagnostics` implements `std::error::Error` and prints `radix/checkbox.lss:14:5: unknown part "indicatr" for checkbox (parts: indicator, glyph, label)` with the source line. The full list of load-time errors is in the language document §11.

## 12. Who owns what

| Piece | Lives in |
|---|---|
| Parser, compiler, cascade, values, resolve, cache, lerp, overrides, inspection data | `xgpui-luma-style` |
| `Config`; SDK style and token files; SDK `TokenSource`; templates using `.apply()`; reading `transition` in the animation runtime | SDK |
| Look style and token files; look `TokenSource`s; variant/size builder helpers | `look-shadcn`, `look-radix` |
| Inspector UI (from Studio's generic display) | SDK feature or a small `style-inspector` crate |
| Control behaviour, layout structure, focus adorner, icons rendering, animation runtime | SDK (unchanged) |

## 13. Testing contract

- **Crate:** unit tests per grammar rule, name-resolution step and cascade rank, with a test config and token source; no Luma.
- **SDK and looks:** one exhaustive test per look that resolves every control × attribute value × state combination, in both modes, with zero diagnostics.
- **Migration parity:** resolved values compared with today's native output per control, as #282's checkbox parity test does; #94's HCL parity tests are reused because the cascade keeps #94's semantics.

## 14. Open questions (API level)

Language questions are all decided (language §15). Remaining API decisions:

1. ~~Crate name.~~ **Decided 2026-11-19: `xgpui-luma-style`**, in `crates/style`. It replaces the #282 spike crate of the same name, which exists only on the #282 branch.
2. ~~File extension.~~ **Decided 2026-11-19: `.lss`.**
3. ~~Color model.~~ **Decided 2026-11-19: CSS Color 4 color spaces, OKLCH-based** (§7.1).
4. ~~Instance overrides.~~ **Decided 2026-11-19: owning, all-states values only** (§9). An instance value replaces its property in every state, including the disabled look; set a disabled value too if needed. Per-state overrides (`set_in(states, …)`) can be added later without breaking callers.

All API questions are decided or deferred.
5. ~~Built-in vs look functions.~~ **Deferred 2026-11-19** to implementation, decided per function with one rule: a function is built in if it needs no look data (pure math and constructors); it is a look function if it reads look data (the CSS catalog, mode deltas, Radix scales). §6's list is the starting point.
