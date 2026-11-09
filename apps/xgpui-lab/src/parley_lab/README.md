# Parley text field (lab prototype)

`ParleyTextField` is a text field and text area edited through GPUI's paragraph API on the Parley text system. It's a prototype for replacing the SDK `TextField` and `TextArea` (#105). This README covers how to test it, its public API, and how it differs from the SDK's conventions.

The code is in `text_field.rs` and `text_field/` here; the lab's screens are in `native.rs` and `text_field/page.rs`.

## Testing

### Headless tests
They run on a Parley-backed `TestAppContext`, and no window opens:

```bash
cargo test -p xgpui-lab --features test-support --bin xgpui-lab text_field
```

| File | Covers |
|---|---|
| `text_field/tests.rs` | editing, IME, selection, drag capture and autoscroll, text areas, per-line direction, options, look resolution, template chrome, accessibility, undo and redo, the lab page |
| `text_field/sdk_parity_tests.rs` | the SDK's text field and text area tests, ported; each names the SDK test it covers |

Test-writing notes:
- The `Fixture` window is `WIDTH` (240px) wide. The field is also the window's root view, so both frames lay it out at the same width.
- GPUI's test windows build no accessibility tree. The accessibility test reads the control's focused element, as the SDK's accessibility tests do.

### Performance measurements
These are manual and ignored by default. Run them in release:

```bash
cargo test --release -p xgpui-lab --features test-support --bin xgpui-lab measure_ -- --ignored --nocapture
```
- `measure_parley_text_area_hot_paths` uses the SDK text area's text and workloads, so the two can be compared.
- `measure_long_one_line_field` covers 10 K, 100 K and 1 M characters.

### In the app
The control needs the whole application on the Parley text system.

```bash
# Standalone Parley window, opened on the text field tabs
cargo run -p xgpui-lab -- --native-text-backend=parley --text-field
```
Its tabs are Text Fields, Text Right-to-left, Text Areas, Text Scrolling and Text States. Each has a hint on what to try.

```bash
# The main XGPUI Lab on Parley: xgpui-parley › Text Field
cargo run -p xgpui-lab -- --text-system=parley
```
Without `--text-system=parley`, the Text Field tab explains how to relaunch.

## Public API

### Building
`ParleyTextField::new(id)` returns a `ParleyTextFieldBuilder`, and `.spawn(cx)` returns an `Entity<ParleyTextField>`.

| Group | Methods |
|---|---|
| Look (Radix-owned) | `look(&Look)`, `variant`, `classic`, `surface`, `soft`, `size(TextFieldSize)`, `look_override`, `with_template_modifier`, `theme` (an SDK-level escape hatch that skips the look) |
| Content | `value`, `placeholder`, `font`, `rows(n)` (makes it a text area) |
| Options (SDK names) | `enabled`, `prefix_icon`, `tab_stop`, `compact`, `full_width`, `clean_on_escape`, `select_all_on_tab_focus`, `propagate_home_end_to_parent`, `max_clipboard_paste_bytes`, `validator` |
| Lab additions | `allow_tab_character`, `aria_label` |
| Callbacks (SDK `EventCallbacks`) | `on_event`, `on_value_changed`, `on_submit`, `on_focus_changed`, `on_enabled_changed` |

The look resolves in this order: the explicit `look`, then the ambient `Look` global, then `Look::built_in()`.

### Control (`ParleyTextField`)
| Group | Methods |
|---|---|
| Value | `value()` (line breaks as `\n`), `set_value` (no `Change` event; clears undo) |
| State | `set_enabled`, `is_enabled`, `set_invalid`, `is_invalid`, `set_validator`, `is_focused`, `set_size`, `set_placeholder`, `set_tab_stop`, `set_clean_on_escape`, `set_max_clipboard_paste_bytes` |
| Editing | `select_all`, `copy_selection`, `cut_selection`, `paste`, `undo`, `redo`, `can_undo`, `can_redo` |
| Inspection | `is_multiline`, `selection()` (byte positions with affinity), `scroll`, `layout_error` |

### Events
`ParleyTextFieldEvent` has `Change { value }`, `Submit { value }`, `FocusChanged { focused }` and `EnabledChanged { enabled }`. These are the SDK `TextFieldEvent` variants and fields.

## Differences from the SDK's conventions

`crates/sdk/src/controls/text/` follows the same pattern as the other SDK controls. This prototype matches its **public names** and reuses its **types**, `TextFieldTheme`, `TextFieldLook`, `TextFieldState`, `TextFieldTemplate` and `TextFieldRenderModel`, but not its **structure**:

| SDK convention | This prototype |
|---|---|
| One directory per control: `mod`, `model`, `state`, `template`, `theme`, `control`, `callback` | `text_field.rs` holds model, state, editing, keys, IME, layout and rendering in one file |
| A separate `textfield` and `textarea`, each with its own model, template and control | One control; `rows(n)` makes a text area, and `stacked.rs` is roughly the counterpart of `textarea/control/layout.rs` |
| Shared editing in `text/editing.rs` (`handle_key_down`, `EditableTextPolicy`, `KeyHandlingResult`) on character indices | Editing written inline, on byte positions with direction affinity |
| Look-neutral SDK builder (`template(..)`, `size(ControlSize)`), with a look-owned builder on top (`radix::TextField`) | One builder that is both, Radix-flavoured; no `template(..)` |
| The template draws the text from the render model | The template draws chrome around an empty value, and the Parley text is an overlay that repeats the template's geometry (`TextFieldTemplate` has no text slot) |
| `AttachmentHost` for role, tooltips and description | Role, value, placeholder and label set directly on the focused element |
| `observe_theme_revision` invalidates caches | The look is resolved every frame; layout caching is GPUI's frame cache plus a per-revision width cache |
| `TextFieldEventExt`, `TextField` entity alias, `#[non_exhaustive]` events | none of these |
| Control setters `set_template`, `set_font_family`, `set_look_override`; `state()`; change, submit, focus and blur counts | missing |
| `full_width` defaults to `false` | defaults to `true` |
| Tests inside each module (`#[cfg(test)] mod tests`) | Tests in separate files under `text_field/` |
| Disabled or invalid fields drop the focus ring's room (a known SDK bug, #105, #110) | A transparent ring keeps the room, so the size never changes |

Before migrating into the SDK, it needs restructuring to that layout, with behaviour and tests unchanged:
- separate field and area controls;
- a model / state / control split;
- shared editing on byte positions;
- a look-neutral builder;
- a text slot in `TextFieldTemplate`.
