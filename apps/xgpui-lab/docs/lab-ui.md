# Shared lab presentation

Color Spaces is the reference for lab presentation, originally reviewed at
`ed58794d72b3aae73e8fdddb9df5cb84e4656c59` (issue #29). Issue #33 applies
these patterns to Italy Photo Comparison. The reusable implementation is
`src/lab_ui.rs`; both panes in `src/color_demos/` consume it.
The full rule set for new and restyled screens (page order, Radix roles,
controls, review checklist) is [XGPUI-LAB-DESIGN.md](XGPUI-LAB-DESIGN.md).

Use the shared lab page patterns and match Color Spaces:

- Flat page background and foreground from `Look::chrome()`. Resolve surfaces,
  borders, muted text, and interaction states from the current Radix Look.
  The app theme generates its accent and gray scales from #5A6169 for both
  modes; never copy screenshot RGB values or add a separate palette.
- Full-width centered container, maximum 1240 logical pixels, p6 outer padding,
  gap5 between breadcrumb, header, and content. Content sections use gap4.
- Muted small breadcrumb, semibold text3xl title, gap2 to one muted subtitle.
- Main visualization directly on the page, without an enclosing body card.
- Wrapping, vertically centered control rows use gap3 and the reference
  Radix switch label typography. Keep actual controls as Luma/Radix entities.
- Supporting panels use p4/gap3, rounded-lg corners, a subtle surface and thin
  border. Comparison cards use p3/gap3 and the same surface/radius hierarchy.
- Captions, status, explanations, and provenance use shared small muted text.

## Supported variations

A visualization owns its geometry. Color Spaces retains its responsive color
field; Italy uses a maximum 600 × 400 logical-pixel photograph with a 3:2
aspect ratio, shrinking with available width rather than stretching the fixture.
Do not apply square field geometry to photography.

Comparison cards may contain numerical values or selectable photographs.
Italy uses three centered, wrapping 200-pixel cards with fixed 174 × 116
(3:2) thumbnails. The main photograph is centered on the same content grid.
Photo selection is through these cards; only presentation uses a dropdown. Selection,
hover, press, and keyboard focus resolve from Radix interaction roles and gray
steps. Photo cards remain Luma buttons so input and accessibility behavior
stay with the control. Keep all cards and the large selected image synchronized.

Shared helpers own chrome only. Models, color conversion, media loading,
rendering paths, presentation negotiation, and error state stay in each lab.
Do not migrate unrelated labs simply because these helpers exist.

## Verification

Compare Color Spaces before/after extraction and Italy alongside it at the
same window size in light and dark themes. Check a narrow window for wrapping,
vertical scrolling, visible controls, and undistorted photos. Run the existing
Italy selection/presentation and fixture tests and the Color Spaces interaction
tests; native wide-gamut fidelity still requires a capable display.

## Desktop window shell

The main Lab window uses published `xgpui-shell 0.2.0` with XGPUI 3.1.0.
`app/titlebar.rs` declares the shell window, toolbar layout, Radix controls and
callbacks together in `open_shell_window`. `app_shell.rs` forwards to that
composition. `LabTitlebar` renders the inline template alongside the
`XgpuiLabApp` content view. Its button cache retains SDK entities and binds
callbacks once, while layout and theme content update on redraw. The shell owns platform decorations, native traffic lights,
window controls and desktop preference handling. The Lab owns its Radix toolbar
controls, theme, navigation and workspace content. Title text and empty toolbar
space use `ShellDragRegion`; interactive controls stay outside drag regions.

Toolbar alignment measures the actual toolbar/body origins, including shell
insets, rather than assuming a macOS traffic-light offset or fixed header height.
The light shell background is `#ECECED` while active and `#F0F0F0` while inactive.
The Lab also sets its native macOS window appearance to Aqua/DarkAqua when the
Radix mode changes, so native traffic lights use the corresponding AppKit artwork.
This is per window and does not change the system appearance.
Window activation observers refresh the outer shell and toolbar together; content
surfaces keep their own Radix palette. The Lab updates `ShellWindow::set_options`
with the 0.2.0 `titlebar_background` and `colors` overrides on theme/activation
changes. The shell paints the complete header, including native-control gutters
and trailing padding; its macOS border uses the same background for a seamless
bottom edge. Other platforms use the Radix border, text and hover colors.
These explicit colors also override the default high-contrast palette; the Lab
owns their contrast. Native AppKit appearance synchronization and the optical
alignment adjustment remain separate from this color API. This replaces the local
deferred edge painting associated with [upstream issue #192](https://github.com/gpui-luma-dev/xgpui/issues/192).
The shell API is alpha. Interactive macOS, Windows and Linux desktop
verification remains necessary, including decoration negotiation, fullscreen,
window operations and scaled text; headless tests cannot establish those results.
