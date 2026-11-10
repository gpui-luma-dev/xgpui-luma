# Upstream Pastel Waves lab notes

Waves-specific Markdown sections carried over from XGPUI at
`e0a7833b6e6ae234eb6c19d2988c085e9524daeb` (Apache-2.0). These snapshots preserve upstream
usage, rendering descriptions and visual-reference attribution without copying
unrelated lab documentation. Commands and source paths refer to upstream XGPUI.
The brushes section describes the older path reference; use the implementation
and [Luma notes](../pastel-waves.md) for current renderer support.

See also the [full upstream performance study](pastel-waves-performance-study.md).

## Source: `apps/xgpui-ext-lab/paint-lab/README.md`

[Original at inspected revision](https://github.com/gpui-luma-dev/xgpui/blob/e0a7833b6e6ae234eb6c19d2988c085e9524daeb/apps/xgpui-ext-lab/paint-lab/README.md)

### Pastel waves

Open the standalone pastel ribbon composition:

```sh
cargo run --locked --config .cargo/validation.toml \
  --manifest-path apps/xgpui-ext-lab/Cargo.toml --bin paint-lab --features portable-shaders -- --pastel-waves
```

The composition uses a pale freeform background and nine overlapping translucent
mesh-gradient ribbons, with fine highlights and optional grain. **Animate** starts
gentle forward/reverse motion with eased turns, **Pause** freezes it,
**Reverse** changes direction, **Grain** toggles dithering and fine threads,
and **Reset** restores the initial still composition. A small bottom-right label
shows the render frame count, measured render FPS, and direction. FPS is not a
GPU presentation measurement. Resize the window to check
that the artwork follows its bounds. The demo lives in `src/pastel_waves.rs`.

**Renderer** cycles three implementations of the same composition:

| Label | Implementation |
| --- | --- |
| `metal` / `d3d11` / `wgpu` | Native producer, chosen at build time by target: a compute pass evaluates every ribbon curve once per pixel column, then one fragment pass shades the target. Metal uses `src/pastel_waves/pastel_waves.metal`; D3D11 (Windows) and wgpu (Linux) share `src/pastel_waves/producer.wgsl`, translated to HLSL shader model 5.0 with naga for D3D11. |
| `wgsl` | One portable instanced WGSL draw (`src/pastel_waves.wgsl`). |
| `paths` | The original brush and native path-coverage reference. |

The native producer is the default on macOS. On Windows and Linux the default
stays `wgsl` until the native producers are verified on that hardware; select
them with **Renderer**. `just wgpu-test` checks that the wgpu producer matches
the Metal producer (on macOS), and the HLSL translation is tested on every
desktop target.

### Visual reference attribution

The pastel waves demo was visually inspired by
[Abstract pastel wave background — pink, blue and purple gradient](https://pngtree.com/freebackground/abstract-pastel-wave-background-pink-blue-purple-gradient_15809178.html)
on PNGTree. Credit for the reference artwork belongs to its original creator.
The demo recreates the visual style with procedural Rust geometry and shaders;
the reference image is not bundled or used as a texture.


## Source: `docs/extensions/brushes.md`

[Original at inspected revision](https://github.com/gpui-luma-dev/xgpui/blob/e0a7833b6e6ae234eb6c19d2988c085e9524daeb/docs/extensions/brushes.md)

### Pastel waves composition

The standalone native example combines nine translucent mesh-gradient ribbons
with fixed pastel background color fields. Separate variable-width contours and
soft inward bleed gradients create the rounded-fold illusion. The shading
follows the ribbon geometry rather than blurring its silhouette.

```sh
cargo run --release --locked --config .cargo/validation.toml \
  --manifest-path apps/xgpui-ext-lab/Cargo.toml --bin paint-lab --features portable-shaders -- --pastel-waves
```

Use **Animate / Pause**, **Reverse**, **Grain**, and **Reset**. Motion travels
forward and backward with eased endpoint turns. The bottom-right label shows
render frame count, measured render FPS and direction; it does not measure GPU
presentation cadence. The background fields stay fixed.

This is a visual reference implementation, not an optimized animation pipeline.
Each foreground ribbon's bleed uses 24 overlapping translucent shapes with
native path coverage. Frame timing is smoothed and capped, so animation can slow
under load. [Issue #35](https://github.com/gpui-luma-dev/xgpui/issues/35) tracks
release profiling, cached-background experiments and directly shaded ribbon
strips, with in-app comparisons and resource measurements. No performance gain
or target frame rate is claimed yet. Source: `apps/xgpui-ext-lab/paint-lab/src/pastel_waves.rs`.



## Source: `docs/extensions/extension-lab.md`

[Original at inspected revision](https://github.com/gpui-luma-dev/xgpui/blob/e0a7833b6e6ae234eb6c19d2988c085e9524daeb/docs/extensions/extension-lab.md)

#### Pastel waves

Open the native paint lab's animated ribbon experiment:

```sh
just lab-paint --pastel-waves
```



