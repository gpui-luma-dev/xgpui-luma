//! Presentation for [`super::ColorStop`].
//!
//! The control owns state, child controls and input; a template only arranges
//! what the render model provides. Interactive children (`remove_button`,
//! `editor`) arrive pre-rendered and already isolated from the
//! surrounding row, so a template cannot break their input handling.

use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Div, IntoElement, MouseButton, Pixels, SharedString, Stateful, Window, div,
    prelude::*, px,
};
use xgpui_luma::color::ColorValue;
use xgpui_luma::infra::ElementExt;
use xgpui_luma::infra::attachments::TooltipHandle;
use xgpui_luma::theme::InteractionState;
use xgpui_luma_color::ColorSwatch;

use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};

pub(crate) type ColorStopClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub(crate) type ColorStopBoundsHandler = Box<dyn Fn(Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub(crate) type ColorStopHoverHandler = Box<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// Behavior the template attaches to its own elements.
pub(crate) struct ColorStopTemplateHandlers {
    /// Attach to the swatch (or whatever the template uses to toggle the editor).
    pub swatch_click: ColorStopClickHandler,
    /// Swatch hover, which drives its tooltip.
    pub swatch_hover: ColorStopHoverHandler,
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
    /// CSS `hsl()` text with alpha, e.g. `hsl(312 48% 44% / 100%)`; see [`super::stop_label`].
    pub label: SharedString,
    /// Out-of-sRGB indicator source; see [`super::StopGamut`].
    pub gamut: super::StopGamut,
    pub alpha: f32,
    /// Normalized `0.0..=1.0`; `None` when the stop has no position.
    pub position: Option<f32>,
    /// Zero-based order within a parent list; `None` when standalone.
    pub index: Option<usize>,
    pub selected: bool,
    pub expanded: bool,
    /// Trailing text: the position (e.g. `50%`) for positioned stops, otherwise
    /// any host-supplied detail (e.g. mesh point coordinates).
    pub detail_label: Option<SharedString>,
    /// Remove action, when the stop is removable.
    pub remove_button: Option<AnyElement>,
    /// Color editor; present only while expanded.
    pub editor: Option<AnyElement>,
    /// Swatch help ("Edit color" / "Close color editor"), for the current state.
    pub swatch_tooltip: Option<TooltipHandle>,
    pub swatch_hovered: bool,
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

/// Swatch · `hsl(h s l / a)` · gamut badge · position · remove, with the
/// editor in a panel below.
/// The remove action shows on hover or while selected.
pub(crate) struct DefaultColorStopTemplate;

/// Height of the default template's single line.
pub(crate) const DEFAULT_LINE_HEIGHT: f32 = 32.0;
const SWATCH_SIZE: f32 = 26.0;
const DETAIL_MIN_WIDTH: f32 = 60.0;

impl ColorStopTemplate for DefaultColorStopTemplate {
    fn render(
        &self,
        model: ColorStopRenderModel<'_>,
        handlers: ColorStopTemplateHandlers,
        _window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let group: SharedString = format!("{}-group", model.id).into();
        let line = div()
            .w_full()
            .h(px(DEFAULT_LINE_HEIGHT))
            .flex()
            .items_center()
            .gap_2()
            .child(swatch(&model, handlers.swatch_click, handlers.swatch_hover, cx))
            .child(label(&model))
            .children(gamut_badge(&model))
            .children(detail_label(&model))
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

/// Color swatch (checkerboard for translucent colors) that toggles the editor,
/// with its tooltip. The press stays with the swatch, so the surrounding row
/// neither takes focus from an open editor nor starts a drag.
pub(crate) fn swatch(
    model: &ColorStopRenderModel<'_>,
    on_click: ColorStopClickHandler,
    on_hover: ColorStopHoverHandler,
    cx: &App,
) -> Stateful<Div> {
    let id: SharedString = format!("{}-swatch", model.id).into();
    let selector = id.to_string();
    let root = div()
        .id(id)
        .debug_selector(move || selector.clone())
        .flex_shrink_0()
        .w(px(SWATCH_SIZE))
        .cursor_pointer()
        .on_hover(move |hovered, window, cx| on_hover(*hovered, window, cx))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(move |event, window, cx| {
            cx.stop_propagation();
            on_click(event, window, cx);
        })
        .child(ColorSwatch::new(model.color).checkerboard(true).height(px(SWATCH_SIZE)).rounded(px(6.0)));
    match &model.swatch_tooltip {
        Some(tooltip) => {
            tooltip.render(root, InteractionState { hovered: model.swatch_hovered, ..Default::default() }, cx)
        }
        None => root,
    }
}

/// `hsl(h s l / a)` label filling the remaining width; truncates rather than
/// running into the trailing badge/detail.
pub(crate) fn label(model: &ColorStopRenderModel<'_>) -> impl IntoElement {
    let chrome = model.look.chrome();
    div()
        .flex_1()
        .min_w_0()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .typography_style(model.look.typography_scale(TextSize::Sm))
        .text_color(chrome.body_text)
        .child(model.label.clone())
}

/// Out-of-sRGB indicator: `P3` for Display P3 colors outside sRGB, `> P3`
/// beyond that. sRGB-only renderers show such colors gamut-mapped.
pub(crate) fn gamut_badge(model: &ColorStopRenderModel<'_>) -> Option<impl IntoElement> {
    model.gamut.badge().map(|text| {
        let selector: SharedString = format!("{}-gamut", model.id).into();
        div().flex_shrink_0().debug_selector(move || selector.to_string()).child(
            xgpui_luma_look_radix::Badge::new(model.look, text).variant(xgpui_luma_look_radix::BadgeVariant::Outline),
        )
    })
}

/// Right-aligned muted detail text (position or host detail), when present.
pub(crate) fn detail_label(model: &ColorStopRenderModel<'_>) -> Option<gpui::Div> {
    let chrome = model.look.chrome();
    model.detail_label.clone().map(|text| {
        div()
            .flex_shrink_0()
            .min_w(px(DETAIL_MIN_WIDTH))
            .flex()
            .justify_end()
            .typography_style(model.look.typography_scale(TextSize::Sm))
            .text_color(chrome.muted_text)
            .child(text)
    })
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
