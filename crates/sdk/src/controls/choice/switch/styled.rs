//! The switch on the shared choice template (issue #285): a track with a sliding thumb.
//!
//! Values come from `.lss` (`switch.lss`); an instance's model can still override the track and thumb
//! size, the extra track length, the orientation and the radius.

use gpui::{App, Div, ElementId, SharedString, Stateful, Window, div, prelude::*, px};
use xgpui_luma_style::{PartId, PartView, box_shadows};

use crate::controls::button::{ButtonRenderModel, ButtonTemplate, button_content_context};
use crate::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
use crate::controls::choice::styled::{
    ChoiceData, StyledChoiceTemplate, focus_adorner, hsl, label_style, length, transparent,
};
use crate::controls::switch::{SwitchData, SwitchOrientation};
use crate::infra::shadow_layout::should_paint_shadow;
use crate::infra::template::TemplateWithModifiers;
use crate::theme::snap_to_pixel;

/// A switch template whose values come from `.lss` files.
pub type StyledSwitchTemplate = StyledChoiceTemplate<SwitchData>;

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

impl ChoiceData for SwitchData {
    const CONTROL: &'static str = "switch";
    const ON_STATE: &'static str = "checked";

    fn is_on(&self) -> bool {
        self.checked
    }

    fn progress(&self) -> f32 {
        self.progress
    }

    fn toward(on: bool) -> Self {
        SwitchData::new(on)
    }

    fn root_id(id: &SharedString) -> ElementId {
        (id.clone(), 0usize).into()
    }

    fn render(
        template: &StyledChoiceTemplate<Self>,
        model: &ButtonRenderModel<Self>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let progress = model.data.progress.clamp(0.0, 1.0);
        let scale_factor = window.scale_factor();
        let (animated, settled) = template.resolve_pair(model, scale_factor);
        let root = settled.root();
        let focus = focus_adorner(root);
        let focus_ring_extent = focus.extent();
        let focus_color = hsl(root.focus_border()).unwrap_or_else(transparent);
        let focused = model.state.focused && !model.state.disabled;
        let border_width = length(root, "border_width");
        let gap = root.gap().unwrap_or(0.0);

        // `.lss` sizes, then the instance's model overrides.
        let snap = |v: f32| snap_to_pixel(v, scale_factor);
        let track_width = model.switch_track_width.map(snap).unwrap_or_else(|| length(root, "track_width"));
        let track_height = model.switch_track_height.map(snap).unwrap_or_else(|| length(root, "track_height"));
        let thumb_size = model.switch_thumb_size.map(snap).unwrap_or_else(|| length(root, "thumb_size"));
        let track_padding = length(root, "track_padding");
        let track_length = snap(track_width + model.switch_track_width_extra);
        let (track_width, track_height) = match model.switch_orientation {
            SwitchOrientation::Horizontal => (track_length, track_height),
            SwitchOrientation::Vertical => (track_height, track_length),
        };
        let track_radius = model.radius_override.get().unwrap_or_else(|| length(root, "track_radius"));
        let thumb_radius = track_radius.min(thumb_size * 0.5);

        let (thumb_left, thumb_top) = match model.switch_orientation {
            SwitchOrientation::Horizontal => (
                snap(lerp(track_padding, track_width - thumb_size - track_padding, progress)),
                snap(((track_height - thumb_size) * 0.5 - border_width).max(0.0)),
            ),
            SwitchOrientation::Vertical => (
                snap(((track_width - thumb_size) * 0.5 - border_width).max(0.0)),
                snap(lerp(track_height - thumb_size - track_padding, track_padding, progress)),
            ),
        };

        let animated_track = animated.part(PartId::new("track"));
        let animated_thumb = animated.part(PartId::new("thumb"));
        let track_background = hsl(animated_track.background()).unwrap_or_else(transparent);
        let track_border = hsl(animated_track.border()).unwrap_or_else(transparent);
        let (label_color, typography, font_family) = label_style(settled.part(PartId::new("label")));

        let mut thumb = div()
            .id("thumb")
            .absolute()
            .left(px(thumb_left))
            .top(px(thumb_top))
            .size(px(thumb_size))
            .flex()
            .items_center()
            .justify_center()
            .bg(hsl(animated_thumb.background()).unwrap_or_else(transparent))
            .border(px(border_width))
            .border_color(hsl(animated_thumb.border()).unwrap_or_else(transparent))
            .rounded(px(thumb_radius));
        if let Some(shadow) = painted_shadow(settled.part(PartId::new("thumb")), model.elevation, model.state.disabled)
        {
            thumb = thumb.shadow(shadow);
        }

        let content_look = ButtonFamilyLook {
            focus_border: transparent(),
            background: track_background,
            foreground: label_color,
            muted_foreground: label_color,
            border: Some(track_border),
            typography,
            font_family: font_family.clone(),
            radius: track_radius,
            padding_x: 0.0,
            padding_y: 0.0,
            gap,
            height: track_height,
            icon_size: thumb_size,
            shadow: None,
        };
        let content_model = button_content_context(model, content_look);
        if let Some(content) = &model.switch_thumb_content {
            thumb = thumb.child(content(&content_model, cx));
        }

        let mut track_visual = div()
            .id("track")
            .relative()
            .w(px(track_width))
            .h(px(track_height))
            .bg(track_background)
            .border(px(border_width))
            .border_color(track_border)
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
        // Each part draws its own shadow: `track.shadow` around the track, `thumb.shadow` under the thumb.
        if let Some(shadow) = painted_shadow(settled.part(PartId::new("track")), model.elevation, model.state.disabled)
        {
            track_visual = track_visual.shadow(shadow);
        }

        let track = div().relative().child(track_visual);
        let track = if focus_ring_extent > 0.0 {
            div()
                .id("track-slot")
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .w(px(track_width + focus_ring_extent * 2.0))
                .h(px(track_height + focus_ring_extent * 2.0))
                .child(track)
                .child(focus.overlay(focus_color, focused, track_radius, false))
                .into_any_element()
        } else {
            track.into_any_element()
        };

        let mut control = div().id(Self::root_id(&model.id)).relative().flex().items_center().rounded(px(track_radius));
        if matches!(model.role, ButtonFamilyRole::Icon) {
            control = control.child(track);
        } else {
            let baseline = length(root, "label_baseline_shift");
            let label = div().mt(px(baseline)).child((model.content)(&content_model, cx));
            control = control
                .gap(px(gap))
                .text_color(label_color)
                .text_size(px(typography.size))
                .line_height(px(typography.line_height))
                .font_family(font_family)
                .font_weight(typography.weight)
                .child(track)
                .child(label);
        }

        let mut root_element = template.apply_modifiers(control, model);
        root_element = crate::style::fade(root_element, crate::style::part_opacity(root));
        if !model.state.disabled {
            root_element = root_element.cursor_pointer();
        }
        root_element
    }
}

/// A part's `shadow` as it is painted: none when the instance has elevation off, is disabled, or the
/// part has no shadow layers.
fn painted_shadow(part: PartView<'_>, elevation: bool, disabled: bool) -> Option<Vec<gpui::BoxShadow>> {
    let layers = part.shadow()?;
    should_paint_shadow(elevation, disabled, !layers.is_empty()).then(|| box_shadows(layers))
}

/// The SDK's default switch template: the SDK's `.lss` files alone.
pub fn default_template() -> std::sync::Arc<dyn ButtonTemplate<SwitchData>> {
    static TEMPLATE: std::sync::OnceLock<std::sync::Arc<dyn ButtonTemplate<SwitchData>>> = std::sync::OnceLock::new();
    TEMPLATE.get_or_init(|| std::sync::Arc::new(StyledSwitchTemplate::sdk())).clone()
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::IntoElement;

    use super::*;

    #[test]
    fn styled_switch_layout_is_stable_across_states_orientations_and_scales() {
        for role in [ButtonFamilyRole::Text, ButtonFamilyRole::Icon] {
            for orientation in [SwitchOrientation::Horizontal, SwitchOrientation::Vertical] {
                let template = StyledSwitchTemplate::sdk();
                crate::infra::focus_adorner::testing::assert_stable_layout(move |state, on, window, cx| {
                    let model = ButtonRenderModel {
                        data: SwitchData::new(on),
                        state,
                        role,
                        switch_orientation: orientation,
                        ..Default::default()
                    };
                    template.render(&model, window, cx).into_any_element()
                });
            }
        }
    }
}

#[cfg(test)]
mod shadow_tests {
    use xgpui_luma_style::{Source, Theme, Value};

    use super::*;
    use crate::style::fixture;

    fn switch(theme: &Theme, states: &[&str]) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        theme.resolve(&theme.key_named("switch", &[], states).expect("key"))
    }

    /// The SDK track reads the control elevation; the thumb has no shadow unless a look sets one.
    #[test]
    fn sdk_switch_shadows_the_track_not_the_thumb() {
        for theme in fixture::themes() {
            fixture::assert_reads(&theme, "switch", &[], "track", "shadow", "elevation.control");
            let resolved = switch(&theme, &[]);
            assert!(painted_shadow(resolved.part(PartId::new("thumb")), true, false).is_none(), "no thumb shadow");
            assert!(painted_shadow(resolved.part(PartId::new("track")), true, false).is_some(), "a track shadow");
        }
    }

    /// A look that sets `thumb.shadow` gets it painted on the thumb, unless elevation is off or the
    /// switch is disabled; the track keeps its own shadow.
    #[test]
    fn a_thumb_shadow_is_painted_on_the_thumb() {
        let theme = fixture::light();
        let sheet = theme
            .stylesheet()
            .with_source("app", Source::new("lifted.lss", "switch { part thumb { shadow = elevation.menu } }"))
            .expect("compiles");
        let theme = theme.with_stylesheet(&sheet).expect("binds");
        let resolved = switch(&theme, &[]);
        let thumb = resolved.part(PartId::new("thumb"));
        let menu_layers = match fixture::token(&theme, "elevation.menu", "shadow") {
            Value::Shadow(layers) => layers.len(),
            other => panic!("a shadow: {other:?}"),
        };
        assert_eq!(painted_shadow(thumb, true, false).map(|s| s.len()), Some(menu_layers));
        assert!(painted_shadow(thumb, false, false).is_none(), "elevation off");
        assert!(painted_shadow(thumb, true, true).is_none(), "disabled");
        assert!(
            painted_shadow(resolved.part(PartId::new("track")), true, false).is_some(),
            "the track keeps its shadow"
        );
    }
}
