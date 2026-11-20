use std::sync::Arc;

use xgpui_luma::controls::sidebar::{SidebarPanelTemplate, SidebarTheme, StyledSidebar, ThemedSidebarPanelTemplate};
use xgpui_luma::theme::{InteractionState};

use super::floating_menu::floating_menu_theme;
use crate::look::ShadcnLook;

/// The Shadcn sidebar: rows and section labels from `.lss` (`style/sidebar.lss`); the shell layout,
/// typography roles and drag preview from the look.
struct ShadcnSidebarTheme {
    theme: ShadcnLook,
    styled: StyledSidebar,
}

impl ShadcnSidebarTheme {
    fn new(theme: ShadcnLook) -> Self {
        let size = theme.size();
        let style = theme.clone();
        let binding = xgpui_luma::style::StyleBinding::new(
            "sidebar",
            xgpui_luma::style::ThemeSource::Live(Arc::new(move || style.style_theme())),
        );
        let binding = if size != crate::ShadcnSize::Md {
            binding.attr("size", size.key())
        } else {
            binding
        };
        Self { theme, styled: StyledSidebar::new(binding) }
    }
}

impl SidebarTheme for ShadcnSidebarTheme {
    fn typography(&self) -> xgpui_luma::theme::LumaTypography {
        self.theme.mode_tokens().typography.clone()
    }
    fn section_font_features(&self) -> gpui::FontFeatures {
        self.theme.stylesheet().common.font_features.resolve("sidebar_section")
    }

    fn geometry(&self) -> xgpui_luma::theme::stylesheet::SidebarGeometry {
        self.theme.mode_tokens().stylesheet().common.sidebar.resolve_geometry(Default::default())
    }
    fn resolve_drag(&self) -> xgpui_luma::infra::drag_drop::DragDropLook {
        let size = self.theme.size();
        let tokens = self.theme.mode_tokens();
        let palette = &tokens.palette;
        xgpui_luma::infra::drag_drop::DragDropLook {
            geometry: self.theme.stylesheet().common.shared_geometry(),
            background: self.theme.token_color("popover").unwrap_or(palette.panel_background),
            foreground: self.theme.token_color("popover-foreground").unwrap_or(palette.body_text),
            valid_marker: self.theme.token_color("ring").unwrap_or(palette.focus_ring),
            invalid_marker: self.theme.token_color("destructive").unwrap_or(palette.destructive_background),
            radius: tokens.control_sizes.radius(size),
            typography: tokens.typography.text.label,
            font_family: tokens.typography.font.sans.family.clone().into(),
        }
    }

    fn foreground(&self) -> gpui::Hsla {
        self.styled.foreground()
    }

    fn resolve_section(&self) -> xgpui_luma::controls::sidebar::SidebarSectionLook {
        self.styled.section()
    }

    fn resolve_branch(&self, state: InteractionState) -> xgpui_luma::controls::sidebar::SidebarItemLook {
        self.styled.branch(state)
    }

    fn resolve_item(&self, selected: bool, state: InteractionState) -> xgpui_luma::controls::sidebar::SidebarItemLook {
        self.styled.item(selected, state)
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics_for(self.theme.size())
    }
}

pub fn sidebar_panel_template(theme: ShadcnLook) -> Arc<dyn SidebarPanelTemplate> {
    let menu_theme = floating_menu_theme(theme.clone());
    Arc::new(ThemedSidebarPanelTemplate::new_with_floating_menu_theme(
        Arc::new(ShadcnSidebarTheme::new(theme.clone())),
        menu_theme,
    ))
}

pub fn sidebar_theme(theme: ShadcnLook) -> Arc<dyn SidebarTheme> {
    Arc::new(ShadcnSidebarTheme::new(theme))
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
                let sized = look.with_size(size);
                let theme = sidebar_theme(sized.clone());
                for selected in [true, false] {
                    for bits in 0..16u8 {
                        let state = InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            ..Default::default()
                        };
                        let got = theme.resolve_item(selected, state);
                        let want = crate::controls::sidebar::sidebar_item_look(&sized, selected, state, size);
                        let case = format!("{mode:?} {size:?} {selected} {state:?}");
                        assert_eq!(got.background.is_some(), want.background.is_some(), "{case} background");
                        if let (Some(g), Some(w)) = (got.background, want.background) {
                            crate::assert_close(g, w);
                        }
                        assert_eq!(got.focus_border.is_some(), want.focus_border.is_some(), "{case} focus");
                        crate::assert_close(got.foreground, want.foreground);
                        assert_eq!(
                            (got.radius, got.height, got.padding_x, got.gap, got.icon_size),
                            (want.radius, want.height, want.padding_x, want.gap, want.icon_size),
                            "{case}"
                        );
                    }
                }
            }
        }
    }
}
