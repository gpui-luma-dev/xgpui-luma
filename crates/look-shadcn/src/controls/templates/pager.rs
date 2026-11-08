use std::sync::Arc;

use xgpui_luma::controls::button::{ButtonTemplate, DefaultButtonTemplate};
use xgpui_luma::controls::button_family::{ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, ButtonFamilyTheme};
use xgpui_luma::controls::pager::{PagerLook, PagerTemplate, PagerTheme, ThemedPagerTemplate};
use xgpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::button::{ShadcnButtonStyle, button_palette};
use crate::controls::pager::{pager_button_look, pager_look};
use crate::look::ShadcnLook;
use crate::look_context::LookContext;

struct ShadcnPagerTheme {
    theme: ShadcnLook,
}

struct ShadcnPagerButtonTheme {
    theme: ShadcnLook,
    pager_look: PagerLook,
}

impl ButtonFamilyTheme for ShadcnPagerButtonTheme {
    fn resolve(&self, role: ButtonFamilyRole, size: ControlSize, state: InteractionState) -> ButtonFamilyPalette {
        let tokens = self.theme.mode_tokens();
        let stylesheet = self.theme.stylesheet();
        let ctx = LookContext::new(tokens.as_ref(), self.theme.mode(), state);
        button_palette(&ctx, stylesheet.as_ref(), ShadcnButtonStyle::Outline, role, size)
    }

    fn resolve_look(
        &self,
        role: ButtonFamilyRole,
        _size: ControlSize,
        state: InteractionState,
        _scale: &xgpui_luma::theme::StandardBoxScale,
        _pill_radius: f32,
    ) -> Option<ButtonFamilyLook> {
        Some(pager_button_look(&self.theme, &self.pager_look, role, state))
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

impl PagerTheme for ShadcnPagerTheme {
    fn resolve(
        &self,
        enabled: bool,
        style: xgpui_luma::controls::pager::PagerStyle,
    ) -> xgpui_luma::controls::pager::PagerLook {
        pager_look(&self.theme, enabled, style)
    }

    fn button_template(&self, pager_look: &PagerLook) -> Arc<dyn ButtonTemplate<()>> {
        Arc::new(DefaultButtonTemplate::new(Arc::new(ShadcnPagerButtonTheme {
            theme: self.theme.clone(),
            pager_look: pager_look.clone(),
        })))
    }
}

pub fn pager_template(theme: ShadcnLook) -> Arc<dyn PagerTemplate> {
    Arc::new(ThemedPagerTemplate::new(pager_theme(theme)))
}

pub fn pager_theme(theme: ShadcnLook) -> Arc<dyn PagerTheme> {
    Arc::new(ShadcnPagerTheme { theme })
}
