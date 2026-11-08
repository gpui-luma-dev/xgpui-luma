# SDK rendering allocation measurements

Issue [#111](https://github.com/gpui-luma-dev/gpui-luma/issues/111) measures shadow storage and representative control rendering after the earlier allocation reductions. The results support retaining public `Option<Vec<BoxShadow>>` fields and the existing public layout-cache API. Two internal changes remove temporary Sidebar containers and limit TextArea prepaint traversal to viewport candidates.

## Reproduction

Run from the repository root, without launching an application:

```sh
cargo bench --locked -p xgpui-luma --bench render_hot_paths -- --measure
cargo test --locked --release -p xgpui-luma --features test-support --lib perf_tests::measure_ -- --ignored --nocapture --test-threads=1
```

The second command uses GPUI's test platform and TestWindow, not a native application window. The measurements are ignored during ordinary tests. Run the commands sequentially, with no other builds in progress. The test fixtures exercise production methods rather than copies of their algorithms.

The recorded environment is arm64 macOS (Darwin 27.0.0), rustc 1.99.0 (`b940084d7`, LLVM 23.1.1), GPUI 1.24.2, on October 21, 2026. Cargo's repository release profile uses thin LTO and one codegen unit. The runtime baseline is `a989c653`, with the new diagnostic harness applied before the production changes.

Each measurement warms the process with three operations, then reports the median and minimum/maximum of five batches. Iteration counts live beside each fixture. A separate pass counts allocation calls, including zeroed allocations and reallocations, on the measuring thread. Requested bytes sum the allocation sizes and the new sizes of reallocations; they are allocation traffic, not peak or retained memory. Deallocation is included in latency but does not subtract from the byte count. Background-thread and GPU allocations are outside this counter.

Fixtures, fonts initialized by the initial TestWindow render, and initial application setup are excluded unless the case explicitly measures construction. Cold cases repeatedly reset a control cache or construct an index/entity in an already initialized process; they do not measure process startup. Redraw cases include notification, headless layout/prepaint/paint, and executor draining. These are CPU diagnostic workloads, not native GPU frame times or statistical confidence intervals.

## Workloads

- Shadows use zero, one, two, and eight copies of the default control shadow layer. Storage cases measure construction from a slice, cloning, and both operations followed by GPUI conversion. The control fixture redraws 100 SDK Buttons and separately constructs Button entities; entity construction does not render the button or resolve its look.
- Sidebar uses 100 and 10,000 flat labeled rows. Cases cover matching and late-mismatching signatures, cold listener binding, warm handler reuse, render-model construction plus binding, and a full headless redraw. Collapsed rail cases open one parent with 10 or 100 children and measure actual submenu listener binding and model construction.
- TreeView uses expanded branches with nine leaves each, for 100 or 10,000 total nodes. Cases measure validated index construction, virtualized warm redraws, and item replacement plus redraw. Replacement includes cloning the input tree, rebuilding metadata/projection, and reconciling state.
- TextArea uses 100 or 10,000 repetitions of a line containing Latin text, accented characters, CJK, emoji, a combining sequence, and Arabic. The viewport is five rows and 300 pixels wide. Cases cover unchanged redraws, cleared layout caches, alternating edits, wrap-width changes through padding, scrolling, changing selection, changing IME ranges, and unchanged selection decorations. Mutation setup and its allocations are included.

## Measured improvements

| Workload | Before µs/op | After µs/op | Allocations before → after |
| --- | ---: | ---: | ---: |
| Sidebar warm handlers, 100 rows | 0.310 | 0.192 | 7 → 0 |
| Sidebar warm handlers, 10,000 rows | 23.804 | 24.395 | 7 → 0 |
| Sidebar cold handlers, 100 rows | 12.808 | 12.450 | 532 → 518 |
| TextArea warm redraw, 100 lines | 23.658 | 23.583 | 228 → 228 |
| TextArea warm redraw, 10,000 lines | 33.692 | 23.683 | 228 → 228 |

Sidebar used struct updates from `Default` after supplying all seven row-container fields. Constructing and immediately dropping the unused default `Rc` containers cost seven allocations and 304 requested bytes per warm binding. Explicitly initializing only the submenu fields removes that traffic; cold binding also avoids a second set of seven defaults. The signature scan still dominates at 10,000 rows, so this is an allocation reduction without an established latency gain at that size.

TextArea previously scanned every cached line to test viewport intersection. It now computes a conservative candidate range and retains the original inclusive intersection test. The 10,000-line warm redraw falls by about 30% in this fixture, while the 100-line warm redraw is effectively unchanged. Allocation counts do not change. Regression tests cover fractional heights and scrolling boundaries; no timing thresholds are used in tests.

Final representative workload medians:

| Workload | µs/op | Allocations/op |
| --- | ---: | ---: |
| 100 Buttons, 0 / 1 / 2 / 8 shadow layers | 1094 / 1140 / 1183 / 1439 | 8719 / 9320 / 9320 / 9320 |
| Sidebar signature hit, 100 / 10,000 rows | 0.182 / 24.151 | 0 / 0 |
| Sidebar full redraw, 100 / 10,000 rows | 717 / 89,300 | 6662 / 671615 |
| Collapsed submenu handlers, 10 / 100 items | 0.582 / 5.190 | 26 / 209 |
| TreeView index rebuild, 100 / 10,000 nodes | 3.319 / 374.827 | 8 / 15 |
| TreeView warm redraw, 100 / 10,000 nodes | 301 / 757 | 2088 / 2094 |
| TreeView replacement plus redraw, 10,000 nodes | 2751 | 8818 |
| TextArea cold layout / edit / resize, 10,000 lines | 9576 / 10560 / 10144 | 150171 / 150267.6 / 150266 |
| TextArea scroll / selection / IME / warm decoration, 10,000 lines | 24.525 / 25.475 / 24.983 / 24.875 | 228 / 241 / 240 / 229 |

Complete [before](render-hot-paths-before.txt) and [after](render-hot-paths-after.txt) outputs include batch ranges and requested bytes. The before output covers the three common collection/text fixtures; the Button fixture was added subsequently and has no before/after runtime change.

## Shadow storage decision

GPUI 1.24.2's `Styled::shadow` takes ownership of a `Vec<BoxShadow>`. Inline SmallVec storage must allocate when converted to that vector, and shared Arc storage must copy its elements. The conversion cases also include the common allocation for GPUI's Div style. Isolated allocation-free clones therefore do not establish a rendering improvement.

Each cell is **ns/op / allocations/op**, including GPUI ownership conversion:

| Layers | Operation | Vec | SmallVec with two inline slots | Arc slice |
| ---: | --- | ---: | ---: | ---: |
| 1 | clone plus GPUI | 84.0 / 2 | 90.8 / 2 | 95.6 / 2 |
| 1 | construct plus GPUI | 82.9 / 2 | 90.3 / 2 | 112.6 / 3 |
| 2 | clone plus GPUI | 84.7 / 2 | 96.5 / 2 | 94.7 / 2 |
| 2 | construct plus GPUI | 83.6 / 2 | 93.7 / 2 | 117.8 / 3 |
| 8 | clone plus GPUI | 92.8 / 2 | 110.5 / 2 | 107.6 / 2 |
| 8 | construct plus GPUI | 93.5 / 2 | 103.6 / 2 | 120.7 / 3 |

The isolated one-layer clones are cheaper with SmallVec and Arc, but the complete conversion cases do not reduce allocations or latency. Arc construction plus conversion adds another allocation. See the [complete shadow measurements](render-hot-paths-shadows.txt), including empty stacks, clone-only costs, construction, ranges, and requested bytes.

Keep the existing public shadow-field types. No downstream migration is required. A future storage proposal needs evidence from complete control rendering with the proposed representation, including construction, GPUI conversion, larger spilled stacks, and native frame profiling.

## Remaining costs

Sidebar signature validation remains proportional to the rendered row count. It checks the current rendered projection, including children disappearing after collapse animations; replacing it with an incomplete revision key could retain stale listeners. Large Sidebar redraws also construct all rows. Virtualizing those rows would be a separate architectural change.

Collapsed submenu listeners are still rebuilt, and TreeView replacement still rebuilds its validated index and projection. The harness makes these costs reproducible. Adding submenu invalidation state or changing tree reconciliation is not justified by the isolated handler/index numbers alone.

TextArea cache misses still shape the entire document. The viewport change only bounds the traversal of already cached lines; it does not promise constant-time editing or resizing. Existing shaping and decoration caches continue to handle unchanged text, selection, and IME ranges separately.

The unused public layout-cache API remains compatible. Deprecation is independent of this profiling work.

## Validation and manual testing

Regression coverage checks that warm Sidebar handler binding allocates nothing, that projection and mode changes still rebuild handlers, and that TextArea viewport candidates preserve the previous inclusive edge behavior across empty documents, fractional line heights, partial scrolling, and translated viewports. Existing headless tests cover wrapping, Unicode glyph geometry, selection, IME decoration, and cache invalidation. The allocation counter has a zeroed-allocation/reallocation check.

Completed checks:

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features
cargo test --locked -p xgpui-luma --all-features --lib
```

The SDK suite passes 811 tests with five manual measurements ignored. All four new release measurement tests and the standalone shadow benchmark also complete successfully. Run the full interaction suite in its normal test profile: GPUI release builds omit debug selectors used by existing interaction tests.

Before opening a PR, manually test Sidebar expansion/collapse, reordering, collapsed rail submenus, and TextArea long-document scrolling, resizing, Unicode selection, and IME composition in a native example app. Native GUI profiling requires explicit launch approval under `AGENTS.md`.
