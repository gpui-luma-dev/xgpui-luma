use xgpui_luma::controls::textfield::TextFieldEventExt;
use xgpui_luma::controls::slider::SliderEventExt;
use std::collections::HashSet;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, TextRun, Window, div, font,
    prelude::*, px,
};
use xgpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionItem, AccordionTrigger};
use xgpui_luma::controls::slider::{Slider, SliderEvent};
use xgpui_luma::controls::textfield::{TextField, TextFieldEvent};
use xgpui_luma::{GridTrack, grid_layout, hstack, vstack};
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};
use xgpui_luma_look_shadcn as shadcn;

use super::colors::sync_token_field_template;
use super::super::model::{METRIC_FIELD_WIDTH, OTHER_CATEGORIES, SHADOW_COLOR_FIELD_WIDTH, SHADOW_SECTION_GAP};
use super::super::parsing::{
    effective_radius_rem, effective_spacing_rem, format_metric_rem, format_palette_hsl_multiplier,
    format_palette_hue_deg, format_shadow_color_input, format_shadow_number, parse_metric_rem,
    parse_palette_hsl_number, parse_shadow_color_input,
};
use crate::studio::app::LumaStudioApp;
use crate::studio::controls::color_picker::{ColorPickerEvent, ColorPickerPopover};
use crate::studio::hs_mixer::{ThemePaletteHsOverride, clamp_palette_temperature_amount, clamp_palette_vividness_amount};
use crate::studio::overrides::{
    PALETTE_HUE_DEG_MAX, PALETTE_HUE_DEG_MIN, PALETTE_LIGHTNESS_MULTIPLIER_MAX, PALETTE_LIGHTNESS_MULTIPLIER_MIN,
    PALETTE_SATURATION_MULTIPLIER_MAX, PALETTE_SATURATION_MULTIPLIER_MIN, RADIUS_REM_MAX, RADIUS_REM_MIN,
    SHADOW_BLUR_MAX, SHADOW_BLUR_MIN, SHADOW_OFFSET_X_MAX, SHADOW_OFFSET_X_MIN, SHADOW_OFFSET_Y_MAX,
    SHADOW_OFFSET_Y_MIN, SHADOW_OPACITY_MAX, SHADOW_OPACITY_MIN, SHADOW_SPREAD_MAX, SHADOW_SPREAD_MIN, SPACING_REM_MAX,
    SPACING_REM_MIN, StudioOverrides, ThemePaletteHslOverride, ThemeShadowOverride, clamp_palette_hue_deg,
    clamp_palette_lightness_multiplier, clamp_palette_saturation_multiplier, clamp_radius_rem, clamp_shadow_blur,
    clamp_shadow_offset_x, clamp_shadow_offset_y, clamp_shadow_opacity, clamp_shadow_spread, clamp_spacing_rem,
    resolved_shadow_override,
};
use super::{category_item_id, expanded_category_ids, spawn_compact_textfield, spawn_slider};

const HSL_GRID_LABELS: [&str; 3] = ["Hue", "Saturation", "Lightness"];
const HS_ADJUSTMENT_LEFT_LABELS: [&str; 2] = ["Neutral", "Warmer"];
const HS_ADJUSTMENT_RIGHT_LABELS: [&str; 2] = ["Vivid", "Cooler"];
const SHADOW_GRID_LABELS: [&str; 5] = ["Opacity", "Blur", "Spread", "Offset X", "Offset Y"];
const SHADOW_GRID_UNIT_WIDTH: f32 = 24.0;
const SLIDER_FIELD_GRID_GAP_X: f32 = 10.0;
const PANEL_SLIDER_STEP: f32 = 0.01;

pub struct OtherPanel {
    look: Arc<ShadcnLook>,
    palette_hsl: ThemePaletteHslOverride,
    palette_hs: ThemePaletteHsOverride,
    shadow_override: ThemeShadowOverride,
    palette_hue_field: TextField,
    palette_saturation_field: TextField,
    palette_lightness_field: TextField,
    palette_hue_slider: Slider,
    palette_saturation_slider: Slider,
    palette_lightness_slider: Slider,
    palette_vividness_slider: Slider,
    palette_temperature_slider: Slider,
    radius_field: TextField,
    spacing_field: TextField,
    radius_slider: Slider,
    spacing_slider: Slider,
    shadow_color_field: TextField,
    shadow_color_picker: Entity<ColorPickerPopover>,
    shadow_opacity_field: TextField,
    shadow_blur_field: TextField,
    shadow_spread_field: TextField,
    shadow_offset_x_field: TextField,
    shadow_offset_y_field: TextField,
    shadow_opacity_slider: Slider,
    shadow_blur_slider: Slider,
    shadow_spread_slider: Slider,
    shadow_offset_x_slider: Slider,
    shadow_offset_y_slider: Slider,
    other_accordion: Option<Entity<AccordionControl>>,
}

impl OtherPanel {
    pub fn new(look: Arc<ShadcnLook>, overrides: &StudioOverrides, cx: &mut Context<Self>) -> Self {
        let palette_hsl = overrides.palette_hsl(look.mode()).clone();
        let palette_hs = overrides.palette_hs(look.mode()).clone();
        let radius_rem = effective_radius_rem(&look, overrides);
        let spacing_rem = effective_spacing_rem(&look, overrides);
        let shadow_override = resolved_shadow_override(&look, overrides);

        let mut this = Self {
            palette_hue_field: build_number_field(
                &look,
                "palette-hue",
                format_palette_hue_deg(palette_hsl.hue_deg),
                cx,
            ),
            palette_saturation_field: build_number_field(
                &look,
                "palette-saturation",
                format_palette_hsl_multiplier(palette_hsl.saturation_multiplier),
                cx,
            ),
            palette_lightness_field: build_number_field(
                &look,
                "palette-lightness",
                format_palette_hsl_multiplier(palette_hsl.lightness_multiplier),
                cx,
            ),
            palette_hue_slider: build_slider(
                &look,
                "palette-hue",
                PALETTE_HUE_DEG_MIN,
                PALETTE_HUE_DEG_MAX,
                1.0,
                palette_hsl.hue_deg,
                cx,
            ),
            palette_saturation_slider: build_slider(
                &look,
                "palette-saturation",
                PALETTE_SATURATION_MULTIPLIER_MIN,
                PALETTE_SATURATION_MULTIPLIER_MAX,
                PANEL_SLIDER_STEP,
                palette_hsl.saturation_multiplier,
                cx,
            ),
            palette_lightness_slider: build_slider(
                &look,
                "palette-lightness",
                PALETTE_LIGHTNESS_MULTIPLIER_MIN,
                PALETTE_LIGHTNESS_MULTIPLIER_MAX,
                PANEL_SLIDER_STEP,
                palette_hsl.lightness_multiplier,
                cx,
            ),
            palette_vividness_slider: build_slider(
                &look,
                "palette-vividness",
                crate::studio::hs_mixer::PALETTE_VIVIDNESS_AMOUNT_MIN,
                crate::studio::hs_mixer::PALETTE_VIVIDNESS_AMOUNT_MAX,
                PANEL_SLIDER_STEP,
                palette_hs.vividness_amount,
                cx,
            ),
            palette_temperature_slider: build_slider(
                &look,
                "palette-temperature",
                crate::studio::hs_mixer::PALETTE_TEMPERATURE_AMOUNT_MIN,
                crate::studio::hs_mixer::PALETTE_TEMPERATURE_AMOUNT_MAX,
                PANEL_SLIDER_STEP,
                palette_hs.temperature_amount,
                cx,
            ),
            radius_field: build_number_field(&look, "radius", format_metric_rem(radius_rem), cx),
            spacing_field: build_number_field(&look, "spacing", format_metric_rem(spacing_rem), cx),
            radius_slider: build_slider(
                &look,
                "radius",
                RADIUS_REM_MIN,
                RADIUS_REM_MAX,
                PANEL_SLIDER_STEP,
                radius_rem,
                cx,
            ),
            spacing_slider: build_slider(
                &look,
                "spacing",
                SPACING_REM_MIN,
                SPACING_REM_MAX,
                PANEL_SLIDER_STEP,
                spacing_rem,
                cx,
            ),
            shadow_color_field: build_number_field(
                &look,
                "shadow-color",
                format_shadow_color_input(shadow_override.color),
                cx,
            ),
            shadow_color_picker: cx
                .new(|cx| ColorPickerPopover::new(look.clone(), shadow_override.color, "shadow-color", cx)),
            shadow_opacity_field: build_number_field(
                &look,
                "shadow-opacity",
                format_shadow_number(shadow_override.opacity()),
                cx,
            ),
            shadow_blur_field: build_number_field(
                &look,
                "shadow-blur",
                format_shadow_number(shadow_override.blur_px),
                cx,
            ),
            shadow_spread_field: build_number_field(
                &look,
                "shadow-spread",
                format_shadow_number(shadow_override.spread_px),
                cx,
            ),
            shadow_offset_x_field: build_number_field(
                &look,
                "shadow-offset-x",
                format_shadow_number(shadow_override.offset_x_px),
                cx,
            ),
            shadow_offset_y_field: build_number_field(
                &look,
                "shadow-offset-y",
                format_shadow_number(shadow_override.offset_y_px),
                cx,
            ),
            shadow_opacity_slider: build_slider(
                &look,
                "shadow-opacity",
                SHADOW_OPACITY_MIN,
                SHADOW_OPACITY_MAX,
                PANEL_SLIDER_STEP,
                shadow_override.opacity(),
                cx,
            ),
            shadow_blur_slider: build_slider(
                &look,
                "shadow-blur",
                SHADOW_BLUR_MIN,
                SHADOW_BLUR_MAX,
                PANEL_SLIDER_STEP,
                shadow_override.blur_px,
                cx,
            ),
            shadow_spread_slider: build_slider(
                &look,
                "shadow-spread",
                SHADOW_SPREAD_MIN,
                SHADOW_SPREAD_MAX,
                PANEL_SLIDER_STEP,
                shadow_override.spread_px,
                cx,
            ),
            shadow_offset_x_slider: build_slider(
                &look,
                "shadow-offset-x",
                SHADOW_OFFSET_X_MIN,
                SHADOW_OFFSET_X_MAX,
                PANEL_SLIDER_STEP,
                shadow_override.offset_x_px,
                cx,
            ),
            shadow_offset_y_slider: build_slider(
                &look,
                "shadow-offset-y",
                SHADOW_OFFSET_Y_MIN,
                SHADOW_OFFSET_Y_MAX,
                PANEL_SLIDER_STEP,
                shadow_override.offset_y_px,
                cx,
            ),
            look: look.clone(),
            palette_hsl,
            palette_hs,
            shadow_override,
            other_accordion: None,
        };
        this.other_accordion = Some(Self::build_other_accordion(cx.entity(), look, &HashSet::new(), cx));
        this
    }

    pub fn apply_theme_snapshot(&mut self, look: Arc<ShadcnLook>, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        let expanded = self.expanded_other_category_ids(cx);
        self.look = look;
        let theme = self.look.clone();
        self.sync_templates(&theme, cx);
        self.sync_values(theme.as_ref(), overrides, cx);
        self.other_accordion = Some(Self::build_other_accordion(cx.entity(), theme, &expanded, cx));
        cx.notify();
    }

    pub fn sync_global_overrides(&mut self, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        let theme = self.look.clone();
        self.sync_values(theme.as_ref(), overrides, cx);
        cx.notify();
    }

    pub fn wire_subscriptions(
        panel: &Entity<Self>,
        cx: &mut Context<LumaStudioApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        let palette_hue_field = panel.read(cx).palette_hue_field.clone();
        palette_hue_field.clone().on_event(cx, |app, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(hue_deg) = parse_palette_hsl_number(value, PALETTE_HUE_DEG_MIN, PALETTE_HUE_DEG_MAX)
                    .map(clamp_palette_hue_deg)
                else {
                    return;
                };
                app.set_palette_hue_deg(hue_deg, cx);
            }
        });

        let palette_saturation_field = panel.read(cx).palette_saturation_field.clone();
        palette_saturation_field.clone().on_event(cx, |app, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(multiplier) = parse_palette_hsl_number(
                    value,
                    PALETTE_SATURATION_MULTIPLIER_MIN,
                    PALETTE_SATURATION_MULTIPLIER_MAX,
                )
                .map(clamp_palette_saturation_multiplier) else {
                    return;
                };
                app.set_palette_saturation_multiplier(multiplier, cx);
            }
        });

        let palette_lightness_field = panel.read(cx).palette_lightness_field.clone();
        palette_lightness_field.clone().on_event(cx, |app, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(multiplier) =
                    parse_palette_hsl_number(value, PALETTE_LIGHTNESS_MULTIPLIER_MIN, PALETTE_LIGHTNESS_MULTIPLIER_MAX)
                        .map(clamp_palette_lightness_multiplier)
                else {
                    return;
                };
                app.set_palette_lightness_multiplier(multiplier, cx);
            }
        });

        let palette_hue_slider = panel.read(cx).palette_hue_slider.clone();
        palette_hue_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_palette_hue_deg(value, cx);
        });

        let palette_saturation_slider = panel.read(cx).palette_saturation_slider.clone();
        palette_saturation_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_palette_saturation_multiplier(value, cx);
        });

        let palette_lightness_slider = panel.read(cx).palette_lightness_slider.clone();
        palette_lightness_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_palette_lightness_multiplier(value, cx);
        });

        let palette_vividness_slider = panel.read(cx).palette_vividness_slider.clone();
        palette_vividness_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_palette_vividness_amount(clamp_palette_vividness_amount(value), cx);
        });

        let palette_temperature_slider = panel.read(cx).palette_temperature_slider.clone();
        palette_temperature_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_palette_temperature_amount(clamp_palette_temperature_amount(value), cx);
        });

        let radius_field = panel.read(cx).radius_field.clone();
        radius_field.clone().on_event(cx, |app, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(rem) = parse_metric_rem(value, RADIUS_REM_MIN, RADIUS_REM_MAX).map(clamp_radius_rem) else {
                    return;
                };
                app.set_radius_rem(rem, cx);
            }
        });

        let spacing_field = panel.read(cx).spacing_field.clone();
        spacing_field.clone().on_event(cx, |app, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(rem) = parse_metric_rem(value, SPACING_REM_MIN, SPACING_REM_MAX).map(clamp_spacing_rem) else {
                    return;
                };
                app.set_spacing_rem(rem, cx);
            }
        });

        let radius_slider = panel.read(cx).radius_slider.clone();
        radius_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_radius_rem(value, cx);
        });

        let spacing_slider = panel.read(cx).spacing_slider.clone();
        spacing_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_spacing_rem(value, cx);
        });

        let shadow_color_field = panel.read(cx).shadow_color_field.clone();
        shadow_color_field.clone().on_event(cx, |app, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(color) = parse_shadow_color_input(value) else {
                    return;
                };
                app.set_shadow_color(color, cx);
            }
        });

        let shadow_color_picker = panel.read(cx).shadow_color_picker.clone();
        subscriptions.push(cx.subscribe(&shadow_color_picker, |app, _, event: &ColorPickerEvent, cx| {
            let ColorPickerEvent::Change(color) = event;
            app.set_shadow_color(*color, cx);
        }));

        let shadow_opacity_field = panel.read(cx).shadow_opacity_field.clone();
        shadow_opacity_field.clone().on_event(cx, |app, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(opacity) =
                    parse_metric_rem(value, SHADOW_OPACITY_MIN, SHADOW_OPACITY_MAX).map(clamp_shadow_opacity)
                else {
                    return;
                };
                app.set_shadow_opacity(opacity, cx);
            }
        });

        let shadow_blur_field = panel.read(cx).shadow_blur_field.clone();
        shadow_blur_field.clone().on_event(cx, |app, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(blur) = parse_metric_rem(value, SHADOW_BLUR_MIN, SHADOW_BLUR_MAX).map(clamp_shadow_blur)
                else {
                    return;
                };
                app.set_shadow_blur(blur, cx);
            }
        });

        let shadow_spread_field = panel.read(cx).shadow_spread_field.clone();
        shadow_spread_field.clone().on_event(cx, |app, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(spread) =
                    parse_metric_rem(value, SHADOW_SPREAD_MIN, SHADOW_SPREAD_MAX).map(clamp_shadow_spread)
                else {
                    return;
                };
                app.set_shadow_spread(spread, cx);
            }
        });

        let shadow_offset_x_field = panel.read(cx).shadow_offset_x_field.clone();
        shadow_offset_x_field.clone().on_event(cx, |app, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(offset_x) =
                    parse_metric_rem(value, SHADOW_OFFSET_X_MIN, SHADOW_OFFSET_X_MAX).map(clamp_shadow_offset_x)
                else {
                    return;
                };
                app.set_shadow_offset_x(offset_x, cx);
            }
        });

        let shadow_offset_y_field = panel.read(cx).shadow_offset_y_field.clone();
        shadow_offset_y_field.clone().on_event(cx, |app, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(offset_y) =
                    parse_metric_rem(value, SHADOW_OFFSET_Y_MIN, SHADOW_OFFSET_Y_MAX).map(clamp_shadow_offset_y)
                else {
                    return;
                };
                app.set_shadow_offset_y(offset_y, cx);
            }
        });

        let shadow_opacity_slider = panel.read(cx).shadow_opacity_slider.clone();
        shadow_opacity_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_shadow_opacity(value, cx);
        });

        let shadow_blur_slider = panel.read(cx).shadow_blur_slider.clone();
        shadow_blur_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_shadow_blur(value, cx);
        });

        let shadow_spread_slider = panel.read(cx).shadow_spread_slider.clone();
        shadow_spread_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_shadow_spread(value, cx);
        });

        let shadow_offset_x_slider = panel.read(cx).shadow_offset_x_slider.clone();
        shadow_offset_x_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_shadow_offset_x(value, cx);
        });

        let shadow_offset_y_slider = panel.read(cx).shadow_offset_y_slider.clone();
        shadow_offset_y_slider.clone().on_event(cx, |app, event: &SliderEvent, cx| {
            let value = match event {
                SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                _ => return,
            };
            app.set_shadow_offset_y(value, cx);
        });
    }

    fn sync_templates(&self, theme: &Arc<ShadcnLook>, cx: &mut Context<Self>) {
        for field in [&self.palette_hue_field, &self.palette_saturation_field, &self.palette_lightness_field] {
            sync_token_field_template(theme, field, cx);
        }
        for field in [&self.radius_field, &self.spacing_field] {
            sync_token_field_template(theme, field, cx);
        }
        for field in [
            &self.shadow_color_field,
            &self.shadow_opacity_field,
            &self.shadow_blur_field,
            &self.shadow_spread_field,
            &self.shadow_offset_x_field,
            &self.shadow_offset_y_field,
        ] {
            sync_token_field_template(theme, field, cx);
        }
        for slider in [
            &self.palette_hue_slider,
            &self.palette_saturation_slider,
            &self.palette_lightness_slider,
            &self.palette_vividness_slider,
            &self.palette_temperature_slider,
            &self.radius_slider,
            &self.spacing_slider,
            &self.shadow_opacity_slider,
            &self.shadow_blur_slider,
            &self.shadow_spread_slider,
            &self.shadow_offset_x_slider,
            &self.shadow_offset_y_slider,
        ] {
            slider.update(cx, |slider, cx| {
                slider.set_template(theme.slider_template(), cx);
            });
        }
    }

    fn sync_values(&mut self, look: &ShadcnLook, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        let palette_hsl = overrides.palette_hsl(self.look.mode()).clone();
        self.palette_hsl = palette_hsl.clone();
        self.palette_hue_field
            .update(cx, |field, cx| field.set_value(format_palette_hue_deg(palette_hsl.hue_deg), cx));
        self.palette_saturation_field.update(cx, |field, cx| {
            field.set_value(format_palette_hsl_multiplier(palette_hsl.saturation_multiplier), cx)
        });
        self.palette_lightness_field.update(cx, |field, cx| {
            field.set_value(format_palette_hsl_multiplier(palette_hsl.lightness_multiplier), cx)
        });
        self.palette_hue_slider.update(cx, |slider, cx| slider.set_value(palette_hsl.hue_deg, cx));
        self.palette_saturation_slider
            .update(cx, |slider, cx| slider.set_value(palette_hsl.saturation_multiplier, cx));
        self.palette_lightness_slider
            .update(cx, |slider, cx| slider.set_value(palette_hsl.lightness_multiplier, cx));

        let palette_hs = overrides.palette_hs(self.look.mode()).clone();
        self.palette_hs = palette_hs.clone();
        self.palette_vividness_slider
            .update(cx, |slider, cx| slider.set_value(palette_hs.vividness_amount, cx));
        self.palette_temperature_slider
            .update(cx, |slider, cx| slider.set_value(palette_hs.temperature_amount, cx));

        let radius_rem = effective_radius_rem(look, overrides);
        let spacing_rem = effective_spacing_rem(look, overrides);
        self.radius_field.update(cx, |field, cx| field.set_value(format_metric_rem(radius_rem), cx));
        self.spacing_field.update(cx, |field, cx| field.set_value(format_metric_rem(spacing_rem), cx));
        self.radius_slider.update(cx, |slider, cx| slider.set_value(radius_rem, cx));
        self.spacing_slider.update(cx, |slider, cx| slider.set_value(spacing_rem, cx));

        let shadow = resolved_shadow_override(look, overrides);
        self.shadow_override = shadow.clone();
        self.shadow_color_field
            .update(cx, |field, cx| field.set_value(format_shadow_color_input(shadow.color), cx));
        self.shadow_color_picker.update(cx, |picker, cx| picker.set_color(shadow.color, cx));
        self.shadow_opacity_field
            .update(cx, |field, cx| field.set_value(format_metric_rem(shadow.opacity()), cx));
        self.shadow_blur_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.blur_px), cx));
        self.shadow_spread_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.spread_px), cx));
        self.shadow_offset_x_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.offset_x_px), cx));
        self.shadow_offset_y_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.offset_y_px), cx));
        self.shadow_opacity_slider.update(cx, |slider, cx| slider.set_value(shadow.opacity(), cx));
        self.shadow_blur_slider.update(cx, |slider, cx| slider.set_value(shadow.blur_px, cx));
        self.shadow_spread_slider.update(cx, |slider, cx| slider.set_value(shadow.spread_px, cx));
        self.shadow_offset_x_slider.update(cx, |slider, cx| slider.set_value(shadow.offset_x_px, cx));
        self.shadow_offset_y_slider.update(cx, |slider, cx| slider.set_value(shadow.offset_y_px, cx));
    }

    fn build_other_accordion(
        panel: Entity<Self>,
        look: Arc<ShadcnLook>,
        expanded_categories: &HashSet<String>,
        cx: &mut Context<Self>,
    ) -> Entity<AccordionControl> {
        let mut accordion_builder = shadcn::Accordion::new("luma-studio-other-accordion")
            .look(look.as_ref())
            .multiple()
            .item_dividers(false)
            .trigger_min_height(28.0)
            .trigger_padding_y(4.0)
            .content_padding_top(0.0)
            .content_padding_bottom(4.0)
            .template(look.accordion_template());

        for category in OTHER_CATEGORIES {
            let panel = panel.clone();
            let id = category_item_id("other", category);
            let expanded = if expanded_categories.is_empty() {
                true
            } else {
                expanded_categories.contains(&id)
            };
            accordion_builder = accordion_builder.item(
                AccordionItem::new(
                    id,
                    AccordionTrigger::new(*category),
                    AccordionContent::custom(move |window, cx| panel.read(cx).category_content(category, window)),
                )
                .expanded(expanded),
            );
        }

        accordion_builder.spawn(cx)
    }

    fn expanded_other_category_ids(&self, cx: &App) -> HashSet<String> {
        expanded_category_ids(
            self.other_accordion.as_ref().expect("other accordion initialized"),
            OTHER_CATEGORIES.iter().copied(),
            "other",
            cx,
        )
    }

    fn category_content(&self, category: &str, window: &mut Window) -> AnyElement {
        match category {
            "HSL ADJUSTMENTS" => self.hsl_adjustments_grid(window),
            "HS MIXER" => self.hs_adjustments_category_content(window),
            "RADIUS" => {
                metric_category_content(self, window, "Radius", self.radius_slider.clone(), self.radius_field.clone())
            }
            "SPACING" => metric_category_content(
                self,
                window,
                "Spacing",
                self.spacing_slider.clone(),
                self.spacing_field.clone(),
            ),
            "SHADOW" => self.shadow_category_content(window),
            _ => self.category_placeholder_content(category),
        }
    }

    fn category_placeholder_content(&self, category: &str) -> AnyElement {
        let chrome = self.look.chrome();
        div()
            .w_full()
            .pt(px(2.0))
            .pb(px(6.0))
            .text_xs()
            .text_color(chrome.muted_text)
            .child(format!("{category} controls coming soon."))
            .into_any_element()
    }

    fn hsl_adjustments_grid(&self, window: &mut Window) -> AnyElement {
        grid_layout! {
            rows: 3,
            columns: [
                GridTrack::Px(max_label_width(self, window, &HSL_GRID_LABELS)),
                GridTrack::Star(1.0),
                GridTrack::Px(METRIC_FIELD_WIDTH),
                GridTrack::Px(SHADOW_GRID_UNIT_WIDTH),
            ],
            gap_x: SLIDER_FIELD_GRID_GAP_X,
            gap_y: SHADOW_SECTION_GAP;
            [0, 0] => slider_field_grid_label(self, "Hue"),
            [0, 1] => self.palette_hue_slider.clone(),
            [0, 2] => self.palette_hue_field.clone(),
            [0, 3] => slider_field_grid_unit(self, "deg"),
            [1, 0] => slider_field_grid_label(self, "Saturation"),
            [1, 1] => self.palette_saturation_slider.clone(),
            [1, 2] => self.palette_saturation_field.clone(),
            [1, 3] => slider_field_grid_unit(self, "x"),
            [2, 0] => slider_field_grid_label(self, "Lightness"),
            [2, 1] => self.palette_lightness_slider.clone(),
            [2, 2] => self.palette_lightness_field.clone(),
            [2, 3] => slider_field_grid_unit(self, "x"),
        }
        .into_any_element()
    }

    fn hs_adjustments_category_content(&self, window: &mut Window) -> AnyElement {
        let left_label_width = max_label_width_for_size(self, window, &HS_ADJUSTMENT_LEFT_LABELS, ShadcnTextSize::Sm);
        let right_label_width = max_label_width_for_size(self, window, &HS_ADJUSTMENT_RIGHT_LABELS, ShadcnTextSize::Sm);

        div()
            .w_full()
            .min_w(px(0.0))
            .child(grid_layout! {
                rows: 2,
                columns: [
                    GridTrack::Px(left_label_width),
                    GridTrack::Star(1.0),
                    GridTrack::Px(right_label_width),
                ],
                gap_x: 16.0,
                gap_y: 0.0;
                [0, 0] => hs_adjustment_label(self, "Neutral", false),
                [0, 1] => div().w_full().min_w(px(0.0)).child(self.palette_vividness_slider.clone()),
                [0, 2] => hs_adjustment_label(self, "Vivid", true),
                [1, 0] => hs_adjustment_label(self, "Warmer", false),
                [1, 1] => div().w_full().min_w(px(0.0)).child(self.palette_temperature_slider.clone()),
                [1, 2] => hs_adjustment_label(self, "Cooler", true),
            })
            .into_any_element()
    }

    fn shadow_category_content(&self, window: &mut Window) -> AnyElement {
        vstack! {
            gap=SHADOW_SECTION_GAP;
            hstack! {
                gap=10 align=center;
                self.shadow_color_picker.clone(),
                div()
                    .w(px(SHADOW_COLOR_FIELD_WIDTH))
                    .max_w_full()
                    .child(self.shadow_color_field.clone()),
            },
            shadow_slider_grid(self, window),
        }
        .w_full()
        .into_any_element()
    }
}

impl Render for OtherPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(10.0))
            .child(div().w_full().child(self.other_accordion.as_ref().expect("other accordion initialized").clone()))
    }
}

fn build_number_field(
    look: &Arc<ShadcnLook>,
    id: &str,
    value: impl Into<SharedString>,
    cx: &mut Context<OtherPanel>,
) -> TextField {
    spawn_compact_textfield(look, id, value, cx)
}

fn build_slider(
    look: &Arc<ShadcnLook>,
    id: &str,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    cx: &mut Context<OtherPanel>,
) -> Slider {
    spawn_slider(look, id, min, max, step, value, cx)
}

fn metric_category_content(
    panel: &OtherPanel,
    window: &mut Window,
    label: &'static str,
    slider: Slider,
    field: TextField,
) -> AnyElement {
    div()
        .w_full()
        .min_w(px(0.0))
        .child(grid_layout! {
            rows: 1,
            columns: [
                GridTrack::Px(max_label_width_for_size(panel, window, &[label], ShadcnTextSize::Xs)),
                GridTrack::Star(1.0),
                GridTrack::Px(METRIC_FIELD_WIDTH),
                GridTrack::Px(SHADOW_GRID_UNIT_WIDTH),
            ],
            gap_x: SLIDER_FIELD_GRID_GAP_X,
            gap_y: 0.0;
            [0, 0] => slider_field_grid_label(panel, label),
            [0, 1] => div().w_full().min_w(px(0.0)).child(slider),
            [0, 2] => field,
            [0, 3] => slider_field_grid_unit(panel, "rem"),
        })
        .into_any_element()
}

fn hs_adjustment_label(panel: &OtherPanel, label: &str, right_aligned: bool) -> AnyElement {
    let chrome = panel.look.chrome();
    let label_style = panel.look.typography_scale(ShadcnTextSize::Sm);

    div()
        .w_full()
        .when(right_aligned, |this| this.text_right())
        .typography_style(label_style)
        .text_color(chrome.body_text)
        .child(label.to_string())
        .into_any_element()
}

fn shadow_slider_grid(panel: &OtherPanel, window: &mut Window) -> AnyElement {
    grid_layout! {
        rows: 5,
        columns: [
            GridTrack::Px(max_label_width(panel, window, &SHADOW_GRID_LABELS)),
            GridTrack::Star(1.0),
            GridTrack::Px(METRIC_FIELD_WIDTH),
            GridTrack::Px(SHADOW_GRID_UNIT_WIDTH),
        ],
        gap_x: SLIDER_FIELD_GRID_GAP_X,
        gap_y: SHADOW_SECTION_GAP;
        [0, 0] => slider_field_grid_label(panel, "Opacity"),
        [0, 1] => panel.shadow_opacity_slider.clone(),
        [0, 2] => panel.shadow_opacity_field.clone(),
        [0, 3] => slider_field_grid_unit(panel, ""),
        [1, 0] => slider_field_grid_label(panel, "Blur"),
        [1, 1] => panel.shadow_blur_slider.clone(),
        [1, 2] => panel.shadow_blur_field.clone(),
        [1, 3] => slider_field_grid_unit(panel, "px"),
        [2, 0] => slider_field_grid_label(panel, "Spread"),
        [2, 1] => panel.shadow_spread_slider.clone(),
        [2, 2] => panel.shadow_spread_field.clone(),
        [2, 3] => slider_field_grid_unit(panel, "px"),
        [3, 0] => slider_field_grid_label(panel, "Offset X"),
        [3, 1] => panel.shadow_offset_x_slider.clone(),
        [3, 2] => panel.shadow_offset_x_field.clone(),
        [3, 3] => slider_field_grid_unit(panel, "px"),
        [4, 0] => slider_field_grid_label(panel, "Offset Y"),
        [4, 1] => panel.shadow_offset_y_slider.clone(),
        [4, 2] => panel.shadow_offset_y_field.clone(),
        [4, 3] => slider_field_grid_unit(panel, "px"),
    }
    .into_any_element()
}

fn max_label_width(panel: &OtherPanel, window: &mut Window, labels: &[&str]) -> f32 {
    max_label_width_for_size(panel, window, labels, ShadcnTextSize::Xs)
}

fn max_label_width_for_size(
    panel: &OtherPanel,
    window: &mut Window,
    labels: &[&str],
    text_size: ShadcnTextSize,
) -> f32 {
    let row_label_typography = panel.look.typography_scale(text_size);
    let mut label_font = font(panel.look.mode_tokens().typography.font.sans.family.clone());
    label_font.weight = row_label_typography.weight;
    let mut max_width = 0.0_f32;

    for label in labels {
        let run = TextRun {
            len: label.len(),
            font: label_font.clone(),
            color: panel.look.chrome().body_text,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window.text_system().shape_line(
            SharedString::from((*label).to_owned()),
            px(row_label_typography.size),
            &[run],
            None,
        );
        max_width = max_width.max(line.x_for_index(label.len()).as_f32());
    }

    max_width.ceil()
}

fn slider_field_grid_label(panel: &OtherPanel, label: &str) -> AnyElement {
    let chrome = panel.look.chrome();
    let row_label_typography = panel.look.typography_scale(ShadcnTextSize::Xs);

    div()
        .w_full()
        .typography_style(row_label_typography)
        .text_color(chrome.body_text)
        .child(label.to_string())
        .into_any_element()
}

fn slider_field_grid_unit(panel: &OtherPanel, unit: &'static str) -> AnyElement {
    let chrome = panel.look.chrome();
    let unit_style = panel.look.typography_scale(ShadcnTextSize::Sm);

    div().typography_style(unit_style).text_color(chrome.muted_text).child(unit).into_any_element()
}
