use crate::infra::template::ModifiedElementTemplate;
use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Hsla, KeyDownEvent, MouseButton, MouseDownEvent, Pixels, ScrollEvent,
    SharedString, Window, div, prelude::*, px,
};

use crate::motion::overlay_presence::OverlayPresence;

pub type ComboBoxKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type ComboBoxScrollWheelHandler = Box<dyn Fn(&ScrollEvent, &mut Window, &mut App) + 'static>;
pub type ComboBoxClearClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ComboBoxTriggerClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ComboBoxTriggerMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type ComboBoxTriggerBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type ComboBoxTemplateModifier = Box<dyn Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static>;

pub struct ComboBoxTemplateHandlers {
    pub key_down: ComboBoxKeyDownHandler,
    pub scroll_wheel: ComboBoxScrollWheelHandler,
    pub clear_click: ComboBoxClearClickHandler,
    pub trigger_click: ComboBoxTriggerClickHandler,
    pub trigger_mouse_down: ComboBoxTriggerMouseDownHandler,
    pub trigger_bounds: ComboBoxTriggerBoundsHandler,
}

fn noop_key_down(_: &KeyDownEvent, _: &mut Window, _: &mut App) {}
fn noop_scroll_wheel(_: &ScrollEvent, _: &mut Window, _: &mut App) {}
fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

impl Default for ComboBoxTemplateHandlers {
    fn default() -> Self {
        Self {
            key_down: Box::new(noop_key_down),
            scroll_wheel: Box::new(noop_scroll_wheel),
            clear_click: Box::new(noop_click),
            trigger_click: Box::new(noop_click),
            trigger_mouse_down: Box::new(noop_mouse_down),
            trigger_bounds: Box::new(noop_bounds),
        }
    }
}

pub struct ComboBoxRenderModel {
    /// Captured shared default glyph choices.
    pub icons: crate::theme::stylesheet::IconsStylesheet,
    /// Captured and size-resolved affordance geometry.
    pub geometry: crate::theme::stylesheet::ComboBoxGeometry,
    pub textfield: AnyElement,
    pub query_is_empty: bool,
    pub show_down_arrow: bool,
    pub show_clear_button: bool,
    pub full_width: bool,
    pub minimum_trigger_width: Pixels,
    pub status_label: SharedString,
    pub status_detail: SharedString,
    pub status_color: Hsla,
    pub muted_text_color: Hsla,
    pub popup_bounds: Option<Bounds<Pixels>>,
    /// The popup panel's style (a `floating_menu` binding, #433).
    pub popup_menu: crate::style::StyleBinding,
    pub popup_content: Option<AnyElement>,
    pub presence: OverlayPresence,
}

pub trait ComboBoxTemplate: Send + Sync {
    fn render(
        &self,
        model: ComboBoxRenderModel,
        handlers: ComboBoxTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement;
}

pub struct DefaultComboBoxTemplate;

pub fn default_combobox_template() -> Arc<dyn ComboBoxTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ComboBoxTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultComboBoxTemplate)).clone()
}

impl ComboBoxTemplate for DefaultComboBoxTemplate {
    fn render(
        &self,
        model: ComboBoxRenderModel,
        handlers: ComboBoxTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement {
        let ComboBoxTemplateHandlers {
            key_down,
            scroll_wheel,
            clear_click,
            trigger_click,
            trigger_mouse_down,
            trigger_bounds,
        } = handlers;

        div()
            .when(model.full_width, |root| root.w_full())
            .flex()
            .flex_col()
            .on_key_down(key_down)
            .on_scroll(scroll_wheel)
            .child(
                div()
                    .on_children_prepainted(move |bounds, window, cx| {
                        if let Some(bounds) = bounds.first() {
                            trigger_bounds(bounds, window, cx);
                        }
                    })
                    .on_mouse_down(MouseButton::Left, trigger_mouse_down)
                    .min_w(model.minimum_trigger_width)
                    .when(model.full_width, |row| row.w_full())
                    .relative()
                    .child(model.textfield)
                    .when(model.show_down_arrow, |row| {
                        row.child(
                            div()
                                .id("combobox-arrow")
                                .absolute()
                                .top(px(0.0))
                                .right(px(model.geometry.affordance_inset))
                                .h_full()
                                .w(px(model.geometry.affordance_width))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .text_color(model.muted_text_color)
                                .hover(|style| style.text_color(model.status_color))
                                .on_click(trigger_click)
                                .child(crate::infra::icon::render_icon_source(
                                    &model.icons.resolve(crate::theme::stylesheet::IconRole::MenuCollapsed),
                                    model.muted_text_color,
                                    model.geometry.affordance_icon_size,
                                )),
                        )
                    })
                    .when(model.show_clear_button && !model.query_is_empty, |row| {
                        row.child(
                            div()
                                .id("combobox-clear")
                                .absolute()
                                .top(px(0.0))
                                .right(px(model.geometry.affordance_inset
                                    + if model.show_down_arrow {
                                        model.geometry.affordance_width + model.geometry.affordance_gap
                                    } else {
                                        0.0
                                    }))
                                .h_full()
                                .w(px(model.geometry.affordance_width))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .text_color(model.muted_text_color)
                                .hover(|style| style.text_color(model.status_color))
                                .on_click(clear_click)
                                .child(crate::infra::icon::render_icon_source(
                                    &model.icons.resolve(crate::theme::stylesheet::IconRole::Clear),
                                    model.muted_text_color,
                                    model.geometry.affordance_icon_size,
                                )),
                        )
                    }),
            )
            .when_some(model.popup_content, |root, popup_content| {
                root.child(div().opacity(model.presence.opacity()).child(popup_content))
            })
            .into_any_element()
    }
}

impl ComboBoxTemplate for ModifiedElementTemplate<dyn ComboBoxTemplate, AnyElement> {
    fn render(
        &self,
        model: ComboBoxRenderModel,
        handlers: ComboBoxTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let element = self.base.render(model, handlers, window, cx);

        self.apply(element, cx)
    }
}

pub(super) fn template_with_modifier<F>(template: Arc<dyn ComboBoxTemplate>, modifier: F) -> Arc<dyn ComboBoxTemplate>
where
    F: Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static,
{
    Arc::new(ModifiedElementTemplate::new(template, Box::new(modifier)))
}

pub use super::items_template::{
    ComboBoxItemsRenderModel, ComboBoxItemsTemplate, ComboBoxItemsTemplateHandlers, default_combobox_items_template,
};
