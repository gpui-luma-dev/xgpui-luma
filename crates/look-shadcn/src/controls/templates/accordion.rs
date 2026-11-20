use std::sync::Arc;

use xgpui_luma::controls::accordion::{AccordionTemplate, AccordionTheme, StyledAccordion, ThemedAccordionTemplate};
use xgpui_luma::theme::{InteractionState};

use crate::look::ShadcnLook;

/// The Shadcn accordion: trigger, content and layout from `.lss` (`style/accordion.lss`); the drag preview,
/// handle icon and tooltip from the look.
struct ShadcnAccordionTheme {
    theme: ShadcnLook,
    styled: StyledAccordion,
}

impl AccordionTheme for ShadcnAccordionTheme {
    fn drag_handle_icon(&self) -> xgpui_luma::infra::icon::IconSource {
        self.theme
            .mode_tokens()
            .stylesheet()
            .common
            .icons
            .resolve(xgpui_luma::theme::stylesheet::IconRole::DragHandle)
    }
    fn drag_tooltip_theme(&self) -> Arc<dyn xgpui_luma::controls::tooltip::TooltipTheme> {
        crate::tooltip_theme(&self.theme)
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

    fn resolve_trigger(&self, state: InteractionState) -> xgpui_luma::controls::accordion::AccordionPalette {
        self.styled.trigger(state)
    }

    fn resolve_content(&self, _expanded: bool) -> xgpui_luma::controls::accordion::AccordionContentPalette {
        self.styled.content()
    }

    fn resolve_scale(&self, scale_factor: f32) -> xgpui_luma::controls::accordion::AccordionScale {
        self.styled.scale(scale_factor)
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics_for(self.theme.size())
    }
}

pub fn accordion_template(theme: ShadcnLook) -> Arc<dyn AccordionTemplate> {
    Arc::new(ThemedAccordionTemplate::new(accordion_theme(theme.clone())))
}

pub fn accordion_theme(theme: ShadcnLook) -> Arc<dyn AccordionTheme> {
    let size = theme.size();
    let style = theme.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "accordion",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || style.style_theme())),
    );
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(ShadcnAccordionTheme { theme, styled: StyledAccordion::new(binding) })
}

#[cfg(test)]
mod drag_tests {
    use super::*;
    #[test]
    fn drag_palette_follows_live_look_in_both_modes() {
        use xgpui_luma::theme::ThemeMode;
        let look = ShadcnLook::built_in();
        let theme = accordion_theme(look.clone());
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let drag = theme.resolve_drag();
            let expected = super::super::tabs::tabs_theme(look.clone()).resolve_drag();
            assert_eq!(drag.background, expected.background);
            assert_eq!(drag.foreground, expected.foreground);
            assert_eq!(drag.valid_marker, expected.valid_marker);
            assert_eq!(drag.invalid_marker, expected.invalid_marker);
        }
    }
}
