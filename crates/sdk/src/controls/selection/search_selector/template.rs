use crate::infra::template::RootModifiers;
use crate::infra::template::ModifiedElementTemplate;
use crate::infra::template::ModifiedTemplate;
use crate::infra::template::RenderModel;
use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Hsla, KeyDownEvent, MouseButton, MouseDownEvent, Pixels, ScrollEvent,
    SharedString, Stateful, Window, div, prelude::*, px,
};

use crate::controls::button::face::{Face, FaceShape};
use crate::controls::selector::SelectorLook;
use crate::controls::selector::theme::selector_face_look;
use crate::controls::selector_list::{SelectorItemsPanelLook, SelectorPanelClickHandler, SelectorPanelHoverHandler};
use crate::motion::overlay_presence::OverlayPresence;

use super::behavior::SelectionItem;
use super::item_template::{SearchSelectorItemRenderModel, SearchSelectorItemTemplate};
use crate::controls::textfield::TextFieldState;
use crate::theme::LumaTextStyle;

pub type SearchSelectorKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type SearchSelectorScrollWheelHandler = Box<dyn Fn(&ScrollEvent, &mut Window, &mut App) + 'static>;
pub type SearchSelectorTriggerClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SearchSelectorTriggerHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SearchSelectorTriggerMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SearchSelectorTriggerMouseUpHandler = Box<dyn Fn(&gpui::MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type SearchSelectorTriggerBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type SearchSelectorTemplateModifier =
    Box<dyn Fn(Stateful<gpui::Div>, &mut App) -> Stateful<gpui::Div> + Send + Sync + 'static>;

pub struct SearchSelectorTemplateHandlers {
    pub key_down: SearchSelectorKeyDownHandler,
    pub scroll_wheel: SearchSelectorScrollWheelHandler,
    pub trigger_click: SearchSelectorTriggerClickHandler,
    pub trigger_hover: SearchSelectorTriggerHoverHandler,
    pub trigger_mouse_down: SearchSelectorTriggerMouseDownHandler,
    pub trigger_mouse_up: SearchSelectorTriggerMouseUpHandler,
    pub trigger_mouse_up_out: SearchSelectorTriggerMouseUpHandler,
    pub trigger_bounds: SearchSelectorTriggerBoundsHandler,
}

fn noop_key_down(_: &KeyDownEvent, _: &mut Window, _: &mut App) {}
fn noop_scroll_wheel(_: &ScrollEvent, _: &mut Window, _: &mut App) {}
fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}
fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_mouse_up(_: &gpui::MouseUpEvent, _: &mut Window, _: &mut App) {}
fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

impl Default for SearchSelectorTemplateHandlers {
    fn default() -> Self {
        Self {
            key_down: Box::new(noop_key_down),
            scroll_wheel: Box::new(noop_scroll_wheel),
            trigger_click: Box::new(noop_click),
            trigger_hover: Box::new(noop_hover),
            trigger_mouse_down: Box::new(noop_mouse_down),
            trigger_mouse_up: Box::new(noop_mouse_up),
            trigger_mouse_up_out: Box::new(noop_mouse_up),
            trigger_bounds: Box::new(noop_bounds),
        }
    }
}

pub struct SearchSelectorRenderModel {
    /// Captured shared default glyph choices.
    pub icons: crate::theme::stylesheet::IconsStylesheet,
    /// Captured and size-resolved affordance geometry.
    pub geometry: crate::theme::stylesheet::SearchSelectorGeometry,
    pub id: SharedString,
    pub trigger_label: SharedString,
    pub trigger_label_is_placeholder: bool,
    pub trigger_state: TextFieldState,
    pub trigger_look: SelectorLook,
    pub trigger_typography_override: Option<LumaTextStyle>,
    pub enabled: bool,
    pub full_width: bool,
    pub minimum_trigger_width: Pixels,
    pub status_label: SharedString,
    pub status_detail: SharedString,
    pub status_color: Hsla,
    pub muted_text_color: Hsla,
    pub popup_content: Option<AnyElement>,
    pub presence: OverlayPresence,
    pub disclosure_progress: f32,
}

pub trait SearchSelectorTemplate: Send + Sync {
    fn render(
        &self,
        model: SearchSelectorRenderModel,
        handlers: SearchSelectorTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<gpui::Div>;
}

pub struct DefaultSearchSelectorTemplate;

pub fn default_search_selector_template() -> Arc<dyn SearchSelectorTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SearchSelectorTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultSearchSelectorTemplate)).clone()
}

impl SearchSelectorTemplate for DefaultSearchSelectorTemplate {
    fn render(
        &self,
        model: SearchSelectorRenderModel,
        handlers: SearchSelectorTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let SearchSelectorTemplateHandlers {
            key_down,
            scroll_wheel,
            trigger_click,
            trigger_hover,
            trigger_mouse_down,
            trigger_mouse_up,
            trigger_mouse_up_out,
            trigger_bounds,
        } = handlers;

        let mut trigger_look = model.trigger_look;
        if let Some(typography) = model.trigger_typography_override {
            trigger_look.trigger_typography = typography;
            trigger_look.trigger_icon_size = typography.size;
        }
        let label_color = if model.trigger_label_is_placeholder {
            trigger_look.trigger_icon
        } else {
            trigger_look.trigger_foreground
        };

        // The trigger is a button face (#430), dense: its focus ring paints inside the edge so focusing never
        // changes the trigger's size (the selector reserves room around its trigger; this one never has).
        let face = Face {
            id: "trigger".into(),
            shape: FaceShape::Padded,
            radius: trigger_look.trigger_radius,
            full_width: true,
            min_height: false,
            focused: model.trigger_state.focus_visible && model.enabled && !model.trigger_state.invalid,
            disabled: !model.enabled,
            elevation: true,
            compact: true,
        };
        // Border visibility changes paint only; invalid/disabled ghost triggers keep the same label and icon geometry.
        let metrics = crate::theme::MetricTokens {
            border_width: model.geometry.border_width,
            ..crate::theme::MetricTokens::default()
        };
        let face_look = selector_face_look(&trigger_look, &metrics);
        let label = div()
            .debug_selector(|| "search-selector-label".into())
            .min_w(px(0.0))
            .truncate()
            .text_color(label_color)
            .child(model.trigger_label)
            .into_any_element();
        let icon = div()
            .id("icon")
            .debug_selector(|| "search-selector-icon".into())
            .w(px(model.geometry.search_icon_width))
            .flex()
            .items_center()
            .justify_center()
            .text_color(trigger_look.trigger_icon)
            .child(crate::infra::icon::render_icon_source(
                &model.icons.resolve(crate::theme::stylesheet::IconRole::Search),
                trigger_look.trigger_icon,
                model.geometry.search_icon_size,
            ))
            .into_any_element();
        let trigger = face
            .control(&face_look, label, Some(icon))
            .on_hover(trigger_hover)
            .on_mouse_down(MouseButton::Left, trigger_mouse_down)
            .on_mouse_up(MouseButton::Left, trigger_mouse_up)
            .on_mouse_up_out(MouseButton::Left, trigger_mouse_up_out)
            .on_click(trigger_click);
        let trigger_chrome = face.root(trigger, &face_look);

        div()
            .id((model.id.clone(), 0usize))
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
                    .min_w(model.minimum_trigger_width)
                    .when(model.full_width, |row| row.w_full())
                    .relative()
                    .child(trigger_chrome),
            )
            .when_some(model.popup_content, |root, popup_content| {
                root.child(div().opacity(model.presence.opacity()).child(popup_content))
            })
    }
}

impl SearchSelectorTemplate for ModifiedElementTemplate<dyn SearchSelectorTemplate, Stateful<gpui::Div>> {
    fn render(
        &self,
        model: SearchSelectorRenderModel,
        handlers: SearchSelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let element = self.base.render(model, handlers, window, cx);

        self.apply(element, cx)
    }
}

pub(super) fn template_with_modifier<F>(
    template: Arc<dyn SearchSelectorTemplate>,
    modifier: F,
) -> Arc<dyn SearchSelectorTemplate>
where
    F: Fn(Stateful<gpui::Div>, &mut App) -> Stateful<gpui::Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedElementTemplate::new(template, Box::new(modifier)))
}

pub struct SearchSelectorItemsRenderModel<'a> {
    pub trailing_icon_size: f32,
    pub menu_id: &'a SharedString,
    pub search_selector_id: &'a SharedString,
    pub items: &'a [SelectionItem],
    pub visible_indices: &'a [usize],
    pub selected_source_index: Option<usize>,
    pub active_visible_index: Option<usize>,
    pub open: bool,
    pub enabled: bool,
    pub item_template: Option<&'a SearchSelectorItemTemplate<SelectionItem>>,
    pub look: SelectorItemsPanelLook,
}

pub struct SearchSelectorItemsTemplateHandlers {
    pub item_hovers: Vec<SelectorPanelHoverHandler>,
    /// One callback per visible row, including disabled rows; consume in visible order.
    pub item_clicks: Vec<SelectorPanelClickHandler>,
}

pub type SearchSelectorItemsTemplateModifier = Box<
    dyn Fn(Stateful<gpui::Div>, &SearchSelectorItemsRenderModel<'_>) -> Stateful<gpui::Div> + Send + Sync + 'static,
>;

pub trait SearchSelectorItemsTemplate: Send + Sync {
    fn render(
        &self,
        model: &SearchSelectorItemsRenderModel<'_>,
        handlers: SearchSelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<gpui::Div>;
}

impl RenderModel for dyn SearchSelectorItemsTemplate {
    type Model<'a> = SearchSelectorItemsRenderModel<'a>;
}

pub struct DefaultSearchSelectorItemsTemplate {
    modifiers: RootModifiers<dyn SearchSelectorItemsTemplate>,
}

impl DefaultSearchSelectorItemsTemplate {
    pub fn new() -> Self {
        Self { modifiers: RootModifiers::default() }
    }

    #[allow(dead_code)]
    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<gpui::Div>, &SearchSelectorItemsRenderModel<'_>) -> Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl Default for DefaultSearchSelectorItemsTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_search_selector_items_template() -> Arc<dyn SearchSelectorItemsTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SearchSelectorItemsTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultSearchSelectorItemsTemplate::new())).clone()
}

pub fn search_selector_items_template_with_modifier<F>(
    template: Arc<dyn SearchSelectorItemsTemplate>,
    modifier: F,
) -> Arc<dyn SearchSelectorItemsTemplate>
where
    F: Fn(Stateful<gpui::Div>, &SearchSelectorItemsRenderModel<'_>) -> Stateful<gpui::Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl SearchSelectorItemsTemplate for DefaultSearchSelectorItemsTemplate {
    fn render(
        &self,
        model: &SearchSelectorItemsRenderModel<'_>,
        handlers: SearchSelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let SearchSelectorItemsTemplateHandlers { item_hovers, item_clicks } = handlers;
        let look = &model.look;

        let mut root =
            div().id((model.menu_id.clone(), 0usize)).relative().flex().flex_col().w_full().p(px(look.padding));
        let mut clicks = item_clicks.into_iter();

        for (visible_index, (source_index, hover)) in model.visible_indices.iter().copied().zip(item_hovers).enumerate()
        {
            let click = clicks.next();
            let Some(item) = model.items.get(source_index) else {
                continue;
            };

            let enabled_item = model.enabled && item.enabled;
            let color = if enabled_item {
                look.foreground
            } else {
                look.item_disabled_foreground
            };
            let selected = model.selected_source_index == Some(source_index);
            let active = enabled_item && model.active_visible_index == Some(visible_index);
            let content = if let Some(item_template) = model.item_template {
                item_template(
                    &SearchSelectorItemRenderModel {
                        search_selector_id: model.search_selector_id,
                        item,
                        source_index,
                        visible_index,
                        selected,
                        active,
                        open: model.open,
                        enabled: enabled_item,
                    },
                    cx,
                )
            } else {
                let mut content = div().flex().items_center().justify_between().flex_1().child(item.label.clone());
                if let Some(icon) = item.trailing_icon {
                    content =
                        content.child(crate::infra::icon::lucide_icon(icon, look.positive, model.trailing_icon_size));
                }
                content.into_any_element()
            };

            let mut row = div()
                .id(("row", visible_index))
                .flex()
                .items_center()
                .min_h(px(look.item_height))
                .px(px(look.item_padding_x))
                .rounded(px(look.item_radius))
                .text_color(color)
                .text_size(px(look.item_typography.size))
                .line_height(px(look.item_typography.line_height))
                .font_weight(look.item_typography.weight)
                .child(content);

            if enabled_item {
                row = row.cursor_pointer().on_hover(hover).hover({
                    let hover_background = look.item_hover_background;
                    let hover_foreground = look.item_hover_foreground;
                    move |style| style.bg(hover_background).text_color(hover_foreground)
                });

                if active {
                    row = row.bg(look.item_hover_background).text_color(look.item_hover_foreground);
                }

                if let Some(click) = click {
                    row = row.on_click(click);
                }
            } else {
                row = crate::style::fade(row, look.item_disabled_opacity);
            }

            root = root.child(row);
        }

        self.modifiers.apply(root, model)
    }
}

impl SearchSelectorItemsTemplate for ModifiedTemplate<dyn SearchSelectorItemsTemplate> {
    fn render(
        &self,
        model: &SearchSelectorItemsRenderModel<'_>,
        handlers: SearchSelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let root = self.base.render(model, handlers, cx);
        self.apply(root, model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "test-support")]
    #[test]
    fn trigger_content_stays_put_across_interaction_states() {
        crate::infra::focus_adorner::testing::assert_search_selector_layout(
            crate::controls::selector::default_selector_theme(),
        );
    }

    #[test]
    fn items_template_with_modifier_wraps_template() {
        let template = default_search_selector_items_template();
        let wrapped = search_selector_items_template_with_modifier(template.clone(), |element, _| element);

        assert!(!Arc::ptr_eq(&wrapped, &template));
    }
}
