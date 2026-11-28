use std::sync::Arc;

use xgpui_luma::controls::tabs::{StyledTabs, TabsTemplate, TabsTheme, ThemedTabsTemplate};

use crate::look::ShadcnLook;

/// The Shadcn tabs: list and tabs from `.lss` (`style/tabs.lss`); the drag preview from the look.
struct ShadcnTabsTheme {
    theme: ShadcnLook,
    styled: StyledTabs,
}

impl TabsTheme for ShadcnTabsTheme {
    fn resolve_drag(&self) -> xgpui_luma::infra::drag_drop::DragDropLook {
        super::drag::drag_look(&self.theme)
    }

    fn style(&self) -> xgpui_luma::style::StyleBinding {
        self.styled.binding()
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
                crate::assert_close(
                    drag.background,
                    look.token_color("popover").unwrap_or(tokens.palette.panel_background),
                );
                crate::assert_close(
                    drag.foreground,
                    look.token_color("popover-foreground").unwrap_or(tokens.palette.body_text),
                );
                crate::assert_close(drag.valid_marker, look.token_color("ring").unwrap_or(tokens.palette.focus_ring));
                crate::assert_close(
                    drag.invalid_marker,
                    look.token_color("destructive").unwrap_or(tokens.palette.destructive_background),
                );
            }
        }
    }
}
