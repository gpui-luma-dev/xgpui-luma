use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, Context, Div, FocusHandle, Hsla, Pixels, Stateful, Window, div, prelude::*, px};

use super::{
    control::{ResizablePanels, ResizablePanelsHandleDrag},
    math::{handle_hit_target_main_axis_px, handle_overlay_geometry, split_positions_px},
    model::{
        PanelSize, ResizeHandleMetrics, ResizeHandleVisibility, ResizablePanelSpec, ResizablePanelsOrientation,
        ResizablePanelsRenderModel,
    },
    theme::ResizablePanelsLook,
};

pub type ResizablePanelsTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &ResizablePanelsRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait ResizablePanelsTemplate: Send + Sync {
    fn render(
        &self,
        model: &ResizablePanelsRenderModel<'_>,
        look: &ResizablePanelsLook,
        handle_focuses: &[FocusHandle],
        window: &mut Window,
        cx: &mut Context<ResizablePanels>,
    ) -> Stateful<Div>;
}

impl RenderModel for dyn ResizablePanelsTemplate {
    type Model<'a> = ResizablePanelsRenderModel<'a>;
}

pub struct ThemedResizablePanelsTemplate {
    modifiers: RootModifiers<dyn ResizablePanelsTemplate>,
}

impl ThemedResizablePanelsTemplate {
    pub fn new() -> Self {
        Self { modifiers: RootModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ResizablePanelsRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl Default for ThemedResizablePanelsTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_resizable_panels_template() -> Arc<dyn ResizablePanelsTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ResizablePanelsTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedResizablePanelsTemplate::new())).clone()
}

pub(super) fn modified_resizable_panels_template<F>(
    template: Arc<dyn ResizablePanelsTemplate>,
    modifier: F,
) -> Arc<dyn ResizablePanelsTemplate>
where
    F: Fn(Stateful<Div>, &ResizablePanelsRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl ResizablePanelsTemplate for ModifiedTemplate<dyn ResizablePanelsTemplate> {
    fn render(
        &self,
        model: &ResizablePanelsRenderModel<'_>,
        look: &ResizablePanelsLook,
        handle_focuses: &[FocusHandle],
        window: &mut Window,
        cx: &mut Context<ResizablePanels>,
    ) -> Stateful<Div> {
        let root = self.base.render(model, look, handle_focuses, window, cx);
        self.apply(root, model)
    }
}

impl ResizablePanelsTemplate for ThemedResizablePanelsTemplate {
    fn render(
        &self,
        model: &ResizablePanelsRenderModel<'_>,
        look: &ResizablePanelsLook,
        handle_focuses: &[FocusHandle],
        window: &mut Window,
        cx: &mut Context<ResizablePanels>,
    ) -> Stateful<Div> {
        let handle_metrics = model.handle_metrics;
        let split_positions = split_positions_px(model.panel_sizes_px);
        let provisional_weight_layout = provisional_weight_layout(model);
        let mut handles = Vec::new();
        if !provisional_weight_layout {
            for (index, &split_px) in split_positions.iter().enumerate() {
                if model.panel_hidden.get(index).copied().unwrap_or(false)
                    || model.panel_hidden.get(index + 1).copied().unwrap_or(false)
                {
                    continue;
                }
                handles.push(render_overlay_handle(
                    index,
                    model,
                    look,
                    &handle_focuses[index],
                    &handle_metrics,
                    split_px,
                    window,
                    cx,
                ));
            }
        }

        let root = div().on_children_prepainted({
            let entity = cx.entity().clone();
            move |bounds, _window, cx| {
                let Some(size) = bounds.iter().map(|b| b.size).reduce(|acc, next| gpui::Size {
                    width: acc.width.max(next.width),
                    height: acc.height.max(next.height),
                }) else {
                    return;
                };
                entity.update(cx, |this, cx| {
                    this.set_measured_size(size, cx);
                });
            }
        });
        let root = paint_surface(root, model, look, handles)
            .on_drag_move(cx.listener(ResizablePanels::handle_drag_move))
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(ResizablePanels::finish_drag))
            .on_mouse_up_out(gpui::MouseButton::Left, cx.listener(ResizablePanels::finish_drag));
        self.modifiers.apply(root, model)
    }
}

/// Paint the SDK panel shell and panes with supplied separator elements.
/// This presentation helper does not bind control events or measure the owner.
pub fn render_resizable_panels_surface(
    model: &ResizablePanelsRenderModel<'_>,
    look: &ResizablePanelsLook,
    handles: impl IntoIterator<Item = AnyElement>,
) -> Stateful<Div> {
    paint_surface(div(), model, look, handles)
}

fn paint_surface(
    root: Div,
    model: &ResizablePanelsRenderModel<'_>,
    look: &ResizablePanelsLook,
    handles: impl IntoIterator<Item = AnyElement>,
) -> Stateful<Div> {
    let provisional_weight_layout = provisional_weight_layout(model);
    let mut root = root.id(model.id.clone()).relative().overflow_hidden().flex().flex_col();
    root = match (model.frame_width, model.frame_height) {
        (Some(width), Some(height)) => root.w(width).h(height),
        (Some(width), None) => root.w(width).h_full(),
        (None, Some(height)) => root.h(height).w_full(),
        (None, None) => root.size_full(),
    };

    if model.show_border {
        root = root.border(px(look.border_width)).border_color(look.border);
    }

    let mut track = div().relative().flex_1().min_h_0().min_w_0().w_full();
    let mut panels_row = div().w_full().h_full().flex().items_stretch();
    if model.orientation == ResizablePanelsOrientation::Vertical {
        panels_row = panels_row.flex_col();
    }

    for (index, panel) in model.panels.iter().enumerate() {
        let main_axis_px = model.panel_sizes_px.get(index).copied().unwrap_or(0.0);
        let main_size = px(main_axis_px.max(0.0));
        panels_row = panels_row.child(render_panel(model.orientation, panel, main_size, provisional_weight_layout));
    }

    track = track.child(panels_row);

    root.child(track.children(handles))
}

fn provisional_weight_layout(model: &ResizablePanelsRenderModel<'_>) -> bool {
    if model.frame_width.is_some() || model.frame_height.is_some() {
        return false;
    }

    let main_axis_px = match model.orientation {
        ResizablePanelsOrientation::Horizontal => model.measured_size.map(|size| size.width.as_f32()).unwrap_or(0.0),
        ResizablePanelsOrientation::Vertical => model.measured_size.map(|size| size.height.as_f32()).unwrap_or(0.0),
    };

    main_axis_px < 64.0
}

fn panel_flex_weight(panel: &ResizablePanelSpec) -> f32 {
    match panel.size {
        PanelSize::Weight(weight) => weight.max(0.0),
        PanelSize::Absolute(px) => px.as_f32().max(1.0),
    }
}

fn render_panel(
    orientation: ResizablePanelsOrientation,
    panel: &ResizablePanelSpec,
    main_size: Pixels,
    provisional_weight_layout: bool,
) -> impl IntoElement {
    let mut panel_node = div().overflow_hidden().child((panel.render.clone())());
    if let Some(background) = panel.background {
        panel_node = panel_node.bg(background);
    }
    if provisional_weight_layout && matches!(panel.size, PanelSize::Weight(_)) {
        let weight = panel_flex_weight(panel);
        return match orientation {
            ResizablePanelsOrientation::Horizontal => {
                panel_node.flex_grow(weight).flex_shrink_0().min_w(px(0.0)).h_full()
            }
            ResizablePanelsOrientation::Vertical => {
                panel_node.flex_grow(weight).flex_shrink_0().min_h(px(0.0)).w_full()
            }
        };
    }
    match orientation {
        ResizablePanelsOrientation::Horizontal => panel_node.w(main_size).h_full().flex_shrink_0(),
        ResizablePanelsOrientation::Vertical => panel_node.h(main_size).w_full().flex_shrink_0(),
    }
}

#[allow(clippy::too_many_arguments)]
fn render_overlay_handle(
    index: usize,
    model: &ResizablePanelsRenderModel<'_>,
    look: &ResizablePanelsLook,
    focus: &FocusHandle,
    handle_metrics: &ResizeHandleMetrics,
    split_px: f32,
    window: &mut Window,
    cx: &mut Context<ResizablePanels>,
) -> AnyElement {
    let focused = crate::infra::state::focus_visible(focus.is_focused(window), window);
    let (_, lane_px) = overlay_handle_lane(index, model, focused);
    let (origin_px, _, _) = handle_overlay_geometry(split_px, lane_px);
    let interaction_layer = render_handle_interaction_layer(
        index,
        model.id.clone(),
        model.orientation,
        px(handle_hit_target_main_axis_px(handle_metrics)),
        split_px,
        origin_px,
        model.enabled,
        ResizablePanelsHandleDrag { id: model.id.clone(), handle_index: index },
        cx,
    );
    render_resizable_panels_handle(index, model, look, split_px, focused)
        .track_focus(focus)
        .tab_index(if model.enabled { 0 } else { -1 })
        .on_key_down(cx.listener(move |this, event, window, cx| {
            this.handle_handle_key_down(index, event, window, cx);
        }))
        .child(interaction_layer)
        .into_any_element()
}

fn overlay_handle_lane(index: usize, model: &ResizablePanelsRenderModel<'_>, focused: bool) -> (bool, f32) {
    let handle_active =
        model.hovered_handle == Some(index) || model.dragging_handle == Some(index) || (model.enabled && focused);
    let show_handle = match model.handle_visibility {
        ResizeHandleVisibility::Always => true,
        ResizeHandleVisibility::Hover => handle_active,
        ResizeHandleVisibility::Hidden => false,
    };
    let handle_hit_px = handle_hit_target_main_axis_px(&model.handle_metrics);
    // Shell must be at least as wide as the hit target. Nesting a 12px hit layer inside a
    // 1px Hover-collapsed lane clips the grab area to a razor line (cursor flashes, drag fails).
    let layout_lane_px = if show_handle {
        model.handle_metrics.lane_px.max(handle_hit_px)
    } else {
        handle_hit_px
    };
    (show_handle, layout_lane_px)
}

/// Paint SDK separator and grip chrome without input bindings, for composed/static surfaces.
/// `split_px` is the seam position along the panel group's main axis.
pub fn render_resizable_panels_handle(
    index: usize,
    model: &ResizablePanelsRenderModel<'_>,
    look: &ResizablePanelsLook,
    split_px: f32,
    focused: bool,
) -> Stateful<Div> {
    let enabled = model.enabled;
    let orientation = model.orientation;
    let handle_id = (model.id.clone(), index);
    let handle_metrics = &model.handle_metrics;
    let (show_handle, lane_px) = overlay_handle_lane(index, model, focused);
    let (origin_px, lane_px, divider_local_px) = handle_overlay_geometry(split_px, lane_px);
    let mut handle = div()
        .id(handle_id)
        .absolute()
        .map(|this| crate::style::fade(this, look.opacity))
        .when(enabled && orientation == ResizablePanelsOrientation::Horizontal, |this| this.cursor_col_resize());

    handle = match orientation {
        ResizablePanelsOrientation::Horizontal => {
            handle.left(px(origin_px)).top(px(0.0)).bottom(px(0.0)).w(px(lane_px))
        }
        ResizablePanelsOrientation::Vertical => handle
            .top(px(origin_px))
            .left(px(0.0))
            .right(px(0.0))
            .h(px(lane_px))
            .when(enabled, |this| this.cursor_row_resize()),
    };

    let grip_color = if model.handle_grip {
        look.grip_emphasis
    } else {
        look.grip
    };

    let divider = match orientation {
        ResizablePanelsOrientation::Horizontal => div()
            .absolute()
            .left(px(divider_local_px))
            .top(px(0.0))
            .bottom(px(0.0))
            .w(px(look.divider_width))
            .bg(look.divider),
        ResizablePanelsOrientation::Vertical => div()
            .absolute()
            .top(px(divider_local_px))
            .left(px(0.0))
            .right(px(0.0))
            .h(px(look.divider_width))
            .bg(look.divider),
    };

    handle = handle.child(divider);
    if show_handle {
        handle = handle.child(render_handle_grip(orientation, grip_color, handle_metrics));
    }

    handle
}

fn render_handle_grip(orientation: ResizablePanelsOrientation, grip_color: Hsla, metrics: &ResizeHandleMetrics) -> Div {
    let cross = metrics.grip_cross_axis_px;
    let main = metrics.grip_main_axis_px;
    div()
        .absolute()
        .left(px(0.0))
        .right(px(0.0))
        .top(px(0.0))
        .bottom(px(0.0))
        .when(orientation == ResizablePanelsOrientation::Horizontal, |this| {
            this.flex().justify_center().items_center()
        })
        .when(orientation == ResizablePanelsOrientation::Vertical, |this| {
            this.flex().flex_col().justify_center().items_center()
        })
        .child(
            div()
                .rounded(px(metrics.grip_radius))
                .bg(grip_color)
                .when(orientation == ResizablePanelsOrientation::Horizontal, |this| this.w(px(cross)).h(px(main)))
                .when(orientation == ResizablePanelsOrientation::Vertical, |this| this.w(px(main)).h(px(cross))),
        )
}

#[allow(clippy::too_many_arguments)]
fn render_handle_interaction_layer(
    index: usize,
    id: gpui::SharedString,
    orientation: ResizablePanelsOrientation,
    handle_hit: Pixels,
    split_px: f32,
    handle_origin_px: f32,
    enabled: bool,
    drag_payload: ResizablePanelsHandleDrag,
    cx: &mut Context<ResizablePanels>,
) -> AnyElement {
    let hit_id = (id, index);
    let hit_px = handle_hit.as_f32();
    let split_local = split_px - handle_origin_px;
    let hit_inset = split_local - hit_px * 0.5;

    if orientation == ResizablePanelsOrientation::Horizontal {
        div()
            .id(hit_id)
            .absolute()
            .left(px(hit_inset))
            .top(px(0.0))
            .bottom(px(0.0))
            .w(handle_hit)
            .when(enabled, |this| this.cursor_col_resize())
            .on_hover(cx.listener(move |this, hovered, _window, cx| {
                this.handle_handle_hover(index, hovered, cx);
            }))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, event, window, cx| {
                    this.handle_handle_mouse_down(index, event, window, cx);
                }),
            )
            .on_drag(drag_payload.clone(), {
                let drag = drag_payload.clone();
                move |_: &ResizablePanelsHandleDrag, _, _, cx| {
                    cx.stop_propagation();
                    cx.new(|_| drag.clone())
                }
            })
            .into_any_element()
    } else {
        div()
            .id(hit_id)
            .absolute()
            .top(px(hit_inset))
            .left(px(0.0))
            .right(px(0.0))
            .h(handle_hit)
            .when(enabled, |this| this.cursor_row_resize())
            .on_hover(cx.listener(move |this, hovered, _window, cx| {
                this.handle_handle_hover(index, hovered, cx);
            }))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, event, window, cx| {
                    this.handle_handle_mouse_down(index, event, window, cx);
                }),
            )
            .on_drag(drag_payload, |drag: &ResizablePanelsHandleDrag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .into_any_element()
    }
}
