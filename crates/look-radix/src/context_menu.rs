//! Radix context menus use the same panel and item treatments as popup menus.
use std::sync::Arc;
use gpui::FontWeight;
use xgpui_luma::controls::context_menu::{
    ContextMenuLook, ContextMenuTemplate, ContextMenuTheme, ThemedContextMenuTemplate,
};
use xgpui_luma::theme::{InteractionState, LumaTextStyle, MetricTokens};
use crate::{Look, SemanticRole, Tone};

/// Solid or Soft highlighted items, shared with Radix popup menus.
pub type ContextMenuVariant = crate::PopupMenuVariant;
struct ContextMenuThemeAdapter {
    look: Look,
    variant: ContextMenuVariant,
    tone: Tone,
}
impl ContextMenuTheme for ContextMenuThemeAdapter {
    fn resolve(&self, state: InteractionState) -> ContextMenuLook {
        let metrics = self.look.metrics();
        let baseline =
            xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default().context_menu.authored_geometry("");
        let recipe = self.look.common_stylesheet().context_menu.authored_geometry("").with_fallback(&baseline);
        let geometry = self.look.common_stylesheet().context_menu.resolve_geometry(
            "",
            xgpui_luma::theme::stylesheet::ContextMenuGeometry {
                min_width: xgpui_luma::controls::context_menu::default_context_menu_theme()
                    .resolve(state)
                    .target_min_width,
                padding_x: metrics.control.md.padding_x,
                padding_y: metrics.control.md.padding_y,
                font_size: xgpui_luma::theme::ThemeTokens::default().typography.text.label.size,
                line_height: xgpui_luma::theme::ThemeTokens::default().typography.text.label.line_height,
                ..Default::default()
            },
        );
        ContextMenuLook {
            border_width: recipe.border_width.unwrap_or_default(),
            disabled_opacity: recipe.disabled_opacity.unwrap_or_default(),
            window_margin: recipe.window_margin.unwrap_or_default(),
            target_background: self.look.resolve_role(SemanticRole::Surface).hsla(),
            target_foreground: self
                .look
                .resolve_role(if state.disabled {
                    SemanticRole::MutedForeground
                } else {
                    SemanticRole::Foreground
                })
                .hsla(),
            target_border: self
                .look
                .resolve_role(if state.focused && !state.disabled {
                    SemanticRole::Focus
                } else {
                    SemanticRole::Border
                })
                .hsla(),
            target_typography: LumaTextStyle {
                size: geometry.font_size,
                line_height: geometry.line_height,
                weight: FontWeight::NORMAL,
            },
            target_radius: metrics.radius.md,
            target_padding_x: geometry.padding_x,
            target_padding_y: geometry.padding_y,
            target_min_width: geometry.min_width,
            floating_menu: crate::popup_menu::floating_menu_look(&self.look, self.variant, self.tone),
        }
    }
    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
}
pub fn context_menu_theme(look: &Look, variant: ContextMenuVariant, tone: Tone) -> Arc<dyn ContextMenuTheme> {
    Arc::new(ContextMenuThemeAdapter { look: look.clone(), variant, tone })
}
pub fn context_menu_template(look: &Look, variant: ContextMenuVariant, tone: Tone) -> Arc<dyn ContextMenuTemplate> {
    Arc::new(ThemedContextMenuTemplate::new(context_menu_theme(look, variant, tone)))
}
#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::ThemeMode;
    #[test]
    fn menu_matches_popup_styling_in_both_modes_and_tones() {
        let look = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for variant in [ContextMenuVariant::Solid, ContextMenuVariant::Soft] {
                for tone in [Tone::Accent, Tone::Gray] {
                    let actual =
                        context_menu_theme(&look, variant, tone).resolve(InteractionState::default()).floating_menu;
                    let expected = crate::popup_menu::floating_menu_look(&look, variant, tone);
                    assert_eq!(actual.background, expected.background);
                    assert_eq!(actual.border, expected.border);
                    assert_eq!(actual.item_hover_background, expected.item_hover_background);
                    assert_eq!(actual.item_hover_foreground, expected.item_hover_foreground);
                }
            }
        }
    }
    #[test]
    fn context_chrome_uses_selected_shared_stylesheet() {
        let look = Look::built_in();
        let sheet = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[
            "[common.context_menu.geometry]\nborder_width=0\ndisabled_opacity=0.2\nwindow_margin=17",
        ])
        .unwrap();
        look.set_common_stylesheet(sheet);
        let resolved =
            context_menu_theme(&look, ContextMenuVariant::Solid, Tone::Accent).resolve(InteractionState::default());
        assert_eq!((resolved.border_width, resolved.disabled_opacity, resolved.window_margin), (0.0, 0.2, 17.0));
    }
    #[test]
    fn focus_and_disabled_target_follow_radix_roles() {
        let look = Look::built_in();
        let theme = context_menu_theme(&look, ContextMenuVariant::Solid, Tone::Accent);
        let focused = theme.resolve(InteractionState { focused: true, ..Default::default() });
        assert_eq!(focused.target_border, look.resolve_role(SemanticRole::Focus).hsla());
        let disabled = theme.resolve(InteractionState { focused: true, disabled: true, ..Default::default() });
        assert_eq!(disabled.target_border, look.resolve_role(SemanticRole::Border).hsla());
        assert_eq!(disabled.target_foreground, look.resolve_role(SemanticRole::MutedForeground).hsla());
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_floating_menu_stylesheet() -> xgpui_luma::theme::stylesheet::FloatingMenuStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    FloatingMenuStylesheet {
        geometry: FloatingMenuGeometryStylesheet {
            separator_inset: Some(8.0),
            font_size: Some(14.0),
            line_height: Some(20.0),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_context_menu_stylesheet() -> xgpui_luma::theme::stylesheet::ContextMenuStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ContextMenuStylesheet {
        geometry: ContextMenuGeometryStylesheet {
            min_width: Some(200.0),
            font_size: Some(14.0),
            line_height: Some(20.0),
            ..Default::default()
        },
        ..Default::default()
    }
}
