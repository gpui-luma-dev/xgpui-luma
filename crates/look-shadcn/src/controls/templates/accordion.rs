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

    fn drag_style(&self) -> Arc<xgpui_luma::style::lss::Resolved> {
        super::drag::drag_style(&self.theme)
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
    use xgpui_luma::style::PartRead;
    #[test]
    fn drag_palette_follows_live_look_in_both_modes() {
        use xgpui_luma::theme::ThemeMode;
        let look = ShadcnLook::built_in();
        let theme = accordion_theme(look.clone());
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let drag = theme.drag_style();
            let expected = super::super::tabs::tabs_theme(look.clone()).drag_style();
            for name in ["background", "foreground", "valid_marker", "invalid_marker"] {
                assert_eq!(drag.root().color(name), expected.root().color(name), "{name}");
            }
        }
    }
}
