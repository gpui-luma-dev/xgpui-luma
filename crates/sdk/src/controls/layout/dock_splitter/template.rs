use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::{Arc, OnceLock};

use gpui::{App, Div, KeyDownEvent, MouseButton, MouseDownEvent, MouseUpEvent, Stateful, Window, div, px, prelude::*};

use super::{DockSplitterLook, DockSplitterDrag, DockSplitterRenderModel, SplitterOrientation};

pub type DockSplitterHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type DockSplitterMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type DockSplitterMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type DockSplitterKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;

pub struct DockSplitterTemplateHandlers {
    pub hover: DockSplitterHoverHandler,
    pub mouse_down: DockSplitterMouseDownHandler,
    pub mouse_up: DockSplitterMouseUpHandler,
    pub mouse_up_out: DockSplitterMouseUpHandler,
    pub key_down: DockSplitterKeyDownHandler,
}

pub trait DockSplitterTemplate: Send + Sync {
    fn render(
        &self,
        model: &DockSplitterRenderModel<'_>,
        look: &DockSplitterLook,
        handlers: DockSplitterTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div>;
}

impl RenderModel for dyn DockSplitterTemplate {
    type Model<'a> = DockSplitterRenderModel<'a>;
}

pub struct ThemedDockSplitterTemplate {
    show_thumb: bool,
    modifiers: RootModifiers<dyn DockSplitterTemplate>,
}

impl ThemedDockSplitterTemplate {
    pub fn new(show_thumb: bool) -> Self {
        Self { show_thumb, modifiers: RootModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &DockSplitterRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl Default for ThemedDockSplitterTemplate {
    fn default() -> Self {
        Self::new(false)
    }
}

pub fn default_dock_splitter_template() -> Arc<dyn DockSplitterTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn DockSplitterTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedDockSplitterTemplate::new(false))).clone()
}

pub(super) fn template_with_modifier<F>(
    template: Arc<dyn DockSplitterTemplate>,
    modifier: F,
) -> Arc<dyn DockSplitterTemplate>
where
    F: Fn(Stateful<Div>, &DockSplitterRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl DockSplitterTemplate for ModifiedTemplate<dyn DockSplitterTemplate> {
    fn render(
        &self,
        model: &DockSplitterRenderModel<'_>,
        look: &DockSplitterLook,
        handlers: DockSplitterTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, look, handlers, window, cx);
        self.apply(root, model)
    }
}

impl DockSplitterTemplate for ThemedDockSplitterTemplate {
    fn render(
        &self,
        model: &DockSplitterRenderModel<'_>,
        look: &DockSplitterLook,
        handlers: DockSplitterTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let root = render_splitter(model, look, handlers, self.show_thumb);
        self.modifiers.apply(root, model)
    }
}

fn render_splitter(
    model: &DockSplitterRenderModel<'_>,
    look: &DockSplitterLook,
    handlers: DockSplitterTemplateHandlers,
    show_thumb: bool,
) -> Stateful<Div> {
    let DockSplitterTemplateHandlers { hover, mouse_down, mouse_up, mouse_up_out, key_down } = handlers;
    let line_color = if model.dragging || model.hovered || model.focused {
        look.hover_color
    } else {
        look.line_color
    };
    let half_inset = ((look.hit_target_px - look.visible_line_px) * 0.5).max(0.0);

    let mut root = div().id((model.id.clone(), 0usize)).relative().flex_shrink_0();

    root = match model.orientation {
        SplitterOrientation::Vertical => root.w(px(look.visible_line_px)).h_full(),
        SplitterOrientation::Horizontal => root.h(px(look.visible_line_px)).w_full(),
    };

    let drag_payload = DockSplitterDrag { id: model.id.clone() };
    let mut hit_target = match model.orientation {
        SplitterOrientation::Vertical => div()
            .id(model.id.clone())
            .absolute()
            .left(px(-half_inset))
            .top(px(0.0))
            .bottom(px(0.0))
            .w(px(look.hit_target_px))
            .track_focus(model.focus_handle)
            .tab_index(if model.enabled { 0 } else { -1 })
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_key_down(key_down)
            .on_drag(drag_payload.clone(), {
                let drag = drag_payload.clone();
                move |_: &DockSplitterDrag, _, _, cx| {
                    cx.stop_propagation();
                    cx.new(|_| drag.clone())
                }
            })
            .when(model.enabled, |this| this.cursor_col_resize())
            .child(
                div()
                    .absolute()
                    .left(px(half_inset))
                    .top(px(0.0))
                    .bottom(px(0.0))
                    .w(px(look.visible_line_px))
                    .bg(line_color),
            ),
        SplitterOrientation::Horizontal => div()
            .id(model.id.clone())
            .absolute()
            .top(px(-half_inset))
            .left(px(0.0))
            .right(px(0.0))
            .h(px(look.hit_target_px))
            .track_focus(model.focus_handle)
            .tab_index(if model.enabled { 0 } else { -1 })
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_key_down(key_down)
            .on_drag(drag_payload, |drag: &DockSplitterDrag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .when(model.enabled, |this| this.cursor_row_resize())
            .child(
                div()
                    .absolute()
                    .top(px(half_inset))
                    .left(px(0.0))
                    .right(px(0.0))
                    .h(px(look.visible_line_px))
                    .bg(line_color),
            ),
    };

    if show_thumb && model.orientation == SplitterOrientation::Vertical {
        hit_target = hit_target.child(
            div().absolute().inset_0().flex().justify_center().items_center().child(
                div()
                    .rounded(px(look.geometry.thumb_radius))
                    .bg(if model.hovered || model.dragging || model.focused {
                        look.thumb_color
                    } else {
                        look.thumb_color.opacity(look.geometry.thumb_rest_opacity)
                    })
                    .w(px(look.geometry.thumb_width))
                    .h(px(look.geometry.thumb_height)),
            ),
        );
    }

    root.child(hit_target)
}
