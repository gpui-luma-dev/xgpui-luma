use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Hsla, Stateful, Window, div, hsla, px, relative, prelude::*};

use super::{ProgressTemplate, ProgressTemplateModifier, apply_template_modifiers};
use super::super::direction::{ProgressDirection, ProgressOrientation};
use super::super::ProgressRenderModel;
use crate::controls::progress::{ProgressTheme, default_progress_theme};

pub struct LinearProgressTemplate {
    theme: Arc<dyn ProgressTheme>,
    modifiers: Vec<ProgressTemplateModifier>,
    track_radius: Option<Arc<dyn Fn(f32) -> f32 + Send + Sync>>,
}

pub type ThemedLinearProgressTemplate = LinearProgressTemplate;

impl LinearProgressTemplate {
    pub fn new(theme: Arc<dyn ProgressTheme>) -> Self {
        Self { theme, modifiers: Vec::new(), track_radius: None }
    }

    /// Resolve the track and fill corner radius from the themed track height, in pixels.
    /// Defaults to half the track height. The result is clamped to that range.
    pub fn with_track_radius<F>(mut self, radius: F) -> Self
    where
        F: Fn(f32) -> f32 + Send + Sync + 'static,
    {
        self.track_radius = Some(Arc::new(radius));
        self
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ProgressRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

pub fn default_linear_progress_template() -> Arc<dyn ProgressTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ProgressTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(LinearProgressTemplate::new(default_progress_theme()))).clone()
}

impl ProgressTemplate for LinearProgressTemplate {
    fn render(&self, model: &ProgressRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let look = self.theme.resolve(model.enabled, model.size);
        let percentage = model.percentage.clamp(0.0, 1.0);
        let track_height = look.track_height;
        let thumb_size = look.thumb_size;
        let track_radius = px(self
            .track_radius
            .as_ref()
            .map_or(track_height / 2.0, |radius| radius(track_height))
            .max(0.0)
            .min(track_height.max(0.0) / 2.0));
        let thumb_radius = thumb_size / 2.0;
        let thumb_offset = -(thumb_size / 2.0);

        let cross_extent = if model.show_thumb && !model.indeterminate {
            thumb_size.max(track_height)
        } else {
            track_height
        };
        let track_inset = (cross_extent - track_height) / 2.0;

        let root = match model.direction.orientation() {
            ProgressOrientation::Horizontal => {
                let track_node = div()
                    .absolute()
                    .rounded(track_radius)
                    .bg(look.track_color)
                    .left_0()
                    .right_0()
                    .top(px(track_inset))
                    .h(px(track_height));

                let fill_node = if model.indeterminate {
                    render_horizontal_shimmer(
                        model.direction,
                        model.phase,
                        track_inset,
                        track_height,
                        track_radius,
                        look.progress_color,
                        &look.geometry,
                    )
                } else {
                    match model.direction {
                        ProgressDirection::LeftToRight => div()
                            .absolute()
                            .rounded(track_radius)
                            .bg(look.progress_color)
                            .left_0()
                            .top(px(track_inset))
                            .h(px(track_height))
                            .w(relative(percentage)),
                        ProgressDirection::RightToLeft => div()
                            .absolute()
                            .rounded(track_radius)
                            .bg(look.progress_color)
                            .right_0()
                            .top(px(track_inset))
                            .h(px(track_height))
                            .w(relative(percentage)),
                        _ => div(),
                    }
                };

                let mut root = div()
                    .id(model.id.clone())
                    .relative()
                    .w_full()
                    .h(px(cross_extent))
                    .child(track_node)
                    .child(fill_node);

                if model.show_thumb && !model.indeterminate {
                    let thumb_node =
                        div().absolute().size(px(thumb_size)).rounded(px(thumb_radius)).bg(look.thumb_color);
                    let thumb_node = match model.direction {
                        ProgressDirection::LeftToRight => thumb_node.left(relative(percentage)).ml(px(thumb_offset)),
                        ProgressDirection::RightToLeft => thumb_node.right(relative(percentage)).mr(px(thumb_offset)),
                        _ => thumb_node,
                    };
                    root = root.child(thumb_node);
                }

                root
            }
            ProgressOrientation::Vertical => {
                let track_node = div()
                    .absolute()
                    .rounded(track_radius)
                    .bg(look.track_color)
                    .top_0()
                    .bottom_0()
                    .left(px(track_inset))
                    .w(px(track_height));

                let fill_node = if model.indeterminate {
                    render_vertical_shimmer(
                        model.direction,
                        model.phase,
                        track_inset,
                        track_height,
                        track_radius,
                        look.progress_color,
                        &look.geometry,
                    )
                } else {
                    match model.direction {
                        ProgressDirection::BottomToTop => div()
                            .absolute()
                            .rounded(track_radius)
                            .bg(look.progress_color)
                            .left(px(track_inset))
                            .bottom_0()
                            .w(px(track_height))
                            .h(relative(percentage)),
                        ProgressDirection::TopToBottom => div()
                            .absolute()
                            .rounded(track_radius)
                            .bg(look.progress_color)
                            .left(px(track_inset))
                            .top_0()
                            .w(px(track_height))
                            .h(relative(percentage)),
                        _ => div(),
                    }
                };

                let mut root = div()
                    .id(model.id.clone())
                    .relative()
                    .w(px(cross_extent))
                    .h_full()
                    .child(track_node)
                    .child(fill_node);

                if model.show_thumb && !model.indeterminate {
                    let thumb_node =
                        div().absolute().size(px(thumb_size)).rounded(px(thumb_radius)).bg(look.thumb_color);
                    let thumb_node = match model.direction {
                        ProgressDirection::BottomToTop => thumb_node.bottom(relative(percentage)).mb(px(thumb_offset)),
                        ProgressDirection::TopToBottom => thumb_node.top(relative(percentage)).mt(px(thumb_offset)),
                        _ => thumb_node,
                    };
                    root = root.child(thumb_node);
                }

                root
            }
        };

        apply_template_modifiers(&self.modifiers, root, model)
    }
}

fn shimmer_band_start(phase: f32, fraction: f32) -> f32 {
    let travel = 1.0 + fraction;
    (phase.clamp(0.0, 1.0) * travel) - fraction
}

fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    hsla(color.h, color.s, color.l, (color.a * alpha).clamp(0.0, 1.0))
}

fn render_horizontal_shimmer(
    direction: ProgressDirection,
    phase: f32,
    track_inset: f32,
    track_height: f32,
    track_radius: gpui::Pixels,
    color: Hsla,
    geometry: &crate::theme::stylesheet::ProgressGeometry,
) -> Div {
    let fraction = geometry.indeterminate_fraction;
    let start = shimmer_band_start(phase, fraction);
    let soft = with_alpha(color, geometry.shimmer_soft_opacity);
    let mid = with_alpha(color, geometry.shimmer_mid_opacity);

    let band = |offset: f32, width: f32, fill: Hsla| {
        let node = div()
            .absolute()
            .rounded(track_radius)
            .bg(fill)
            .top(px(track_inset))
            .h(px(track_height))
            .w(relative(width.max(0.0)));
        match direction {
            ProgressDirection::RightToLeft => node.right(relative(offset)),
            _ => node.left(relative(offset)),
        }
    };

    div()
        .absolute()
        .left_0()
        .right_0()
        .top(px(track_inset))
        .h(px(track_height))
        .overflow_hidden()
        .child(band(start, fraction, soft))
        .child(band(start + fraction * geometry.shimmer_mid_offset, fraction * geometry.shimmer_mid_width, mid))
        .child(band(start + fraction * geometry.shimmer_core_offset, fraction * geometry.shimmer_core_width, color))
}

fn render_vertical_shimmer(
    direction: ProgressDirection,
    phase: f32,
    track_inset: f32,
    track_height: f32,
    track_radius: gpui::Pixels,
    color: Hsla,
    geometry: &crate::theme::stylesheet::ProgressGeometry,
) -> Div {
    let fraction = geometry.indeterminate_fraction;
    let start = shimmer_band_start(phase, fraction);
    let soft = with_alpha(color, geometry.shimmer_soft_opacity);
    let mid = with_alpha(color, geometry.shimmer_mid_opacity);

    let band = |offset: f32, height: f32, fill: Hsla| {
        let node = div()
            .absolute()
            .rounded(track_radius)
            .bg(fill)
            .left(px(track_inset))
            .w(px(track_height))
            .h(relative(height.max(0.0)));
        match direction {
            ProgressDirection::TopToBottom => node.top(relative(offset)),
            _ => node.bottom(relative(offset)),
        }
    };

    div()
        .absolute()
        .top_0()
        .bottom_0()
        .left(px(track_inset))
        .w(px(track_height))
        .overflow_hidden()
        .child(band(start, fraction, soft))
        .child(band(start + fraction * geometry.shimmer_mid_offset, fraction * geometry.shimmer_mid_width, mid))
        .child(band(start + fraction * geometry.shimmer_core_offset, fraction * geometry.shimmer_core_width, color))
}
