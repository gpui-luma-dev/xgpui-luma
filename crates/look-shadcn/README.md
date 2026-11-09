# xgpui-luma-look-shadcn

Source repository: [gpui-luma-dev/xgpui-luma](https://github.com/gpui-luma-dev/xgpui-luma).

Shadcn themes and styled XGPUI-Luma controls for **xgpui**.

Provides token resolution, CSS stylesheet ingestion, shadow ladders, light/dark mode transitions, and concrete control builders adhering to the Shadcn design language.

Built on [xgpui](https://xgpui.dev/) and the [XGPUI-Luma SDK](https://github.com/gpui-luma-dev/xgpui-luma/tree/main/crates/sdk).

## Features

* **CSS Token Engine**: Parses CSS variable catalogs (`parse_css_catalog`) with embedded fallback stylesheet support.
* **Control Builders**: Pre-styled builders for Button, Input, Checkbox, Radio, Switch, Slider, Tabs, TreeView, Table, and Overlays.
* **Theme Inspection**: Optional `inspect` feature providing runtime token resolution introspection for developer tooling.

## Installation

```sh
cargo add xgpui-luma@1.0.1
cargo add xgpui-luma-look-shadcn@1.0.1
cargo add xgpui@3.1.0 --rename gpui
```

The package is named `xgpui-luma-look-shadcn`; Rust imports use `xgpui_luma_look_shadcn`.

## Quick Start

```rust,no_run
use gpui::{Context, Entity};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_shadcn::{ShadcnLook, Button, ShadcnSize};

fn save_button<M: 'static>(
    look: &ShadcnLook,
    cx: &mut Context<M>,
) -> Entity<xgpui_luma::controls::button::Button> {
    Button::new("save")
        .look(look)
        .primary()
        .size(ShadcnSize::Md)
        .label("Save")
        .spawn(cx)
}
```

Use `ShadcnLook::built_in()` for the bundled fallback theme, or
`ShadcnLook::from_css_str` to parse your application's theme CSS.

To enable optional theme inspection:

```sh
cargo add xgpui-luma-look-shadcn@1.0.1 --features inspect
```

## Build

In your application, run `cargo check` or `cargo build`.
From the Luma repository, run `cargo check -p xgpui-luma-look-shadcn`.

## License

Apache-2.0. See `LICENSE` in the package. Third-party material retains its
own license terms.
