use std::sync::Arc;

use xgpui_luma::controls::tabs::{TabsTemplate, TabsTheme, ThemedTabsTemplate};
use xgpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::tabs::{tabs_item_look_with_stylesheet, tabs_list_look_with_stylesheet};
use crate::look::ShadcnLook;

struct ShadcnTabsTheme {
    theme: ShadcnLook,
}

impl TabsTheme for ShadcnTabsTheme {
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

    fn resolve_list(&self, enabled: bool, size: ControlSize) -> xgpui_luma::controls::tabs::TabsListLook {
        let tokens = self.theme.mode_tokens();
        tabs_list_look_with_stylesheet(tokens.as_ref(), &self.theme.stylesheet(), enabled, size)
    }

    fn resolve_item(
        &self,
        active: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> xgpui_luma::controls::tabs::TabsItemLook {
        let tokens = self.theme.mode_tokens();
        tabs_item_look_with_stylesheet(tokens.as_ref(), &self.theme.stylesheet(), active, state, size)
    }

    fn font_family(&self) -> gpui::SharedString {
        self.theme.mode_tokens().typography.font.sans.family.clone().into()
    }
}

pub fn tabs_template(theme: ShadcnLook) -> Arc<dyn TabsTemplate> {
    Arc::new(ThemedTabsTemplate::new(tabs_theme(theme.clone())))
}

pub fn tabs_theme(theme: ShadcnLook) -> Arc<dyn TabsTheme> {
    Arc::new(ShadcnTabsTheme { theme: theme.clone() })
}

#[cfg(test)]
mod drag_tests {
    use super::*;
    use xgpui_luma::theme::ThemeMode;
    #[test]
    fn navigation_drag_surfaces_follow_table_tokens_in_both_modes() {
        let look = crate::test_support::fallback_look();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let tabs = tabs_theme(look.clone()).resolve_drag(ControlSize::Md);
            let sidebar = super::super::sidebar::sidebar_theme(look.clone()).resolve_drag(ControlSize::Md);
            let tokens = look.mode_tokens();
            for drag in [tabs, sidebar] {
                assert_eq!(drag.background, look.token_color("popover").unwrap_or(tokens.palette.panel_background));
                assert_eq!(drag.foreground, look.token_color("popover-foreground").unwrap_or(tokens.palette.body_text));
                assert_eq!(drag.valid_marker, look.token_color("ring").unwrap_or(tokens.palette.focus_ring));
                assert_eq!(
                    drag.invalid_marker,
                    look.token_color("destructive").unwrap_or(tokens.palette.destructive_background)
                );
            }
        }
    }
}
