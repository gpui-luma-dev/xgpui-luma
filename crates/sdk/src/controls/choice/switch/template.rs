use gpui::{App, Div, Hsla, Stateful, Window, div, hsla, px, prelude::*};

use crate::infra::shadow_layout::should_paint_shadow;
use crate::controls::button::{ButtonRenderModel, ButtonTemplate, button_content_context};
use crate::controls::button_family::ButtonFamilyLook;

use crate::controls::switch::{SwitchData, SwitchOrientation, SwitchPalette, SwitchTheme, default_switch_theme};
use crate::infra::template::TemplateWithModifiers;
use crate::define_control_template;
use crate::theme::snap_to_pixel;

define_control_template!(
    ThemedSwitchTemplate,
    dyn SwitchTheme,
    ButtonRenderModel<SwitchData>,
    ButtonTemplate<SwitchData>,
    default_switch_theme()
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

// Interpolate only colors; typography and shadows come from the owned settled palette.
struct SwitchColors {
    track_background: Hsla,
    track_border: Hsla,
    thumb_background: Hsla,
    thumb_border: Hsla,
}

fn lerp_switch_palette(off: &SwitchPalette, on: &SwitchPalette, progress: f32) -> SwitchColors {
    let t = progress.clamp(0.0, 1.0);
    SwitchColors {
        track_background: lerp_hsla(off.track_background, on.track_background, t),
        track_border: lerp_hsla(off.track_border, on.track_border, t),
        thumb_background: lerp_hsla(off.thumb_background, on.thumb_background, t),
        thumb_border: lerp_hsla(off.thumb_border, on.thumb_border, t),
    }
}

impl ButtonTemplate<SwitchData> for ThemedSwitchTemplate {
    fn render(&self, model: &ButtonRenderModel<SwitchData>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let progress = model.data.progress;
        let checked = model.data.checked;
        let focused = model.state.focused && !model.state.disabled;
        let off_palette = self.theme.resolve(false, model.state, model.size);
        let on_palette = self.theme.resolve(true, model.state, model.size);
        let palette = lerp_switch_palette(&off_palette, &on_palette, progress);
        let settled_palette = if checked { on_palette } else { off_palette };
        let focus = crate::infra::focus_adorner::FocusAdorner::new(self.theme.metrics().focus);
        let focus_ring_extent = focus.extent();
        let scale_factor = window.scale_factor();
        let mut scale = self.theme.scale(model.size, scale_factor);
        if let Some(track_width) = model.switch_track_width {
            scale.track_width = snap_to_pixel(track_width, scale_factor);
        }
        if let Some(track_height) = model.switch_track_height {
            scale.track_height = snap_to_pixel(track_height, scale_factor);
        }
        if let Some(thumb_size) = model.switch_thumb_size {
            scale.thumb_size = snap_to_pixel(thumb_size, scale_factor);
        }
        let track_length = snap_to_pixel(scale.track_width + model.switch_track_width_extra, scale_factor);
        let (track_width, track_height) = match model.switch_orientation {
            SwitchOrientation::Horizontal => (track_length, scale.track_height),
            SwitchOrientation::Vertical => (scale.track_height, track_length),
        };
        scale.track_width = track_width;
        scale.track_height = track_height;
        let track_radius = model.radius_override.get().unwrap_or(scale.track_radius);
        let thumb_radius = track_radius.min(scale.thumb_size * 0.5);

        let (thumb_left, thumb_top) = match model.switch_orientation {
            SwitchOrientation::Horizontal => {
                let thumb_left_off = scale.track_padding;
                let thumb_left_on = scale.track_width - scale.thumb_size - scale.track_padding;
                let thumb_left = snap_to_pixel(lerp_f32(thumb_left_off, thumb_left_on, progress), scale_factor);
                let thumb_top = snap_to_pixel(
                    ((scale.track_height - scale.thumb_size) * 0.5 - self.theme.metrics().border_width.default)
                        .max(0.0),
                    scale_factor,
                );
                (thumb_left, thumb_top)
            }
            SwitchOrientation::Vertical => {
                let thumb_left = snap_to_pixel(
                    ((scale.track_width - scale.thumb_size) * 0.5 - self.theme.metrics().border_width.default).max(0.0),
                    scale_factor,
                );
                let thumb_top_on = scale.track_padding;
                let thumb_top_off = scale.track_height - scale.thumb_size - scale.track_padding;
                let thumb_top = snap_to_pixel(lerp_f32(thumb_top_off, thumb_top_on, progress), scale_factor);
                (thumb_left, thumb_top)
            }
        };

        let mut thumb = div()
            .id("thumb")
            .absolute()
            .left(px(thumb_left))
            .top(px(thumb_top))
            .size(px(scale.thumb_size))
            .flex()
            .items_center()
            .justify_center()
            .bg(palette.thumb_background)
            .border(px(self.theme.metrics().border_width.default))
            .border_color(palette.thumb_border)
            .rounded(px(thumb_radius));

        let content_look = ButtonFamilyLook {
            focus_border: gpui::transparent_black(),
            background: palette.track_background,
            foreground: settled_palette.label_color,
            muted_foreground: settled_palette.label_color,
            border: Some(palette.track_border),
            typography: settled_palette.label_typography,
            font_family: settled_palette.label_font_family.clone(),
            radius: track_radius,
            padding_x: 0.0,
            padding_y: 0.0,
            gap: scale.gap,
            height: scale.track_height,
            icon_size: scale.thumb_size,
            shadow: None,
        };
        let content_model = button_content_context(model, content_look.clone());
        if let Some(content) = &model.switch_thumb_content {
            thumb = thumb.child(content(&content_model, cx));
        }

        let mut track_visual = div()
            .id("track")
            .relative()
            .w(px(scale.track_width))
            .h(px(scale.track_height))
            .bg(palette.track_background)
            .border(px(self.theme.metrics().border_width.default))
            .border_color(palette.track_border)
            .rounded(px(track_radius));

        if let Some(content) = &model.switch_track_content {
            track_visual = track_visual.child(
                div()
                    .absolute()
                    .top_0()
                    .right_0()
                    .bottom_0()
                    .left_0()
                    .overflow_hidden()
                    .rounded(px(track_radius))
                    .child(content(&content_model, cx)),
            );
        }

        track_visual = track_visual.child(thumb);

        if should_paint_shadow(model.elevation, model.state.disabled, !settled_palette.thumb_shadow.is_empty()) {
            track_visual = track_visual.shadow(settled_palette.thumb_shadow);
        }

        let track = div().relative().child(track_visual);
        let track = if focus_ring_extent > 0.0 {
            let mut slot = div()
                .id("track-slot")
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .w(px(scale.track_width + (focus_ring_extent * 2.0)))
                .h(px(scale.track_height + (focus_ring_extent * 2.0)))
                .child(track);

            slot = slot.child(focus.overlay(settled_palette.focus_border, focused, track_radius, false));

            slot.into_any_element()
        } else {
            track.into_any_element()
        };

        let indicator_only = matches!(model.role, crate::controls::button_family::ButtonFamilyRole::Icon);
        let mut control =
            div().id((model.id.clone(), 0usize)).relative().flex().items_center().rounded(px(track_radius));

        if indicator_only {
            control = control.child(track);
        } else {
            let label = div().mt(px(scale.label_baseline_shift)).child((model.content)(&content_model, cx));
            control = control
                .gap(px(scale.gap))
                .text_color(settled_palette.label_color)
                .text_size(px(settled_palette.label_typography.size))
                .line_height(px(settled_palette.label_typography.line_height))
                .font_family(settled_palette.label_font_family.clone())
                .font_weight(settled_palette.label_typography.weight)
                .child(track)
                .child(label);
        }

        let mut root = self.apply_modifiers(control, model);

        if model.state.disabled {
            root = root.opacity(scale.disabled_opacity);
        } else {
            root = root.cursor_pointer();
        }

        root
    }
}
