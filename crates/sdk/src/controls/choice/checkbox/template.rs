use gpui::{AnyElement, App, Div, Hsla, Stateful, Window, div, hsla, px, prelude::*};

use crate::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
use crate::infra::shadow_layout::should_paint_shadow;
use crate::controls::button::{ButtonRenderModel, ButtonTemplate, button_content_context};
use crate::controls::checkbox::{CheckboxData, CheckboxPalette, CheckboxTheme, default_checkbox_theme};
use crate::infra::template::TemplateWithModifiers;
use crate::define_control_template;

define_control_template!(
    ThemedCheckboxTemplate,
    dyn CheckboxTheme,
    ButtonRenderModel<CheckboxData>,
    ButtonTemplate<CheckboxData>,
    default_checkbox_theme()
);

fn lerp_f32(start: f32, end: f32, t: f32) -> f32 {
    start + ((end - start) * t)
}

fn lerp_hsla(start: Hsla, end: Hsla, t: f32) -> Hsla {
    hsla(
        lerp_f32(start.h, end.h, t),
        lerp_f32(start.s, end.s, t),
        lerp_f32(start.l, end.l, t),
        lerp_f32(start.a, end.a, t),
    )
}

fn lerp_optional_hsla(start: Option<Hsla>, end: Option<Hsla>, t: f32) -> Option<Hsla> {
    match (start, end) {
        (Some(a), Some(b)) => Some(lerp_hsla(a, b, t)),
        (Some(a), None) if t < 0.5 => Some(a),
        (None, Some(b)) if t >= 0.5 => Some(b),
        _ => None,
    }
}

// Interpolate only colors; typography and shadows come from the owned settled palette.
struct CheckboxColors {
    control_background: Option<Hsla>,
    control_border: Option<Hsla>,
    indicator_background: Hsla,
    indicator_border: Hsla,
    checkmark_color: Hsla,
}

fn lerp_checkbox_palette(off: &CheckboxPalette, on: &CheckboxPalette, progress: f32) -> CheckboxColors {
    let t = progress.clamp(0.0, 1.0);
    CheckboxColors {
        control_background: lerp_optional_hsla(off.control_background, on.control_background, t),
        control_border: lerp_optional_hsla(off.control_border, on.control_border, t),
        indicator_background: lerp_hsla(off.indicator_background, on.indicator_background, t),
        indicator_border: lerp_hsla(off.indicator_border, on.indicator_border, t),
        checkmark_color: lerp_hsla(off.checkmark_color, on.checkmark_color, t),
    }
}

impl ButtonTemplate<CheckboxData> for ThemedCheckboxTemplate {
    fn render(&self, model: &ButtonRenderModel<CheckboxData>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let progress = model.data.progress.clamp(0.0, 1.0);
        let checked = model.data.checked;
        let focused = model.state.focused && !model.state.disabled;
        let off_palette = self.theme.resolve(false, model.state, model.size);
        let on_palette = self.theme.resolve(true, model.state, model.size);
        let palette = lerp_checkbox_palette(&off_palette, &on_palette, progress);
        let mut settled_palette = if checked { on_palette } else { off_palette };
        let focus = crate::infra::focus_adorner::FocusAdorner::new(self.theme.metrics().focus);
        let focus_ring_extent = focus.extent();
        let scale_factor = window.scale_factor();
        let scale = self.theme.scale(model.size, scale_factor);

        let indicator_visual = {
            let mut indicator = div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(scale.indicator_size))
                .bg(palette.indicator_background)
                .border(px(self.theme.metrics().border_width.default))
                .border_color(palette.indicator_border)
                .rounded(px(scale.indicator_radius))
                .child(render_checkmark(
                    progress,
                    scale.glyph_size,
                    palette.checkmark_color,
                    &model.data.icons.selected,
                ));

            if should_paint_shadow(
                model.elevation,
                model.state.disabled,
                settled_palette.indicator_shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()),
            ) && let Some(shadows) = settled_palette.indicator_shadow.take()
            {
                indicator = indicator.shadow(shadows);
            }

            indicator
        };

        let indicator_only = matches!(model.role, ButtonFamilyRole::Icon);
        let indicator = div().relative().child(indicator_visual);
        let indicator = if focus_ring_extent > 0.0 {
            let mut slot = div()
                .id("indicator-slot")
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .size(px(scale.indicator_size + (focus_ring_extent * 2.0)))
                .child(indicator);

            slot = slot.child(focus.overlay(settled_palette.focus_border, focused, scale.indicator_radius, false));

            slot.into_any_element()
        } else {
            indicator.into_any_element()
        };

        let mut control = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .text_color(settled_palette.label_color)
            .text_size(px(settled_palette.label_typography.size))
            .line_height(px(settled_palette.label_typography.line_height))
            .font_family(settled_palette.label_font_family.clone())
            .font_weight(settled_palette.label_typography.weight)
            .rounded(px(scale.control_radius))
            .cursor_pointer();

        if indicator_only {
            control = control.child(indicator);
        } else {
            let content_look = ButtonFamilyLook {
                focus_border: gpui::transparent_black(),
                background: settled_palette.control_background.unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.0)),
                foreground: settled_palette.label_color,
                muted_foreground: settled_palette.label_color,
                border: settled_palette.control_border,
                typography: settled_palette.label_typography,
                font_family: settled_palette.label_font_family.clone(),
                radius: scale.control_radius,
                padding_x: scale.control_padding_x,
                padding_y: scale.control_padding_y,
                gap: scale.gap,
                height: scale.height,
                icon_size: scale.glyph_size,
                shadow: None,
            };
            let content_model = button_content_context(model, content_look);
            let label = div().mt(px(scale.label_baseline_shift)).child((model.content)(&content_model, cx));
            control = control
                .gap(px(scale.gap))
                .min_h(px(scale.height))
                .px(px(scale.control_padding_x))
                .py(px(scale.control_padding_y))
                .child(indicator)
                .child(label);
        }

        if let Some(background) = palette.control_background {
            control = control.bg(background);
        }

        // Optional paint must not add/remove a layout border when focus changes.
        control = control
            .border(px(self.theme.metrics().border_width.default))
            .border_color(palette.control_border.unwrap_or_else(gpui::transparent_black));

        let mut root = self.apply_modifiers(control, model);

        if model.state.disabled {
            root = root.opacity(scale.disabled_opacity);
        } else {
            root = root.cursor_pointer();
        }

        root
    }
}

fn render_checkmark(progress: f32, size: f32, color: gpui::Hsla, icon: &crate::infra::icon::IconSource) -> AnyElement {
    let progress = progress.clamp(0.0, 1.0);
    if progress <= f32::EPSILON {
        return div().size(px(size)).into_any_element();
    }

    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .opacity(progress)
        .child(crate::infra::icon::render_icon_source(icon, color, size))
        .into_any_element()
}
