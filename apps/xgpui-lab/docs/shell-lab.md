# Embedded shell lab

The explorer's **xgpui-shell** section ports the diagnostics from
`xgpui/apps/xgpui-ext-lab/shell-lab/src/main.rs` into the Lab's shared Radix
presentation. It uses the published `xgpui-shell` dependency; no sibling checkout
patch or upstream source modification is required.

- **Shell Lab:** the actual `ClientFramePreview` renderer with simulated
  floating/maximized/tiled/fullscreen state, appearance, activation, Close focus
  and hover, and platform/fallback, close-only or left-aligned buttons. The style
  inspector displays the resolved colors, typography, geometry and actions.
  The same page includes contrast, focus accent (including black), logical scale,
  typography, reduced motion and titlebar action overrides, plus reset. Overrides
  are retained across explorer navigation and affect only the preview.
  **New window** and **New fixed-size window** open independent full Lab windows,
  directly on Shell Lab. Each starts with its own theme and preview defaults.
  Fixed-size windows disable native resizing; the page reports the actual mode.
  Close test windows with their native controls; no restart or CLI flags are needed.
- **Desktop Preferences:** current platform appearance, preferences, capabilities,
  decoration mode, window state and device scale. Missing desktop readings are
  shown as “not reported”; preference detection is currently Linux-only.

All interactive settings use SDK selectors and Radix builders. The sample content
keeps the Lab's own theme, independently of simulated frame appearance. Desktop,
window-state, appearance, display and button-layout observers refresh diagnostics;
the event log retains the twelve most recent distinct messages.

This is the first port. Additional real window operation controls, decoration
requests, and applying overrides to a selected real window remain to be ported. Simulated frame controls never move, resize, minimize or
close the host Lab. The whole-titlebar theming gap remains tracked by
[xgpui#192](https://github.com/gpui-luma-dev/xgpui/issues/192).

Headless tests exercise SDK selector callbacks, override resolution and reset,
page retention, explorer routing and preservation of host window/desktop state.
No GUI launch or manual platform acceptance is implied by these tests.
