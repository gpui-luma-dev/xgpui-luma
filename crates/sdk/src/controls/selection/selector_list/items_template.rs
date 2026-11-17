use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Div, MouseButton, MouseDownEvent, Pixels, SharedString, Stateful, Window, div,
    prelude::*, px,
};
use crate::infra::icon::{IconSource, render_icon_source};
use crate::infra::locale::{LayoutDirection, LocaleContext, LogicalStyled};
use super::item_template::{SelectorItemRenderModel, SelectorItemTemplate};
use super::model::{SelectorItemLike, SelectorPath};
use crate::infra::state::ControlFocusState;
use crate::theme::{ControlSize, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SelectorItemsPanelLook {
    /// Positive completion/status adornments.
    pub positive: gpui::Hsla,
    pub background: gpui::Hsla,
    pub foreground: gpui::Hsla,
    pub border: gpui::Hsla,
    /// Shell border thickness, also used when aligning popup rows.
    pub border_width: f32,
    /// Additional opacity applied to disabled rows.
    pub disabled_opacity: f32,
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

pub fn default_selector_items_panel_look(tokens: &ThemeTokens, size: ControlSize) -> SelectorItemsPanelLook {
    SelectorItemsPanelLook::from_menu(
        crate::controls::floating_menu::default_floating_menu_look(tokens, size),
        &tokens.metrics,
        &tokens.stylesheet,
        size.stylesheet_key(),
    )
}

impl SelectorItemsPanelLook {
    /// Share menu chrome, metric border width and popover placement before panel overrides.
    pub fn from_menu(
        menu: crate::controls::floating_menu::FloatingMenuLook,
        metrics: &crate::theme::MetricTokens,
        stylesheet: &crate::theme::stylesheet::CommonStylesheet,
        key: &str,
    ) -> Self {
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().popover.authored_geometry("panel");
        let placement = stylesheet.popover.authored_geometry("panel").with_fallback(&baseline);
        Self {
            positive: menu.positive,
            background: menu.background,
            foreground: menu.foreground,
            border: menu.border,
            border_width: metrics.border_width.default,
            disabled_opacity: menu.disabled_opacity,
            popup_offset_y: placement.offset_y.unwrap_or_default(),
            window_margin: placement.window_margin.unwrap_or_default(),
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
        .with_stylesheet(&stylesheet.selector_list, key)
    }

    /// Apply independent panel overrides after resolving the shared menu baseline.
    /// Looks use their own semantic size key; consumer look callbacks run afterward.
    pub fn with_stylesheet(mut self, stylesheet: &crate::theme::stylesheet::SelectorListStylesheet, key: &str) -> Self {
        crate::theme::stylesheet::apply_common_geometry!(stylesheet, key, SelectorListGeometry, self,
            [padding => padding, min_width => min_width, item_height => item_height,
             item_padding_x => item_padding_x, item_gap => item_gap, item_icon_size => item_icon_size,
             font_size => item_typography.size, line_height => item_typography.line_height,
             radius => radius, item_radius => item_radius, border_width => border_width,
             disabled_opacity => disabled_opacity, popup_offset_y => popup_offset_y, window_margin => window_margin]);
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
    pub look: SelectorItemsPanelLook,
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

pub type SelectorItemsTemplateModifier<T> =
    Box<dyn for<'a> Fn(Stateful<Div>, &SelectorItemsRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static>;

pub struct DefaultSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    modifiers: Vec<SelectorItemsTemplateModifier<T>>,
}

struct ModifiedSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    base: Arc<dyn SelectorItemsTemplate<T>>,
    modifiers: Vec<SelectorItemsTemplateModifier<T>>,
}

impl<T> DefaultSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &SelectorItemsRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &SelectorItemsRenderModel<'_, T>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
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

impl<T> ModifiedSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn new(base: Arc<dyn SelectorItemsTemplate<T>>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &SelectorItemsRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &SelectorItemsRenderModel<'_, T>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
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
        let look = &model.look;
        let direction = LocaleContext::current(cx).direction;
        let content_max_height = (model.max_height - px((look.padding + look.border_width) * 2.0)).max(px(0.0));
        let mut rows = div().relative().flex().flex_col().w_full();

        if let Some(report) = popup_geometry {
            let alignment_index =
                model.selected_index.or_else(|| model.items.iter().position(|item| item.is_enabled()));
            let inset = px(look.padding + look.border_width);
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
                look.foreground
            } else {
                look.item_disabled_foreground
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
                    .flex_row_logical(direction)
                    .items_center()
                    .gap(px(look.item_gap))
                    .child(
                        div()
                            .flex_1()
                            .when(direction == LayoutDirection::Rtl, |label| label.text_right())
                            .child(item.label().clone()),
                    )
                    .into_any_element()
            };

            let content_slot = div().flex_1().child(content);
            let selection_slot =
                div().child(render_selection_checkmark(selected, color, look.item_icon_size, model.selection_icon));
            #[cfg(feature = "test-support")]
            let content_slot = content_slot.debug_selector(move || format!("selector-row-content-{index}"));
            #[cfg(feature = "test-support")]
            let selection_slot = selection_slot.debug_selector(move || format!("selector-row-check-{index}"));
            let mut row = div()
                .id((item.id().clone(), 0usize))
                .flex()
                .flex_row_logical(direction)
                .items_center()
                .gap(px(look.item_gap))
                .min_h(px(look.item_height))
                .flex_shrink_0()
                .px(px(look.item_padding_x))
                .rounded(px(look.item_radius))
                .text_color(color)
                .text_size(px(look.item_typography.size))
                .line_height(px(look.item_typography.line_height))
                .font_weight(look.item_typography.weight)
                .child(content_slot)
                .child(selection_slot);
            #[cfg(feature = "test-support")]
            {
                row = row.debug_selector(move || format!("selector-row-{index}"));
            }

            if enabled_item {
                row = row.cursor_pointer().on_hover(hover).hover({
                    let hover_background = look.item_hover_background;
                    let hover_foreground = look.item_hover_foreground;
                    move |style| style.bg(hover_background).text_color(hover_foreground)
                });

                if active {
                    row = row.bg(look.item_hover_background).text_color(look.item_hover_foreground);
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
                row = row.opacity(look.disabled_opacity);
            }

            rows = rows.child(row);
        }

        let root = div()
            .id((model.menu_id.clone(), 0usize))
            .relative()
            .min_w(px(look.min_width))
            .max_h(model.max_height)
            .p(px(look.padding))
            .bg(look.background)
            .border(px(look.border_width))
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow.clone())
            .overflow_hidden()
            .when(policy_routing, |root| root.block_mouse_except_scroll())
            .when(!policy_routing, |root| root.occlude())
            .when_some(scroll_wheel, |root, handler| root.on_scroll(handler))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .child(rows);

        self.apply_modifiers(root, model)
    }
}

impl<T> SelectorItemsTemplate<T> for ModifiedSelectorItemsTemplate<T>
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
        self.apply_modifiers(root, model)
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
    Arc::new(ModifiedSelectorItemsTemplate::new(template).with_modifier(modifier))
}

fn render_selection_checkmark(selected: bool, color: gpui::Hsla, size: f32, icon: &IconSource) -> AnyElement {
    if selected {
        render_icon_source(icon, color, size)
    } else {
        div().size(px(size)).into_any_element()
    }
}
