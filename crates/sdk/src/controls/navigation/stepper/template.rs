use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Div, FontWeight, SharedString, Stateful, Window, div, px, relative, prelude::*};

use super::model::{StepperLabelPlacement, StepperRenderModel};
use super::theme::{StepperTheme, default_stepper_theme};
use crate::style::PartRead;
use xgpui_luma_style::PartView;
use super::StepState;
use crate::controls::progress::{ProgressDirection, ProgressOrientation};

pub type StepperTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &StepperRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait StepperTemplate: Send + Sync {
    fn render(&self, model: &StepperRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

impl RenderModel for dyn StepperTemplate {
    type Model<'a> = StepperRenderModel<'a>;
}

pub struct ThemedStepperTemplate {
    theme: Arc<dyn StepperTheme>,
    modifiers: Vec<StepperTemplateModifier>,
}

impl ThemedStepperTemplate {
    pub fn new(theme: Arc<dyn StepperTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &StepperRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

pub fn default_stepper_template() -> Arc<dyn StepperTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn StepperTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedStepperTemplate::new(default_stepper_theme()))).clone()
}

pub(super) fn template_with_modifier<F>(template: Arc<dyn StepperTemplate>, modifier: F) -> Arc<dyn StepperTemplate>
where
    F: Fn(Stateful<Div>, &StepperRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl StepperTemplate for ModifiedTemplate<dyn StepperTemplate> {
    fn render(&self, model: &StepperRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let root = self.base.render(model, window, cx);
        self.apply(root, model)
    }
}

impl StepperTemplate for ThemedStepperTemplate {
    fn render(&self, model: &StepperRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let resolved = super::styled::resolve_stepper(&self.theme.style(), model.enabled);
        let look = resolved.root();
        let display_indices = display_indices(model.step_count, model.direction);
        let horizontal = model.direction.orientation() == ProgressOrientation::Horizontal;

        let indicator = if horizontal {
            render_horizontal_stepper(model, &display_indices, look)
        } else {
            render_vertical_stepper(model, &display_indices, look)
        };

        let root = if model.has_content_panel() {
            let panel = render_content_panel(model, horizontal, look, window, cx);
            if horizontal {
                div()
                    .id((model.id.clone(), 0usize))
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(look.number("content_gap")))
                    .child(indicator)
                    .child(panel)
            } else {
                div()
                    .id((model.id.clone(), 0usize))
                    .h_full()
                    .w_full()
                    .flex()
                    .flex_row()
                    .gap(px(look.number("content_gap")))
                    .child(indicator)
                    .child(panel.flex_1())
            }
        } else {
            indicator
        };

        apply_template_modifiers(&self.modifiers, root, model)
    }
}

fn display_indices(step_count: usize, direction: ProgressDirection) -> Vec<usize> {
    if direction.is_reversed() {
        (0..step_count).rev().collect()
    } else {
        (0..step_count).collect()
    }
}

fn segment_fill(display_step: f32, from_index: usize) -> f32 {
    (display_step - from_index as f32).clamp(0.0, 1.0)
}

fn render_horizontal_stepper(
    model: &StepperRenderModel<'_>,
    display_indices: &[usize],
    look: PartView<'_>,
) -> Stateful<Div> {
    let badge_size = look.number("step_badge_size");
    let track_thickness = look.number("track_thickness");

    let mut root = div().id(model.id.clone()).w_full().flex().flex_row().items_start();

    for (position, &step_index) in display_indices.iter().enumerate() {
        let state = model.step_states.get(step_index).copied().unwrap_or(StepState::Incomplete);
        let label = model.labels.get(step_index);
        let is_first = position == 0;
        let is_last = position + 1 == display_indices.len();

        let left_fill = if is_first {
            0.0
        } else {
            segment_fill(model.display_step, display_indices[position - 1])
        };
        let right_fill = if is_last {
            0.0
        } else {
            segment_fill(model.display_step, step_index)
        };

        root = root.child(
            div()
                .id(("step", step_index))
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(look.number("label_gap")))
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .items_center()
                        .child(render_horizontal_track(
                            is_first,
                            left_fill,
                            track_thickness,
                            badge_size,
                            look.color("track_active"),
                            look.color("track_muted"),
                            step_index * 2,
                        ))
                        .child(render_badge(step_index, state, badge_size, look, model.icons))
                        .child(render_horizontal_track(
                            is_last,
                            right_fill,
                            track_thickness,
                            badge_size,
                            look.color("track_active"),
                            look.color("track_muted"),
                            step_index * 2 + 1,
                        )),
                )
                .children(label.map(|label| render_step_label(label, look, false).text_center())),
        );
    }

    root
}

fn render_vertical_stepper(
    model: &StepperRenderModel<'_>,
    display_indices: &[usize],
    look: PartView<'_>,
) -> Stateful<Div> {
    let badge_size = look.number("step_badge_size");
    let track_thickness = look.number("track_thickness");
    let side_labels = model.label_placement != StepperLabelPlacement::Below;

    let mut root = div()
        .id(model.id.clone())
        .h_full()
        .w_full()
        .min_w(px(if side_labels {
            badge_size + look.number("side_label_width")
        } else {
            badge_size + look.number("label_width")
        }))
        .flex()
        .flex_col()
        .items_stretch();

    for (position, &step_index) in display_indices.iter().enumerate() {
        let state = model.step_states.get(step_index).copied().unwrap_or(StepState::Incomplete);
        let label = model.labels.get(step_index);
        let is_first = position == 0;
        let is_last = position + 1 == display_indices.len();

        let above_fill = if is_first {
            0.0
        } else {
            segment_fill(model.display_step, display_indices[position - 1])
        };
        let below_fill = if is_last {
            0.0
        } else {
            segment_fill(model.display_step, step_index)
        };

        let badge_column = div()
            .w(px(badge_size))
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .child(render_vertical_track(
                is_first,
                above_fill,
                track_thickness,
                badge_size,
                look.color("track_active"),
                look.color("track_muted"),
                step_index * 2,
            ))
            .child(render_badge(step_index, state, badge_size, look, model.icons))
            .child(render_vertical_track(
                is_last,
                below_fill,
                track_thickness,
                badge_size,
                look.color("track_active"),
                look.color("track_muted"),
                step_index * 2 + 1,
            ));

        root = root.child(match model.label_placement {
            StepperLabelPlacement::Below => div()
                .id(("step", step_index))
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .flex_col()
                .items_center()
                .child(badge_column)
                .children(label.map(|label| render_step_label(label, look, false))),
            StepperLabelPlacement::Start => div()
                .id(("step", step_index))
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(look.number("side_label_gap")))
                .child(render_side_label_slot(label, look, true))
                .child(badge_column)
                .child(div().flex_1()),
            StepperLabelPlacement::End => div()
                .id(("step", step_index))
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(look.number("side_label_gap")))
                .child(div().flex_1())
                .child(badge_column)
                .child(render_side_label_slot(label, look, false)),
        });
    }

    root
}

fn render_content_panel(
    model: &StepperRenderModel<'_>,
    horizontal: bool,
    look: PartView<'_>,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    let progress = model.transition_progress.clamp(0.0, 1.0);
    let max_index = model.step_count.saturating_sub(1);
    let from_index = model.from_step.round().clamp(0.0, max_index as f32) as usize;
    let to_index = model.to_step.min(max_index);
    let settled = progress >= 1.0 - f32::EPSILON || from_index == to_index;
    let forward = (to_index as f32) >= model.from_step;

    let mut host = div().id((model.id.clone(), 1usize)).relative().overflow_hidden().w_full();

    if let Some(height) = model.content_height {
        host = host.h(px(height));
    } else {
        host = host.h(px(look.number("content_height")));
    }

    if settled {
        if let Some(renderer) = model.step_contents.get(to_index).and_then(|slot| slot.as_ref()) {
            host = host.child(positioned_panel(renderer(window, cx), horizontal, 0.0, "current"));
        }
    } else {
        // Slide outgoing out and incoming in along the stepper axis.
        let (from_offset, to_offset) = if forward {
            (-progress, 1.0 - progress)
        } else {
            (progress, -(1.0 - progress))
        };
        if let Some(renderer) = model.step_contents.get(from_index).and_then(|slot| slot.as_ref()) {
            host = host.child(positioned_panel(renderer(window, cx), horizontal, from_offset, "from"));
        }
        if let Some(renderer) = model.step_contents.get(to_index).and_then(|slot| slot.as_ref()) {
            host = host.child(positioned_panel(renderer(window, cx), horizontal, to_offset, "to"));
        }
    }

    host
}

fn positioned_panel(content: AnyElement, horizontal: bool, offset: f32, key: &'static str) -> Stateful<Div> {
    let mut panel = div().id(key).absolute().top_0().left_0().size_full();

    if horizontal {
        panel = panel.left(relative(offset));
    } else {
        panel = panel.top(relative(offset));
    }

    panel.child(content)
}

fn render_step_label(label: &SharedString, look: PartView<'_>, align_end: bool) -> Stateful<Div> {
    let mut label_node = div()
        .id((label.clone(), 0usize))
        .text_size(px(look.typography(12.0, 16.0).size))
        .line_height(px(look.typography(12.0, 16.0).line_height))
        .text_color(look.color("incomplete_foreground"))
        .child(label.clone());

    if align_end {
        label_node = label_node.text_right();
    }

    label_node
}

fn render_side_label_slot(label: Option<&SharedString>, look: PartView<'_>, align_end: bool) -> Stateful<Div> {
    let mut slot = div().id(("label-slot", usize::from(align_end))).flex_1().min_w(px(0.0)).flex().items_center();
    slot = if align_end {
        slot.justify_end()
    } else {
        slot.justify_start()
    };

    if let Some(label) = label {
        slot = slot.child(render_step_label(label, look, align_end));
    }

    slot
}

fn render_horizontal_track(
    is_edge: bool,
    fill: f32,
    track_thickness: f32,
    badge_size: f32,
    active_color: gpui::Hsla,
    muted_color: gpui::Hsla,
    segment_id: usize,
) -> Stateful<Div> {
    if is_edge {
        return div().id(("track-h-edge", segment_id)).flex_1().h(px(badge_size));
    }

    let fill = fill.clamp(0.0, 1.0);
    div().id(("track-h", segment_id)).flex_1().h(px(badge_size)).flex().items_center().child(
        div()
            .relative()
            .w_full()
            .h(px(track_thickness))
            .bg(muted_color)
            .child(div().absolute().top_0().left_0().h_full().w(relative(fill)).bg(active_color)),
    )
}

fn render_vertical_track(
    is_edge: bool,
    fill: f32,
    track_thickness: f32,
    badge_size: f32,
    active_color: gpui::Hsla,
    muted_color: gpui::Hsla,
    segment_id: usize,
) -> Stateful<Div> {
    if is_edge {
        return div().id(("track-v-edge", segment_id)).flex_1().w(px(badge_size));
    }

    let fill = fill.clamp(0.0, 1.0);
    div().id(("track-v", segment_id)).flex_1().w(px(badge_size)).flex().justify_center().child(
        div()
            .relative()
            .w(px(track_thickness))
            .h_full()
            .bg(muted_color)
            .child(div().absolute().top_0().left_0().w_full().h(relative(fill)).bg(active_color)),
    )
}

fn render_badge(
    step_index: usize,
    state: StepState,
    badge_size: f32,
    look: PartView<'_>,
    icons: &crate::infra::icon::SelectionStatusIcons,
) -> AnyElement {
    let badge_radius = badge_size / 2.0;
    let (background, foreground, border) = match state {
        StepState::Complete => (
            look.color("complete_background"),
            look.color("complete_foreground"),
            look.color("complete_background"),
        ),
        StepState::InProgress => {
            (look.color("current_background"), look.color("current_foreground"), look.color("current_background"))
        }
        StepState::Incomplete => (
            look.color("incomplete_background"),
            look.color("incomplete_foreground"),
            look.color("incomplete_border"),
        ),
    };

    div()
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .size(px(badge_size))
        .rounded(px(badge_radius))
        .bg(background)
        .border(px(look.number("border_width")))
        .border_color(border)
        .child(match state {
            StepState::Complete => {
                render_checkmark(foreground, badge_size * look.number("glyph_scale"), &icons.completed)
            }
            StepState::InProgress | StepState::Incomplete => render_step_number(
                step_index + 1,
                foreground,
                badge_size * look.number("number_scale"),
                look.font_weight(),
            ),
        })
        .into_any_element()
}

fn render_checkmark(color: gpui::Hsla, size: f32, icon: &crate::infra::icon::IconSource) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size))
        .child(crate::infra::icon::render_icon_source(icon, color, size))
        .into_any_element()
}

fn render_step_number(step_number: usize, color: gpui::Hsla, size: f32, weight: FontWeight) -> AnyElement {
    div()
        .font_weight(weight)
        .text_color(color)
        .text_size(px(size))
        .child(step_number.to_string())
        .into_any_element()
}

fn apply_template_modifiers(
    modifiers: &[StepperTemplateModifier],
    mut root: Stateful<Div>,
    model: &StepperRenderModel<'_>,
) -> Stateful<Div> {
    for modifier in modifiers {
        root = modifier(root, model);
    }
    root
}
