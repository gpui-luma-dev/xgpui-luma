# xgpui-luma-look-radix

Source repository: [gpui-luma-dev/xgpui-luma](https://github.com/gpui-luma-dev/xgpui-luma).

Radix themes and styled XGPUI-Luma controls for **xgpui**.

Provides palette scales, semantic token resolution, custom color generation, light/dark mode handling, and concrete control builders adhering to the Radix design language.

Built on [xgpui](https://xgpui.dev/) and the [XGPUI-Luma SDK](https://github.com/gpui-luma-dev/xgpui-luma/tree/main/crates/sdk).

## Features

* **Stepped Palette Scales**: Full 12-step Radix color scales for accent and neutral palettes.
* **Accessible States**: Built-in high-contrast variants, focus metrics, and alpha steps.
* **Control Builders**: Pre-styled builders for Button, Checkbox, Radio, Switch, Slider, Tabs, Toolbar, PopupMenu, and TreeView.
* **Display Elements**: Direct element implementations for Avatar, Badge, Card, and Callout.

## Installation

```sh
cargo add xgpui-luma@1.0.1
cargo add xgpui-luma-look-radix@1.0.1
cargo add xgpui@3.1.0 --rename gpui
```

The package is named `xgpui-luma-look-radix`; Rust imports use `xgpui_luma_look_radix`.

## Quick Start

```rust,no_run
use gpui::{Context, Entity};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_radix::{Look, Button, ButtonSize};

fn save_button<M: 'static>(look: &Look, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::button::Button> {
    Button::new("save")
        .look(look)
        .solid()
        .size(ButtonSize::Two)
        .label("Save")
        .spawn(cx)
}
```

Use `Look::built_in()` to create the default look. Pass a reference to that look
to builders so controls share the same palette and styling settings.

## Build

In your application, run `cargo check` or `cargo build`.
From the Luma repository, run `cargo check -p xgpui-luma-look-radix`.

## License

Apache-2.0. See `LICENSE` in the package. Radix and Color.js attribution is in
`src/custom_colors/LICENSES.txt`.
