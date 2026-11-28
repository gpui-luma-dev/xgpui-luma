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
use crate::theme::{LumaTextStyle, ThemeTokens};

/// A selector popup panel's values, copied out of its `floating_menu` style (#433). A compatibility type: panel
/// templates draw from the style itself (see [`SelectorItemsRenderModel::menu`]).
#[deprecated(
    note = "panel templates draw from the `floating_menu` style; resolve it and read its parts instead (#433)"
)]
#[derive(Clone, Debug)]
pub struct SelectorItemsPanelLook {
    /// Positive completion/status adornments.
    pub positive: gpui::Hsla,
    pub background: gpui::Hsla,
    pub foreground: gpui::Hsla,
    pub border: gpui::Hsla,
    /// Shell border thickness, also used when aligning popup rows.
    pub border_width: f32,
    /// A disabled row's opacity (the floating menu's `item` part).
    pub item_disabled_opacity: f32,
    /// Vertical offset for anchored popups; inline panels ignore this value.
    pub popup_offset_y: f32,
    /// Minimum viewport edge margin for anchored popup placement.
    pub window_margin: f32,
    pub shadow: Vec<gpui::BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub item_disabled_foreground: gpui::Hsla,
    pub item_hover_background: gpui::Hsla,
    pub item_hover_foreground: gpui::Hsla,
    pub item_typography: LumaTextStyle,
    pub item_height: f32,
    pub item_padding_x: f32,
    pub item_gap: f32,
    pub item_icon_size: f32,
    pub item_radius: f32,
}

/// The SDK's selector items panel: the SDK's `.lss` floating menu with `tokens`' metrics and geometry.
#[deprecated(note = "use `StyleBinding::sdk(\"floating_menu\")`; panel templates draw from the style (#433)")]
#[allow(deprecated)]
pub fn default_selector_items_panel_look(tokens: &ThemeTokens) -> SelectorItemsPanelLook {
    styled_selector_items_panel_look(&crate::style::StyleBinding::sdk("floating_menu"), &tokens.metrics)
}

/// A selector items panel from a `.lss` floating-menu binding (the `floating_menu` control, or one that
/// extends it), placed by the same theme's `popover.lss` (#397).
#[deprecated(note = "panel templates draw from the `floating_menu` binding itself (#433)")]
#[allow(deprecated)]
pub fn styled_selector_items_panel_look(
    binding: &crate::style::StyleBinding,
    metrics: &crate::theme::MetricTokens,
) -> SelectorItemsPanelLook {
    let popover = crate::style::StyleBinding::new("popover", crate::style::ThemeSource::Fixed(binding.theme()));
    SelectorItemsPanelLook::from_menu(crate::controls::floating_menu::styled_floating_menu_look(binding), metrics)
        .with_placement(crate::theme::stylesheet::PopoverGeometry::from_binding(&popover))
}

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

#[allow(deprecated)]
impl SelectorItemsPanelLook {
    /// The panel `menu` (a `floating_menu` binding) describes, placed by its theme's `popover.lss`.
    pub fn from_style(menu: &crate::style::StyleBinding) -> Self {
        Self::from_menu(
            crate::controls::floating_menu::styled_floating_menu_look(menu),
            &crate::theme::MetricTokens::default(),
        )
        .with_placement(selector_panel_placement(menu))
        .with_border_width(PanelGeometry::of(menu).border_width)
    }

    fn with_border_width(mut self, border_width: f32) -> Self {
        self.border_width = border_width;
        self
    }

    /// Share menu chrome, metric border width and the SDK's popover panel placement (`popover.lss`).
    pub fn from_menu(
        menu: crate::controls::floating_menu::FloatingMenuLook,
        metrics: &crate::theme::MetricTokens,
    ) -> Self {
        let placement = crate::theme::stylesheet::PopoverGeometry::sdk();
        Self {
            positive: menu.positive,
            background: menu.background,
            foreground: menu.foreground,
            border: menu.border,
            border_width: metrics.border_width,
            item_disabled_opacity: menu.item_disabled_opacity,
            popup_offset_y: placement.panel_offset_y,
            window_margin: placement.window_margin,
            shadow: menu.shadow,
            radius: menu.radius,
            padding: menu.padding,
            min_width: menu.min_width,
            item_disabled_foreground: menu.item_disabled_foreground,
            item_hover_background: menu.item_hover_background,
            item_hover_foreground: menu.item_hover_foreground,
            item_typography: menu.item_typography,
            item_height: menu.item_height,
            item_padding_x: menu.item_padding_x,
            item_gap: menu.item_gap,
            item_icon_size: menu.item_icon_size,
            item_radius: menu.item_radius,
        }
    }

    /// Anchored popup placement from a look's `popover.lss` instead of the SDK's.
    pub fn with_placement(mut self, placement: crate::theme::stylesheet::PopoverGeometry) -> Self {
        self.popup_offset_y = placement.panel_offset_y;
        self.window_margin = placement.window_margin;
        self
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
#[allow(deprecated)]
mod parity_tests {
    use super::*;
    use crate::controls::floating_menu::MenuStyle;
    use crate::style::StyleBinding;

    fn assert_panel_matches(menu: &StyleBinding, look: &SelectorItemsPanelLook) {
        let style = MenuStyle::resolve(menu);
        let (panel, item, colors) = (style.panel(), style.item(), style.row_colors());
        assert_eq!(
            (panel.color("background"), panel.color("border"), panel.box_shadow()),
            (look.background, look.border, look.shadow.clone())
        );
        assert_eq!(
            (
                panel.number("border_width"),
                panel.number("radius"),
                panel.number("padding"),
                panel.number("min_width")
            ),
            (look.border_width, look.radius, look.padding, look.min_width)
        );
        assert_eq!(
            (colors.foreground, colors.disabled_foreground, colors.hover_background, colors.hover_foreground),
            (
                look.foreground,
                look.item_disabled_foreground,
                look.item_hover_background,
                look.item_hover_foreground
            )
        );
        assert_eq!(
            (item.number("height"), item.number("padding_x"), item.number("gap"), style.item_radius()),
            (look.item_height, look.item_padding_x, look.item_gap, look.item_radius)
        );
        assert_eq!(item.typography(14.0, 20.0), look.item_typography);
        assert_eq!(
            (style.icon_size(), style.disabled_opacity(), panel.color("positive")),
            (look.item_icon_size, look.item_disabled_opacity, look.positive)
        );
        let placement = selector_panel_placement(menu);
        assert_eq!((placement.panel_offset_y, placement.window_margin), (look.popup_offset_y, look.window_margin));
    }

    #[test]
    fn the_selector_panel_draws_what_the_look_held() {
        let tokens = ThemeTokens::default();
        assert_panel_matches(&StyleBinding::sdk("floating_menu"), &default_selector_items_panel_look(&tokens));
    }

    #[test]
    fn the_selection_panel_draws_what_the_look_held() {
        let tokens = ThemeTokens::default();
        let look = crate::controls::selection_panel::default_selection_panel_look(&tokens);
        let menu = StyleBinding::sdk("selection_panel");
        let style = MenuStyle::resolve(&menu);
        let (panel, colors) = (style.panel(), style.row_colors());
        assert_eq!(
            (
                panel.color("background"),
                panel.number("border_width"),
                panel.number("padding"),
                panel.number("min_width")
            ),
            (look.background, look.border_width, look.padding, look.min_width)
        );
        assert_eq!((colors.foreground, colors.hover_background), (look.foreground, look.item_hover_background));
        assert_eq!((style.item().number("height"), style.icon_size()), (look.item_height, look.item_icon_size));
    }
}
