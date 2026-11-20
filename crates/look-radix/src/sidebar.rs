//! Radix sidebar row, section, rail-menu, and scrollbar adapters.
use std::sync::Arc;
use gpui::SharedString;
use xgpui_luma::controls::sidebar::*;
use xgpui_luma::theme::{InteractionState, MetricTokens};
use crate::{Look, PopupMenuVariant, Tone};
/// The Radix sidebar: rows and section labels from `.lss` (`style/sidebar.lss`); the shell layout,
/// typography roles, section font features and drag preview from the look.
struct SidebarThemeAdapter {
    look: Look,
    styled: StyledSidebar,
}
impl SidebarTheme for SidebarThemeAdapter {
    fn typography(&self) -> xgpui_luma::theme::LumaTypography {
        let mut typography = xgpui_luma::theme::LumaTypography::default();
        typography.font.sans.family = crate::typography::font_family(&self.look).to_string();
        typography
    }
    fn section_font_features(&self) -> gpui::FontFeatures {
        self.look.common_stylesheet().font_features.resolve("sidebar_section")
    }
    fn geometry(&self) -> xgpui_luma::theme::stylesheet::SidebarGeometry {
        self.look.common_stylesheet().sidebar.resolve_geometry(Default::default())
    }
    fn foreground(&self) -> gpui::Hsla {
        self.styled.foreground()
    }
    fn resolve_section(&self) -> SidebarSectionLook {
        self.styled.section()
    }
    fn resolve_branch(&self, state: InteractionState) -> SidebarItemLook {
        self.styled.branch(state)
    }
    fn resolve_item(&self, selected: bool, state: InteractionState) -> SidebarItemLook {
        self.styled.item(selected, state)
    }
    fn metrics(&self) -> MetricTokens {
        self.look.metrics_for(self.look.size())
    }
    fn resolve_drag(&self) -> xgpui_luma::infra::drag_drop::DragDropLook {
        let size = self.look.size();
        crate::navigation::drag_look(&self.look, size)
    }
}
struct RailMenuTheme {
    look: Look,
}
impl xgpui_luma::controls::floating_menu::FloatingMenuTheme for RailMenuTheme {
    fn resolve(&self) -> xgpui_luma::controls::floating_menu::FloatingMenuLook {
        crate::floating_menu_look(&self.look, PopupMenuVariant::Solid, Tone::Accent)
    }
}
/// Live Radix navigation rows, section labels, focus and drag feedback.
pub fn sidebar_theme(look: &Look) -> Arc<dyn SidebarTheme> {
    let size = look.size();
    let binding = xgpui_luma::style::StyleBinding::new("sidebar", crate::textfield::live_source(look));
    let binding = if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(SidebarThemeAdapter { look: look.clone(), styled: StyledSidebar::new(binding) })
}
/// SDK sidebar template, including Radix floating menus for collapsed branches.
pub fn sidebar_template(look: &Look) -> Arc<dyn SidebarPanelTemplate> {
    Arc::new(ThemedSidebarPanelTemplate::new_with_floating_menu_theme(
        sidebar_theme(look),
        Arc::new(RailMenuTheme { look: look.clone() }),
    ))
}
/// Themed SDK sidebar builder. Configure structure, selection, presentation, and callbacks afterward.
pub fn sidebar_builder(look: &Look, id: impl Into<SharedString>) -> SidebarControlBuilder {
    SidebarControlBuilder::new(id)
        .stylesheet(&look.sized_stylesheet(look.size()))
        .motion_defaults(look.motion_defaults().sidebar)
        .panel_template(sidebar_template(look))
        .scrollbar_template(crate::scrollbar_template(look))
}
