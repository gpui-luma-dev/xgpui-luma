use std::sync::Arc;

use gpui::{
    Anchor, App, Bounds, ClickEvent, Div, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Point, Size, Stateful,
    Window, anchored, deferred, div, point, px, prelude::*,
};

use super::{SelectorOpeningMode, SelectorPlacement, SelectorRenderModel};
use super::item_template::render_item_content;

use crate::controls::button_family::button_family_effective_border;
use crate::infra::ElementExt;
use crate::controls::selector_list::{
    SelectorItem, SelectorItemLike, SelectorItemsRenderModel, SelectorItemsTemplate, SelectorItemsTemplateHandlers,
    default_selector_items_template,
};
use crate::theme::{StandardBoxScale, snap_to_pixel};

use super::theme::{SelectorLook, SelectorTheme, default_selector_theme};

pub type SelectorBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type SelectorClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SelectorHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SelectorMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SelectorMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type SelectorItemHoverHandler = Arc<dyn Fn(usize, &bool, &mut Window, &mut App) + 'static>;
pub type SelectorItemMouseDownHandler = Arc<dyn Fn(usize, &MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SelectorItemClickHandler = Arc<dyn Fn(usize, &ClickEvent, &mut Window, &mut App) + 'static>;
pub type SelectorTemplateModifier<T> =
    Box<dyn for<'a> Fn(Stateful<Div>, &SelectorRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static>;

pub struct SelectorTemplateHandlers {
    /// Bind inside the popup with `SelectorRenderModel::popup_scroll`, without native wheel movement.
    pub popup_scroll_wheel: Option<crate::controls::selector_list::SelectorPanelScrollWheelHandler>,
    /// Forward to the panel to report actual row geometry for centered opening.
    pub popup_geometry: Option<crate::controls::selector_list::SelectorPopupGeometryHandler>,
    pub trigger_bounds: SelectorBoundsHandler,
    pub trigger_click: SelectorClickHandler,
    pub trigger_hover: SelectorHoverHandler,
    pub trigger_mouse_down: SelectorMouseDownHandler,
    pub trigger_mouse_up: SelectorMouseUpHandler,
    pub trigger_mouse_up_out: SelectorMouseUpHandler,
    pub root_mouse_down_out: SelectorMouseDownHandler,
    pub on_item_hover: SelectorItemHoverHandler,
    pub on_item_mouse_down: SelectorItemMouseDownHandler,
    pub on_item_click: SelectorItemClickHandler,
}

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}
fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}
fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

impl Default for SelectorTemplateHandlers {
    fn default() -> Self {
        Self {
            popup_scroll_wheel: None,
            popup_geometry: None,
            trigger_bounds: Box::new(noop_bounds),
            trigger_click: Box::new(noop_click),
            trigger_hover: Box::new(noop_hover),
            trigger_mouse_down: Box::new(noop_mouse_down),
            trigger_mouse_up: Box::new(noop_mouse_up),
            trigger_mouse_up_out: Box::new(noop_mouse_up),
            root_mouse_down_out: Box::new(noop_mouse_down),
            on_item_hover: Arc::new(|_, _, _, _| {}),
            on_item_mouse_down: Arc::new(|_, _, _, _| {}),
            on_item_click: Arc::new(|_, _, _, _| {}),
        }
    }
}

pub trait SelectorTemplate<T = SelectorItem>: Send + Sync
where
    T: SelectorItemLike + 'static,
{
    fn resolve_look(&self, model: &SelectorRenderModel<'_, T>, window: &Window, cx: &mut App) -> SelectorLook;

    fn render(
        &self,
        model: &SelectorRenderModel<'_, T>,
        handlers: SelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedSelectorTemplate<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    theme: Arc<dyn SelectorTheme>,
    items_template: Arc<dyn SelectorItemsTemplate<T>>,
    modifiers: Vec<SelectorTemplateModifier<T>>,
}

struct ModifiedSelectorTemplate<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    base: Arc<dyn SelectorTemplate<T>>,
    modifiers: Vec<SelectorTemplateModifier<T>>,
}

impl<T> ThemedSelectorTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    pub fn new(theme: Arc<dyn SelectorTheme>, items_template: Arc<dyn SelectorItemsTemplate<T>>) -> Self {
        Self { theme, items_template, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &SelectorRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &SelectorRenderModel<'_, T>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

impl<T> ModifiedSelectorTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn new(base: Arc<dyn SelectorTemplate<T>>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: SelectorTemplateModifier<T>) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &SelectorRenderModel<'_, T>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }

    fn into_arc(self) -> Arc<dyn SelectorTemplate<T>> {
        Arc::new(self)
    }
}

pub fn default_selector_template<T>() -> Arc<dyn SelectorTemplate<T>>
where
    T: SelectorItemLike + 'static,
{
    Arc::new(ThemedSelectorTemplate::new(default_selector_theme(), default_selector_items_template::<T>()))
}

pub(super) fn modified_selector_template<T, F>(
    template: Arc<dyn SelectorTemplate<T>>,
    modifier: F,
) -> Arc<dyn SelectorTemplate<T>>
where
    T: SelectorItemLike + 'static,
    F: for<'a> Fn(Stateful<Div>, &SelectorRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
{
    ModifiedSelectorTemplate::new(template).with_modifier(Box::new(modifier)).into_arc()
}

impl<T> SelectorTemplate<T> for ModifiedSelectorTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn resolve_look(&self, model: &SelectorRenderModel<'_, T>, window: &Window, cx: &mut App) -> SelectorLook {
        self.base.resolve_look(model, window, cx)
    }

    fn render(
        &self,
        model: &SelectorRenderModel<'_, T>,
        handlers: SelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }
}

impl<T> SelectorTemplate<T> for ThemedSelectorTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn resolve_look(&self, model: &SelectorRenderModel<'_, T>, window: &Window, _cx: &mut App) -> SelectorLook {
        let scale_factor = window.scale_factor();
        let scale = StandardBoxScale::compute(model.size, &self.theme.metrics(), scale_factor);
        self.theme.resolve_visual_look(
            model.trigger_style,
            model.visual_state,
            model.size,
            &scale,
            model.without_elevation,
        )
    }

    fn render(
        &self,
        model: &SelectorRenderModel<'_, T>,
        handlers: SelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let SelectorTemplateHandlers {
            popup_scroll_wheel,
            popup_geometry,
            trigger_bounds,
            trigger_click,
            trigger_hover,
            trigger_mouse_down,
            trigger_mouse_up,
            trigger_mouse_up_out,
            root_mouse_down_out,
            on_item_hover,
            on_item_mouse_down,
            on_item_click,
        } = handlers;
        let scale_factor = window.scale_factor();
        let scale = StandardBoxScale::compute(model.size, &self.theme.metrics(), scale_factor);
        let mut look = self.theme.resolve_visual_look(
            model.trigger_style,
            model.visual_state,
            model.size,
            &scale,
            model.without_elevation,
        );
        let focused = model.visual_state.interaction.focused
            && !model.visual_state.interaction.disabled
            && !model.visual_state.invalid;
        let trigger_icon_size = snap_to_pixel(look.trigger_icon_size, scale_factor);
        let border = button_family_effective_border(look.trigger_border);
        let focus = crate::infra::focus_adorner::FocusAdorner::new(self.theme.metrics().focus);
        let focus_extent = focus.extent();
        let trigger_content = render_item_content(model, &look, cx);
        let mut trigger = div()
            .id("trigger")
            .flex()
            .items_center()
            .justify_between()
            .gap(px(look.trigger_gap))
            .px(px(look.trigger_padding_x))
            .py(px(look.trigger_padding_y))
            .h(px(look.trigger_height))
            .w_full()
            .bg(look.trigger_background)
            .rounded(px(look.trigger_radius))
            .cursor_pointer()
            .relative()
            .on_prepaint(move |bounds, window, cx| {
                trigger_bounds(&bounds, window, cx);
            })
            .on_hover(trigger_hover)
            .on_mouse_down(MouseButton::Left, trigger_mouse_down)
            .on_mouse_up(MouseButton::Left, trigger_mouse_up)
            .on_mouse_up_out(MouseButton::Left, trigger_mouse_up_out)
            .on_click(trigger_click)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(look.trigger_gap))
                    .min_w(px(0.0))
                    .flex_1()
                    .text_color(look.trigger_foreground)
                    .text_size(px(look.trigger_typography.size))
                    .line_height(px(look.trigger_typography.line_height))
                    .font_weight(look.trigger_typography.weight)
                    .child(trigger_content),
            )
            .child(div().flex().items_center().justify_center().flex_shrink_0().text_color(look.trigger_icon).child(
                crate::infra::icon::render_icon_source(&model.icons.trigger, look.trigger_icon, trigger_icon_size),
            ));

        trigger = trigger.border(px(self.theme.metrics().border_width.default)).border_color(border);

        if !model.state.disabled
            && !model.without_elevation
            && let Some(shadows) = look.trigger_shadow.take().filter(|shadows| !shadows.is_empty())
        {
            trigger = trigger.shadow(shadows);
        }

        let trigger = self.apply_modifiers(trigger, model);
        let mut root = div().id(model.id.clone()).w_full().relative().on_mouse_down_out(root_mouse_down_out);
        if focus_extent > 0.0 {
            root = root.p(px(focus_extent)).child(trigger);
        } else {
            root = root.child(trigger);
        }

        root = root.child(focus.overlay(look.trigger_focus_border, focused, look.trigger_radius, false));

        if model.presence.should_paint() {
            let mut popup_metrics = resolve_selector_popup_metrics(
                model.trigger_bounds,
                model.placement,
                &look,
                model.items.len(),
                window.viewport_size(),
            );
            let centered = model.opening_mode == SelectorOpeningMode::Centered && !model.items.is_empty();
            if centered {
                let index = model.selected_index.or_else(|| model.items.iter().position(|item| item.is_enabled()));
                if let Some(index) = index {
                    let geometry =
                        model.popup_geometry.unwrap_or(crate::controls::selector_list::SelectorPopupGeometry {
                            row_center: px(look.items_panel.item_height) * (index as f32 + 0.5),
                            content_height: px(look.items_panel.item_height) * model.items.len(),
                            inset: px(look.items_panel.padding + look.items_panel.border_width),
                        });
                    popup_metrics = resolve_centered_popup_metrics(
                        model.trigger_bounds,
                        geometry,
                        window.viewport_size(),
                        look.items_panel.window_margin,
                    );
                    if model.initialize_popup_scroll
                        && let Some(scroll) = model.popup_scroll
                    {
                        scroll.set_offset(point(px(0.0), -popup_metrics.initial_scroll));
                    }
                }
            }
            let panel_template = model.panel_template.unwrap_or(self.items_template.as_ref());
            let item_hovers = (0..model.items.len())
                .map(|model_index| {
                    let on_item_hover = on_item_hover.clone();
                    Box::new(move |hovered: &bool, window: &mut Window, cx: &mut App| {
                        on_item_hover(model_index, hovered, window, cx);
                    }) as SelectorHoverHandler
                })
                .collect::<Vec<_>>();
            let item_mouse_downs = (0..model.items.len())
                .map(|model_index| {
                    let on_item_mouse_down = on_item_mouse_down.clone();
                    Box::new(move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on_item_mouse_down(model_index, event, window, cx);
                    }) as crate::controls::selector_list::SelectorPanelMouseDownHandler
                })
                .collect::<Vec<_>>();
            let item_clicks = (0..model.items.len())
                .map(|model_index| {
                    let on_item_click = on_item_click.clone();
                    Box::new(move |event: &ClickEvent, window: &mut Window, cx: &mut App| {
                        on_item_click(model_index, event, window, cx);
                    }) as SelectorClickHandler
                })
                .collect::<Vec<_>>();
            let menu = panel_template.render(
                &SelectorItemsRenderModel {
                    menu_id: model.id,
                    selector_id: model.id,
                    items: model.items,
                    selected_index: model.selected_index,
                    active_path: model.active_path,
                    open: model.open,
                    enabled: model.enabled,
                    focus: model.focus,
                    item_template: model.item_template,
                    look: look.items_panel.clone(),
                    max_height: popup_metrics.max_height,
                    scrolling: popup_metrics.scrolling,
                    scroll_handle: model.popup_scroll,
                    selection_icon: &model.icons.selected,
                },
                SelectorItemsTemplateHandlers {
                    item_hovers,
                    item_mouse_downs,
                    item_clicks,
                    scroll_wheel: popup_scroll_wheel,
                    popup_geometry: if centered { popup_geometry } else { None },
                },
                cx,
            );
            let menu = menu
                .when(centered, |menu| menu.min_w(model.trigger_bounds.map_or(px(0.0), |bounds| bounds.size.width)));
            let overlay = anchored()
                .snap_to_window_with_margin(px(look.items_panel.window_margin))
                .anchor(popup_metrics.anchor)
                .position(popup_metrics.position)
                .offset(popup_metrics.offset)
                .child(div().opacity(model.presence.opacity()).child(menu));

            root = root.child(deferred(overlay).with_priority(1));
        }

        self.apply_modifiers(root, model)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResolvedSelectorPlacement {
    anchor: Anchor,
    position: Point<Pixels>,
    offset: Point<Pixels>,
    max_height: Pixels,
    scrolling: bool,
    initial_scroll: Pixels,
}

pub(crate) fn resolve_selector_popup_metrics(
    trigger_bounds: Option<Bounds<Pixels>>,
    placement: SelectorPlacement,
    look: &SelectorLook,
    item_count: usize,
    viewport_size: Size<Pixels>,
) -> ResolvedSelectorPlacement {
    let trigger_bounds = trigger_bounds.unwrap_or_else(|| {
        Bounds::new(point(px(0.0), px(0.0)), Size { width: px(0.0), height: px(look.trigger_height) })
    });
    let menu_size = estimated_menu_size(look, item_count, trigger_bounds.size.width);
    let offset_y = px(look.menu_offset_y);
    let viewport_margin = px(look.items_panel.window_margin);
    let viewport_top = viewport_margin;
    let viewport_bottom = viewport_size.height - viewport_margin;
    let available_below = (viewport_bottom - (trigger_bounds.bottom() + offset_y)).max(px(0.0));
    let available_above = (trigger_bounds.top() - offset_y - viewport_top).max(px(0.0));
    let minimum_height = px((look.items_panel.padding * 2.0) + look.items_panel.item_height);
    let resolved = match placement {
        SelectorPlacement::Smart => {
            if trigger_bounds.bottom() + offset_y + menu_size.height <= viewport_bottom {
                SelectorPlacement::BelowStart
            } else if trigger_bounds.top() - offset_y - menu_size.height >= viewport_top {
                SelectorPlacement::AboveStart
            } else if available_below >= available_above {
                SelectorPlacement::BelowStart
            } else {
                SelectorPlacement::AboveStart
            }
        }
        placement => placement,
    };

    let available_height = match resolved {
        SelectorPlacement::Smart | SelectorPlacement::BelowStart => available_below,
        SelectorPlacement::AboveStart => available_above,
        SelectorPlacement::CenteredOnTrigger => viewport_size.height - (viewport_margin * 2.0),
        SelectorPlacement::OverlayOnTrigger => viewport_bottom - trigger_bounds.top(),
    }
    .max(minimum_height);
    let max_height = menu_size.height.min(available_height);
    let scrolling = menu_size.height > max_height;

    match resolved {
        SelectorPlacement::Smart | SelectorPlacement::BelowStart => ResolvedSelectorPlacement {
            anchor: Anchor::TopLeft,
            position: point(trigger_bounds.left(), trigger_bounds.bottom()),
            offset: point(px(0.0), offset_y),
            max_height,
            scrolling,
            initial_scroll: px(0.0),
        },
        SelectorPlacement::AboveStart => ResolvedSelectorPlacement {
            anchor: Anchor::BottomLeft,
            position: point(trigger_bounds.left(), trigger_bounds.top()),
            offset: point(px(0.0), -offset_y),
            max_height,
            scrolling,
            initial_scroll: px(0.0),
        },
        SelectorPlacement::CenteredOnTrigger => ResolvedSelectorPlacement {
            anchor: Anchor::TopLeft,
            position: trigger_bounds.center(),
            offset: point(-(menu_size.width * 0.5), -(max_height * 0.5)),
            max_height,
            scrolling,
            initial_scroll: px(0.0),
        },
        SelectorPlacement::OverlayOnTrigger => ResolvedSelectorPlacement {
            anchor: Anchor::TopLeft,
            position: trigger_bounds.origin,
            offset: point(px(0.0), px(0.0)),
            max_height,
            scrolling,
            initial_scroll: px(0.0),
        },
    }
}

/// Resolve placement and an opening scroll offset together, so clamping does not
/// hide the alignment row in long panels.
fn resolve_centered_popup_metrics(
    trigger: Option<Bounds<Pixels>>,
    geometry: crate::controls::selector_list::SelectorPopupGeometry,
    viewport: Size<Pixels>,
    window_margin: f32,
) -> ResolvedSelectorPlacement {
    let trigger = trigger.unwrap_or_else(|| Bounds::new(point(px(0.0), px(0.0)), gpui::size(px(0.0), px(0.0))));
    let margin = px(window_margin).min((viewport.height * 0.5).max(px(0.0)));
    let available = (viewport.height - margin * 2.0).max(px(0.0));
    let panel_height = geometry.content_height + geometry.inset * 2.0;
    let max_height = panel_height.min(available);
    let row_center = geometry.inset + geometry.row_center;
    let top = (trigger.center().y - row_center)
        .max(margin)
        .min((viewport.height - margin - max_height).max(margin));
    let max_scroll = (panel_height - max_height).max(px(0.0));
    let initial_scroll = (row_center - (trigger.center().y - top)).max(px(0.0)).min(max_scroll);
    ResolvedSelectorPlacement {
        anchor: Anchor::TopLeft,
        position: point(trigger.left(), top),
        offset: point(px(0.0), px(0.0)),
        max_height,
        scrolling: panel_height > max_height,
        initial_scroll,
    }
}

fn estimated_menu_size(look: &SelectorLook, item_count: usize, trigger_width: Pixels) -> Size<Pixels> {
    let menu_min_width = px(look.items_panel.min_width);
    Size {
        width: if trigger_width > menu_min_width {
            trigger_width
        } else {
            menu_min_width
        },
        height: px(look.items_panel.padding * 2.0) + px(look.items_panel.item_height) * item_count,
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, point, px, size};

    use super::*;
    use crate::controls::selector::SelectorTriggerStyle;
    use crate::theme::{ControlSize, InteractionState, StandardBoxScale};

    fn look() -> SelectorLook {
        let theme = default_selector_theme();
        theme.resolve_look(
            SelectorTriggerStyle::Outline,
            InteractionState::default(),
            ControlSize::Md,
            &StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), 1.0),
            false,
        )
    }

    #[test]
    fn centered_rows_align_first_middle_and_last_when_the_panel_fits() {
        for index in [0, 4, 9] {
            let geometry = crate::controls::selector_list::SelectorPopupGeometry {
                row_center: px(index as f32 * 32.0 + 16.0),
                content_height: px(320.0),
                inset: px(5.0),
            };
            let trigger = Bounds::new(point(px(40.0), px(480.0)), size(px(180.0), px(32.0)));
            let metrics = resolve_centered_popup_metrics(Some(trigger), geometry, size(px(800.0), px(1000.0)), 8.0);
            assert_eq!(metrics.position.x, trigger.left());
            assert_eq!(
                metrics.position.y + geometry.inset + geometry.row_center - metrics.initial_scroll,
                trigger.center().y
            );
            assert_eq!(metrics.initial_scroll, px(0.0));
            assert!(!metrics.scrolling);
        }
    }

    #[test]
    fn centered_long_list_scrolls_and_clamps_at_viewport_edges() {
        for trigger_y in [8.0, 180.0, 360.0] {
            for index in [0, 40, 79] {
                let geometry = crate::controls::selector_list::SelectorPopupGeometry {
                    row_center: px(index as f32 * 32.0 + 16.0),
                    content_height: px(2560.0),
                    inset: px(5.0),
                };
                let trigger = Bounds::new(point(px(30.0), px(trigger_y)), size(px(150.0), px(32.0)));
                let metrics = resolve_centered_popup_metrics(Some(trigger), geometry, size(px(320.0), px(400.0)), 8.0);
                let row = geometry.inset + geometry.row_center - metrics.initial_scroll;
                assert!(metrics.scrolling);
                assert_eq!(metrics.position.y, px(8.0));
                assert!(row >= px(0.0) && row <= metrics.max_height);
                assert!(metrics.initial_scroll >= px(0.0));
                assert!(metrics.initial_scroll <= geometry.content_height + geometry.inset * 2.0 - metrics.max_height);
                if index == 40 {
                    assert_eq!(metrics.position.y + row, trigger.center().y);
                }
            }
        }
    }

    #[test]
    fn centered_tiny_viewport_never_produces_negative_height() {
        let geometry = crate::controls::selector_list::SelectorPopupGeometry {
            row_center: px(16.0),
            content_height: px(32.0),
            inset: px(5.0),
        };
        let metrics = resolve_centered_popup_metrics(None, geometry, size(px(20.0), px(10.0)), 8.0);
        assert_eq!(metrics.max_height, px(0.0));
        assert!(metrics.initial_scroll >= px(0.0));
    }

    #[test]
    fn smart_placement_uses_below_when_it_fits() {
        let look = look();
        let trigger = Bounds::new(point(px(12.0), px(80.0)), size(px(160.0), px(32.0)));
        let popup_metrics = resolve_selector_popup_metrics(
            Some(trigger),
            SelectorPlacement::Smart,
            &look,
            3,
            size(px(320.0), px(360.0)),
        );

        assert_eq!(popup_metrics.anchor, Anchor::TopLeft);
        assert_eq!(popup_metrics.position, point(px(12.0), px(112.0)));
        assert!(!popup_metrics.scrolling);
    }

    #[test]
    fn overlay_placement_anchors_to_trigger_origin() {
        let look = look();
        let trigger = Bounds::new(point(px(30.0), px(70.0)), size(px(150.0), px(30.0)));
        let popup_metrics = resolve_selector_popup_metrics(
            Some(trigger),
            SelectorPlacement::OverlayOnTrigger,
            &look,
            6,
            size(px(320.0), px(360.0)),
        );

        assert_eq!(popup_metrics.anchor, Anchor::TopLeft);
        assert_eq!(popup_metrics.position, point(px(30.0), px(70.0)));
        assert_eq!(popup_metrics.offset, point(px(0.0), px(0.0)));
    }

    #[test]
    fn smart_placement_prefers_larger_visible_side_and_enables_scroll() {
        let look = look();
        let trigger = Bounds::new(point(px(12.0), px(210.0)), size(px(160.0), px(32.0)));
        let popup_metrics = resolve_selector_popup_metrics(
            Some(trigger),
            SelectorPlacement::Smart,
            &look,
            20,
            size(px(320.0), px(280.0)),
        );

        assert_eq!(popup_metrics.anchor, Anchor::BottomLeft);
        assert!(popup_metrics.scrolling);
        assert!(popup_metrics.max_height < estimated_menu_size(&look, 20, trigger.size.width).height);
    }
}
