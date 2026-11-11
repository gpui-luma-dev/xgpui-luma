use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, Div, DragMoveEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Stateful, TextAlign,
    Window, canvas, div, fill, font, point, px, prelude::*, size, svg,
};

use super::{TextFieldDrag, TextFieldLook, TextFieldRenderModel, TextFieldState};
use crate::controls::button::ControlIcon;
use crate::controls::textfield::{TextFieldTheme, default_textfield_theme};
use crate::theme::{ControlSize, StandardBoxScale};

const CARET_EDGE_OFFSET: f32 = 0.0;

fn char_to_byte_offset(text: &str, char_offset: usize) -> usize {
    text.chars().take(char_offset).map(char::len_utf8).sum()
}

pub type TextFieldHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type TextFieldMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type TextFieldMouseMoveHandler = Box<dyn Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static>;
pub type TextFieldMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type TextFieldKeyDownHandler = Box<dyn Fn(&gpui::KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type TextFieldClickHandler = Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>;
pub type TextFieldDragMoveHandler = Box<dyn Fn(&DragMoveEvent<TextFieldDrag>, &mut Window, &mut App) + 'static>;

pub struct TextFieldTemplateHandlers {
    pub hover: TextFieldHoverHandler,
    pub mouse_down: TextFieldMouseDownHandler,
    pub mouse_move: TextFieldMouseMoveHandler,
    pub mouse_up: TextFieldMouseUpHandler,
    pub mouse_up_out: TextFieldMouseUpHandler,
    pub click: TextFieldClickHandler,
    pub key_down: TextFieldKeyDownHandler,
    pub drag_move: TextFieldDragMoveHandler,
}

pub type TextFieldTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait TextFieldTemplate: Send + Sync {
    fn resolve_look(&self, state: TextFieldState, enabled: bool, size: ControlSize) -> TextFieldLook {
        default_textfield_theme().resolve_look(
            state,
            enabled,
            size,
            &StandardBoxScale::compute(size, &default_textfield_theme().metrics(), 1.0),
        )
    }

    fn resolve_look_with_scale(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        _scale_factor: f32,
        _cx: &mut App,
    ) -> TextFieldLook {
        self.resolve_look(state, enabled, size)
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedTextFieldTemplate {
    theme: Arc<dyn TextFieldTheme>,
    modifiers: Vec<TextFieldTemplateModifier>,
}

impl ThemedTextFieldTemplate {
    pub fn new(theme: Arc<dyn TextFieldTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TextFieldRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

struct ModifiedTextFieldTemplate {
    base: Arc<dyn TextFieldTemplate>,
    modifiers: Vec<TextFieldTemplateModifier>,
}

impl ModifiedTextFieldTemplate {
    fn new(base: Arc<dyn TextFieldTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: TextFieldTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TextFieldRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

pub fn default_textfield_template() -> Arc<dyn TextFieldTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn TextFieldTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedTextFieldTemplate::new(default_textfield_theme()))).clone()
}

pub(super) fn template_with_modifier<F>(template: Arc<dyn TextFieldTemplate>, modifier: F) -> Arc<dyn TextFieldTemplate>
where
    F: Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTextFieldTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl TextFieldTemplate for ModifiedTextFieldTemplate {
    fn resolve_look(&self, state: TextFieldState, enabled: bool, size: ControlSize) -> TextFieldLook {
        self.base.resolve_look(state, enabled, size)
    }

    fn resolve_look_with_scale(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale_factor: f32,
        cx: &mut App,
    ) -> TextFieldLook {
        self.base.resolve_look_with_scale(state, enabled, size, scale_factor, cx)
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }
}

impl TextFieldTemplate for ThemedTextFieldTemplate {
    fn resolve_look(&self, state: TextFieldState, enabled: bool, size: ControlSize) -> TextFieldLook {
        self.theme
            .resolve_look(state, enabled, size, &StandardBoxScale::compute(size, &self.theme.metrics(), 1.0))
    }

    fn resolve_look_with_scale(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale_factor: f32,
        _cx: &mut App,
    ) -> TextFieldLook {
        let scale = StandardBoxScale::compute(size, &self.theme.metrics(), scale_factor);
        self.theme.resolve_look(state, enabled, size, &scale)
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let mut look = model.look.clone();
        let show_placeholder = model.value.is_empty() && !model.state.focused;
        let chars = model.value.chars().collect::<Vec<_>>();
        let cursor = model.state.cursor.min(chars.len());
        let selection = model.state.selection_range();
        let caret_height = look.typography.size + look.caret_height_extra;

        let mut text_viewport = if show_placeholder {
            div()
                .min_w(px(0.0))
                .flex_1()
                .flex()
                .items_center()
                .overflow_hidden()
                .text_color(look.placeholder)
                .text_size(px(look.typography.size))
                .line_height(px(look.typography.line_height))
                .font_family(look.font_family.clone())
                .font_weight(look.typography.weight)
                .child(
                    div()
                        .h(px(caret_height))
                        .flex()
                        .items_center()
                        .text_size(px(look.typography.size))
                        .line_height(px(look.typography.line_height))
                        .font_family(look.font_family.clone())
                        .font_weight(look.typography.weight)
                        .child(model.placeholder.clone()),
                )
        } else {
            let value = model.value.clone();
            let marked_range = model.marked_range.clone();
            let horizontal_scroll = model.horizontal_scroll;
            let enabled = model.enabled;
            let caret_visible = model.caret_visible;
            let caret = cursor;
            let line_height = px(look.typography.line_height);
            let canvas_look = look.clone();

            div().min_w(px(0.0)).flex_1().h(px(look.typography.line_height)).overflow_hidden().child(
                canvas(
                    move |bounds, window, _| {
                        let line = window.text_system().shape_line(
                            value.clone(),
                            px(canvas_look.typography.size),
                            &crate::controls::text::runs::colored_runs(
                                value.as_ref(),
                                0,
                                selection,
                                marked_range
                                    .as_ref()
                                    .map(|range| (range, canvas_look.helper_geometry.composition_underline_width)),
                                {
                                    let mut font = font(canvas_look.font_family.clone());
                                    font.weight = canvas_look.typography.weight;
                                    font
                                },
                                canvas_look.foreground,
                                canvas_look.selection_foreground,
                            ),
                            None,
                        );
                        let selection_quad = selection.and_then(|(start, end)| {
                            let start_x = line.x_for_index(char_to_byte_offset(value.as_ref(), start));
                            let end_x = line.x_for_index(char_to_byte_offset(value.as_ref(), end));
                            (end_x > start_x).then(|| {
                                fill(
                                    Bounds::from_corners(
                                        point(bounds.left() + start_x - px(horizontal_scroll), bounds.top()),
                                        point(bounds.left() + end_x - px(horizontal_scroll), bounds.bottom()),
                                    ),
                                    canvas_look.selection_background,
                                )
                            })
                        });
                        let caret_quad = if enabled && caret_visible && selection.is_none() {
                            let x = line.x_for_index(char_to_byte_offset(value.as_ref(), caret));
                            Some(fill(
                                Bounds::new(
                                    point(bounds.left() + x - px(horizontal_scroll + CARET_EDGE_OFFSET), bounds.top()),
                                    size(px(canvas_look.caret_width), bounds.size.height),
                                ),
                                canvas_look.caret,
                            ))
                        } else {
                            None
                        };
                        (line, selection_quad, caret_quad)
                    },
                    move |bounds, (line, selection_quad, caret_quad), window, cx| {
                        if let Some(selection_quad) = selection_quad {
                            window.paint_quad(selection_quad);
                        }
                        line.paint(
                            point(bounds.left() - px(horizontal_scroll), bounds.top()),
                            line_height,
                            TextAlign::Left,
                            None,
                            window,
                            cx,
                        )
                        .ok();
                        if let Some(caret_quad) = caret_quad {
                            window.paint_quad(caret_quad);
                        }
                    },
                )
                .size_full(),
            )
        };

        if let Some(icon) = model.prefix_icon {
            text_viewport = div()
                .min_w(px(0.0))
                .flex_1()
                .flex()
                .items_center()
                .gap(px(look.gap))
                .child(render_prefix_icon(icon, look.icon, look.icon_size))
                .child(text_viewport);
        }

        // Always fill the focus-ring root so a stretched ring cannot leave a content-sized
        // entry box centered inside it (e.g. fixed-width hosts without full_width).
        let focused = model.state.focused && model.state.focus_visible && model.enabled;
        let focus = crate::infra::focus_adorner::FocusAdorner::new(self.theme.metrics().focus);
        let focus_extent = focus.extent();

        let mut control = div()
            .id("control")
            .relative()
            .w_full()
            .min_w(px(0.0))
            .h(px(look.min_height))
            .flex()
            .items_center()
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
            .when(model.enabled, |root| root.cursor_text())
            .when(!model.enabled, |root| root.cursor_not_allowed().opacity(look.disabled_opacity))
            .child(text_viewport);

        if model.enabled
            && let Some(shadows) = look.shadow.take().filter(|shadows| !shadows.is_empty())
        {
            control = control.shadow(shadows);
        }

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

        root = root
            .on_hover(handlers.hover)
            .on_mouse_down(gpui::MouseButton::Left, handlers.mouse_down)
            .on_mouse_move(handlers.mouse_move)
            .on_mouse_up(gpui::MouseButton::Left, handlers.mouse_up)
            .on_mouse_up_out(gpui::MouseButton::Left, handlers.mouse_up_out)
            .on_click(handlers.click)
            .on_key_down(handlers.key_down)
            .on_drag(TextFieldDrag::new(model.id.clone()), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(handlers.drag_move);

        root = root.child(focus.overlay(look.focus_border, focused, look.radius, false));

        self.apply_modifiers(root, model)
    }
}

fn render_prefix_icon(icon: &ControlIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    match icon {
        ControlIcon::Lucide(icon) => div()
            .size(px(size))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(size))
            .line_height(px(size))
            .text_color(color)
            .child(crate::infra::icon::lucide_icon(*icon, color, size))
            .into_any_element(),
        ControlIcon::SvgPath(path) => svg().size(px(size)).text_color(color).path(path.clone()).into_any_element(),
    }
}
