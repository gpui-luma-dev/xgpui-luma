# Luma style language — design (draft for review)

Status: implemented in `crates/style` on `issue-285/style-lang` (2026-11-19). All open questions in §15 are decided; §17 lists what implementation changed.
Scope: the authoring language for control styles and tokens. The Rust API around it is in [style-crate-api.md](style-crate-api.md), which defers to this document for syntax, name resolution and the cascade.

---

## 1. Evidence: what the #94 HCL corpus needs

All 103 HCL files on `issue-94/hcl-spike` (SDK, Shadcn, Radix) were inventoried. The language must express everything in them.

| Measure | Count |
|---|---|
| Files / non-comment lines | 103 / 5,643 |
| `state` blocks | 527 |
| `part` blocks | 280 |
| `variant` blocks / distinct variant names | 86 / 24 |
| `size` blocks / distinct size keys | 85 / 12 (`sm md lg`, `1`–`9`) |
| `motion` blocks | 26 |
| `extends` | 32 (variant→variant, part→part, part→other control's part) |
| Distinct property names | ~493 (top 15 are box/paint; ~470 are control-specific geometry) |
| States used | `disabled` 182, `hover` 115, `pressed` 96, `checked` 43, `selected` 31, `high_contrast` 27, `focused` 15, `invalid` 13, `active` 5 |
| Deepest state nesting | 3 (`checked:high_contrast:disabled` ×16) |
| Token roots | `palette` 259, `css` 229, `radix` 194, `shadcn` 129, `metrics` 90, `typography` 51, `elevation` 9 |
| Cross-control references | 13 (`button.root`, `popover.panel.offset_y`, …) |
| Functions | `gray` 51, `alpha` 49, `first` 33, `text` 25, `shadow` 24, `lucide` 23, `shift` 10, `first_layer` 9, `layer` 8, `css_shadow` 8, `timed` 6, `max` 7, Radix shadow helpers 12, … |

Two facts drive the cascade design (§8):

- **Base states leak into variants on purpose.** Of 76 variant assignments where the base part has state rules for the same property, **57 omit some of those states** and rely on the base's state rule still applying (52 of them `disabled`; also `selected`, `hover`, `pressed`, `invalid`). Example: SDK `popup_menu` variant `outline` sets only `background` and expects the base's hover/pressed fills.
- **The other 19 restate every base state** to stop the leak (Radix `soft`, Shadcn `content_only`). These are the verbose files.
- **Interaction states are exclusive.** Only one rule (`disabled` + `hover`) combines two of `hover`/`pressed`/`disabled`; everything else follows "disabled beats pressed beats hover".

## 2. Goals

1. Cover every construct in the corpus; convert the corpus mechanically with identical results.
2. One state level: `:checked:hover { … }` instead of nested `state` blocks.
3. The style files are the schema: declarations carry kinds and doc comments, so **no per-control Rust** is needed for validation or inspection.
4. No units: a property's declared kind says what a number means.
5. Every value keeps its source location for the inspector.

## 3. Files and top level

A file holds any of three top-level items:

```
const  { … }      // lexical constants
tokens { … }      // token definitions (SDK palette, Shadcn `shadcn.*`, Radix `radix.*`)
button { … }      // a control block (any other identifier)
```

Each file is loaded into one **layer**, chosen by the consumer (`sdk`, `look`, `app`). Several files may contribute to the same control or token group; they merge (§8).

## 4. Control blocks

```
/// Push button and the base of toggle, popup trigger and others.
button {
  /// Height of the control row.
  height: length = metrics.control.control_height
  icon_size: length = metrics.control.icon_size
  font_size = typography.text.label.size            // core property: no kind needed

  /// The whole button face.
  part root {
    background = palette.surface.subtle.background
    foreground = palette.app.foreground
    border     = palette.border.default
    :hover    { background = palette.state.hover.background }
    :pressed  { background = palette.state.pressed.background }
    :disabled { background = palette.state.disabled.background
                foreground = palette.state.disabled.foreground }
  }

  variant ghost {
    part root { background = transparent  border = none  shadow = none }
  }

  size sm { padding_x = 12  font_size = 12  icon_size = 14 }

  part root { transition = motion.default }   // motion is an ordinary part property (§16)
}
```

| Construct | Meaning |
|---|---|
| `name: kind = expr` | **Declaration.** Allowed only in the control's declaring layer (the lowest layer that mentions the control). Gives the property a kind and, with `///`, a doc. |
| `name = expr` | Assignment to a core property or a declared property. Unknown name → error. |
| `name := expr` | **Owning assignment** (§8.4): rules for `name` from less specific scopes stop applying. |
| `part p { … }` | A part. Parts are declared by the declaring layer; other layers may only use them. The control block itself is part `root`. |
| `variant v { … }`, `size s { … }` | **Scopes**: rules inside apply only when the instance has that attribute value. Any attribute the consumer declares works the same way (`variant`, `size`, `orientation`, …). Scopes nest. |
| `:s1:s2 { … }` | **State block**: properties only, no nesting. Applies when all listed states hold. |
| `extends x` | On a control, part or scope header: start from another definition (§8.5). |
| `///` | Doc comment for the next declaration, part or control; shown by the inspector. |

Control-level properties (`height`, `icon_size`) belong to `root`. Parts read them by bare name (§7).

## 5. Token blocks

```
tokens {
  metrics {
    border_width = 1
    control { height = 36  control_height = height  padding_x = 14  gap = 8  radius = 6 }
  }
  palette {
    app   { background = gray(98.04, 1)  foreground = gray(11.18, 1) }
    focus { ring = gray(20, 1) }
  }
  elevation {
    control = [shadow(0.05, 0, 1, 2, 0)]
  }
}

tokens {                                  // a Shadcn file
  shadcn {
    primary { background = css.primary  hover = layer(css.primary, hover) }
    mode dark { state_delta { filled_hover = 0.03 } }
  }
}
```

- Groups nest; a token's path is its group path (`palette.focus.ring`).
- Tokens can be scoped by attribute, like controls: `metrics { control { size sm { height = 28 } } }`. The most specific scope matching the instance wins, so `metrics.control.control_height` follows the instance's size (Shadcn needs this).
- `mode light { … }` / `mode dark { … }` hold values used only in that mode. Control blocks never branch on mode.
- Token kinds are inferred from their values: function return kinds give `color`, `shadow`, `text`, `motion`; strings give `font`; plain numbers give `number`, which may be used in any `length` or `number` property (the property's kind decides the meaning). An optional annotation, `height: length = 36`, makes the kind exact and can carry a `///` doc.
- A token overridden in a later layer must keep the original's kind; a token defined in several `mode` blocks must have the same kind in each.
- **Native roots** come from Rust through the token source, not from files: Shadcn `css.*` (the CSS catalog), Radix `gray[n]`, `accent[n]`, `tone[n]`, `destructive[n]`.

## 6. Expressions

| Form | Example | Notes |
|---|---|---|
| Number | `12`, `0.56` | Unit comes from the property's kind. |
| Hex color | `#1e293b`, `#1e293b80` | sRGB. Other color forms are functions: `oklch(…)`, `oklab(…)`, `color(display_p3, …)` and the other CSS Color 4 spaces (API §9). |
| String | `"System UI"` | Font families, icon names. |
| Keyword | `none`, `transparent`, `instant`, `out_cubic`, `black` | Built into the crate (API §8); `none` = not painted (an absent `Option`). |
| Path | `palette.app.foreground`, `typography.text.body.size` | Reaches into record values (`text(…)` has `size`, `line_height`, `weight`). |
| Index | `gray[7]`, `tone[10]` | For scale-like native roots. |
| Call | `alpha(css.ring, 0.5)`, `first(css.card, css.background)` | Built-in functions (`min`, `max`, `first`, `alpha`, `gray`, `shadow`, `text`, `timed`, …) plus look functions from the token source (`layer`, `shift`, `css_shadow`, …); API §8. |
| List | `[shadow(0.1, 0, 1, 3, 0), shadow(0.1, 0, 1, 2, -1)]` | Shadow layers. |
| Arithmetic | `max(indicator_size - glyph_inset * 2, 0)` | `+ - * /`, unary `-`, parentheses. |

Kinds: `length`, `number`, `color`, `shadow`, `text`, `font`, `weight`, `icon`, `motion`. Every expression is kind-checked at load time against its property.

## 7. Name resolution

A bare or dotted name resolves in this order; matching more than one is an **error**, not a silent pick.

1. **Same part, same key**: `border = background` reads this part's resolved `background` for the current states and attributes.
2. **Root**: a control-level property (`indicator_size`).
3. **Another part**: `glyph.foreground`.
4. **Constants**: innermost `const` block outward, then file, then global (SDK).
5. **Tokens**: `metrics`, `palette`, `elevation`, `typography`, then the look's roots (`shadcn`, `radix`, `css`, …).
6. **Other controls** (geometry only): `popover.panel.offset_y`.

A property cannot refer to itself: in a look, `gap = gap * 2` is a cycle error. Use the token instead (`gap = metrics.control.gap * 2`).

Reference cycles are errors, reported with the full chain.

## 8. Cascade (normative)

For an instance key (control, attributes, states) and one `(part, property)`:

### 8.1 Normalize the key
The consumer declares an **exclusive state group**, for Luma `disabled > pressed > hover`: only the highest active member stays in the key. `:checked:hover` therefore does not apply while pressed. (The one corpus rule that needs `disabled` and `hover` together is rewritten.)

### 8.2 Collect candidates
Every assignment whose control matches (or is inherited through `extends`), whose scope attributes are all in the key, and whose states are all in the key.

### 8.3 Rank (highest wins)
1. **State count** of the rule (more wins).
2. **Interaction state**: a rule containing a member of the exclusive group beats one that doesn't (a `:hover` beats a `:selected` at equal count, as in #94).
3. **Attribute count** of its scope (a `variant ghost` rule beats the base rule for the same states).
4. **Layer** (`app` > `look` > `sdk`). A rule reached through `extends` counts at the layer where the `extends` was written, if that is higher than its own.
5. **Own over inherited**: the control's own rules beat rules reached through `extends`.
6. **Source order** (later wins).

Instance overrides (API §6) sit above all of this: a value set on one control instance owns its property in every state, like `:=` in a top layer.

This is #94's semantics written as one rank: state rules from the base still apply inside a variant unless the variant writes the same state; a look overrides the SDK coordinate by coordinate.

### 8.4 Owning assignment `:=`
`background := tone[5]` written in layer *L* inside scope *S* removes, before ranking, every candidate for that `(part, property)` whose scope attributes are a subset of *S*'s, in layer *L* or below, **except** candidates in layer *L* with exactly *S*'s attributes. So the owning scope's own state rules still apply; everything less specific, and everything the layers below say for that scope, does not.

This is the fix for the 19 "restate every base state" cases:

```
// Radix soft, was ~50 lines of nested state blocks
variant soft {
  part indicator {
    background := tone[5]
    border     := tone[5]
    :disabled { background = transparent  border = transparent }
  }
  part glyph {
    foreground := tone[11]
    :high_contrast { foreground = tone[12] }
    :disabled      { foreground = gray[8] }
  }
}
```

A look's base scope can also own a property against the SDK: `background := radix.role.surface` stops the SDK's hover and pressed fills, replacing Radix's current "restate hover and pressed" workaround.

### 8.5 `extends`
| Where | Effect |
|---|---|
| `toggle extends button { … }` | Every `button` rule from every layer is a candidate for `toggle`, below toggle's own rules at the same layer. So a look's `button` overrides reach `toggle`. |
| `variant ghost extends outline { … }` | Rules scoped `variant outline` also match `variant ghost`, ranked below ghost's own rules. |
| `part branch extends item { … }` | Rules for part `item` also apply to part `branch`, ranked below branch's own. |
| `part trigger extends button.root { … }` | Rules for `button`'s `root` (base and the variant of the same name) apply to `trigger`. The only way to share paint across controls. Written in a look, these rules beat the SDK's own trigger rules (rank 4). |

A part can extend another part only outside variant, size and state blocks. Cycles are errors.

### 8.6 Evaluate
The winning expression is evaluated for the key. `none` leaves the property unset. Lengths snap to device pixels on output unless the value is `exact(…)` or the consumer's policy says otherwise. A property that reads another property sees its unsnapped value, so rounding does not compound (`line_height = font_size * …`).

## 9. Properties and kinds

**Core properties** (declared by the consumer once; `.apply()` maps them to gpui):

| Kind | Properties |
|---|---|
| color | `background`, `foreground`, `border`, `focus_border` |
| length | `border_width`, `radius`, `padding` (`padding_x`, `padding_y`), `margin`, `gap`, `width`, `height`, `min_width`, `max_width`, `min_height`, `max_height`, `font_size`, `line_height` |
| number | `opacity` |
| shadow | `shadow` |
| font / weight | `font_family`, `font_weight` |
| motion | `transition` (not applied by `.apply()`; read by the SDK's animation runtime, §16) |

**Declared properties**: everything else (`indicator_size`, `track_height`, `icon`, `chevron`, `caret`, `placeholder`, `period`, `drop_transition` …), declared in the control's declaring layer with a kind and a doc comment. About 470 names in today's corpus.

## 10. Grammar (pest)

```pest
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
COMMENT    = _{ !"///" ~ "//" ~ (!"\n" ~ ANY)* | "/*" ~ (!"*/" ~ ANY)* ~ "*/" }
doc        = @{ "///" ~ (!"\n" ~ ANY)* }

file         = { SOI ~ (const_block | tokens_block | control)* ~ EOI }

const_block  = { kw_const ~ "{" ~ binding* ~ "}" }
binding      = { doc* ~ ident ~ (":" ~ ident)? ~ "=" ~ expr }       // optional kind annotation

tokens_block = { kw_tokens ~ "{" ~ token_item* ~ "}" }
token_item   = _{ mode_block | const_block | token_scope | group | binding }
mode_block   = { kw_mode ~ ident ~ "{" ~ token_item* ~ "}" }
token_scope  = { ident ~ key ~ "{" ~ token_item* ~ "}" }             // size sm { … } inside tokens
group        = { doc* ~ ident ~ "{" ~ token_item* ~ "}" }

control      = { doc* ~ ident ~ extends? ~ "{" ~ item* ~ "}" }
item         = _{ const_block | part | states | scope | decl }
part         = { doc* ~ kw_part ~ ident ~ extends? ~ "{" ~ (states | decl)* ~ "}" }
scope        = { ident ~ key ~ extends? ~ "{" ~ item* ~ "}" }       // variant ghost { }  size sm { }  size 1 { }
key          = @{ (ASCII_ALPHANUMERIC | "_")+ }
states       = { state+ ~ "{" ~ decl* ~ "}" }
state        = ${ ":" ~ ident }
extends      = { kw_extends ~ path }
decl         = { doc* ~ ident ~ (":" ~ ident)? ~ assign ~ expr }    // name: kind = expr
assign       = { ":=" | "=" }

expr         = { prefix* ~ postfix ~ (infix ~ prefix* ~ postfix)* }   // Pratt
prefix       = { "-" }
infix        = { "+" | "-" | "*" | div }
div          = @{ "/" ~ !("/" | "*") }                              // never the start of a comment
postfix      = { primary ~ index* }
index        = { "[" ~ expr ~ "]" }
primary      = _{ "(" ~ expr ~ ")" | call | list | number | hex_color | string | path }
call         = { path ~ "(" ~ (expr ~ ("," ~ expr)* ~ ","?)? ~ ")" }
list         = { "[" ~ (expr ~ ("," ~ expr)* ~ ","?)? ~ "]" }

number       = @{ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }
hex_color    = @{ "#" ~ (ASCII_HEX_DIGIT{8} | ASCII_HEX_DIGIT{6}) }
string       = @{ "\"" ~ (!("\"" | "\n") ~ ANY)* ~ "\"" }
path         = @{ ident ~ ("." ~ ident)* }
ident        = @{ !keyword ~ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }

keyword      = @{ ("const" | "tokens" | "mode" | "part" | "extends") ~ !(ASCII_ALPHANUMERIC | "_") }
kw_const     = @{ "const"   ~ !(ASCII_ALPHANUMERIC | "_") }
kw_tokens    = @{ "tokens"  ~ !(ASCII_ALPHANUMERIC | "_") }
kw_mode      = @{ "mode"    ~ !(ASCII_ALPHANUMERIC | "_") }
kw_part      = @{ "part"    ~ !(ASCII_ALPHANUMERIC | "_") }
kw_extends   = @{ "extends" ~ !(ASCII_ALPHANUMERIC | "_") }
```

Notes:
- Statements need no separators: an expression ends at the first token that cannot continue it, so `background = transparent  border = none` on one line is fine.
- Identifiers have no `-` (it is subtraction). Two corpus names change: `minimal-edge` → `minimal_edge`.
- `scope` is "identifier, key, `{`"; keys may start with a digit (Radix sizes `1`–`9`). Whether the first identifier is a declared attribute is checked at load time, with a clear error.
- There is no `motion` keyword and no record syntax: motion is the `transition` property (§16).

## 11. Diagnostics

Every error carries file, line and column, with the source line shown. Load-time errors:

| Error | Example |
|---|---|
| Syntax | `expected "=" after property name` |
| Unknown control / part / attribute / state | `unknown part "indicatr" for checkbox (parts: indicator, glyph, label)` |
| Unknown property | `"backgroud" is not a core or declared property of checkbox` |
| Declaration outside declaring layer | `"track_height" can only be declared in the sdk layer` |
| Kind mismatch | `border expects color, found length (metrics.control.radius)` |
| Ambiguous name | `"panel" is both a part of popover and a token group` |
| Unknown token / function / wrong arity | `alpha expects 2 arguments, found 1` |
| Cycle | `cycle: glyph_size → indicator_size → glyph_size` |
| `extends` problem | `variant ghost extends unknown variant "outlin"` |

## 12. Examples converted from the corpus

### SDK `popup_menu` (HCL 52 lines → 30)
```
popup_menu {
  height        = metrics.control.control_height
  padding_x     = metrics.control.padding_x
  padding_y     = metrics.control.padding_y
  gap           = metrics.control.gap
  icon_size     = metrics.control.icon_size
  radius        = metrics.control.radius
  font_size     = typography.text.label.size
  line_height   = typography.text.label.line_height
  /// Gap between the trigger and the menu.
  menu_offset_y: length = gap * 0.5

  /// The button that opens the menu.
  part trigger {
    background   = palette.surface.subtle.background
    foreground   = palette.app.foreground
    border       = palette.border.default
    shadow       = elevation.control
    focus_border = palette.focus.ring
    :hover    { background = palette.state.hover.background }
    :pressed  { background = palette.state.pressed.background }
    :disabled { background = palette.state.disabled.background
                foreground = palette.state.disabled.foreground }
  }

  variant outline { part trigger { background = palette.app.background } }   // hover/pressed still apply
  variant ghost   { part trigger { background = transparent  border = none  shadow = none
                                   :pressed { background = palette.state.hover.background } } }

  // Motion (was a `motion { }` block): each role is the transition of the part that moves.
  part menu      { transition = motion.default }    // presence: same timing in and out; a :open value would differ them
  part highlight { transition = timed(180, out_cubic) }
  part chevron   { transition = motion.default }                                           // disclosure
  part submenu   { transition = motion.default }
}
```

### Shadcn `button` variants (`content_only`: 17 lines → 4)
```
button {
  part root {
    background   = shadcn.primary.background
    foreground   = shadcn.primary.foreground
    border       = background
    focus_border = css.ring
    shadow       = none
    :hover    { background = shadcn.primary.hover }
    :pressed  { background = shadcn.primary.pressed }
    :disabled { background = css.muted  foreground = css.muted_foreground }
  }

  variant secondary {
    part root {
      background = shadcn.secondary.background
      foreground = shadcn.secondary.foreground
      :hover   { background = shadcn.secondary.hover }
      :pressed { background = shadcn.secondary.pressed }
    }                                                     // base :disabled still applies
  }

  variant content_only {
    part root { background := transparent  border := transparent  focus_border := transparent
                foreground := css.foreground }
  }

  size sm { padding_x = 12  font_size = 12  icon_size = 14 }
  size md { padding_x = 16  font_size = 14  icon_size = 16 }
  size lg { padding_x = 20  font_size = 16  icon_size = 18 }
}
```

### Shadcn `popup_menu` (cross-control `extends`)
```
popup_menu {
  part trigger extends button.root { }
  variant outline { part trigger extends button.root { } }   // paints like button variant outline
  variant ghost   { part trigger extends button.root { } }
}
```

### Radix `checkbox` (HCL 164 lines → ~55)
```
checkbox {
  gap = 8
  indicator_radius = none                     // derived from the radius prop

  size 1 { indicator_size = 14  glyph_size = 9 }
  size 2 { indicator_size = 16  glyph_size = 10 }
  size 3 { indicator_size = 20  glyph_size = 12 }

  part root  { focus_border = radix.role.focus }
  part label { foreground = radix.role.foreground  :disabled { foreground = radix.role.muted_foreground } }

  // Surface: an unchecked face never changes, so own background against the SDK.
  part indicator {
    background := radix.role.surface
    border      = gray[7]
    shadow      = none
    :checked                         { background = radix.role.primary  border = background }
    :checked:hover                   { background = tone[10] }
    :checked:high_contrast           { background = tone[12] }
    :disabled                        { background = transparent  border = gray[6] }
    :checked:high_contrast:disabled  { background = transparent  border = gray[6] }
  }
  part glyph {
    foreground = gray[11]
    :checked                         { foreground = radix.role.primary_foreground }
    :checked:high_contrast           { foreground = tone[1] }
    :disabled                        { foreground = gray[8] }
    :checked:high_contrast:disabled  { foreground = gray[8] }
  }

  variant classic {
    part indicator {
      border = gray[3]
      shadow = resting_shadow(gray[5], alpha(gray[2], 0.5))
      :checked  { shadow = checked_shadow(alpha(glyph.foreground, 0.35), alpha(black, 0.28)) }
      :disabled { shadow = resting_shadow(gray[5], alpha(gray[3], 0.4)) }
      :checked:high_contrast:disabled { shadow = resting_shadow(gray[5], alpha(gray[3], 0.4)) }
    }
  }

  variant soft {
    part indicator { background := tone[5]  border := tone[5]
                     :disabled { background = transparent  border = transparent } }
    part glyph     { foreground := tone[11]
                     :high_contrast { foreground = tone[12] }
                     :disabled      { foreground = gray[8] } }
  }
}
```
The `:checked:high_contrast:disabled` rules exist because a three-state rule outranks `:disabled` (§8.3 rank 1). That is #94's behaviour too; see open question 2.

## 13. Converting the #94 corpus

| HCL | New | Mechanical? |
|---|---|---|
| `part "x" { }` | `part x { }` | yes |
| `state "a" { state "b" { } }` | `:a:b { }` (flattened) | yes |
| `variant "v" { }`, `size "s" { }` | `variant v { }`, `size s { }` | yes |
| `extends = y` inside a block | `extends y` on the header | yes |
| `const`, values, functions, `gray[n]` | unchanged | yes |
| `motion { role = … }` | `part <moving part> { transition = … }` per the §16 table; `{ enter, exit }` becomes base and `:open` values | yes, from the table; new parts (`highlight`, `selection`, `submenu`) declared by hand |
| `tokens { }`, look namespaces (`shadcn { }`, `radix { }`) | inside `tokens { }` | yes |
| Control geometry and named colors | add `: kind` in the SDK file | **manual or from Rust field types** (~470 names) |
| The one `disabled` + `hover` rule | rewrite | manual |
| `minimal-edge` | `minimal_edge` | manual (2 places) |

Expected size after conversion: roughly 3.5–4k lines from 5,643, mostly from flattening state blocks; more where `:=` replaces restated states (the 19 cases).

## 14. Parser notes (non-binding)

- pest + `pest_derive` for the grammar; pest's `PrattParser` for expressions.
- AST construction by hand over `Pairs`, or with `pest_consume` to shorten it; either works, the grammar is the same.
- Comments are discarded by the parser; phase-2 editing splices source text by span so comments survive.

## 15. Open questions

1. ~~Declarations in files vs Rust.~~ **Decided 2026-11-19: in the style files.** Control-specific properties are declared (`name: kind = value`, with `///` docs) in the control's declaring-layer style file, normally the SDK's file beside the control. Rust declares only global configuration: states, the exclusive group, attributes, layers and core properties.
2. ~~State count before attribute count (§8.3).~~ **Decided 2026-11-19: a variant is a set of overrides on a copy of the base style.** A variant assignment replaces only the state it is written for, so `variant outline { background = C }` changes the resting background and the base's `:hover` still applies. This is the §8.3 ranking (and #94's behaviour). A variant that should replace a property in every state uses `:=`. Variants are a look feature; the SDK's current variants are expected to be removed before this work starts (some are really orientation, role or kind).
3. ~~`:=` spelling.~~ **Decided 2026-11-19: keep `:=`.** It is already in the grammar and examples, and the inspector marks owning assignments, so being easy to miss in text matters less. Renaming later is a small refactor.
4. ~~Motion.~~ **Decided 2026-11-19: motion is an ordinary part property** (`transition`), chosen by the destination state, with presence through `:open`. No `motion` block, `presence` kind or record syntax. See §16.
5. ~~Token kinds.~~ **Decided 2026-11-19: inferred, with optional annotations** (§5). The only unchecked case is a number token used with the wrong numeric meaning (an opacity used as a padding); the inspector shows value and source for those.
6. ~~Units.~~ **Decided 2026-11-19: no units.** Lengths are logical pixels, durations are milliseconds, and the property's declared kind gives every number its meaning. Optional units (`px`, `ms`) can be added later without breaking existing files, if wrong-meaning number tokens turn out to be a problem.

All six questions are decided.

## 16. Motion as a part property (decided 2026-11-19)

### Today
Motion is a second styling system beside the stylesheet:
- 26 `motion_profile!` role structs in the SDK (`state`, `selection`, `thumb`, `presence`, …), a `MotionDefaults` baseline (`theme/stylesheet/motion.rs`, 511 lines; `theme/motion.rs`, 132 lines), look-level `look.motion(|m| …)` edits, and instance `.motion(|m| …)` edits, each with its own resolution order (`motion/DESIGN.md`).
- In #94 it became a `motion { }` block per control, with a separate `presence` record kind (`{ enter, exit }`).
- None of it uses parts, states, variants, layers or the inspector the way paint does. Studio's inspector has its own ~780-line `motion.rs` for it.

### Proposal
**A motion role is the `transition` property of the part that moves.** One core property, kind `motion` (`timed(…)`, `spring(…)`, `instant`), set and overridden like any other.

```
switch {
  part thumb {
    transition = default_timed
  }
}

popup_menu {
  part menu {                          // presence: the menu appearing and disappearing
    transition = instant               // exit (leaving :open) is immediate …
    :open { transition = default_timed }   // … enter (arriving at :open) is timed
  }
  part highlight { transition = timed(180, out_cubic) }
  part chevron   { transition = default_timed }        // disclosure
  part submenu   { transition = default_timed }
}
```

Two rules make this enough:

1. **Destination state picks the transition** (as in CSS). A change from state A to state B uses the `transition` resolved for B. For presence that gives enter = the value at `:open`, exit = the value without `:open`, so `dismissal` (timed in, instant out) needs no record type.
2. **The runtime is unchanged.** `VisualTransition`, springs, trajectories, overlay presence and `motion_enabled(false)` stay in the SDK. The SDK only asks the style for a part's `transition` instead of reading a role struct.

### Mapping the current roles

| Role (controls) | Becomes |
|---|---|
| `state` (checkbox, radio, switch, toggle) | `transition` of `indicator` / `thumb` / `root` |
| `selection` (control group, stepper) | `transition` of `selection` (the moving highlight) |
| `indicator` (tabs) | `transition` of `indicator` |
| `highlight` (popup menu) | `transition` of `highlight` |
| `thumb` (slider) | `transition` of `thumb` |
| `value` (progress) | `transition` of `fill` |
| `period_ms` (progress) | declared `period: number` on `fill` (a loop, not a transition) |
| `extent` (split view, resizable panels) | `transition` of `pane` |
| `expansion` (accordion, tree, sidebar) | `transition` of `content`, state `:expanded` |
| `disclosure` (tabs, menus, search selector) | `transition` of `chevron` |
| `content` (tabs) | `transition` of `panel` |
| `presence`, `popup`, `submenu`, `tabs_overlay` | `transition` of the appearing part, state `:open` |
| `position` (tree view) | `transition` of `row` |
| `drop_layout` (table, tree, tabs, sidebar, accordion) | declared `drop_transition: motion` on the item part (it shares `row` with `position`) |
| `scroll` (app) | `transition` of `viewport` |
| `transition` (app-wide default) | token `motion.default`, read by SDK files |

New states needed: `open`, `expanded` (`expanded` was already listed as a flag in #94).

### What it removes
- The 26 role structs, `MotionDefaults`, the look and builder motion plumbing, and the `motion { }` block and `presence` kind from the language.
- Studio's separate motion inspector: motion appears as one more row per part, with its source, like paint.

### What it needs elsewhere
- **Instance overrides.** Today `.motion(|m| m.content = instant())` edits one instance. In this model that is an instance-level style override (a top layer for one key), which the crate API must provide anyway for per-instance paint. Design it once for both.
- **Parts that only exist for motion** (`highlight`, `selection`, `submenu`) must be real parts of those controls. Most already are visual pieces; a few will be new.

## 17. What implementation changed (2026-11-19)

The crate implements this design. Writing it against the converted #94 files changed six things, now reflected above:

1. **Token scopes.** Token files can scope values by attribute (`size sm { height = 28 }`); Shadcn's size-dependent metrics need it (§5, grammar).
2. **`extends` and layers.** A rule reached through `extends` ranks at the layer where the `extends` was written. Otherwise Shadcn's `part trigger extends button.root` would lose to the SDK's own trigger rules (§8.3, §8.5).
3. **Snapping on output only.** Property references see unsnapped values (§8.6).
4. **`rec2020`** uses CSS Color 4's current 2.4 gamma; the five-space example in the colors spec then agrees within 0.002.
5. **Misspelled token roots** (`pallete.x`) are reported when the theme is bound, because native roots (`css`, `gray`) are known only then. The message suggests the closest token group.
6. **Part `extends`** is allowed only outside variant, size and state blocks (§8.5).

The grammar also needed one fix: `/` must not start `//` or `/*`, or a `///` doc comment after a value is read as division.
