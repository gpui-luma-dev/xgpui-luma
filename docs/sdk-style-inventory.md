# SDK styling inventory (#146)

The SDK Rust constructors own configurable vanilla appearance. All SDK control families and
shared styling helpers are accounted for below. The typed public schema and
factories supply the same baseline to SDK-only apps and custom looks.

## Family coverage

| Family/helper | Authoritative common owner | Retained behavior / integration |
| --- | --- | --- |
| Button / IconButton / Toggle | `button, toggle; semantic recipes; focus/disabled tokens` | Icon-only zero padding and circular radius derive from the role. |
| SplitButton | `popup_menu plus independent half interaction state` | Shares PopupMenu trigger chrome; halves retain separate feedback. |
| Checkbox / Radio / Switch | `checkbox, radio, switch; semantic recipes; icons` | Track/indicator proportions, label spacing, strokes and opacity are data; selection interpolation remains math. |
| ControlGroup / RadioGroup / IconGroup | `control_group, radio_group, icon_group; shared row helpers` | Hosts resolved item chrome; row selection/focus dispatch remains behavior. |
| Toolbar | `toolbar; shared palette/metric tokens` | Separator and frame thickness are resolved geometry. |
| ListBox | `host look/theme; common tokens` | Owns measured/virtualized flow, not an independent item skin; 48px unmeasured estimate and overscan are layout heuristics. |
| Table | `table; semantic recipes; common tokens` | Paint and height calculations share divider widths; drag/column styling uses the same resolved fields. |
| TreeView | `tree_view; icons; semantic recipes; drag_preview elevation` | Drop indentation/strokes/opacity and preview chrome use the selected scale. |
| Frame / rounded panel helpers | `explicit frame styles; shared metric tokens` | Zero defaults mean no authored fill/spacing; corner calculations derive from inputs. |
| DockSplitter / SplitView | `dock_splitter, split_view; shared initial pane dimensions` | Pointer capture and click suppression stay interaction behavior. |
| ResizablePanels | `resizable_panels; metric tokens` | Preset conversion derives from authored lane widths; split solving/bootstrap threshold stays layout. |
| ScrollContainer / Scrollbar / PopupScrollSurface | `scrollbar; shared initial helper dimensions` | Range/visible fraction, wheel/page steps, scrolling and virtualization stay behavior/data. |
| Accordion | `accordion; icons; semantic recipes; shared drag geometry` | Expansion/transfer/drop hit regions remain interaction and layout. |
| Pager | `pager; icons; shared page-size trigger width` | Pagination indices/page sizes and minimum usable slot count remain data/behavior. |
| Sidebar / panel engine | `sidebar; icons; semantic recipes; shared drag geometry` | Host disclosure defaults survive replacement; explicit panel choices remain final. |
| Stepper | `stepper; icons; semantic recipes` | Badge/track/label/glyph geometry is data; completed/current dispatch remains behavior. |
| Tabs / indicator / content | `tabs geometry/variants; icons; semantic recipes` | Indicator baseline comes from data; keyed motion/content lifetime stays behavior. |
| FloatingMenu / ContextMenu | `floating_menu, context_menu; semantic recipes` | Placement, separator, target and item chrome are data; navigation/dismissal remain behavior. |
| PopupMenu / PopoverButton | `popup_menu, popover; icons; semantic recipes` | Anchored/centered popup margins and border-dependent alignment are resolved. |
| OverlayWindow / SlidePanel | `overlay_window, slide_panel, resizable_panels; elevation` | Insets, header drag region and dividers are data; resize/clamp/presence stay behavior. |
| Tooltip | `tooltip; common tokens` | Pointer/placement/shortcut presentation is data; timing remains common.motion. |
| Progress / Slider | `progress, slider; semantic recipes` | Sweep/shimmer/radial geometry is data; phase, legal paths and pointer-value mapping remain algorithms. |
| Autocomplete / ComboBox / SearchSelector | `autocomplete, combobox, search_selector; icons; row_limits` | Affordance geometry, search reservation and viewport budgets are captured; explicit limits win. |
| Selector / SelectorList / SelectionPanel | `selector, selector_list, selection_panel; icons; row_limits` | Panel chrome derives from the shared menu with final family overrides; selection/filtering stays behavior. |
| TextField / TextArea / text runs | `textfield, textarea; shared compact/resize chrome; typography/metrics` | Caret paint and template agree; editing, IME ranges and cursor hit/reveal padding remain behavior. |
| Shared layouts / form labels | `shared; typography and metric tokens` | WideMiddle budgets and field gaps are data; responsive thresholds derive from them. |
| Drag helpers / geometry primitives | `shared drag geometry; tokens; drag_preview elevation` | Path topology, item-slot distribution and before/into/after hit regions remain algorithms. |
| TitleBar / window actions | `shell; palette.*.shell; icons.window_*` | Public constants are authored in Rust; native traffic-light positions and custom colors retain their APIs. |
| Tokens / provenance / color / motion | `tokens and motion in the native stylesheet` | Conversion tolerances, unit domains, gamut math, validation and absent-color sentinels remain Rust. |

## State/variant and icon coverage

`common.recipes.<family>` authors the semantic palette bindings consumed by SDK
state recipes. Disabled > pressed > hovered > default is interaction precedence;
focus/invalid remain independent. Binding sources read the original projected
palette, preserving programmatic token overrides without cascading remaps.

Shadcn authors color/elevation rules in its existing Rust configuration modules. Radix button state rules,
tone-step choices, Page/GhostQuiet exceptions and Classic decoration tuning are
authored in its existing Rust button configuration. Tests compare every previous button variant,
mode, contrast, selection and pointer/focus state. The disabled-active-Tab focus
paint defect found by the audit is corrected.

Sidebar header/subtitle typography and section OpenType features use selected theme snapshots; explicit empty feature sets clear small caps. Table emphasis and Stepper number weights reuse shared title/label weight tokens. SVG wrappers no longer reset an unused font weight.

All control-owned default glyphs, including shell window actions, resolve through
`common.icons`. Explicit icon objects and disclosure pairs retain precedence.
Tabs support external SVG accessories. Sidebar defaults survive runtime content
replacement; panel-local choices win. User-provided item icons remain content.

## Literal reconciliation

[sdk-style-candidates.csv](sdk-style-candidates.csv) contains 1,800 classified
entries, each with an owner/classification and rationale. It preserves historical
expressions and includes the newly authored shared/shell/recipe fields.
Line numbers for historical rows are snapshot locations; line zero is a
family-level summary. A `superseded` row records an expression absent from the
current source and does not assert that a live literal is migrated.

After the `.lss` migration (#285, #287, #355) the CSV is a historical record:
rows that were `migrated` to a `common.*` family other than `recipes`, `icons`,
`row_limits`, `font_features` or `motion` are `superseded`, since those values now
come from `.lss`; their rationale still names the old field. Rows whose source file
was deleted in #285 point at the `.lss` file that replaced it and keep the old
file's line number.

| Classification | Rationale |
| --- | --- |
| Migrated | Typed common field/token is authoritative. |
| Mathematical / derived | Centering, radii from dimensions, interpolation endpoints, path topology, zero/empty identity. |
| Interaction / behavior / data | Pointer and keyboard policy, accessible hit/reveal targets, scroll steps, value ranges and pagination data. |
| Layout | Measured geometry, prefix sums, virtualization estimates/overscan, frame tolerances and bootstrap decisions. |
| Validation | Domain/range guards and color/configuration ingress. |
| Absence | Transparent fill/border or empty geometry sentinel. |
| Test fixture | Literal independent input/expected output in headless regression tests. |
| Superseded | Archived expression removed or replaced during migration. |

Specific retained limits include 4px caret hitboxes and 24px horizontal reveal
padding (editing interaction), 48px unmeasured ListBox row estimates and 480px
overscan (virtualization), the 64px unmeasured weighted-panel bootstrap switch
(layout), 48px column resize floor (usable resize target), drag autoscroll speed/
activation zones, before/into/after drop partitions, circular-path epsilon guards,
and 0/1 animation/value endpoints. None chooses a second painted default.

Run `python3 scripts/check_sdk_style_inventory.py` to reject unknown statuses,
missing rationales and broken source references.

## Verification contract

- Embedded baseline parses through the strict runtime schema and compiled token constructors.
- SDK-only factories and both looks exercise inheritance, explicit zero/false,
  selected family recipes, row budgets and consumer precedence.
- Color-space source preservation and motion snapshot/reduced-motion contracts remain intact.
- Workspace formatting, strict Clippy, relevant headless suites, SDK doc tests and
  website example compilation cover the implementation. No GUI launch is required.

See [the stylesheet contract](shared-stylesheet.md) for schema, validation,
lifetimes, API ownership and SDK-only/custom-look examples.
