# xgpui-luma-plot

Source repository: [gpui-luma-dev/xgpui-luma](https://github.com/gpui-luma-dev/xgpui-luma).

Native charts for XGPUI-Luma, with separate data, plot specification,
presentation, and view layers. Supports Cartesian line, area, and bar charts,
pie and donut charts, radar charts, and radial charts.

This crate is part of the **xgui ecosystem**, built on [xgpui](https://xgpui.dev/).
The XGPUI-Luma crates generally release shortly after xgpui releases to track
framework updates. Luma and xgpui have separate version numbers; use the
compatible versions shown in the installation examples.

The framework dependency is the Cargo package **`xgpui`**. In these examples,
`gpui` is a local dependency alias for `xgpui`, so `gpui::` imports refer to xgpui.
The `gpui-unofficial` package and packages ending in `-gpui-unofficial` are distinct
registry packages; retain their full package names when referring to them. The
original [gpui-luma](https://github.com/gpui-luma-dev/gpui-luma), based on
`gpui-unofficial`, remains a separate project.

## Installation

Once version 1.0.1 is published to crates.io:

```sh
cargo add xgpui-luma@1.0.1 xgpui-luma-plot@1.0.1
cargo add xgpui@2.0.1 --rename gpui
```

Rust imports use `xgpui_luma_plot`.

## Usage

```rust,no_run
use xgpui_luma_plot::{CartesianChartBuilder, Dataset, DatasetError, LineLayer, Sample, Series};

fn chart() -> Result<(), DatasetError> {
    let data = Dataset::new().with_series(Series::new(
        "temperature",
        "Temperature",
        [Sample::new(0.0, 18.0), Sample::new(1.0, 21.0)],
    ))?;
    let _view = CartesianChartBuilder::new(data)
        .line(LineLayer::new("temperature"))
        .build();
    Ok(())
}
```

See the [plot documentation](https://gpui-luma.dev/plots.html) for styling,
interaction, and chart families.

## Build

From the repository root, run `cargo check -p xgpui-luma-plot`.

## License

Apache-2.0. See [LICENSE](LICENSE). Third-party material retains its own license terms.
