# xgpui-luma-color

Source repository: [gpui-luma-dev/xgpui-luma](https://github.com/gpui-luma-dev/xgpui-luma).

Color primitives and interactive color controls for **xgpui**, built with XGPUI-Luma.

Provides domain renderers, checkerboard painting, palette calculations, and interactive color manipulation components built on the lookless Luma SDK.

Built on [xgpui](https://xgpui.dev/) and the [XGPUI-Luma SDK](https://github.com/gpui-luma-dev/xgpui-luma/tree/main/crates/sdk).

## Components

* **`ColorField`**: 2D saturation/brightness or hue/saturation selection surface.
* **`ColorRing` & `ColorArc`**: Radial hue and chromaticity selectors.
* **`ColorSlider`**: 1D gradient sliders for alpha, hue, saturation, and lightness.
* **`ColorSwatch` & `ColorSwatchButtonTemplate`**: Swatches with checkerboard transparency backdrops and selected states.
* **`CheckerboardPaint`**: GPU-friendly alpha background rendering.

## Installation

```sh
cargo add xgpui-luma@1.0.1
cargo add xgpui-luma-color@1.0.1
cargo add xgpui@3.1.0 --rename gpui
```

The package is named `xgpui-luma-color`; Rust imports use `xgpui_luma_color`.

## Usage

Create a swatch to include in an xgpui view:

```rust,no_run
use xgpui_luma::color::ColorValue;
use xgpui_luma_color::ColorSwatch;

fn accent_swatch() -> ColorSwatch {
    ColorSwatch::new(ColorValue::srgb(0.2, 0.4, 0.8, 1.0))
}
```

## Build

In your application, run `cargo check` or `cargo build`.
From the Luma repository, run `cargo check -p xgpui-luma-color`.

## License

Apache-2.0. See `LICENSE` in the package.
