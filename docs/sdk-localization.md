# SDK localization contracts

Applications publish one `LocaleContext` with `set_locale_context`, or use the
`xgpui-luma-l10n` SDK adapter. Locale changes refresh windows. Direction controls
SDK chrome; application content, data geometry, and custom presenters remain
application-owned. User text keeps automatic paragraph direction.

## Control behavior

- Horizontal groups, toolbar/tabs, choice indicators, selection rows, menus,
  sidebars, trees, and stepper flow follow logical start/end. Physical arrow keys
  follow the visual direction in horizontal navigation and tree/menu expansion.
- Tree indentation and drag-target insets move to logical start. Tree item order,
  vertical movement, selection IDs, and disabled-item rules do not change.
- Table columns reverse visually while retaining source indices. Header/body
  widths remain paired; resize handles move to logical column boundaries and
  horizontal deltas reverse. Switching direction during a resize cancels the
  resize and restores its original column configuration. Row reordering remains
  vertical. Default text cells align at start, numeric cells at end; custom
  content and explicit styles retain ownership of their alignment.
- Explicit slider/progress directions and explicit horizontal stepper directions
  remain physical overrides. A stepper without a direction override follows the
  locale. Vertical stepper side labels follow logical start/end.
- Popover `*Start` placements resolve against the live locale, including smart
  above/below fallback. Tooltip `AboveStart`, `AboveEnd`, `BelowStart`, and
  `BelowEnd` align logically and retain viewport clamping/fallback. Existing
  tooltip `Above` and `Below` remain centered.
- Templates that have no `App` receive an explicit snapshot: for example
  `FloatingMenuRenderModel.direction` and `TooltipRenderModel.direction`.
  The legacy `render_tabs_overlay_host` helper keeps its LTR contract; localized
  callers use `render_tabs_overlay_host_with_direction` and pass the current
  context direction.

## Icons and application overrides

`ControlIcon::mirrored_in_rtl()` and `MenuItemIcon::mirrored_in_rtl()` opt directional
artwork into horizontal reflection. Plain icons, logos, clocks, and media artwork
are unchanged. Standard right/down disclosure pairs mirror with continuous
left/down animation; custom disclosure pairs retain their supplied artwork.
`ControlIcon::element(color, size)` resolves the live direction during rendering.
Popup/split-button trailing icons accept `ControlIcon` as well as Lucide icons.

SDK-provided defaults use `SdkStrings`; explicit labels/placeholders, including
empty strings, take precedence. Selector fallback copy is `select_placeholder`;
autocomplete uses `search_placeholder`. Autocomplete status framing has an adapter
hook; selected labels and machine event/key identifiers are data. The l10n adapter
localizes status names, formats match counts, and isolates interpolated data.
Both look text-field builders already default to an empty placeholder; English
`Search…` occurrences in their builder tests are explicit application examples.

## Source compatibility

Consumers with exhaustive matches must handle the new `Mirrored` icon variants.
External full struct literals must supply `TooltipRenderModel.direction`,
`StepperRenderModel.direction_explicit`, `AutocompleteItemsRenderModel.direction`,
and `SdkStrings.autocomplete_status`.
`PopupMenuRenderModel.end_icon` now holds `Option<ControlIcon>`; convert Lucide
values with `.into()`. Existing builder calls with Lucide values still work.
For forced physical stepper previews set `direction_explicit: true`; for automatic
horizontal locale flow set it to `false`. The existing Lucide-only
`PopupMenu::set_end_icon` setter is retained; `set_end_control_icon` accepts the
full icon contract.

## Development checks

With `test-support`, `l10n_conformance!(checkbox_template, selector_template)` runs
one shared headless fixture against the supplied look templates. Both Radix and
Shadcn run it. The fixture uses `en-XA` expanded copy and `ar-XB` RTL copy to check
slot order, trigger bounds, implicit localized copy, explicit copy precedence,
and focus/selection preservation. It does not require a localization engine.
Family-specific tests additionally cover tree expansion, table resizing, overlay
placement, stepper overrides, autocomplete defaults, and existing migrated controls.

Lab string extraction and real pseudo-locale generation are documented in
`apps/xgpui-lab/localization/DEVELOPMENT.md`. The independent
[audit #258](l10n/audit-258.md) records verified behavior, bounded fixes, and
remaining completion gaps, including untranslated listbox accessibility defaults.
Passing the development suites is not a full localization certification.
TextField/TextArea editing, dock/splitters/panels/title bar remain outside #226.
