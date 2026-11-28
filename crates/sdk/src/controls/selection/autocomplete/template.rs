use crate::infra::template::ElementModifiers;
use crate::infra::template::RootModifiers;
use crate::infra::template::ModifiedTemplate;
use crate::infra::template::RenderModel;
use crate::infra::template::ModifiedElementTemplate;
use crate::style::PartRead;
use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Hsla, KeyDownEvent, Pixels, ScrollEvent, SharedString, Stateful, Window, div,
    prelude::*, px,
};

use crate::controls::selector_list::{SelectorItem, SelectorPanelClickHandler, SelectorPanelHoverHandler};
use crate::motion::overlay_presence::OverlayPresence;
use crate::theme::LumaTextStyle;

pub type AutocompleteKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type AutocompleteScrollWheelHandler = Box<dyn Fn(&ScrollEvent, &mut Window, &mut App) + 'static>;
pub type AutocompleteClearClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type AutocompleteTriggerBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type AutocompleteTemplateModifier =
    Box<dyn Fn(Stateful<gpui::Div>, &mut App) -> Stateful<gpui::Div> + Send + Sync + 'static>;

pub struct AutocompleteTemplateHandlers {
    pub key_down: AutocompleteKeyDownHandler,
    pub scroll_wheel: AutocompleteScrollWheelHandler,
    pub clear_click: AutocompleteClearClickHandler,
    pub trigger_bounds: AutocompleteTriggerBoundsHandler,
}

fn noop_key_down(_: &KeyDownEvent, _: &mut Window, _: &mut App) {}
fn noop_scroll_wheel(_: &ScrollEvent, _: &mut Window, _: &mut App) {}
fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

impl Default for AutocompleteTemplateHandlers {
    fn default() -> Self {
        Self {
            key_down: Box::new(noop_key_down),
            scroll_wheel: Box::new(noop_scroll_wheel),
            clear_click: Box::new(noop_click),
            trigger_bounds: Box::new(noop_bounds),
        }
    }
}

pub struct AutocompleteRenderModel {
    /// Captured shared default glyph choices.
    pub icons: crate::theme::stylesheet::IconsStylesheet,
    /// Captured and size-resolved affordance geometry.
    pub geometry: crate::theme::stylesheet::AutocompleteGeometry,
    pub id: SharedString,
    pub textfield: AnyElement,
    pub query_is_empty: bool,
    pub popup_width: Pixels,
    pub status_label: SharedString,
    pub status_detail: SharedString,
    pub status_color: Hsla,
    pub status_typography: LumaTextStyle,
    pub muted_text_color: Hsla,
    /// Where the popup opens beside the field; without it the popup is not drawn.
    pub popup_placement: Option<crate::infra::popup_placement::ResolvedPopupPlacement>,
    /// The popup panel's style (a `floating_menu` binding, #433).
    pub popup_menu: crate::style::StyleBinding,
    pub popup_content: Option<AnyElement>,
    pub presence: OverlayPresence,
}

pub trait AutocompleteTemplate: Send + Sync {
    fn render(
        &self,
        model: AutocompleteRenderModel,
        handlers: AutocompleteTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<gpui::Div>;
}

pub struct DefaultAutocompleteTemplate {
    modifiers: ElementModifiers<Stateful<gpui::Div>>,
}

impl DefaultAutocompleteTemplate {
    pub fn new() -> Self {
        Self { modifiers: ElementModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<gpui::Div>, &mut App) -> Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl Default for DefaultAutocompleteTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_autocomplete_template() -> Arc<dyn AutocompleteTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn AutocompleteTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultAutocompleteTemplate::new())).clone()
}

pub(super) fn modified_autocomplete_template<F>(
    template: Arc<dyn AutocompleteTemplate>,
    modifier: F,
) -> Arc<dyn AutocompleteTemplate>
where
    F: Fn(Stateful<gpui::Div>, &mut App) -> Stateful<gpui::Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedElementTemplate::new(template, Box::new(modifier)))
}

impl AutocompleteTemplate for ModifiedElementTemplate<dyn AutocompleteTemplate, Stateful<gpui::Div>> {
    fn render(
        &self,
        model: AutocompleteRenderModel,
        handlers: AutocompleteTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply(root, cx)
    }
}

impl AutocompleteTemplate for DefaultAutocompleteTemplate {
    fn render(
        &self,
        model: AutocompleteRenderModel,
        handlers: AutocompleteTemplateHandlers,
        _window: &mut Window,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let AutocompleteTemplateHandlers { key_down, scroll_wheel, clear_click, trigger_bounds } = handlers;

        let has_status_text = !model.status_label.is_empty() || !model.status_detail.is_empty();
        let status_style = model.status_typography;

        let status_row = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(model.geometry.status_gap))
            .child(
                div()
                    .text_size(px(status_style.size))
                    .line_height(px(status_style.line_height))
                    .font_weight(status_style.weight)
                    .text_color(model.status_color)
                    .child(model.status_label),
            )
            .child(
                div()
                    .text_size(px(status_style.size))
                    .line_height(px(status_style.line_height))
                    .font_weight(status_style.weight)
                    .text_color(model.muted_text_color)
                    .child(model.status_detail),
            );

        let root = div()
            .id((model.id.clone(), 0usize))
            .w_full()
            .flex()
            .flex_col()
            .gap(px(model.geometry.gap))
            .when(has_status_text, |root| root.child(status_row))
            .on_key_down(key_down)
            .on_scroll(scroll_wheel)
            .child(
                div()
                    .on_children_prepainted(move |bounds, window, cx| {
                        if let Some(bounds) = bounds.first() {
                            trigger_bounds(bounds, window, cx);
                        }
                    })
                    .w_full()
                    .relative()
                    .child(model.textfield)
                    .when(!model.query_is_empty, |row| {
                        row.child(
                            div()
                                .id("prototype-autocomplete-clear")
                                .absolute()
                                .top(px(0.0))
                                .right(px(model.geometry.clear_inset))
                                .h_full()
                                .w(px(model.geometry.clear_width))
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
                                    model.geometry.clear_icon_size,
                                )),
                        )
                    }),
            )
            .when_some(model.popup_content, |root, popup_content| {
                root.when_some(model.popup_placement, |root, placement| {
                    let style = crate::controls::floating_menu::MenuStyle::resolve(&model.popup_menu);
                    let window_margin =
                        crate::controls::selector_list::selector_panel_placement(&model.popup_menu).window_margin;
                    let shell = style
                        .surface(div().id("prototype-autocomplete-popup-shell"))
                        .w(model.popup_width)
                        .overflow_hidden()
                        .block_mouse_except_scroll()
                        .child(div().opacity(model.presence.opacity()).child(popup_content));
                    root.child(crate::infra::popup_placement::anchored_popup(&placement, px(window_margin), shell))
                })
            });

        self.modifiers.apply(root, cx)
    }
}

pub struct AutocompleteItemsRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: &'a [SelectorItem],
    /// The panel's style (a `floating_menu` binding, #433): the template resolves it and draws from its parts.
    pub menu: crate::style::StyleBinding,
    pub highlighted_index: Option<usize>,
}

pub struct AutocompleteItemsTemplateHandlers {
    pub item_hovers: Vec<SelectorPanelHoverHandler>,
    pub item_clicks: Vec<SelectorPanelClickHandler>,
}

pub type AutocompleteItemsTemplateModifier = Box<
    dyn for<'a> Fn(Stateful<gpui::Div>, &AutocompleteItemsRenderModel<'a>) -> Stateful<gpui::Div>
        + Send
        + Sync
        + 'static,
>;

pub trait AutocompleteItemsTemplate: Send + Sync {
    fn render(
        &self,
        model: &AutocompleteItemsRenderModel<'_>,
        handlers: AutocompleteItemsTemplateHandlers,
    ) -> Stateful<gpui::Div>;
}

impl RenderModel for dyn AutocompleteItemsTemplate {
    type Model<'a> = AutocompleteItemsRenderModel<'a>;
}

pub struct DefaultAutocompleteItemsTemplate {
    modifiers: RootModifiers<dyn AutocompleteItemsTemplate>,
}

impl DefaultAutocompleteItemsTemplate {
    pub fn new() -> Self {
        Self { modifiers: RootModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<gpui::Div>, &AutocompleteItemsRenderModel<'a>) -> Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl Default for DefaultAutocompleteItemsTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_autocomplete_items_template() -> Arc<dyn AutocompleteItemsTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn AutocompleteItemsTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultAutocompleteItemsTemplate::new())).clone()
}

impl AutocompleteItemsTemplate for DefaultAutocompleteItemsTemplate {
    fn render(
        &self,
        model: &AutocompleteItemsRenderModel<'_>,
        handlers: AutocompleteItemsTemplateHandlers,
    ) -> Stateful<gpui::Div> {
        let AutocompleteItemsTemplateHandlers { item_hovers, item_clicks } = handlers;
        let style = crate::controls::floating_menu::MenuStyle::resolve(&model.menu);
        let colors = style.row_colors();
        let typography = style.item().typography(14.0, 20.0);

        let mut root = div()
            .id((model.id.clone(), 1usize))
            .relative()
            .flex()
            .flex_col()
            .min_w(px(style.panel().number("min_width")))
            .p(px(style.panel().number("padding")));
        let mut clicks = item_clicks.into_iter();

        for (index, (item, hover)) in model.items.iter().zip(item_hovers).enumerate() {
            let enabled_item = item.is_enabled();
            let color = if enabled_item {
                colors.foreground
            } else {
                colors.disabled_foreground
            };
            let mut row = div()
                .id(("row", index))
                .flex()
                .items_center()
                .min_h(px(style.item().number("height")))
                .px(px(style.item().number("padding_x")))
                .rounded(px(style.item_radius()))
                .text_color(color)
                .text_size(px(typography.size))
                .line_height(px(typography.line_height))
                .font_weight(typography.weight)
                .child(div().flex_1().min_w(px(0.0)).truncate().child(item.label_text().clone()));

            if enabled_item {
                row = row.cursor_pointer().on_hover(hover).hover({
                    let hover_background = colors.hover_background;
                    let hover_foreground = colors.hover_foreground;
                    move |style| style.bg(hover_background).text_color(hover_foreground)
                });

                if model.highlighted_index.is_some_and(|active| active == index) {
                    row = row.bg(colors.hover_background).text_color(colors.hover_foreground);
                }

                if let Some(click) = clicks.next() {
                    row = row.on_click(click);
                }
            } else {
                row = crate::style::fade(row, style.disabled_opacity());
            }

            root = root.child(row);
        }

        self.modifiers.apply(root, model)
    }
}

impl AutocompleteItemsTemplate for ModifiedTemplate<dyn AutocompleteItemsTemplate> {
    fn render(
        &self,
        model: &AutocompleteItemsRenderModel<'_>,
        handlers: AutocompleteItemsTemplateHandlers,
    ) -> Stateful<gpui::Div> {
        let root = self.base.render(model, handlers);
        self.apply(root, model)
    }
}

pub(super) fn modified_autocomplete_items_template<F>(
    template: Arc<dyn AutocompleteItemsTemplate>,
    modifier: F,
) -> Arc<dyn AutocompleteItemsTemplate>
where
    F: for<'a> Fn(Stateful<gpui::Div>, &AutocompleteItemsRenderModel<'a>) -> Stateful<gpui::Div>
        + Send
        + Sync
        + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}
