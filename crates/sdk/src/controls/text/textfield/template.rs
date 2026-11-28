use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, Div, DragMoveEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Stateful, TextAlign,
    Window, canvas, div, fill, font, point, px, prelude::*, size, svg,
};

use super::{TextFieldDrag, TextFieldRenderModel, TextFieldState};
use crate::controls::text::{input_metrics, input_typography, resolve_text_field};
use crate::style::{PartRead, StyleBinding};
use crate::style::lss::Overrides;
use xgpui_luma_style::Resolved;
use crate::controls::button::ControlIcon;
use crate::controls::textfield::{TextFieldTheme, default_textfield_theme};

const CARET_EDGE_OFFSET: f32 = 0.0;

/// The values a text field draws, read from its resolved style (#433).
#[derive(Clone)]
pub(super) struct FieldValues {
    pub helper_geometry: crate::theme::stylesheet::SharedGeometry,
    pub caret_width: f32,
    pub opacity: f32,
    pub background: gpui::Hsla,
    pub foreground: gpui::Hsla,
    pub border: gpui::Hsla,
    pub focus_border: gpui::Hsla,
    pub placeholder: gpui::Hsla,
    pub icon: gpui::Hsla,
    pub selection_background: gpui::Hsla,
    pub selection_foreground: gpui::Hsla,
    pub caret: gpui::Hsla,
    pub shadow: Option<Vec<gpui::BoxShadow>>,
    pub typography: crate::theme::LumaTextStyle,
    pub font_family: gpui::SharedString,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub radius: f32,
    pub border_width: f32,
    pub icon_size: f32,
    pub focus: crate::theme::FocusMetricTokens,
}

impl FieldValues {
    pub(super) fn read(resolved: &Resolved) -> Self {
        let root = resolved.root();
        Self {
            helper_geometry: crate::theme::stylesheet::SharedGeometry::sdk(),
            caret_width: root.number("caret_width"),
            opacity: root.opacity(),
            background: root.color("background"),
            foreground: root.color("foreground"),
            border: root.color("border"),
            focus_border: root.color("focus_border"),
            placeholder: root.color("placeholder"),
            icon: root.color("icon"),
            selection_background: root.color("selection_background"),
            selection_foreground: root.color("selection_foreground"),
            caret: root.color("caret"),
            shadow: root.box_shadow_opt(),
            typography: input_typography(root),
            font_family: root.font_family_name(),
            min_height: root.height().unwrap_or(0.0),
            padding_x: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            gap: root.gap().unwrap_or(0.0),
            radius: root.radius().unwrap_or(0.0),
            border_width: root.number("border_width"),
            icon_size: root.number("icon_size"),
            focus: input_metrics(root).focus,
        }
    }
}

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

pub trait TextFieldTemplate: Send + Sync {
    /// The `textfield` binding the field draws from (#433).
    fn style(&self) -> StyleBinding {
        default_textfield_theme().style()
    }

    /// The field's resolved style in `state`, with `overrides` set over it (#433): the field draws from its root.
    fn resolve_style(&self, state: TextFieldState, enabled: bool, overrides: &Overrides) -> Arc<Resolved> {
        resolve_text_field(&self.style().with_overrides(overrides), state, enabled)
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

impl RenderModel for dyn TextFieldTemplate {
    type Model<'a> = TextFieldRenderModel<'a>;
}

pub struct ThemedTextFieldTemplate {
    theme: Arc<dyn TextFieldTheme>,
    modifiers: RootModifiers<dyn TextFieldTemplate>,
}

impl ThemedTextFieldTemplate {
    pub fn new(theme: Arc<dyn TextFieldTheme>) -> Self {
        Self { theme, modifiers: RootModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
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
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl TextFieldTemplate for ModifiedTemplate<dyn TextFieldTemplate> {
    fn style(&self) -> StyleBinding {
        self.base.style()
    }

    fn resolve_style(&self, state: TextFieldState, enabled: bool, overrides: &Overrides) -> Arc<Resolved> {
        self.base.resolve_style(state, enabled, overrides)
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply(root, model)
    }
}

impl TextFieldTemplate for ThemedTextFieldTemplate {
    fn style(&self) -> StyleBinding {
        self.theme.style()
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let mut look = FieldValues::read(&model.style);
        let show_placeholder = model.value.is_empty() && !model.state.focused;
        let chars = model.value.chars().collect::<Vec<_>>();
        let cursor = model.state.cursor.min(chars.len());
        let selection = model.state.selection_range();

        let editor_viewport = {
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

        // Keep intrinsic placeholder measurement independent of focus. The editor
        // paints over the same viewport instead of replacing its layout subtree.
        let mut text_viewport = div()
            .relative()
            .min_w(px(0.0))
            .flex_1()
            .h(px(look.typography.line_height))
            .overflow_hidden()
            .text_color(look.placeholder)
            .child(div().opacity(if show_placeholder { 1.0 } else { 0.0 }).child(model.placeholder.clone()))
            .child(editor_viewport.absolute().inset_0());

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
        let focus = crate::infra::focus_adorner::FocusAdorner::new(look.focus);
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
            .when(!model.enabled, |root| root.cursor_not_allowed())
            .map(|root| crate::style::fade(root, look.opacity))
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

        self.modifiers.apply(root, model)
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
