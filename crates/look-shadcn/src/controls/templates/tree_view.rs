use std::sync::Arc;

use xgpui_luma::controls::tree_view::{TreeViewTemplate, TreeViewTheme, ThemedTreeViewTemplate};
use xgpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::tree_view::tree_view_row_palette;
use crate::look::ShadcnLook;

struct ShadcnTreeViewTheme {
    theme: ShadcnLook,
}

impl TreeViewTheme for ShadcnTreeViewTheme {
    fn drag_preview_shadow(&self) -> Vec<gpui::BoxShadow> {
        self.theme
            .stylesheet()
            .common
            .theme_tokens(self.theme.mode(), xgpui_luma::color::GamutMapping::CssLocalMinde)
            .map(|tokens| tokens.elevation.drag_preview.to_box_shadows())
            .unwrap_or_default()
    }

    fn icon(&self, role: xgpui_luma::theme::stylesheet::IconRole) -> xgpui_luma::infra::icon::IconSource {
        self.theme.mode_tokens().stylesheet().common.icons.resolve(role)
    }
    fn resolve_row(
        &self,
        state: InteractionState,
        selected: bool,
        size: ControlSize,
    ) -> xgpui_luma::controls::tree_view::TreeViewPalette {
        let tokens = self.theme.mode_tokens();
        tree_view_row_palette(tokens.as_ref(), selected, state, size)
    }

    fn resolve_scale(&self, size: ControlSize, scale_factor: f32) -> xgpui_luma::controls::tree_view::TreeViewScale {
        xgpui_luma::controls::tree_view::TreeViewScale::compute_with_stylesheet(
            size,
            &self.metrics(),
            scale_factor,
            &self.theme.mode_tokens().stylesheet().common,
        )
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn tree_view_template<T>(theme: ShadcnLook) -> Arc<dyn TreeViewTemplate<T>>
where
    T: Send + Sync + 'static,
{
    Arc::new(ThemedTreeViewTemplate::new(tree_view_theme(theme.clone())))
}

pub fn tree_view_theme(theme: ShadcnLook) -> Arc<dyn TreeViewTheme> {
    Arc::new(ShadcnTreeViewTheme { theme: theme.clone() })
}
