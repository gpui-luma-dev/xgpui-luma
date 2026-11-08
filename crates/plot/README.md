# xgpui-luma-plot

Source repository: [gpui-luma-dev/xgpui-luma](https://github.com/gpui-luma-dev/xgpui-luma).

Native charts for XGPUI-Luma, with separate data, plot specification,
presentation, and view layers. Supports Cartesian line, area, and bar charts,
pie and donut charts, radar charts, and radial charts.

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
