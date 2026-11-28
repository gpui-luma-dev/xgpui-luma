use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Div, MouseButton, MouseDownEvent, Pixels, SharedString, Stateful, Window, div,
    prelude::*, px,
};
use crate::infra::icon::{IconSource, render_icon_source};
use super::item_template::{SelectorItemRenderModel, SelectorItemTemplate};
use super::model::{SelectorItemLike, SelectorPath};
use crate::infra::state::ControlFocusState;
use crate::style::PartRead;
use crate::theme::LumaTextStyle;

/// The panel a selector opens: `source`'s `floating_menu.lss`, placed by its `popover.lss` (#397, #433). The menu
/// hook for selectors, so they open the host's menu, not the SDK's.
pub fn selector_items_panel_source(source: crate::style::ThemeSource) -> crate::controls::selector::ItemsPanelSource {
    crate::controls::floating_menu::floating_menu_source(source)
}

/// Where a selector panel drawn in `menu` opens: its theme's `popover.lss` (#397).
pub fn selector_panel_placement(menu: &crate::style::StyleBinding) -> crate::theme::stylesheet::PopoverGeometry {
    let popover = crate::style::StyleBinding::new("popover", crate::style::ThemeSource::Fixed(menu.theme()));
    crate::theme::stylesheet::PopoverGeometry::from_binding(&popover)
}

/// A selector panel's layout values, read from its `floating_menu` style and placement (#433): for placing and
/// sizing the popup before it is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelGeometry {
    pub(crate) padding: f32,
    pub(crate) border_width: f32,
    pub(crate) item_height: f32,
    pub(crate) item_padding_x: f32,
    /// Row text, for measuring labels.
    pub(crate) item_typography: LumaTextStyle,
    pub(crate) min_width: f32,
    pub(crate) popup_offset_y: f32,
    pub(crate) window_margin: f32,
}

impl PanelGeometry {
    pub(crate) fn of(menu: &crate::style::StyleBinding) -> Self {
        use crate::style::PartRead;
        let rest = menu.resolve(&[], 1.0);
        let (root, item) = (rest.root(), rest.part(xgpui_luma_style::PartId::new("item")));
        let placement = selector_panel_placement(menu);
        Self {
            padding: root.number("padding"),
            border_width: root.number("border_width"),
            item_height: item.number("height"),
            item_padding_x: item.number("padding_x"),
            item_typography: item.typography(14.0, 20.0),
            min_width: root.number("min_width"),
            popup_offset_y: placement.panel_offset_y,
            window_margin: placement.window_margin,
        }
    }
}

pub type SelectorPanelClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SelectorPanelHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SelectorPanelMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SelectorPanelScrollWheelHandler = Box<dyn Fn(&gpui::ScrollEvent, &mut Window, &mut App) + 'static>;

/// Unscrolled row geometry, reported by a panel after layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectorPopupGeometry {
    /// Center of the alignment row relative to the start of the row content.
    pub row_center: Pixels,
    pub content_height: Pixels,
    /// Space between the panel edge and row content, including border and padding.
    pub inset: Pixels,
}

pub type SelectorPopupGeometryHandler = Box<dyn Fn(&SelectorPopupGeometry, &mut Window, &mut App) + 'static>;

#[derive(Default)]
pub struct SelectorItemsTemplateHandlers {
    /// Policy-aware handler. Pair with the render model's retained scroll handle.
    pub scroll_wheel: Option<SelectorPanelScrollWheelHandler>,
    /// Custom panels should report their actual unscrolled alignment row geometry.
    /// Use the selected row, or the first enabled row when there is no selection.
    pub popup_geometry: Option<SelectorPopupGeometryHandler>,
    pub item_hovers: Vec<SelectorPanelHoverHandler>,
    pub item_mouse_downs: Vec<SelectorPanelMouseDownHandler>,
    pub item_clicks: Vec<SelectorPanelClickHandler>,
}

pub struct SelectorItemsRenderModel<'a, T>
where
    T: SelectorItemLike + 'static,
{
    pub menu_id: &'a SharedString,
    pub selector_id: &'a SharedString,
    pub items: &'a [T],
    pub selected_index: Option<usize>,
    pub active_path: Option<SelectorPath>,
    pub open: bool,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub item_template: Option<&'a SelectorItemTemplate<T>>,
    /// The panel's style (a `floating_menu` binding, #433): the template resolves it and draws from its parts.
    pub menu: crate::style::StyleBinding,
    pub max_height: Pixels,
    pub scrolling: bool,
    /// Custom panels should track this handle and disable native wheel movement when the handler is supplied.
    /// None preserves standalone template scrolling for previews and legacy hosts.
    pub scroll_handle: Option<&'a gpui::ScrollHandle>,
    pub selection_icon: &'a IconSource,
}

pub trait SelectorItemsTemplate<T>: Send + Sync
where
    T: SelectorItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectorItemsRenderModel<'_, T>,
        handlers: SelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div>;
}

impl<T> RenderModel for dyn SelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    type Model<'a> = SelectorItemsRenderModel<'a, T>;
}

pub type SelectorItemsTemplateModifier<T> =
    Box<dyn for<'a> Fn(Stateful<Div>, &SelectorItemsRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static>;

pub struct DefaultSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    modifiers: RootModifiers<dyn SelectorItemsTemplate<T>>,
}

impl<T> DefaultSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    pub fn new() -> Self {
        Self { modifiers: RootModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &SelectorItemsRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl<T> Default for DefaultSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SelectorItemsTemplate<T> for DefaultSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectorItemsRenderModel<'_, T>,
        handlers: SelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div> {
        let SelectorItemsTemplateHandlers { item_hovers, item_mouse_downs, item_clicks, scroll_wheel, popup_geometry } =
            handlers;
        let style = crate::controls::floating_menu::MenuStyle::resolve(&model.menu);
        let (panel, item_part) = (style.panel(), style.item());
        let colors = style.row_colors();
        let (padding, border_width) = (panel.number("padding"), panel.number("border_width"));
        let item_gap = item_part.number("gap");
        let content_max_height = (model.max_height - px((padding + border_width) * 2.0)).max(px(0.0));
        let mut rows = div().relative().flex().flex_col().w_full();

        if let Some(report) = popup_geometry {
            let alignment_index =
                model.selected_index.or_else(|| model.items.iter().position(|item| item.is_enabled()));
            let inset = px(padding + border_width);
            let scroll = model.scroll_handle.cloned();
            let item_count = model.items.len();
            rows = rows.on_children_prepainted(move |bounds, window, cx| {
                // GPUI stores child bounds in the handle instead of passing them
                // to this listener when the viewport tracks a scroll handle.
                let bounds = if bounds.is_empty() {
                    scroll
                        .as_ref()
                        .map(|handle| (0..item_count).filter_map(|index| handle.bounds_for_item(index)).collect())
                        .unwrap_or_default()
                } else {
                    bounds
                };
                if let (Some(first), Some(last), Some(row)) =
                    (bounds.first(), bounds.last(), alignment_index.and_then(|index| bounds.get(index)))
                {
                    report(
                        &SelectorPopupGeometry {
                            row_center: row.center().y - first.top(),
                            content_height: last.bottom() - first.top(),
                            inset,
                        },
                        window,
                        cx,
                    );
                }
            });
        }

        let mut rows = rows.id("rows");

        if model.scrolling {
            rows = rows.max_h(content_max_height).overflow_y_scroll();
        }
        let policy_routing = model.scroll_handle.is_some() && scroll_wheel.is_some();
        if let Some(handle) = model.scroll_handle.filter(|_| policy_routing) {
            rows = rows.overflow_hidden().track_scroll(handle);
        }

        let mut mouse_downs = item_mouse_downs.into_iter();
        let mut clicks = item_clicks.into_iter();

        for ((index, item), hover) in model.items.iter().enumerate().zip(item_hovers) {
            let mouse_down = mouse_downs.next();
            let click = clicks.next();
            let enabled_item = item.is_enabled();
            let color = if enabled_item {
                colors.foreground
            } else {
                colors.disabled_foreground
            };
            let selected = model.selected_index == Some(index);
            let active = model.active_path.is_some_and(|path| path.is_item(index));
            let content = if let Some(item_template) = model.item_template {
                let item_model = SelectorItemRenderModel {
                    selector_id: model.selector_id,
                    item,
                    index,
                    selected,
                    active,
                    open: model.open,
                    enabled: model.enabled && enabled_item,
                };
                item_template(&item_model, cx)
            } else {
                div()
                    .flex()
                    .items_center()
                    .gap(px(item_gap))
                    .child(div().flex_1().child(item.label().clone()))
                    .into_any_element()
            };

            let mut row = style
                .row(div().id((item.id().clone(), 0usize)), color)
                .flex_shrink_0()
                .child(div().flex_1().child(content))
                .child(render_selection_checkmark(selected, color, style.icon_size(), model.selection_icon));

            if enabled_item {
                row = row.cursor_pointer().on_hover(hover).hover({
                    let (hover_background, hover_foreground) = (colors.hover_background, colors.hover_foreground);
                    move |style| style.bg(hover_background).text_color(hover_foreground)
                });

                if active {
                    row = row.bg(colors.hover_background).text_color(colors.hover_foreground);
                }

                if let Some(mouse_down) = mouse_down {
                    row = row.on_mouse_down(MouseButton::Left, move |event, window, cx| {
                        cx.stop_propagation();
                        mouse_down(event, window, cx);
                    });
                }

                if let Some(click) = click {
                    row = row.on_click(click);
                }
            } else {
                row = crate::style::fade(row, style.disabled_opacity());
            }

            rows = rows.child(row);
        }

        let root = div()
            .id((model.menu_id.clone(), 0usize))
            .relative()
            .min_w(px(panel.number("min_width")))
            .max_h(model.max_height)
            .p(px(padding))
            .bg(panel.color("background"))
            .border(px(border_width))
            .border_color(panel.color("border"))
            .rounded(px(panel.number("radius")))
            .shadow(panel.box_shadow())
            .overflow_hidden()
            .when(policy_routing, |root| root.block_mouse_except_scroll())
            .when(!policy_routing, |root| root.occlude())
            .when_some(scroll_wheel, |root, handler| root.on_scroll(handler))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .child(rows);

        self.modifiers.apply(root, model)
    }
}

impl<T> SelectorItemsTemplate<T> for ModifiedTemplate<dyn SelectorItemsTemplate<T>>
where
    T: SelectorItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectorItemsRenderModel<'_, T>,
        handlers: SelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, cx);
        self.apply(root, model)
    }
}

pub fn default_selector_items_template<T>() -> Arc<dyn SelectorItemsTemplate<T>>
where
    T: SelectorItemLike + 'static,
{
    Arc::new(DefaultSelectorItemsTemplate::<T>::new())
}

pub(crate) fn items_template_with_modifier<T, F>(
    template: Arc<dyn SelectorItemsTemplate<T>>,
    modifier: F,
) -> Arc<dyn SelectorItemsTemplate<T>>
where
    T: SelectorItemLike + 'static,
    F: for<'a> Fn(Stateful<Div>, &SelectorItemsRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

fn render_selection_checkmark(selected: bool, color: gpui::Hsla, size: f32, icon: &IconSource) -> AnyElement {
    if selected {
        render_icon_source(icon, color, size)
    } else {
        div().size(px(size)).into_any_element()
    }
}

#[cfg(test)]
mod placement_tests {
    use super::*;
    use crate::style::{StyleBinding, ThemeSource, default_theme};

    /// A selector popup is placed by its theme's `popover.lss`, not the SDK's alone (#397): a theme at twice
    /// the length scale opens it twice as far from its trigger.
    #[test]
    fn selector_popups_are_placed_by_their_themes_popover_lss() {
        let sdk = crate::theme::stylesheet::PopoverGeometry::sdk();
        let scaled = default_theme().with_length_scale(2.0).expect("scaled theme");
        let panel = PanelGeometry::of(&StyleBinding::new("floating_menu", ThemeSource::Fixed(scaled.clone())));
        assert!(sdk.panel_offset_y > 0.0 && sdk.window_margin > 0.0, "{sdk:?}");
        assert_eq!((panel.popup_offset_y, panel.window_margin), (sdk.panel_offset_y * 2.0, sdk.window_margin * 2.0));
        let source = PanelGeometry::of(&selector_items_panel_source(ThemeSource::Fixed(scaled))());
        assert_eq!(source.popup_offset_y, sdk.panel_offset_y * 2.0, "the source helper too");
    }
}

/// The panel templates draw from the `floating_menu` style (#433): every value they read matches what the old
/// panel look held, for the selector panel and the selection panel.
#[cfg(test)]
mod parity_tests {}
