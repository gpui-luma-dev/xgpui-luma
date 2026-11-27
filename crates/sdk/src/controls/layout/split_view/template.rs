use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, ClickEvent, Div, DragMoveEvent, Hsla, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Stateful,
    Window, div, prelude::*, px,
};

use super::{SplitViewRenderModel, SplitViewSeparatorVisibility, control::SplitViewSeparatorDrag};

const CENTERED_CUE_OFFSET_FACTOR: f32 = 0.5;

pub type SplitViewHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SplitViewMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SplitViewMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type SplitViewClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SplitViewDragMoveHandler =
    Box<dyn Fn(&DragMoveEvent<SplitViewSeparatorDrag>, &mut Window, &mut App) + 'static>;

pub struct SplitViewTemplateHandlers {
    pub separator_hover: SplitViewHoverHandler,
    pub separator_mouse_down: SplitViewMouseDownHandler,
    pub separator_click: SplitViewClickHandler,
    pub mouse_up: SplitViewMouseUpHandler,
    pub mouse_up_out: SplitViewMouseUpHandler,
    pub drag_move: SplitViewDragMoveHandler,
}

pub type SplitViewTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &SplitViewRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait SplitViewTemplate: Send + Sync {
    fn render(
        &self,
        model: &SplitViewRenderModel<'_>,
        sidebar: AnyElement,
        content: AnyElement,
        handlers: SplitViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

impl RenderModel for dyn SplitViewTemplate {
    type Model<'a> = SplitViewRenderModel<'a>;
}

pub struct ThemedSplitViewTemplate {
    modifiers: RootModifiers<dyn SplitViewTemplate>,
}

impl ThemedSplitViewTemplate {
    pub fn new() -> Self {
        Self { modifiers: RootModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &SplitViewRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl Default for ThemedSplitViewTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_split_view_template() -> Arc<dyn SplitViewTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SplitViewTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedSplitViewTemplate::new())).clone()
}

pub(super) fn modified_split_view_template<F>(
    template: Arc<dyn SplitViewTemplate>,
    modifier: F,
) -> Arc<dyn SplitViewTemplate>
where
    F: Fn(Stateful<Div>, &SplitViewRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl SplitViewTemplate for ModifiedTemplate<dyn SplitViewTemplate> {
    fn render(
        &self,
        model: &SplitViewRenderModel<'_>,
        sidebar: AnyElement,
        content: AnyElement,
        handlers: SplitViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, sidebar, content, handlers, window, cx);
        self.apply(root, model)
    }
}

impl SplitViewTemplate for ThemedSplitViewTemplate {
    fn render(
        &self,
        model: &SplitViewRenderModel<'_>,
        sidebar: AnyElement,
        content: AnyElement,
        handlers: SplitViewTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let SplitViewTemplateHandlers {
            separator_hover,
            separator_mouse_down,
            separator_click,
            mouse_up,
            mouse_up_out,
            drag_move,
        } = handlers;
        let look = model.theme.resolve(model.separator_hovered, model.enabled);
        let separator_color = model.separator_color_override.unwrap_or(look.separator);
        let separator_hover_color = model.separator_hover_color_override.unwrap_or(look.separator_hover);
        let cue_color = match model.separator_visibility {
            SplitViewSeparatorVisibility::Always if model.separator_hovered => Some(separator_hover_color),
            SplitViewSeparatorVisibility::Always => Some(separator_color),
            SplitViewSeparatorVisibility::Hover if model.separator_hovered => Some(separator_hover_color),
            SplitViewSeparatorVisibility::Hover => None,
        };
        let cue_width = separator_cue_width(model.separator_hovered, &look.geometry);
        let resize_enabled = model.enabled && model.resizable && !model.collapsed;

        let mut row = div()
            .size_full()
            .flex()
            .child(div().h_full().w(model.effective_sidebar_width).flex_none().overflow_hidden().child(sidebar));

        let expand_separator = if model.collapsed {
            Some(
                div()
                    .id("expand-separator")
                    .absolute()
                    .left(collapsed_expand_separator_left(model.effective_sidebar_width))
                    .top(px(look.geometry.expand_inset_y))
                    .bottom(px(look.geometry.expand_inset_y))
                    .w(px(look.geometry.hitbox_width))
                    .on_hover(separator_hover)
                    .on_click(separator_click)
                    .cursor_e_resize()
                    .child(render_separator_cue(look.geometry.expand_cue_left, cue_width, cue_color, &look.geometry)),
            )
        } else {
            let mut separator = div()
                .id("separator")
                .relative()
                .h_full()
                .w(px(look.geometry.hitbox_width))
                .flex_none()
                .on_hover(separator_hover)
                .on_click(separator_click)
                .on_mouse_down(MouseButton::Left, separator_mouse_down)
                .when(resize_enabled, |this| this.cursor_col_resize())
                .when(!resize_enabled, |this| this.cursor_pointer())
                .child(render_separator_cue(
                    centered_separator_cue_left(cue_width, look.geometry.hitbox_width),
                    cue_width,
                    cue_color,
                    &look.geometry,
                ));

            if resize_enabled {
                separator = separator.on_drag(
                    SplitViewSeparatorDrag::new(model.id.clone()),
                    |drag: &SplitViewSeparatorDrag, _, _, cx: &mut App| {
                        cx.stop_propagation();
                        cx.new(|_| drag.clone())
                    },
                );
            }

            row = row.child(separator);
            None
        };

        row = row.child(div().h_full().flex_1().overflow_hidden().child(content));

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .size_full()
            .overflow_hidden()
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_drag_move(drag_move)
            .child(row);

        if let Some(expand_separator) = expand_separator {
            root = root.child(expand_separator);
        }

        self.modifiers.apply(root, model)
    }
}

fn separator_cue_width(hovered: bool, geometry: &crate::theme::stylesheet::SplitViewGeometry) -> f32 {
    if hovered {
        geometry.cue_hovered_width
    } else {
        geometry.cue_width
    }
}

fn centered_separator_cue_left(width: f32, hitbox_width: f32) -> f32 {
    (hitbox_width - width) * CENTERED_CUE_OFFSET_FACTOR
}

pub(crate) fn collapsed_expand_separator_left(effective_sidebar_width: Pixels) -> Pixels {
    effective_sidebar_width.max(px(0.0))
}

fn render_separator_cue(
    left: f32,
    width: f32,
    color: Option<Hsla>,
    geometry: &crate::theme::stylesheet::SplitViewGeometry,
) -> Div {
    let mut cue = div()
        .absolute()
        .left(px(left))
        .top(px(geometry.cue_inset_y))
        .bottom(px(geometry.cue_inset_y))
        .w(px(width))
        .rounded(px(geometry.cue_radius));

    if let Some(color) = color {
        cue = cue.bg(color);
    }

    cue
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use super::collapsed_expand_separator_left;

    #[test]
    fn collapsed_expand_separator_starts_at_effective_sidebar_edge() {
        assert_eq!(collapsed_expand_separator_left(px(0.0)), px(0.0));
        assert_eq!(collapsed_expand_separator_left(px(56.0)), px(56.0));
    }
}
