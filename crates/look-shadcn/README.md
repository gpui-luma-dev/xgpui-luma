# xgpui-luma-look-shadcn

Source repository: [gpui-luma-dev/xgpui-luma](https://github.com/gpui-luma-dev/xgpui-luma).

Shadcn theme and look adapter for **XGPUI-Luma** ([gpui-luma.dev](https://gpui-luma.dev)).

Provides token resolution, CSS stylesheet ingestion, shadow ladders, light/dark mode transitions, and concrete control builders adhering to the Shadcn design language.

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

## Features

* **CSS Token Engine**: Parses CSS variable catalogs (`parse_css_catalog`) with embedded fallback stylesheet support.
* **Control Builders**: Pre-styled builders for Button, Input, Checkbox, Radio, Switch, Slider, Tabs, TreeView, Table, and Overlays.
* **Theme Inspection**: Optional `inspect` feature providing runtime token resolution introspection for developer tooling.

## Installation

The installation commands below apply once version 1.0.1 is published to crates.io.

```sh
cargo add xgpui-luma@1.0.1
cargo add xgpui-luma-look-shadcn@1.0.1
cargo add xgpui@2.0.1 --rename gpui
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
