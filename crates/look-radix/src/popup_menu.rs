//! Radix popup-menu theme adapter (Copy / Actions menus).

use std::sync::Arc;

use gpui::{BoxShadow, FontWeight, black, point, px};
use xgpui_luma::controls::floating_menu::FloatingMenuLook;
use xgpui_luma::controls::popup_menu::{
    PopupMenuPalette, PopupMenuTemplate, PopupMenuTheme, PopupMenuTriggerMetrics, PopupMenuTriggerStyle,
    ThemedPopupMenuTemplate, compose_popup_menu_look,
};
use xgpui_luma::theme::{InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale};

use crate::look::Look;
use crate::semantic::SemanticRole;
use crate::tone::Tone;

/// Radix Themes menu content variants, as carried by the SDK popup menu control.
///
/// The variant only governs the highlighted item: the panel itself is identical.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopupMenuVariant {
    /// Highlighted item takes the solid accent (`accent-9` on `accent-contrast`).
    #[default]
    Solid,
    /// Highlighted item takes a soft accent tint (`accent-4`), text unchanged.
    Soft,
}

struct PopupMenuThemeAdapter {
    look: Look,
    variant: PopupMenuVariant,
    tone: Tone,
}

impl PopupMenuTheme for PopupMenuThemeAdapter {
    fn resolve(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> PopupMenuPalette {
        let _ = metrics;
        let layer = state.layer();
        let tone = self.tone;
        let step = |step| tone.step(&self.look, step);
        let (mut background, mut foreground, mut border) = match trigger_style {
            PopupMenuTriggerStyle::Primary => (step(9), tone.contrast(&self.look), Some(step(9))),
            PopupMenuTriggerStyle::Secondary => (step(3), step(11), Some(step(3))),
            PopupMenuTriggerStyle::Outline => (
                self.look.resolve_role(SemanticRole::Background).hsla(),
                self.look.resolve_role(SemanticRole::Foreground).hsla(),
                Some(self.look.resolve_role(SemanticRole::Border).hsla()),
            ),
            PopupMenuTriggerStyle::Ghost => {
                (gpui::hsla(0.0, 0.0, 0.0, 0.0), self.look.resolve_role(SemanticRole::Foreground).hsla(), None)
            }
        };

        match layer {
            InteractionLayer::Disabled => {
                background = self.look.resolve_role(SemanticRole::Surface).hsla();
                foreground = self.look.resolve_role(SemanticRole::MutedForeground).hsla();
                border = Some(self.look.resolve_role(SemanticRole::Border).hsla());
            }
            InteractionLayer::Hovered | InteractionLayer::Pressed => {
                if matches!(trigger_style, PopupMenuTriggerStyle::Ghost | PopupMenuTriggerStyle::Outline) {
                    background = step(3);
                } else if matches!(trigger_style, PopupMenuTriggerStyle::Primary) {
                    background = step(10);
                } else if matches!(trigger_style, PopupMenuTriggerStyle::Secondary) {
                    background = step(4);
                }
            }
            InteractionLayer::Default => {}
        }

        if state.focused && !state.disabled {
            border = Some(step(8));
        }

        let geometry = self.look.common_stylesheet().popup_menu.resolve_geometry(
            "",
            xgpui_luma::theme::stylesheet::PopupMenuGeometry {
                font_size: xgpui_luma::theme::ThemeTokens::default().typography.text.label.size,
                line_height: xgpui_luma::theme::ThemeTokens::default().typography.text.label.line_height,
                ..Default::default()
            },
        );
        PopupMenuPalette {
            trigger_background: background,
            trigger_foreground: foreground,
            trigger_border: border,
            trigger_shadow: None,
            trigger_typography: LumaTextStyle {
                size: geometry.font_size,
                line_height: geometry.line_height,
                weight: FontWeight::MEDIUM,
            },
            floating_menu: floating_menu_look(&self.look, self.variant, self.tone),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn resolve_look(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
        scale_factor: f32,
        _cx: &mut gpui::App,
    ) -> xgpui_luma::controls::popup_menu::PopupMenuLook {
        let mut scale = StandardBoxScale::compute(metrics.size, &self.metrics(), scale_factor);
        let geometry = self.look.common_stylesheet().popup_menu.resolve_geometry(
            "",
            xgpui_luma::theme::stylesheet::PopupMenuGeometry {
                height: scale.height,
                padding_x: scale.padding_x,
                padding_y: scale.padding_y,
                gap: scale.gap,
                ..Default::default()
            },
        );
        scale.height = geometry.height;
        scale.padding_x = geometry.padding_x;
        scale.padding_y = geometry.padding_y;
        scale.gap = geometry.gap;
        let mut look = compose_popup_menu_look(&self.resolve(trigger_style, metrics, state), &scale).with_stylesheet(
            &self.look.common_stylesheet().popup_menu,
            metrics.size.stylesheet_key(),
            scale_factor,
        );
        if let Some(radius) = metrics.trigger_radius_override {
            look.trigger_radius = radius;
        }
        look
    }
}

/// Resolve shared Radix menu panel, row, and separator styling for SDK composition.
///
/// Pass the result to the SDK floating-menu renderer. This is a resolved snapshot:
/// resolve again during render after changing the supplied Look, and notify the owner.
/// Explicit and forked Looks are used directly; no ambient Look is consulted.
/// Uses the existing Radix menu size (`"2"`) and its shared stylesheet overrides.
pub fn floating_menu_look(look: &Look, variant: PopupMenuVariant, tone: Tone) -> FloatingMenuLook {
    floating_menu_look_for(look, variant, tone, xgpui_luma::theme::ControlSize::Md)
}

/// Size-aware shared menu chrome. Size keys are Radix `1`, `2`, and `3`.
/// Like [`floating_menu_look`], this resolves a snapshot from the supplied Look.
pub fn floating_menu_look_for(
    look: &Look,
    variant: PopupMenuVariant,
    tone: Tone,
    size: xgpui_luma::theme::ControlSize,
) -> FloatingMenuLook {
    let metrics = look.metrics();
    let key = crate::look::sdk_size_key(size);
    let control = metrics.for_size(size);
    let (item_hover_background, item_hover_foreground) = match variant {
        PopupMenuVariant::Solid => (tone.step(look, 9), tone.contrast(look)),
        PopupMenuVariant::Soft => (tone.step(look, 4), look.resolve_role(SemanticRole::Foreground).hsla()),
    };

    let base_recipe =
        xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default().floating_menu.authored_geometry(key);
    let recipe = look.common_stylesheet().floating_menu.authored_geometry(key).with_fallback(&base_recipe);
    let mut resolved = FloatingMenuLook {
        window_margin: look
            .common_stylesheet()
            .with_sdk_defaults()
            .popover
            .authored_geometry("panel")
            .window_margin
            .unwrap_or_default(),
        border_width: recipe.border_width.unwrap_or(metrics.border_width.default),
        positive: xgpui_luma::color::gpui_bridge::to_hsla(
            look.common_stylesheet().tokens.source(look.mode()).palette.data.positive,
            xgpui_luma::color::GamutMapping::CssLocalMinde,
        )
        .unwrap_or_else(|_| xgpui_luma::theme::ThemeTokens::default().palette.data.positive),
        background: look.resolve_role(SemanticRole::Surface).hsla(),
        foreground: look.resolve_role(SemanticRole::Foreground).hsla(),
        border: look.resolve_role(SemanticRole::Border).hsla(),
        shadow: vec![BoxShadow {
            offset: point(px(0.0), px(10.0)),
            blur_radius: px(28.0),
            spread_radius: px(-12.0),
            color: gpui::Hsla { a: 0.28, ..black() },
            inset: false,
        }],
        radius: metrics.radius.lg,
        padding: metrics.spacing.s2,
        min_width: recipe.min_width.unwrap_or_default(),
        item_disabled_foreground: look.resolve_role(SemanticRole::MutedForeground).hsla(),
        item_hover_background,
        item_hover_foreground,
        item_typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
        item_height: control.height * recipe.item_height_scale.unwrap_or_default(),
        item_padding_x: control.padding_x * recipe.item_padding_scale.unwrap_or_default(),
        item_gap: metrics.spacing.s2,
        item_icon_size: control.icon_size,
        item_radius: metrics.radius.sm,
        disabled_opacity: recipe.disabled_opacity.unwrap_or_default(),
        separator_color: look.resolve_role(SemanticRole::Border).hsla(),
        separator_thickness: recipe.separator_thickness.unwrap_or_default(),
        separator_spacing: metrics.spacing.s2,
        separator_inset: metrics.spacing.s3,
        submenu_offset_x: metrics.spacing.s1,
    };
    let geometry = look.common_stylesheet().floating_menu.resolve_geometry(
        key,
        xgpui_luma::theme::stylesheet::FloatingMenuGeometry {
            border_width: resolved.border_width,
            padding: resolved.padding,
            min_width: resolved.min_width,
            item_height: resolved.item_height,
            item_padding_x: resolved.item_padding_x,
            item_gap: resolved.item_gap,
            item_icon_size: resolved.item_icon_size,
            font_size: resolved.item_typography.size,
            line_height: resolved.item_typography.line_height,
            submenu_offset_x: resolved.submenu_offset_x,
            separator_thickness: resolved.separator_thickness,
            separator_spacing: resolved.separator_spacing,
            separator_inset: resolved.separator_inset,
            disabled_opacity: resolved.disabled_opacity,
            ..Default::default()
        },
    );
    resolved.border_width = geometry.border_width;
    resolved.padding = geometry.padding;
    resolved.min_width = geometry.min_width;
    resolved.item_height = geometry.item_height;
    resolved.item_padding_x = geometry.item_padding_x;
    resolved.item_gap = geometry.item_gap;
    resolved.item_icon_size = geometry.item_icon_size;
    resolved.item_typography.size = geometry.font_size;
    resolved.item_typography.line_height = geometry.line_height;
    resolved.submenu_offset_x = geometry.submenu_offset_x;
    resolved.separator_thickness = geometry.separator_thickness;
    resolved.separator_spacing = geometry.separator_spacing;
    resolved.separator_inset = geometry.separator_inset;
    resolved.disabled_opacity = geometry.disabled_opacity;

    resolved
}

pub fn popup_menu_theme(look: &Look, variant: PopupMenuVariant, tone: Tone) -> Arc<dyn PopupMenuTheme> {
    Arc::new(PopupMenuThemeAdapter { look: look.clone(), variant, tone })
}

pub fn popup_menu_template(look: &Look, variant: PopupMenuVariant, tone: Tone) -> Arc<dyn PopupMenuTemplate> {
    Arc::new(ThemedPopupMenuTemplate::new(popup_menu_theme(look, variant, tone)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trigger_palette(look: &Look, tone: Tone) -> PopupMenuPalette {
        popup_menu_theme(look, PopupMenuVariant::default(), tone).resolve(
            PopupMenuTriggerStyle::Primary,
            PopupMenuTriggerMetrics::default(),
            InteractionState::default(),
        )
    }

    #[test]
    fn primary_trigger_uses_accent() {
        let look = Look::built_in();
        let palette = trigger_palette(&look, Tone::Accent);

        assert_eq!(palette.trigger_background.l, look.resolve_role(SemanticRole::Primary).hsla().l);
    }

    #[test]
    fn gray_tone_locks_the_trigger_to_the_neutral_scale() {
        let look = Look::built_in();
        let gray = trigger_palette(&look, Tone::Gray);

        assert_eq!(gray.trigger_background, Tone::Gray.step(&look, 9));
        assert_ne!(gray.trigger_background, trigger_palette(&look, Tone::Accent).trigger_background);
    }

    #[test]
    fn solid_variant_highlights_items_with_the_solid_accent() {
        let look = Look::built_in();
        let menu = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Accent);

        assert_eq!(menu.item_hover_background, look.resolve_role(SemanticRole::Primary).hsla());
        assert_eq!(menu.item_hover_foreground, look.resolve_role(SemanticRole::PrimaryForeground).hsla());
    }

    #[test]
    fn soft_variant_tints_items_and_keeps_the_panel_text() {
        let look = Look::built_in();
        let solid = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Accent);
        let soft = floating_menu_look(&look, PopupMenuVariant::Soft, Tone::Accent);

        assert_eq!(soft.item_hover_foreground, look.resolve_role(SemanticRole::Foreground).hsla());
        assert!(soft.item_hover_background.l > solid.item_hover_background.l);
        assert_eq!(soft.background, solid.background);
    }

    #[test]
    fn gray_tone_keeps_the_panel_and_only_swaps_the_highlight() {
        let look = Look::built_in();
        let accent = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Accent);
        let gray = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Gray);

        assert_eq!(gray.item_hover_background, Tone::Gray.step(&look, 9));
        assert_ne!(gray.item_hover_background, accent.item_hover_background);
        assert_eq!(gray.background, accent.background);
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_popup_menu_stylesheet() -> xgpui_luma::theme::stylesheet::PopupMenuStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    PopupMenuStylesheet {
        geometry: PopupMenuGeometryStylesheet { font_size: Some(14.0), line_height: Some(20.0), ..Default::default() },
        ..Default::default()
    }
}
