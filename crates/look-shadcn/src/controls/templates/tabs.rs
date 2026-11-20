use std::sync::Arc;

use xgpui_luma::controls::tabs::{StyledTabs, TabsTemplate, TabsTheme, ThemedTabsTemplate};
use xgpui_luma::theme::{InteractionState};

use crate::look::ShadcnLook;

/// The Shadcn tabs: list and tabs from `.lss` (`style/tabs.lss`); the drag preview from the look.
struct ShadcnTabsTheme {
    theme: ShadcnLook,
    styled: StyledTabs,
}

impl TabsTheme for ShadcnTabsTheme {
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

    fn resolve_list(&self, enabled: bool) -> xgpui_luma::controls::tabs::TabsListLook {
        self.styled.list(enabled)
    }

    fn resolve_item(&self, active: bool, state: InteractionState) -> xgpui_luma::controls::tabs::TabsItemLook {
        self.styled.item(active, state)
    }

    fn font_family(&self) -> gpui::SharedString {
        self.theme.mode_tokens().typography.font.sans.family.clone().into()
    }
}

pub fn tabs_template(theme: ShadcnLook) -> Arc<dyn TabsTemplate> {
    Arc::new(ThemedTabsTemplate::new(tabs_theme(theme.clone())))
}

pub fn tabs_theme(theme: ShadcnLook) -> Arc<dyn TabsTheme> {
    let size = theme.size();
    let style = theme.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "tabs",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || style.style_theme())),
    );
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(ShadcnTabsTheme { theme, styled: StyledTabs::new(binding) })
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
            let tabs = tabs_theme(look.clone()).resolve_drag();
            let sidebar = super::super::sidebar::sidebar_theme(look.clone()).resolve_drag();
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
