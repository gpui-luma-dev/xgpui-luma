# xgpui-luma

Source repository: [gpui-luma-dev/xgpui-luma](https://github.com/gpui-luma-dev/xgpui-luma).

Lookless control SDK and interaction engine for **XGPUI-Luma** ([gpui-luma.dev](https://gpui-luma.dev)).

Provides component logic, event routing, keyboard navigation, focus management, drag-and-drop, and theme contracts independent of any visual theme or styling adapter.

## Modules

* **`controls`**: Buttons, text inputs, checkboxes, radios, switches, sliders, tree views, list boxes, menus, and overlays.
* **`focus`**: Two-dimensional focus navigation, ring indicators, and directional traversal.
* **`interaction`**: Hover tracking, active press state, double-click detection, and pointer capture.
* **`layouts`**: Dock panels, grid systems, wide-middle splits, and layer stacks.
* **`motion`**: Transitions, overlay enter/exit scaling, and continuous phases.
* **`theme`**: Look-agnostic theme provenance and style resolution traits.

## Usage

The installation commands below apply once version 1.0.1 is published to crates.io.

Depend on `xgpui-luma` directly and choose a look adapter for styled controls:

```sh
cargo add xgpui-luma@1.0.1
```
Choose a look adapter:

```sh
cargo add xgpui-luma-look-shadcn@1.0.1
# Or:
cargo add xgpui-luma-look-radix@1.0.1
```

The package is named `xgpui-luma`; Rust imports use `xgpui_luma`.

Build your application with `cargo check` or `cargo build`. In this repository,
check the SDK with `cargo check -p xgpui-luma`.

## License

Apache-2.0; see [LICENSE](LICENSE). Bundled third-party assets retain their
licenses; see the files under `licenses/`.
