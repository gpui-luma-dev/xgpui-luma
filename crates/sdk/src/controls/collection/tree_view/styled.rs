//! Tree-view rows read from `.lss` (issue #285): `tree_view.lss`. Looks keep their icons and drag shadow and
//! delegate rows and layout to [`StyledTreeView`].

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value};

use super::theme::{TreeViewPalette, TreeViewScale, TreeViewTheme};
use crate::style::StyleBinding;
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens};

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// The tree view's `.lss` values for one binding (the `tree_view` control).
#[derive(Clone)]
pub struct StyledTreeView {
    binding: StyleBinding,
}

impl StyledTreeView {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("tree_view") }
    }

    fn resolve(&self, state: InteractionState, selected: bool) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        let states = [
            ("selected", selected),
            ("hover", state.hovered),
            ("pressed", state.pressed),
            ("focused", state.focused),
            ("disabled", state.disabled),
        ];
        self.binding.resolve(&states, 1.0)
    }

    /// A row in `state`, selected or not.
    pub fn row(&self, state: InteractionState, selected: bool) -> TreeViewPalette {
        let resolved = self.resolve(state, selected);
        let root = resolved.root();
        let background = color(root, "background");
        let icon = color(root, "icon_color");
        TreeViewPalette {
            background: (background.a > 0.0).then_some(background),
            foreground: color(root, "foreground"),
            icon_color: icon,
            chevron_color: icon,
            typography: LumaTextStyle {
                size: root.font_size().unwrap_or(14.0),
                line_height: root.line_height().unwrap_or(20.0),
                weight: gpui::FontWeight(number(root, "font_weight").max(100.0)),
            },
            font_family: root.font_family().unwrap_or("").to_string().into(),
        }
    }

    /// The row's inset outline in `state`, if any.
    pub fn outline(&self, state: InteractionState) -> Option<Hsla> {
        Some(color(self.resolve(state, false).root(), "outline")).filter(|color| color.a > 0.0)
    }

    /// The row layout at `scale_factor` device pixels per pixel.
    pub fn scale(&self, scale_factor: f32) -> TreeViewScale {
        let resolved = self.binding.resolve(&[], scale_factor);
        let root = resolved.root();
        TreeViewScale {
            drag_marker_width: number(root, "drag_marker_width"),
            drag_insert_width: number(root, "drag_insert_width"),
            drag_opacity: number(root, "drag_opacity"),
            scrollbar_width: number(root, "scrollbar_width"),
            disabled_opacity: number(root, "disabled_opacity"),
            preview_padding_x: number(root, "preview_padding_x"),
            preview_padding_y: number(root, "preview_padding_y"),
            row_height: number(root, "height"),
            base_padding_x: number(root, "padding_x"),
            indentation_width: number(root, "indentation_width"),
            inner_gap: number(root, "gap"),
            radius: number(root, "radius"),
            icon_size: number(root, "icon_size"),
            chevron_size: number(root, "chevron_size"),
        }
    }
}

/// The SDK's tree-view theme: `tree_view.lss`, with the SDK's icons and drag shadow.
pub struct StyledTreeViewTheme(StyledTreeView);

impl StyledTreeViewTheme {
    pub fn sdk() -> Self {
        Self(StyledTreeView::sdk())
    }
}

impl TreeViewTheme for StyledTreeViewTheme {
    fn resolve_row(&self, state: InteractionState, selected: bool) -> TreeViewPalette {
        self.0.row(state, selected)
    }

    fn row_outline(&self, state: InteractionState) -> Option<Hsla> {
        self.0.outline(state)
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }

    fn resolve_scale(&self, scale_factor: f32) -> TreeViewScale {
        self.0.scale(scale_factor)
    }
}
