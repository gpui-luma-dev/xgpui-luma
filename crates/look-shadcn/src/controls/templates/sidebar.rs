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
        super::drag::drag_look(&self.theme)
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
