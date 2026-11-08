//! Radix pagination using shared button styling and SDK pager geometry.
use std::sync::Arc;
use gpui::SharedString;
use xgpui_luma::controls::pager::*;
use crate::{ButtonVariant, Look, Paint, ScaleFamily, SemanticRole, Tone};
struct PagerThemeAdapter {
    look: Look,
}
impl PagerTheme for PagerThemeAdapter {
    fn resolve(&self, enabled: bool, style: PagerStyle) -> PagerLook {
        let mut resolved =
            DefaultPagerTheme::new(crate::navigation::geometry_tokens(&self.look)).resolve(enabled, style);
        let gray = |step| self.look.resolve_step(ScaleFamily::Gray, step).hsla();
        resolved.panel_background = self.look.resolve_role(SemanticRole::Surface).hsla();
        resolved.border = self.look.resolve_role(SemanticRole::Border).hsla();
        resolved.body_text = self
            .look
            .resolve_role(if enabled {
                SemanticRole::Foreground
            } else {
                SemanticRole::MutedForeground
            })
            .hsla();
        resolved.muted_text = self.look.resolve_role(SemanticRole::MutedForeground).hsla();
        resolved.selected_background = if enabled {
            Tone::Accent.step(&self.look, 4)
        } else {
            gray(3)
        };
        resolved.selected_foreground = if enabled {
            Tone::Accent.step(&self.look, 11)
        } else {
            gray(8)
        };
        resolved.hover_background = Tone::Accent.step(&self.look, 3);
        resolved.shadow.clear();
        resolved
    }
    fn button_template(&self, _: &PagerLook) -> Arc<dyn xgpui_luma::controls::button::ButtonTemplate<()>> {
        crate::button_template(&self.look, ButtonVariant::Ghost, Paint::accent())
    }
}
/// Live Radix pagination chrome, including page-size popup colors.
pub fn pager_theme(look: &Look) -> Arc<dyn PagerTheme> {
    Arc::new(PagerThemeAdapter { look: look.clone() })
}
/// SDK pager template using Radix button and popup chrome.
pub fn pager_template(look: &Look) -> Arc<dyn PagerTemplate> {
    Arc::new(ThemedPagerTemplate::new(pager_theme(look)))
}
/// Themed SDK builder retaining pagination, formatter, callback, and template APIs.
pub fn pager_builder(look: &Look, id: impl Into<SharedString>) -> PagerBuilder {
    PagerBuilder::new(id).stylesheet(&look.common_stylesheet()).theme(pager_theme(look))
}
