use std::sync::Arc;

use xgpui_luma::controls::accordion::{AccordionTemplate, AccordionTheme, ThemedAccordionTemplate};
use xgpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::accordion::{accordion_content_palette, accordion_trigger_palette};
use crate::look::ShadcnLook;

struct ShadcnAccordionTheme {
    theme: ShadcnLook,
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

    fn resolve_trigger(
        &self,
        state: InteractionState,
        size: ControlSize,
    ) -> xgpui_luma::controls::accordion::AccordionPalette {
        let tokens = self.theme.mode_tokens();
        accordion_trigger_palette(tokens.as_ref(), self.theme.mode(), state, size)
    }

    fn resolve_content(&self, expanded: bool) -> xgpui_luma::controls::accordion::AccordionContentPalette {
        let tokens = self.theme.mode_tokens();
        accordion_content_palette(tokens.as_ref(), self.theme.mode(), expanded)
    }

    fn resolve_scale(&self, size: ControlSize, scale_factor: f32) -> xgpui_luma::controls::accordion::AccordionScale {
        crate::tables::metrics::resolve_accordion_metrics(
            &self.theme.mode_tokens(),
            self.theme.mode(),
            size,
            scale_factor,
        )
        .scale()
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn accordion_template(theme: ShadcnLook) -> Arc<dyn AccordionTemplate> {
    Arc::new(ThemedAccordionTemplate::new(accordion_theme(theme.clone())))
}

pub fn accordion_theme(theme: ShadcnLook) -> Arc<dyn AccordionTheme> {
    Arc::new(ShadcnAccordionTheme { theme: theme.clone() })
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
            let drag = theme.resolve_drag(ControlSize::Md);
            let expected = super::super::tabs::tabs_theme(look.clone()).resolve_drag(ControlSize::Md);
            assert_eq!(drag.background, expected.background);
            assert_eq!(drag.foreground, expected.foreground);
            assert_eq!(drag.valid_marker, expected.valid_marker);
            assert_eq!(drag.invalid_marker, expected.invalid_marker);
        }
    }
}
