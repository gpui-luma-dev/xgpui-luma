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

#[cfg(test)]
mod lss_tests {
    use super::*;

    #[test]
    fn lss_rows_match_the_native_rows() {
        let look = crate::test_support::fallback_look();
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for size in [crate::ShadcnSize::Sm, crate::ShadcnSize::Md, crate::ShadcnSize::Lg] {
                let theme = tree_view_theme(look.with_size(size));
                let tokens = look.mode_tokens();
                for selected in [true, false] {
                    for bits in 0..16u8 {
                        let state = InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            ..Default::default()
                        };
                        let got = theme.resolve_row(state, selected);
                        let want =
                            crate::controls::tree_view::tree_view_row_palette(tokens.as_ref(), selected, state, size);
                        let case = format!("{mode:?} {size:?} {selected} {state:?}");
                        assert_eq!(got.background.is_some(), want.background.is_some(), "{case}");
                        if let (Some(g), Some(w)) = (got.background, want.background) {
                            crate::assert_close(g, w);
                        }
                        crate::assert_close(got.foreground, want.foreground);
                        crate::assert_close(got.icon_color, want.icon_color);
                        assert_eq!(
                            (got.typography.size, got.typography.weight),
                            (want.typography.size, want.typography.weight),
                            "{case}"
                        );
                    }
                }
            }
        }
    }
}
