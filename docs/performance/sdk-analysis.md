# SDK performance map and optimization priorities

The first SDK-wide baseline identifies shared stylesheet cloning, large navigation scenes, repeated element/layout work, vector color geometry, and chart tessellation as the main investigation areas. **Radix stylesheet cloning is the clearest immediate optimization candidate:** it accounts for 32.88% of sampled main-thread CPU in the driven form workload.

This is a representative baseline for [issue 167](https://github.com/gpui-luma-dev/gpui-luma/issues/167), not a claim that every control or interaction has been profiled. The [coverage matrix](sdk-coverage.csv) lists measured representatives and remaining scenarios across the SDK, both looks, color, plotting, and shared infrastructure. The original baseline predates production optimization. A subsequent [Radix stylesheet optimization and comparison](radix-stylesheet-optimization.md) is now included on this branch; the baseline data remains unchanged.

Priority update after the Radix optimization: the [Table and TreeView interaction analysis](collection-analysis.md) takes precedence over the original large-navigation stress ranking. The [fixed-height Table follow-up](table-fixed-height-optimization.md) implements the first collection optimization. A 10,000-item Sidebar is a documented scaling limit, not a representative product requirement; navigation optimization should be driven by measured usage near its expected size (roughly 100 items or fewer).

## Evidence and reproduction

- [Protocol, commands, workload definitions, and limitations](sdk-protocol.md).
- [Headless summary](sdk-baseline-macos-arm64.md) and [complete JSONL](sdk-baseline-macos-arm64.jsonl): 22 configurations, 126 scene/phase measurements, three infrastructure microbenchmarks, and lifecycle checks.
- [Native capture manifest](sdk-native-captures.json): seven usable driven CPU traces, one idle CPU trace, Animation Hitches and Metal System Trace captures, and two isolated process-memory runs. The native Allocations capture failed to start within 120 seconds.
- [Native CPU attribution](sdk-native-cpu.json) and [GPU/animation observations](sdk-native-gpu.json). Full traces and exports remain locally under `target/perf-traces/issue-167`; they are intentionally excluded from Git because of their size.

Measurements were collected on October 21, 2026, on an arm64 MacBook Pro with 128 GiB memory, macOS 27.0.1, rustc 1.99.0, GPUI 1.24.2, and Instruments 27.0. Headless metadata records the baseline revision and dirty worktree state. Release settings retain the repository's thin LTO and single codegen unit. The native app requests a 1200 × 800 logical window and registers the bundled example fonts and Lucide assets. The headless platform verifies its actual viewport is 1200 × 800 and uses mock fonts/graphics.

## Shared costs and scaling

The form workload repeats Button, Checkbox, Switch, TextField, TextArea, Slider, and Progress. A size of 100 means 700 primary controls. All tracked controls are invalidated in the warm phase; the driven native phase additionally edits all input values on each frame. This is a deliberate fanout stress workload, not ordinary single-field typing.

| 100 form repetitions | Headless warm redraw ms | Allocations per redraw | Native estimated main-thread CPU |
| --- | ---: | ---: | ---: |
| SDK defaults | 11.291 | 82,832 | 82.70% |
| Shadcn defaults | 14.287 | 152,436 | 99.82% |
| Radix defaults | 23.790 | 483,275 | 99.96% |

Practical look defaults differ in appearance and composition, so these are not isolated measurements of adapter overhead. However, native attribution identifies a specific shared cost in Radix: `CommonStylesheet::clone` has 15.18% self weight and 32.88% inclusive weight; `Look::common_stylesheet` has 31.74% inclusive weight. These overlap and must not be added. The source getter clones the full stylesheet, and controls such as TextField and TextArea call it multiple times during look resolution.

Across all three native form traces, memory copying is the largest individual leaf symbol (15–18% of main-thread weight), with allocation/free routines also prominent. Investigate ownership, repeated look resolution, and GPUI element construction before assuming a GPU limitation. These samples include SDK, GPUI, layout engine, and system work; allocation traffic cannot be assigned entirely to Luma without stack attribution.

| Other workload | Headless warm redraw | Native estimated main-thread CPU under continuous redraw/update |
| --- | --- | ---: |
| Navigation | 2.301 ms at 100 items; 124.134 ms at 10,000 | 70.68% at 1,000 items |
| Table and TreeView together | 0.377 ms at 100 records; 0.451 ms at 10,000 | 19.18% at 10,000 records |
| Frames and SplitView | 0.694 ms at 100 frames; 6.479 ms at 1,000 | Unavailable after fixture correction |
| Vector ColorField and HueRing pairs | 2.716 ms at 10 pairs; 23.797 ms at 100 | 29.61% at 10 pairs |
| Line chart without decimation | 0.926 ms at 10,000 points; 2.512 ms at 100,000 | 29.46% at 100,000 points |

Collection and SplitView controls use bounded panels to keep them visible. Earlier unbounded collection/layout captures are excluded; the corrected layout capture failed when macOS reported no available displays and needs a retry.

Navigation is a composed scene, not a Sidebar-only benchmark. Its large-row scaling is a stress observation and does not establish priority at realistic navigation sizes. Table/TreeView's nearly flat warm redraw is a positive result consistent with viewport virtualization, but does not describe interaction costs. The [collection follow-up](collection-analysis.md) separately measures startup, wheel/keyboard input, keyed selection, data replacement, projection and column resizing.

In the color workload, GPUI `paint_path` is 18.57% of main-thread leaf weight. HueRing draws segmented geometry each paint, while the field uses its vector renderer. Compare cached geometry and raster alternatives with quality and memory checks rather than replacing vector rendering indiscriminately.

In the 100,000-point plot workload, stroke tessellation is 23.86% of main-thread leaf weight and `line_runs` is 15.65%. The fixture explicitly uses the default `Decimation::None`. Compare the existing MinMax mode, visible-range handling, and geometry reuse before adding new chart APIs. This result does not cover bars, area charts, labels, annotations, or other chart families.

## Lifecycle, idle, and memory

All **24,666 tracked primary control entities** released after scene removal, executor draining, and two additional frames. Across the ten additional lifecycle cycles, the largest RSS increase after that release checkpoint was about 2.2 MiB in the SDK 7,000-control stress fixture. This does not prove that all internal entities or subscriptions are leak-free: probes track added primary controls, and RSS includes allocator and GPUI cache retention. The complete sequential headless run reached roughly 2.0 GB RSS; use an isolated `--scene/--look/--size` process to attribute memory to a case.

The historical sequential headless RSS above also includes test contexts retained by the original harness before explicit shutdown was added. Treat it as a historical process observation, not retained SDK heap. The collection follow-up checks release of both the primary control and the headless App.

Isolated native process peaks, measured without Instruments allocation hooks:

| Native driven workload | Maximum resident bytes | Approximate MiB |
| --- | ---: | ---: |
| Shadcn forms, 100 repetitions / 700 primary controls | 173,670,400 | 165.6 |
| Vector color, 100 pairs / 200 primary controls | 137,527,296 | 131.2 |

The separate Metal trace for Shadcn forms reported a peak `MTLDevice.currentAllocatedSize` of 72,712,192 bytes (69.3 MiB). That is allocated Metal resource capacity, not GPU residency; it must not be added to process RSS as disjoint memory.

The Shadcn form idle trace estimates 1.31% main-thread CPU in the steady window, versus 99.82% under the deliberately continuous update driver. Background-thread idle work and energy still need a dedicated assessment. Allocations instrumentation stalled before launching its target and was terminated at the 120-second limit, so this baseline has no native heap ownership/leak report.

## Native timing boundaries

CPU estimates sum running main-thread sample weights in trace seconds 2–12 and divide by that ten-second window. They describe one core's sampled CPU use, not wall-clock rendering duration or GPU utilization. Native runs are single captures, not a statistical comparison across repeated trials. The CPU JSON retains raw sample counts, weights, leaf symbols, and inclusive symbols matching `xgpui_luma`, including GPUI generics instantiated with SDK types.

The animation trace identified one 150 ms potentially expensive update during startup and no app-attributed hitches in the selected steady window. This is a short, instrumented observation, not a smoothness guarantee. Display frame-lifetime/render tables lack process identity; application presentation percentiles have not been derived. Driver callback counts are not presented FPS. Event-to-paint latency and display-specific frame-budget misses remain unmeasured.

## Ranked follow-up work

| Priority | Investigation | Evidence and proposed boundary |
| --- | --- | --- |
| 1 | Avoid full stylesheet clones in Radix render-time resolution | 32.88% inclusive main-thread weight in the driven form trace. Preserve the public owned getter; evaluate internal borrowed access or revision-aware shared snapshots, with mutation/fork semantics and theme invalidation regressions. |
| 2 | Reduce Table full-dataset measurement on startup, replacement and resizing | Collection fixtures expose explicit fill-height measurement visiting every row. Evaluate fixed-height hints and correct invalidation while preserving scrollbar range, variable-height content and header/body alignment. |
| 3 | Characterize TreeView visible-state and data/projection work | Nested expanded/collapsed fixtures separate viewport presenter counts from loaded/displayed-node traversal. Investigate refresh/replacement and keyed reveal before changing APIs; preserve expansion, selection, sorting, filtering, scroll anchors and animation. |
| 4 | Compare color geometry reuse and raster caching | Path painting dominates a substantial part of the color trace. Compare stable versus changing hue, renderer modes, sizes, memory, and visual fidelity. |
| 5 | Characterize chart reduction and tessellation choices | Large undecimated lines spend time in projection and stroke tessellation. Benchmark existing decimation and interaction features independently; preserve data/gap/hit-test semantics. |
| 6 | Complete interaction, ownership, and platform coverage | Open overlays, filtering, dragging, focus/IME, actual theme switching, concurrent animations, browser runtime, and other native platforms remain listed gaps. Include common element/layout/copy churn and revisit Sidebar only with representative usage evidence. |

These are reviewable work boundaries, not implemented fixes or new GitHub issues. Re-run identical scenarios for each proposed optimization and retain correctness and memory tradeoffs with the results.

## Validation and remaining issue work

Completed: formatting, workspace Clippy with all targets/features, 1,641 library tests across the SDK/color/looks/plot packages, the profiling package's build and visible-panel regression checks, all 30 Python helper tests, the eight-case headless smoke run, and the full headless baseline. The profiling library also type-checks for `wasm32-unknown-unknown`; this is not a browser performance measurement.

Issue 167 should remain open. The next measurement tranche needs application-attributed presentation distributions and input latency, additional interaction/control fixtures, actual theme changes, native heap attribution, animation settling, and WASM/other-platform runtime comparisons. The protocol and coverage matrix make those gaps explicit rather than treating representative screenshots or microbenchmarks as complete SDK coverage.
