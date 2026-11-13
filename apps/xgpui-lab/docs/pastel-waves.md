# Pastel Waves renderer refresh (#150)

Source inspected and ported: XGPUI `e0a7833b6e6ae234eb6c19d2988c085e9524daeb`,
`apps/xgpui-ext-lab/paint-lab/src/pastel_waves.rs` and its producer/shader assets.
This is source/API inspection; hardware visual and performance acceptance is pending.
Published dependencies remain xgpui 3.1.0, shader 0.4.3, apple 4.0.1 and wgpu 2.2.0.

## Rendering and app integration

The SDK/Radix Renderer selector directly selects native, WGSL or paths.
Animate and Grain use Radix switches; Reverse and Reset use Radix buttons.
Phase scrubbing pairs a Radix slider with a synchronized numeric field in the
`LabScreen` parameter panel. Defaults follow
upstream: native on macOS, WGSL elsewhere. Native means Metal on macOS, D3D11
on Windows and wgpu on Linux. The paths implementation keeps Luma's retained
brush/material and paused-geometry caches. Reset preserves the chosen renderer.

WGSL compile/draw errors fall back to paths in the same paint. Native preparation
runs later in the platform renderer: failures are recorded, request a subsequent
UI redraw, and use paths thereafter. Selecting a different renderer clears the failure and
allows a retry. The first failed native frame may show only the background.
Requested/effective renderer and the preparation error are visible. Errors after
successful native preparation (for example device loss) remain platform errors.
No core GPUI API was added.

Grain, reverse, reset, scrub, reduced motion, retained page state and SDR
presentation remain app-owned. Suspension stops playback; returning never
starts it. Reduced motion freezes automatic phase changes and keeps scrubbing
available. Native producers create immutable per-frame content and cache pipelines
by device/queue identity.

Timing follows upstream's 0.15 interval smoothing, 50 ms elapsed cap and 1/30 s
step cap at 0.22 radians/second nominal speed. Below 30 render calls/second,
phase deliberately advances slower than wall time. There is no catch-up after
navigation or a long stall. The cosine easing remains unchanged.

Frame count and render FPS count view render calls, including redraws caused by
controls; they are not presented FPS. Paused/reduced-motion FPS displays zero.
CPU draw and separate readback wall time are capture-probe metrics. GPU execution
time and actual presentation cadence are unmeasured. No speedup is claimed.

## Features and validation

Linux enables its direct wgpu producer dependency by target, independently of
`grid-wgpu`. On macOS/Windows `waves-wgpu` optionally adds the wgpu producer;
normal Waves builds do not require Grid's feature. Windows adds the D3D compiler
features needed by the producer. Naga validates WGSL and translates shader model
5.0 HLSL on every test host. macOS-only Metal APIs compile against published crates.

Headless SDK tests cover switching, reset preserving renderer, scrub/reverse/grain,
reduced motion, and suspend/reactivate. Existing app navigation coverage verifies
entity retention and stop-before-navigation. Pure tests cover timing, generated
fill bounds, portable shader compilation, producer validation/HLSL translation,
and unsupported native preparation error reporting.

`--waves-check OUTPUT` opens a native window and requires separate explicit
launch approval under AGENTS.md. It retains exact retained-versus-rebuilt path
checks, material counts and paused geometry reuse. It additionally compares native
and WGSL output against paths at two logical sizes, both grain modes and four
phases, saving each image and reporting actual scale factor and separate CPU
wall times. It rejects silent fallback and flat output. Cross-algorithm thresholds
are provisional: mean RGB channel error ≤8/255 and fraction of RGB channels with
error >16/255 ≤10%. Override with `WAVES_MAX_MEAN_ERROR` and
`WAVES_MAX_BAD_FRACTION`. These thresholds need review against authorized captures;
they are not a claim of verified visual equivalence. Exact path-cache comparisons
remain exact. A failed comparison exits nonzero; inspect saved images and stderr.

Pending hardware acceptance: run captures at display scale factors 1 and 2;
visually inspect ribbons, fold edges and grain; calibrate tolerances; run Windows
D3D11 and Linux wgpu builds/captures. The optional Metal/wgpu producer comparison
is ignored by default because it requires explicit capture authorization.

## Imported lab inventory and follow-ups

All upstream paths below are relative to `apps/xgpui-ext-lab/` at the inspected
revision above. Provenance of older imports is retained in `apps/xgpui-lab/NOTICE`.
This inventory records source organization and demonstrated gaps, not runtime parity.

| Embedded area | Upstream sources | Intentional differences / follow-up |
| --- | --- | --- |
| Pastel Waves | `paint-lab/src/pastel_waves{.rs,.wgsl,/}` | New native/WGSL renderers ported; retained paths, SDK controls, scrub, suspension and SDR are Luma-specific. Hardware acceptance remains open. |
| Paint/brush, Hermite, mesh, portable editor | `paint-lab/src/{main,brushes,editor,portable,hermite_editor,mesh_tangent}.rs`, `checks/` | Luma decomposes pages into `shader/`, SDK controls and embedded checks. Follow-up: compare retained editor and mesh behavior by module; do not replace app navigation or benchmark ownership. |
| Shader examples, shadows, typography | `shader-examples/src/{examples,shadows,typography}.rs` | SDK controls and stop-on-navigation replace standalone launcher/chrome. Follow-up: compare each shader fixture and typography interaction; no additional missing renderer established in this pass. |
| Gamut/chromaticity | `gamut-lab/src/`, `extensions-web/src/color_samples.rs` | Luma has embedded benchmarks, sRGB preview controls and dedicated color pages. Follow-up: reconcile fixture/check changes against older NOTICE revisions. |
| Color | `color-lab/src/`, `extensions-web/src/color.rs` and `color_samples.rs` | Shared conversion references, app-specific pickers and SDR previews; follow-up: source/fixture parity audit. |
| Media/video/photo | `media-lab/src/{video_lab,fixtures,model,ui,float_lab,tiled_lab,upload_bench,ui_perf,scroll_perf,headless}.rs`, shared tiled demo | Video lifecycle synchronized previously; retained app reports and user-selected files are intentional. Follow-up: compare upstream tiled lab against embedded tiled modes; do not assume identical page organization. |
| SVG | `svg-lab/src/{main,scene,gallery,camera,benchmark}.rs`, `media-lab/src/paths.rs` | Bundled redistributable corpus subset and user-file loading are intentional. Follow-up: compare camera/benchmark changes without expanding bundled artwork. |
| Parley/text | `text-lab/src/{samples,timings,paragraph,input,axes,workload}.rs` | Luma uses SDK-hosted text input, retained pages and isolated raster/font tests. Upstream has an `axes` module; dedicated embedded parity needs a focused investigation. |
| Canvas/Mandelbrot | `canvas-lab/src/{main,interaction,mandelbrot,minimap,overview_map,profile}.rs`, shared Mandelbrot | Retained SDK controls, suspension and result management intentional. Follow-up: compare interaction/profile modules against prior 74b8e4e import. |
| Shell/preferences | `shell-lab/src/main.rs` | Uses published shell 0.2.0 with Luma preview and setting controls. Follow-up: audit current preference options independently. |
| Producer benchmarks | `quad-bench/src/main.rs`, `color-field-bench/src/`, `metal-producer-regression/src/main.rs` | Embedded reports and bounded same-executable worker isolation replace standalone launchers. Follow-up: workload/protocol parity audit. |

Non-Waves rows are follow-up investigation scope, not a claim that all current
upstream functionality has been exhaustively reconciled. No unrelated lab was rewritten.

## Local implementation checks

macOS default, `waves-wgpu`, and combined `grid-wgpu,waves-wgpu` builds pass.
Formatting and Clippy pass. Waves unit/SDK tests and the existing navigation
regression pass without launching the app. Cross-target checks were attempted:
Windows stops in GPUI's resource build because `llvm-rc` is unavailable; Linux
stops in system-library discovery because no cross-compilation pkg-config
sysroot is configured. Neither establishes Windows/Linux compile acceptance.
GUI/capture probes have not been run.

## Carried-over app documentation

[Pastel Waves app documentation and visual attribution](upstream/pastel-waves-lab-notes.md)
preserves only the Waves section of the upstream paint-lab README. No Markdown
exists inside upstream `src/pastel_waves/` at the inspected revision.

[Remove Pastel Waves Lab App — xgpui#195](https://github.com/gpui-luma-dev/xgpui/issues/195)
tracks eventual removal from XGPUI.
