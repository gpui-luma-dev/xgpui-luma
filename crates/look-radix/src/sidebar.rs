//! Radix sidebar row, section, rail-menu, and scrollbar adapters.
use std::sync::Arc;
use gpui::SharedString;
use xgpui_luma::controls::sidebar::*;
use xgpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, MetricTokens};
use crate::{Look, PopupMenuVariant, ScaleFamily, SemanticRole, Tone};
struct SidebarThemeAdapter {
    look: Look,
}
impl SidebarThemeAdapter {
    fn item(&self, selected: bool, state: InteractionState, size: ControlSize) -> SidebarItemLook {
        let mut item = DefaultSidebarTheme::new(crate::navigation::geometry_tokens(&self.look))
            .resolve_item(selected, state, size);
        let accent = |step| Tone::Accent.step(&self.look, step);
        item.background = match (selected, state.layer()) {
            (_, InteractionLayer::Disabled) => None,
            (_, InteractionLayer::Pressed) => Some(accent(5)),
            (_, InteractionLayer::Hovered) => Some(accent(4)),
            (true, _) => Some(accent(3)),
            _ => None,
        };
        item.foreground = if state.disabled {
            self.look.resolve_role(SemanticRole::MutedForeground).hsla()
        } else if state.invalid {
            self.look.resolve_role(SemanticRole::Destructive).hsla()
        } else if selected {
            accent(11)
        } else {
            self.look.resolve_role(SemanticRole::Foreground).hsla()
        };
        item.icon_color = item.foreground;
        item.focus_border =
            (state.focused && !state.disabled).then(|| self.look.resolve_role(SemanticRole::Focus).hsla());
        item
    }
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
        self.look.common_stylesheet().sidebar.resolve_geometry("", Default::default())
    }
    fn foreground(&self) -> gpui::Hsla {
        self.look.resolve_role(SemanticRole::Foreground).hsla()
    }
    fn resolve_section(&self) -> SidebarSectionLook {
        let mut section = DefaultSidebarTheme::new(crate::navigation::geometry_tokens(&self.look)).resolve_section();
        section.label_color = self.look.resolve_step(ScaleFamily::Gray, 11).hsla();
        section
    }
    fn resolve_branch(&self, state: InteractionState, size: ControlSize) -> SidebarItemLook {
        self.item(false, state, size)
    }
    fn resolve_item(&self, selected: bool, state: InteractionState, size: ControlSize) -> SidebarItemLook {
        self.item(selected, state, size)
    }
    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
    fn resolve_drag(&self, size: ControlSize) -> xgpui_luma::infra::drag_drop::DragDropLook {
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
    Arc::new(SidebarThemeAdapter { look: look.clone() })
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
        .stylesheet(&look.common_stylesheet())
        .motion_defaults(look.motion_defaults().sidebar)
        .panel_template(sidebar_template(look))
        .scrollbar_template(crate::scrollbar_template(look))
}
