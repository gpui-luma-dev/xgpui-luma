//! Presentation for [`super::ColorStop`].
//!
//! The control owns state, child controls and input; a template only arranges
//! what the render model provides. Interactive children (`position_field`,
//! `remove_button`, `editor`) arrive pre-rendered and already isolated from the
//! surrounding row, so a template cannot break their input handling.

use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Bounds, ClickEvent, IntoElement, Pixels, SharedString, Window, div, prelude::*, px};
use xgpui_luma::color::ColorValue;
use xgpui_luma::infra::ElementExt;
use xgpui_luma_color::ColorSwatch;

use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};

pub(crate) type ColorStopClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub(crate) type ColorStopBoundsHandler = Box<dyn Fn(Bounds<Pixels>, &mut Window, &mut App) + 'static>;

/// Behavior the template attaches to its own elements.
pub(crate) struct ColorStopTemplateHandlers {
    /// Attach to the swatch (or whatever the template uses to open the editor).
    pub swatch_click: ColorStopClickHandler,
    /// Attach to the root (e.g. via `on_prepaint`) so the control knows its
    /// laid-out size; lists use it to size their viewport.
    pub bounds: ColorStopBoundsHandler,
}

// Template data: not every template reads every field.
#[allow(dead_code)]
pub(crate) struct ColorStopRenderModel<'a> {
    pub id: &'a SharedString,
    pub look: &'a Look,
    pub color: ColorValue,
    /// sRGB hex, e.g. `#A73B93`.
    pub label: SharedString,
    pub alpha: f32,
    /// Normalized `0.0..=1.0`; `None` when the stop has no position.
    pub position: Option<f32>,
    /// Zero-based order within a parent list; `None` when standalone.
    pub index: Option<usize>,
    pub selected: bool,
    pub expanded: bool,
    /// Editable position, when the stop is positioned.
    pub position_field: Option<AnyElement>,
    /// Remove action, when the stop is removable.
    pub remove_button: Option<AnyElement>,
    /// Color editor; present only while expanded.
    pub editor: Option<AnyElement>,
}

pub(crate) trait ColorStopTemplate: Send + Sync {
    fn render(
        &self,
        model: ColorStopRenderModel<'_>,
        handlers: ColorStopTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement;
}

struct FnColorStopTemplate<F>(F);

impl<F, E> ColorStopTemplate for FnColorStopTemplate<F>
where
    F: for<'a> Fn(ColorStopRenderModel<'a>, ColorStopTemplateHandlers, &mut Window, &mut App) -> E + Send + Sync,
    E: IntoElement,
{
    fn render(
        &self,
        model: ColorStopRenderModel<'_>,
        handlers: ColorStopTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        (self.0)(model, handlers, window, cx).into_any_element()
    }
}

/// Adapt a closure or function to a [`ColorStopTemplate`].
pub(crate) fn make_color_stop_template<F, E>(template: F) -> Arc<dyn ColorStopTemplate>
where
    F: for<'a> Fn(ColorStopRenderModel<'a>, ColorStopTemplateHandlers, &mut Window, &mut App) -> E
        + Send
        + Sync
        + 'static,
    E: IntoElement + 'static,
{
    Arc::new(FnColorStopTemplate(template))
}

pub(crate) fn default_color_stop_template() -> Arc<dyn ColorStopTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ColorStopTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultColorStopTemplate)).clone()
}

/// Swatch · hex + alpha · position · remove, with the editor in a panel below.
/// The remove action shows on hover or while selected.
pub(crate) struct DefaultColorStopTemplate;

/// Height of the default template's single line.
pub(crate) const DEFAULT_LINE_HEIGHT: f32 = 32.0;
const SWATCH_SIZE: f32 = 26.0;
const POSITION_WIDTH: f32 = 60.0;

impl ColorStopTemplate for DefaultColorStopTemplate {
    fn render(
        &self,
        model: ColorStopRenderModel<'_>,
        handlers: ColorStopTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement {
        let group: SharedString = format!("{}-group", model.id).into();
        let line = div()
            .w_full()
            .h(px(DEFAULT_LINE_HEIGHT))
            .flex()
            .items_center()
            .gap_2()
            .child(swatch(&model, handlers.swatch_click))
            .child(label(&model))
            .when_some(model.position_field, |line, field| {
                line.child(div().flex_shrink_0().w(px(POSITION_WIDTH)).child(field))
            })
            .when_some(model.remove_button, |line, button| {
                line.child(
                    div()
                        .flex_shrink_0()
                        .when(!model.selected, |slot| {
                            slot.invisible().group_hover(group.clone(), |style| style.visible())
                        })
                        .child(button),
                )
            });

        div()
            .group(group)
            .relative()
            .w_full()
            .flex()
            .flex_col()
            .child(line)
            .when_some(model.editor, |root, editor| root.child(editor_panel(model.look, editor)))
            .on_prepaint(move |bounds, window, cx| (handlers.bounds)(bounds, window, cx))
            .into_any_element()
    }
}

// Building blocks shared with custom templates.

/// Clickable color swatch with checkerboard for translucent colors.
pub(crate) fn swatch(model: &ColorStopRenderModel<'_>, on_click: ColorStopClickHandler) -> impl IntoElement {
    div()
        .id(format!("{}-swatch", model.id))
        .flex_shrink_0()
        .w(px(SWATCH_SIZE))
        .cursor_pointer()
        .on_click(move |event, window, cx| on_click(event, window, cx))
        .child(ColorSwatch::new(model.color).checkerboard(true).height(px(SWATCH_SIZE)).rounded(px(6.0)))
}

/// Hex label with muted alpha, filling the remaining width.
pub(crate) fn label(model: &ColorStopRenderModel<'_>) -> impl IntoElement {
    let chrome = model.look.chrome();
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .items_center()
        .gap_2()
        .typography_style(model.look.typography_scale(TextSize::Sm))
        .child(div().text_color(chrome.body_text).child(model.label.clone()))
        .child(div().text_color(chrome.muted_text).child(format!("{}%", (model.alpha * 100.0).round() as i32)))
}

/// Card around the editor.
pub(crate) fn editor_panel(look: &Look, editor: AnyElement) -> impl IntoElement {
    let chrome = look.chrome();
    div().w_full().pt(px(2.0)).pb(px(6.0)).child(
        div()
            .w_full()
            .p(px(10.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.content_background)
            .child(editor),
    )
}
