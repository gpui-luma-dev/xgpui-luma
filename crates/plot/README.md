# xgpui-luma-plot

Source repository: [gpui-luma-dev/xgpui-luma](https://github.com/gpui-luma-dev/xgpui-luma).

Native charts for **xgpui**, built with XGPUI-Luma. Separates data, plot specifications,
presentation, and views. Supports Cartesian line, area, and bar charts,
pie and donut charts, radar charts, and radial charts.

Built on [xgpui](https://xgpui.dev/) and the [XGPUI-Luma SDK](https://github.com/gpui-luma-dev/xgpui-luma/tree/main/crates/sdk).

## Installation

```sh
cargo add xgpui-luma@1.0.1 xgpui-luma-plot@1.0.1
cargo add xgpui@3.1.0 --rename gpui
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

See the [Plot Viz examples](https://github.com/gpui-luma-dev/xgpui-luma/tree/main/apps/luma-plot-viz) for styling,
interaction, and chart families.

## Build

From the repository root, run `cargo check -p xgpui-luma-plot`.

## License

Apache-2.0. See [LICENSE](LICENSE). Third-party material retains its own license terms.
