use std::sync::Arc;

use xgpui_luma::controls::tree_view::{StyledTreeView, TreeViewTemplate, TreeViewTheme, ThemedTreeViewTemplate};
use xgpui_luma::theme::{InteractionState};

use crate::look::ShadcnLook;

/// The Shadcn tree view: rows and layout from `.lss` (`style/tree_view.lss`); icons and the drag shadow from
/// the look.
struct ShadcnTreeViewTheme {
    theme: ShadcnLook,
    styled: StyledTreeView,
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
        self.styled.row(state, selected)
    }

    fn row_outline(&self, state: InteractionState) -> Option<gpui::Hsla> {
        self.styled.outline(state)
    }

    fn resolve_scale(&self, scale_factor: f32) -> xgpui_luma::controls::tree_view::TreeViewScale {
        self.styled.scale(scale_factor)
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
    let size = theme.size();
    let style = theme.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "tree_view",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || style.style_theme())),
    );
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(ShadcnTreeViewTheme { theme, styled: StyledTreeView::new(binding) })
}
