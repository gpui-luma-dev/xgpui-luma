use std::sync::Arc;

use xgpui_luma::controls::tree_view::{TreeViewTemplate, TreeViewTheme, ThemedTreeViewTemplate};
use xgpui_luma::theme::{InteractionState};

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
    fn resolve_row(&self, state: InteractionState, selected: bool) -> xgpui_luma::controls::tree_view::TreeViewPalette {
        let size = self.theme.size();
        let tokens = self.theme.mode_tokens();
        tree_view_row_palette(tokens.as_ref(), selected, state, size)
    }

    fn resolve_scale(&self, scale_factor: f32) -> xgpui_luma::controls::tree_view::TreeViewScale {
        let size = self.theme.size();
        xgpui_luma::controls::tree_view::TreeViewScale::compute_with_stylesheet(
            &self.metrics(),
            scale_factor,
            self.theme.mode_tokens().common_for(size),
        )
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics_for(self.theme.size())
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
