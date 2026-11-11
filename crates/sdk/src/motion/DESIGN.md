# Control motion contract (#145)

Controls define typed roles beside their public builder. The SDK’s
`theme/stylesheet/motion.rs` authors the native motion baseline; control profile defaults
and both looks resolve from those native definitions.
Switch owns `state`;
Tabs owns indicator, content, disclosure and drop-layout roles. A look supplies values for
those roles alongside its templates. Interaction targets, accessibility, events,
presence, and frame scheduling remain in SDK controls.

```rust,ignore
// ShadcnLook and Radix Look accept the same SDK-defined defaults.
look.motion(|m| {
    m.switch.state = Transition::timed(Duration::from_millis(160), Easing::OutCubic);
    m.tabs.indicator = Transition::spring(300.0, 28.0);
});

// The look builder forwards this edit to the SDK builder.
shadcn::Tabs::new("settings")
    .look(&look)
    .motion(|m| m.content = Transition::instant())
    .spawn(cx);
```

## Resolution and lifetime

1. Build the look's profile from existing stylesheet defaults.
2. Apply programmatic look edits in registration order.
3. Snapshot that profile into the SDK builder when the look builder spawns.
4. Apply instance `.motion` edits in registration order. Assigning a role replaces its complete
   transition; untouched roles inherit. `.look`/`.motion` order is independent.
5. Control-wide `motion_enabled(false)` makes motion immediate without erasing the
   authored profile. Re-enabling uses that profile on subsequent interactions.

An SDK builder's `motion_defaults` supplies the look's base profile; consumer
edits remain deferred until resolution, independent of setter call order.
Look clones share edits. Radix forks and Shadcn independent copies copy edits;
subsequent changes are isolated. Existing controls retain their snapshots, and
running content entrances retain their trajectories. No live theme animation
reconfiguration is implied. Applications can route a reduced-motion preference
through `motion_enabled(false)`; this change adds no platform preference detection.

`motion_profile()` exposes the captured authored roles. Each transition’s
`description()` reports timed parameters, spring parameters, instant, or custom.

## Runtime extension

`Transition` shares a `MotionDefinition`. It creates a `RunningMotion` trajectory
from a scalar value, velocity, and target. Sampling uses absolute elapsed time,
is deterministic, and returns a value, velocity, and completion decision. The
existing `VisualTransition` owns the clock and delegates sampling; controls never
match on timed versus spring versus custom implementations.

Retargeting starts from the last displayed value and velocity. Springs preserve
both; timed easing preserves position but intentionally starts its own easing
curve. Tab indicator left and width are persistent scalar channels in pixel
units, rather than restarted normalized progress. Completion snaps exactly to
the target and releases the running trajectory, ending frame requests. Invalid
samples also settle safely. Custom implementations must obey the same contract.

Spring samples can overshoot. Switch thumb position accepts overshoot; colors and
tab content opacity clamp at paint time. Springs use unit mass and positive,
finite stiffness/damping. Invalid parameters settle immediately. The analytical
implementation supports under-, critically-, and over-damped motion.

`VisualTransition::with_transition` accepts a typed transition and arbitrary scalar targets. Runtime helpers are now
`Clone`, rather than `Copy`, because they hold shared configuration/trajectories.

## Migrated control roles

All existing target-based motion paths now use the shared transition contract.
Controls declare roles beside their models; `MotionConfig` and the SDK's
builder/forwarding macros implement inheritance once. Look wrappers declare no
animation engines or per-parameter setters.

| Control | Roles | Preserved defaults |
| --- | --- | --- |
| Checkbox, Radio, Switch, Toggle | `state` | 200 ms cubic out |
| ControlGroup; IconGroup, checkbox/radio/menu groups | `selection` | 200 ms cubic out; selection animation remains opt-in |
| Slider | `thumb` | 200 ms cubic out; direct dragging still snaps |
| Progress | `value`, `period` | 200 ms cubic out; indeterminate phase 1200 ms |
| Stepper | `selection` | 200 ms cubic out |
| Tabs | `indicator`, `content`, `disclosure`, `drop_layout` | Indicator/disclosure 200 ms; content instant in SDK, 300 ms in both looks |
| Accordion | `expansion`, `drop_layout` | Shared expansion/disclosure progress, 200 ms |
| TreeView | `expansion`, `position`, `drop_layout` | 200 ms; virtualization still settles very large expansions |
| Sidebar and its internal engine | `expansion`, `popup`, `drop_layout` | Expansion and rail presence 200 ms |
| SplitView, ResizablePanels | `extent` | 200 ms |
| PopupMenu | `presence`, `highlight`, `disclosure`, `submenu` | Presence/disclosure/submenu 200 ms; highlight 180 ms |
| ContextMenu | `presence`, `submenu`, `disclosure` | Main/submenu enter and disclosure 200 ms; both exits immediate |
| PopoverButton/PopoverPanel; Radix HoverCard | `presence` | Enter/exit 200 ms |
| Selector, SearchSelector, Autocomplete, ComboBox | `presence`; SearchSelector also `disclosure` | Enter/disclosure 200 ms; immediate exit |
| Table | `drop_layout` | 200 ms cubic out |
| ListBox | `scroll`, `drop_layout` | Scroll and layout 200 ms cubic out |
| ScrollContainer | `scroll` | 200 ms |
| SlidePanelState | `presence` | Enter/exit 220 ms |
| TabsOverlayState (app-owned tab dropdown/overflow host) | `enter`, `exit` | 200 ms |

Collection drop roles use the same typed profiles as all other motion. Themes
and templates supply appearance; motion defaults come from the look profile.
`Transition::default()` is an authored transition, not an inheritance marker.

Both looks expose the SDK profiles through `look.motion(...)`. Their supported
builders forward consumer edits and defaults, including semantic group wrappers.
The SDK-only composition paths accept `motion_defaults(...)` explicitly. Look
helpers `slide_panel_state` and `tabs_overlay_state` seed app-owned state; Shadcn
also supplies `scroll_container`. Configure those states before first use.

Persistent ListBox controls capture their look profile on first composition.
Subsequent renders reuse that snapshot and do not re-run configuration closures.
Raw, host-owned ListBox composition can also configure its look builder's motion;
retain custom transition values when reconstructing that composition.

`motion_enabled(false)` applies to every role, including looping progress and
TreeView smooth positioning. To disable expansion while retaining scrolling,
set only `m.expansion = Transition::instant()`. ControlGroup retains its previous
opt-in selection default; use `motion_enabled(true)` to enable it.

## Completion and geometry

Closing presence remains painted until its exit implementation completes, even
when spring progress crosses zero early. Disabling motion settles presence and
retains the authored profile for subsequent interactions. SlidePanel focus
transfer/restoration waits for completion rather than an overshoot threshold.

Stepper animates actual step position. Drop layout retains two position channels
per mounted key, preserving sampled velocity across interrupted drops. Tracking
remains bounded to 512 visible keys, and resize/scroll/reduced-motion changes
still settle the layout as before.

ScrollContainer and ListBox share one pixel-based adapter. It retains platform
clamping, anchor correction, destination remeasurement and cancellation behavior.
`MotionDefinition::retarget` handles a changing measured destination within an
interaction: timed definitions keep the original deadline, while springs keep
velocity. Custom definitions can choose their corresponding retarget policy.
New user interactions still use `start`; controls do not inspect engine types.
TreeView keeps its existing measured/estimated row-space positioning algorithm,
with its interaction timeline sampled by the selected definition.

Indeterminate progress retains `ContinuousPhase`: a looping phase is not a
settling transition. Its period is look-owned, and the control-wide flag stops
it. Hover delays, tooltip hide delays, and scrollbar auto-hide timers remain
interaction policy rather than target motion.

## Stylesheet motion and SDK-only applications

SDK controls work without a look, with vanilla templates and per-control motion
fallbacks. `MotionDefaults::default()` exposes that baseline publicly. Applications
and look adapters can resolve `CommonStylesheet::motion_defaults()` and supply
component profiles with the public `motion_defaults(...)` builder method.
`MotionDefaults::from_stylesheet` is the same resolver used by both bundled looks;
`sheet.motion.apply_to(&mut defaults)` overlays authored roles onto a custom base.

```toml
[common.motion.switch.state]
kind = "timed"
duration_ms = 160
easing = "out_cubic"

[common.motion.tabs.indicator]
kind = "spring"
stiffness = 300.0
damping = 28.0

[common.motion.context_menu.presence.exit]
kind = "instant"

[common.motion.progress]
period_ms = 1200
```

The schema mirrors every typed control profile. Missing roles inherit; an
explicit role replaces the complete transition. Resolve SDK defaults, typed stylesheet roles, programmatic look edits,
then instance motion edits. Disabling motion is final policy.
The existing control snapshot lifetime is unchanged.

Timed definitions require integer `u32` milliseconds and `linear`, `out_cubic`, or
`in_out_cubic` easing. `InOutCubic` accelerates over the first half and decelerates
over the second, with zero velocity at both endpoints. Spring parameters must be
positive and finite. Instant definitions have no parameters. Unknown fields, roles, kinds, and invalid parameters fail parsing.
Custom implementations remain a Rust extension through `Transition::custom`.
`Transition::description()` reports the full built-in configuration or `Custom`.

### Embedded vanilla baseline

`CommonStylesheet::sdk_default()` returns the cached native Rust baseline, and
`MotionDefaults::vanilla()` returns the compiled profiles. `CommonStylesheet::default()`
remains an empty overlay whose resolution inherits the SDK baseline.
Control profile `Default` implementations clone their corresponding cached profile;
the baseline compiler constructs profiles directly, avoiding recursive defaults.
The standalone `Transition::default()` uses `common.motion.transition`.

Look constructors supply overrides, not copies of the SDK defaults. Their typed roles
apply over the SDK baseline, followed by programmatic look edits and consumer edits. Tests require the SDK asset to parse and author
every role. If a packaged baseline is corrupt or incomplete, missing transitions
settle immediately rather than panicking or falling back to duplicated Rust values.
