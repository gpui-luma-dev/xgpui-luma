//! Tree-view rows read from `.lss` (issue #285): `tree_view.lss`. Looks keep their icons and drag shadow and
//! delegate rows and layout to [`StyledTreeView`].

use crate::style::PartRead;
use gpui::Hsla;

use super::theme::{TreeViewPalette, TreeViewScale, TreeViewTheme};
use crate::style::StyleBinding;
use crate::theme::{InteractionState, MetricTokens};

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
        let background = root.color("background");
        let icon = root.color("icon_color");
        TreeViewPalette {
            background: (background.a > 0.0).then_some(background),
            foreground: root.color("foreground"),
            icon_color: icon,
            chevron_color: icon,
            typography: root.typography(14.0, 20.0),
            font_family: root.font_family_name(),
            opacity: root.opacity(),
        }
    }

    /// The row's inset outline in `state`, if any.
    pub fn outline(&self, state: InteractionState) -> Option<Hsla> {
        Some(self.resolve(state, false).root().color("outline")).filter(|color| color.a > 0.0)
    }

    /// The row layout at `scale_factor` device pixels per pixel.
    pub fn scale(&self, scale_factor: f32) -> TreeViewScale {
        let resolved = self.binding.resolve(&[], scale_factor);
        let root = resolved.root();
        TreeViewScale {
            drag_marker_width: root.number("drag_marker_width"),
            drag_insert_width: root.number("drag_insert_width"),
            drag_opacity: root.number("drag_opacity"),
            scrollbar_width: root.number("scrollbar_width"),
            preview_padding_x: root.number("preview_padding_x"),
            preview_padding_y: root.number("preview_padding_y"),
            row_height: root.number("height"),
            base_padding_x: root.number("padding_x"),
            indentation_width: root.number("indentation_width"),
            inner_gap: root.number("gap"),
            radius: root.number("radius"),
            icon_size: root.number("icon_size"),
            chevron_size: root.number("chevron_size"),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A theme that supplies only metrics, given the SDK's own metrics, lays out exactly as the
    /// SDK's `.lss` tree view: the default path takes its fixed sizes and proportions from the `.lss`.
    #[test]
    fn the_default_scale_matches_the_sdk_lss() {
        for scale_factor in [1.0, 1.5, 2.0] {
            let default = crate::controls::tree_view::TreeViewScale::compute(
                &crate::theme::MetricTokens::default(),
                scale_factor,
            );
            assert_eq!(default, StyledTreeView::sdk().scale(scale_factor), "@{scale_factor}");
        }
    }
}
