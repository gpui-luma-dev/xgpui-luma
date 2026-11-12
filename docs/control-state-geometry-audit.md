# Control state geometry audit (#110)

This audit distinguishes stable layout from state-dependent paint. Focus,
hover, press, disabling and validation should not move a control's face,
content or following siblings. Opening a popup, expanding an accordion/tree,
changing content or moving a splitter can intentionally change layout; those
changes must be distinguished from decoration-induced movement.

## Confirmed defect and fix

The default SearchSelector trigger reserved its border only when the resolved
border color had nonzero alpha. With the SDK ghost theme, setting invalid
added a 1 px border and moved the label from x = 14 px to x = 15 px. Disabling
the invalid trigger removed that border and moved the label back. The trigger's
fixed height did not reveal this internal displacement.

`crates/sdk/src/controls/selection/search_selector/template.rs` now always
reserves `geometry.border_width`, including for transparent paint, matching the
plain Selector's approach. The fix preserves the existing appearance policy;
it does not introduce new focus decoration or alter popup behavior.

## Verification inventory

“Covered” identifies regression coverage, not a claim that every public variant,
custom template or runtime interaction has been exhaustively tested.

| Control family | State/geometry evidence | Status |
| --- | --- | --- |
| TextField, TextArea | Existing SDK, Radix and Shadcn template tests check outer bounds and following sibling placement across focus, hover, press, disabled and invalid combinations; placeholder and full-width cases included | Existing outer-layout coverage; internal text/caret checks remain |
| Button, icon/toggle button roles | Existing Radix/Shadcn tests cover sizes, compact forms and style variants with the shared state matrix | Existing outer-layout coverage |
| Checkbox, RadioButton, Switch | Existing look tests and SDK custom-border regressions exercise focus and selection geometry | Existing outer-layout coverage |
| SearchSelector | New SDK/Radix/Shadcn checks cover Outline/Ghost, Sm/Md/Lg, all 64 combinations of focus/hover/press/disabled/invalid/selection, and scale factors 1/1.5/2; label and icon bounds are checked as well as outer/sibling bounds | Conditional-border defect fixed; template geometry covered |
| Selector, PopupMenu (including split trigger) | Existing shared tests exercise ghost/outline triggers and split faces | Existing outer-layout coverage; internal bounds remain |
| ControlGroup, Sidebar | Existing group marker and sidebar row regressions cover decoration appearing/disappearing; Radix composite tests cover custom/transparent focus paint | Partial coverage; whole composite layouts remain |
| ComboBox, Autocomplete, SelectionPanel, SelectorList | Source review shows shared text-field/selection infrastructure and paint-only disabled/active row treatments | Dedicated per-control content/state matrix remains |
| ListBox | Radix row border is always reserved; Shadcn surface keeps a fixed border width when focus changes | Source-reviewed focus path; dedicated state matrix remains |
| Table, TreeView, Accordion, Tabs, Pager, Stepper | Several paths delegate to shared button/group machinery; expansion, active content and animated row heights can intentionally change layout | Per-control matrix and explicit layout-change classifications remain |
| Slider, Progress | Slider track/thumb layers use positioned geometry; disabled-track paths primarily change paint/opacity | Per-control state matrix remains |
| DockSplitter, SplitView, ResizablePanels | SplitView's hovered cue changes inside a fixed hitbox; ResizablePanels handle lane changes are absolute overlays | Source-reviewed decoration paths; panel/content bound checks remain |
| ScrollContainer, Scrollbar, ContextMenu, FloatingMenu, OverlayWindow, PopoverButton, SlidePanel, Tooltip, RadioGroup, IconGroup, Toolbar | Not exhaustively exercised by this patch | Audit remains open |

## Shared harness

`assert_stable_layout` retains its existing behavior. The new
`assert_stable_layout_with_content` also compares explicitly named internal
content selectors. Each selector must remain mounted throughout the state
matrix. SearchSelector is the first consumer, checking both label and icon.

The matrix is a template-level geometry test with a closed popup. It does not
replace actual control dispatch tests, native accessibility/device checks, or
validation of custom themes that deliberately change dimensions. No GUI launch
is required by these headless regressions.

The broader checklist in #110 remains open. Next, apply internal-position
checks to the editable controls and add independent coverage for composite
selectors and collection/navigation controls.
