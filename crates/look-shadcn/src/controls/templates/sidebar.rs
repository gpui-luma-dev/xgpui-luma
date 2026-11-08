use std::sync::Arc;

use xgpui_luma::controls::sidebar::{SidebarPanelTemplate, SidebarTheme, ThemedSidebarPanelTemplate};
use xgpui_luma::theme::{ControlSize, InteractionState};

use super::floating_menu::floating_menu_theme;
use crate::controls::sidebar::{sidebar_branch_look, sidebar_container_look, sidebar_item_look, sidebar_section_look};
use crate::look::ShadcnLook;

struct ShadcnSidebarTheme {
    theme: ShadcnLook,
}

impl SidebarTheme for ShadcnSidebarTheme {
    fn typography(&self) -> xgpui_luma::theme::LumaTypography {
        self.theme.mode_tokens().typography.clone()
    }
    fn section_font_features(&self) -> gpui::FontFeatures {
        self.theme.stylesheet().common.font_features.resolve("sidebar_section")
    }

    fn geometry(&self) -> xgpui_luma::theme::stylesheet::SidebarGeometry {
        self.theme.mode_tokens().stylesheet().common.sidebar.resolve_geometry("", Default::default())
    }
    fn resolve_drag(&self, size: ControlSize) -> xgpui_luma::infra::drag_drop::DragDropLook {
        let tokens = self.theme.mode_tokens();
        let palette = &tokens.palette;
        xgpui_luma::infra::drag_drop::DragDropLook {
            geometry: self.theme.stylesheet().common.shared_geometry(),
            background: self.theme.token_color("popover").unwrap_or(palette.panel_background),
            foreground: self.theme.token_color("popover-foreground").unwrap_or(palette.body_text),
            valid_marker: self.theme.token_color("ring").unwrap_or(palette.focus_ring),
            invalid_marker: self.theme.token_color("destructive").unwrap_or(palette.destructive_background),
            radius: tokens.metrics.radius(size),
            typography: tokens.typography.text.label,
            font_family: tokens.typography.font.sans.family.clone().into(),
        }
    }

    fn foreground(&self) -> gpui::Hsla {
        sidebar_container_look(self.theme.mode_tokens().as_ref()).foreground
    }

    fn resolve_section(&self) -> xgpui_luma::controls::sidebar::SidebarSectionLook {
        sidebar_section_look(&self.theme)
    }

    fn resolve_branch(
        &self,
        state: InteractionState,
        size: ControlSize,
    ) -> xgpui_luma::controls::sidebar::SidebarItemLook {
        sidebar_branch_look(&self.theme, state, size)
    }

    fn resolve_item(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> xgpui_luma::controls::sidebar::SidebarItemLook {
        sidebar_item_look(&self.theme, selected, state, size)
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn sidebar_panel_template(theme: ShadcnLook) -> Arc<dyn SidebarPanelTemplate> {
    let menu_theme = floating_menu_theme(theme.clone());
    Arc::new(ThemedSidebarPanelTemplate::new_with_floating_menu_theme(
        Arc::new(ShadcnSidebarTheme { theme: theme.clone() }),
        menu_theme,
    ))
}

pub fn sidebar_theme(theme: ShadcnLook) -> Arc<dyn SidebarTheme> {
    Arc::new(ShadcnSidebarTheme { theme: theme.clone() })
}
