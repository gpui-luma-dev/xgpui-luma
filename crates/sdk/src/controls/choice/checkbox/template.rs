use gpui::{AnyElement, App, Div, Stateful, Window, div, px, prelude::*};
use xgpui_luma_style::{Lerp as _, State, StateSet};

use crate::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
use crate::infra::shadow_layout::should_paint_shadow;
use crate::controls::button::{ButtonRenderModel, ButtonTemplate, button_content_context};
use crate::controls::checkbox::{CheckboxData, CheckboxTheme, ResolvedCheckboxStyle, default_checkbox_theme};
use crate::theme::LumaTextStyle;
use crate::infra::template::TemplateWithModifiers;
use crate::define_control_template;

define_control_template!(
    ThemedCheckboxTemplate,
    dyn CheckboxTheme,
    ButtonRenderModel<CheckboxData>,
    ButtonTemplate<CheckboxData>,
    default_checkbox_theme()
);

impl ButtonTemplate<CheckboxData> for ThemedCheckboxTemplate {
    fn render(&self, model: &ButtonRenderModel<CheckboxData>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let progress = model.data.progress.clamp(0.0, 1.0);
        let focused = model.state.focused && !model.state.disabled;
        let scale_factor = window.scale_factor();
        let at = StateSet::from(model.state);
        // Off and on both resolve so the check animation can blend their colors.
        let off = self.theme.resolved_style(at, scale_factor);
        let on = self.theme.resolved_style(at.with(State::Checked), scale_factor);
        let paint = ResolvedCheckboxStyle::lerp(&off, &on, progress);
        // Geometry, typography and shadow come from the settled state.
        let s = if model.data.checked { on } else { off };
        let focus = crate::infra::focus_adorner::FocusAdorner::new(self.theme.metrics().focus);
        let focus_ring_extent = focus.extent();

        let mut indicator_visual = div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(s.indicator_size))
            .bg(paint.indicator.background)
            .border(px(s.border_width))
            .border_color(paint.indicator.border)
            .rounded(px(s.indicator_radius))
            .child(render_checkmark(progress, s.glyph_size, paint.glyph.foreground, &model.data.icons.selected));
        if let Some(shadows) = s
            .indicator
            .shadow
            .clone()
            .filter(|shadows| should_paint_shadow(model.elevation, model.state.disabled, !shadows.is_empty()))
        {
            indicator_visual = indicator_visual.shadow(shadows);
        }

        let indicator = div().relative().child(indicator_visual);
        let indicator = if focus_ring_extent > 0.0 {
            div()
                .id("indicator-slot")
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .size(px(s.indicator_size + (focus_ring_extent * 2.0)))
                .child(indicator)
                .child(focus.overlay(s.root.focus_border, focused, s.indicator_radius, false))
                .into_any_element()
        } else {
            indicator.into_any_element()
        };

        let mut control = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .text_color(s.label.foreground)
            .text_size(px(s.label.font_size))
            .line_height(px(s.label.line_height))
            .font_family(s.label.font_family.clone())
            .font_weight(s.label.font_weight)
            .rounded(px(s.control_radius))
            .cursor_pointer();

        if matches!(model.role, ButtonFamilyRole::Icon) {
            control = control.child(indicator);
        } else {
            let content_model = button_content_context(model, content_look(&s));
            let label = div().mt(px(s.label_baseline_shift)).child((model.content)(&content_model, cx));
            control = control
                .gap(px(s.gap))
                .min_h(px(s.height))
                .px(px(s.control_padding_x))
                .py(px(s.control_padding_y))
                .child(indicator)
                .child(label);
        }

        if let Some(background) = paint.root.background {
            control = control.bg(background);
        }

        // Optional paint must not add/remove a layout border when focus changes.
        control = control
            .border(px(s.border_width))
            .border_color(paint.root.border.unwrap_or_else(gpui::transparent_black));

        let root = self.apply_modifiers(control, model);
        if model.state.disabled {
            root.opacity(s.disabled_opacity)
        } else {
            root.cursor_pointer()
        }
    }
}

/// The label content's look, from the settled style.
fn content_look(s: &ResolvedCheckboxStyle) -> ButtonFamilyLook {
    let typography =
        LumaTextStyle { size: s.label.font_size, line_height: s.label.line_height, weight: s.label.font_weight };
    ButtonFamilyLook {
        focus_border: gpui::transparent_black(),
        background: s.root.background.unwrap_or_else(gpui::transparent_black),
        foreground: s.label.foreground,
        muted_foreground: s.label.foreground,
        border: s.root.border,
        typography,
        font_family: s.label.font_family.clone(),
        radius: s.control_radius,
        padding_x: s.control_padding_x,
        padding_y: s.control_padding_y,
        gap: s.gap,
        height: s.height,
        icon_size: s.glyph_size,
        shadow: None,
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
