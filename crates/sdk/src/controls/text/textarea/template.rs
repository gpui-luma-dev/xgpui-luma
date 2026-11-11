use std::sync::{Arc, OnceLock};

use gpui::{
    App, AppContext as _, Div, DragMoveEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Stateful, Window, div, px,
    prelude::*,
};

use super::{TextAreaDrag, TextAreaRenderModel};
use crate::controls::textarea::{TextAreaTheme, default_textarea_theme};
use crate::theme::StandardBoxScale;

const TEXTAREA_TRAILING_HITBOX_WIDTH: f32 = 4.0;
const TEXTAREA_CARET_EDGE_OFFSET: f32 = 0.0;
const TEXTAREA_MIN_WIDTH: f32 = 0.0;

pub type TextAreaHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type TextAreaMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaMouseMoveHandler = Box<dyn Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaKeyDownHandler = Box<dyn Fn(&gpui::KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaClickHandler = Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaDragMoveHandler = Box<dyn Fn(&DragMoveEvent<TextAreaDrag>, &mut Window, &mut App) + 'static>;

pub struct TextAreaTemplateHandlers {
    pub hover: TextAreaHoverHandler,
    pub mouse_down: TextAreaMouseDownHandler,
    pub mouse_move: TextAreaMouseMoveHandler,
    pub mouse_up: TextAreaMouseUpHandler,
    pub mouse_up_out: TextAreaMouseUpHandler,
    pub click: TextAreaClickHandler,
    pub key_down: TextAreaKeyDownHandler,
    pub drag_move: TextAreaDragMoveHandler,
}

pub type TextAreaTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &TextAreaRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait TextAreaTemplate: Send + Sync {
    fn theme(&self) -> Option<Arc<dyn TextAreaTheme>> {
        None
    }

    fn render(
        &self,
        model: &TextAreaRenderModel<'_>,
        handlers: TextAreaTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedTextAreaTemplate {
    theme: Arc<dyn TextAreaTheme>,
    modifiers: Vec<TextAreaTemplateModifier>,
}

impl ThemedTextAreaTemplate {
    pub fn new(theme: Arc<dyn TextAreaTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TextAreaRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TextAreaRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

struct ModifiedTextAreaTemplate {
    base: Arc<dyn TextAreaTemplate>,
    modifiers: Vec<TextAreaTemplateModifier>,
}

impl ModifiedTextAreaTemplate {
    fn new(base: Arc<dyn TextAreaTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: TextAreaTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TextAreaRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

pub fn default_textarea_template() -> Arc<dyn TextAreaTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn TextAreaTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedTextAreaTemplate::new(default_textarea_theme()))).clone()
}

pub(super) fn template_with_modifier<F>(template: Arc<dyn TextAreaTemplate>, modifier: F) -> Arc<dyn TextAreaTemplate>
where
    F: Fn(Stateful<Div>, &TextAreaRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTextAreaTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl TextAreaTemplate for ModifiedTextAreaTemplate {
    fn theme(&self) -> Option<Arc<dyn TextAreaTheme>> {
        self.base.theme()
    }

    fn render(
        &self,
        model: &TextAreaRenderModel<'_>,
        handlers: TextAreaTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }
}

impl TextAreaTemplate for ThemedTextAreaTemplate {
    fn theme(&self) -> Option<Arc<dyn TextAreaTheme>> {
        Some(Arc::clone(&self.theme))
    }

    fn render(
        &self,
        model: &TextAreaRenderModel<'_>,
        handlers: TextAreaTemplateHandlers,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let scale_factor = window.scale_factor();
        let size = model.size;
        let scale = StandardBoxScale::compute(size, &self.theme.metrics(), scale_factor);
        let mut look = self.theme.resolve_look(model.state, model.enabled, size, &scale);
        let show_placeholder = model.value.is_empty() && !model.state.focused;
        let selection = model.state.selection_range();
        let cursor = model.state.cursor.min(model.value.chars().count());
        let caret_height = look.typography.size + look.caret_height_extra;
        let row_height = look.typography.line_height;
        let viewport_height = row_height * model.rows.max(1) as f32;

        let editor_viewport = {
            let mut lines = div().relative().top(px(-model.vertical_scroll)).flex().flex_col();

            for line in &model.line_metrics {
                let line_chars = line.text.chars().collect::<Vec<_>>();
                let mut row = div().relative().h(px(line.height)).flex().items_center().text_color(look.foreground);

                for local_ix in 0..=line_chars.len() {
                    let global_ix = line.start + local_ix;
                    if let Some(ch) = line_chars.get(local_ix) {
                        let selected =
                            selection.map(|(start, end)| global_ix >= start && global_ix < end).unwrap_or(false);
                        let width = line
                            .character_offsets
                            .get(local_ix + 1)
                            .zip(line.character_offsets.get(local_ix))
                            .map(|(next, current)| (next - current).max(0.0))
                            .unwrap_or(0.0);

                        row = row.child(
                            div()
                                .relative()
                                .flex_none()
                                .w(px(width))
                                .h(px(caret_height))
                                .flex()
                                .items_center()
                                .when(selected, |cell| {
                                    cell.bg(look.selection_background).text_color(look.selection_foreground)
                                })
                                .child(ch.to_string())
                                .when(
                                    model.enabled && model.caret_visible && cursor == global_ix && selection.is_none(),
                                    |cell| {
                                        cell.child(
                                            div()
                                                .absolute()
                                                .left(px(TEXTAREA_CARET_EDGE_OFFSET))
                                                .top(px(TEXTAREA_CARET_EDGE_OFFSET))
                                                .w(px(look.caret_width))
                                                .h(px(caret_height))
                                                .bg(look.caret),
                                        )
                                    },
                                ),
                        );
                    } else {
                        row = row.child(
                            div()
                                .relative()
                                .flex_none()
                                .w(px(TEXTAREA_TRAILING_HITBOX_WIDTH))
                                .h(px(caret_height))
                                .when(
                                    model.enabled && model.caret_visible && cursor == global_ix && selection.is_none(),
                                    |cell| {
                                        cell.child(
                                            div()
                                                .absolute()
                                                .left(px(TEXTAREA_CARET_EDGE_OFFSET))
                                                .top(px(TEXTAREA_CARET_EDGE_OFFSET))
                                                .w(px(look.caret_width))
                                                .h(px(caret_height))
                                                .bg(look.caret),
                                        )
                                    },
                                ),
                        );
                    }
                }

                lines = lines.child(row);
            }

            div().min_w(px(TEXTAREA_MIN_WIDTH)).w_full().h(px(viewport_height)).overflow_hidden().child(lines)
        };

        // Placeholder visibility changes paint only, including for intrinsic-width
        // templates. Both empty and populated editors retain the same sizing anchor.
        let text_viewport = div()
            .relative()
            .min_w(px(TEXTAREA_MIN_WIDTH))
            .w_full()
            .h(px(viewport_height))
            .overflow_hidden()
            .text_color(look.placeholder)
            .child(div().opacity(if show_placeholder { 1.0 } else { 0.0 }).child(model.placeholder.clone()))
            .child(editor_viewport.absolute().inset_0());

        let mut control = div()
            .id("control")
            .relative()
            .h(px(look.padding_y * 2.0 + viewport_height))
            .flex()
            .items_start()
            .px(px(look.padding_x))
            .py(px(look.padding_y))
            .bg(look.background)
            .border(px(look.border_width))
            .border_color(look.border)
            .rounded(px(look.radius))
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .font_family(look.font_family.clone())
            .font_weight(look.typography.weight)
            .when(model.full_width, |root| root.w_full())
            .when(model.enabled, |root| root.cursor_text())
            .when(!model.enabled, |root| root.cursor_not_allowed().opacity(look.disabled_opacity))
            .child(text_viewport);

        if model.enabled
            && let Some(shadows) = look.shadow.take().filter(|shadows| !shadows.is_empty())
        {
            control = control.shadow(shadows);
        }

        let focused = model.state.focused && model.state.focus_visible && model.enabled;
        let focus = crate::infra::focus_adorner::FocusAdorner::new(self.theme.metrics().focus);
        let focus_extent = focus.extent();
        let adorned = div().id("adorned").relative().child(control);
        let mut root = div().id(model.id.clone()).relative();
        if focus_extent > 0.0 {
            root = root.p(px(focus_extent)).child(adorned);
        } else {
            root = root.child(adorned);
        }

        if model.full_width {
            root = root.w_full();
        }

        root = root.child(focus.overlay(look.focus_border, focused, look.radius, false));

        self.apply_modifiers(root, model)
            .on_hover(handlers.hover)
            .on_mouse_down(gpui::MouseButton::Left, handlers.mouse_down)
            .on_mouse_move(handlers.mouse_move)
            .on_mouse_up(gpui::MouseButton::Left, handlers.mouse_up)
            .on_mouse_up_out(gpui::MouseButton::Left, handlers.mouse_up_out)
            .on_click(handlers.click)
            .on_key_down(handlers.key_down)
            .on_drag(TextAreaDrag::new(model.id.clone()), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(handlers.drag_move)
    }
}
