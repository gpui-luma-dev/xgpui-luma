use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::Arc;

use gpui::{
    Anchor, App, Bounds, ClickEvent, Div, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Point, Size, Stateful,
    Window, anchored, deferred, div, point, px, prelude::*,
};

use super::{SelectorOpeningMode, SelectorPlacement, SelectorRenderModel};
use super::item_template::render_item_content;

use crate::controls::button::face::{Face, FaceShape};
use crate::infra::ElementExt;
use crate::infra::popup_placement::{PopupPlacementRequest, PopupSide, PopupSpace};
use crate::controls::selector_list::{
    SelectorItem, SelectorItemLike, SelectorItemsRenderModel, SelectorItemsTemplate, SelectorItemsTemplateHandlers,
    default_selector_items_template,
};
use crate::controls::button::face::FaceLook;
use crate::style::PartRead;
use crate::theme::snap_to_pixel;

use super::theme::{SelectorTheme, default_selector_theme};

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
    fn render(
        &self,
        model: &SelectorRenderModel<'_, T>,
        handlers: SelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

impl<T> RenderModel for dyn SelectorTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    type Model<'a> = SelectorRenderModel<'a, T>;
}

pub struct ThemedSelectorTemplate<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    theme: Arc<dyn SelectorTheme>,
    items_template: Arc<dyn SelectorItemsTemplate<T>>,
    modifiers: RootModifiers<dyn SelectorTemplate<T>>,
}

impl<T> ThemedSelectorTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    pub fn new(theme: Arc<dyn SelectorTheme>, items_template: Arc<dyn SelectorItemsTemplate<T>>) -> Self {
        Self { theme, items_template, modifiers: RootModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &SelectorRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
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
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl<T> SelectorTemplate<T> for ModifiedTemplate<dyn SelectorTemplate<T>>
where
    T: SelectorItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectorRenderModel<'_, T>,
        handlers: SelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply(root, model)
    }
}

impl<T> SelectorTemplate<T> for ThemedSelectorTemplate<T>
where
    T: SelectorItemLike + 'static,
{
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
        // Drawn straight from the resolved style (#433): the face through the shared face reader.
        let resolved =
            super::styled::resolve_trigger(&self.theme.style(), model.trigger_style, model.visual_state, 1.0);
        let root = resolved.root();
        let mut face_look = FaceLook::from_part(root);
        if model.without_elevation {
            face_look.shadow = None;
        }
        let trigger_icon_size = snap_to_pixel(root.number("icon_size"), scale_factor);
        let anchor = SelectorAnchor {
            trigger_height: face_look.height,
            offset_y: root.number("menu_offset_y"),
            panel: self.theme.items_panel(),
        };
        // The trigger is a button face (#430): its fill, border, shadow, focus ring and fade come from one place.
        let face = Face {
            id: "trigger".into(),
            shape: FaceShape::Padded,
            radius: root.number("radius"),
            full_width: true,
            min_height: false,
            focused: model.visual_state.interaction.focused
                && !model.visual_state.interaction.disabled
                && !model.visual_state.invalid,
            disabled: model.state.disabled,
            elevation: !model.without_elevation,
            compact: false,
        };
        let trigger_content = render_item_content(model, face_look.gap, cx);
        let chevron =
            crate::infra::icon::render_icon_source(&model.icons.trigger, root.color("icon"), trigger_icon_size);
        let trigger = face
            .control(&face_look, trigger_content, Some(chevron))
            .on_prepaint(move |bounds, window, cx| {
                trigger_bounds(&bounds, window, cx);
            })
            .on_hover(trigger_hover)
            .on_mouse_down(MouseButton::Left, trigger_mouse_down)
            .on_mouse_up(MouseButton::Left, trigger_mouse_up)
            .on_mouse_up_out(MouseButton::Left, trigger_mouse_up_out)
            .on_click(trigger_click);
        let trigger = face.root(self.modifiers.apply(trigger, model), &face_look);
        let mut root =
            div().id(model.id.clone()).w_full().relative().on_mouse_down_out(root_mouse_down_out).child(trigger);

        if model.presence.should_paint() {
            let mut popup_metrics = resolve_selector_popup_metrics(
                model.trigger_bounds,
                model.placement,
                &anchor,
                model.items.len(),
                PopupSpace::of(window, cx),
            );
            let centered = model.opening_mode == SelectorOpeningMode::Centered && !model.items.is_empty();
            if centered {
                let index = model.selected_index.or_else(|| model.items.iter().position(|item| item.is_enabled()));
                if let Some(index) = index {
                    let geometry =
                        model.popup_geometry.unwrap_or(crate::controls::selector_list::SelectorPopupGeometry {
                            row_center: px(anchor.panel.item_height) * (index as f32 + 0.5),
                            content_height: px(anchor.panel.item_height) * model.items.len(),
                            inset: px(anchor.panel.padding + anchor.panel.border_width),
                        });
                    popup_metrics = resolve_centered_popup_metrics(
                        model.trigger_bounds,
                        geometry,
                        window.viewport_size(),
                        anchor.panel.window_margin,
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
                    look: anchor.panel.clone(),
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
                .snap_to_window_with_margin(px(anchor.panel.window_margin))
                .anchor(popup_metrics.anchor)
                .position(popup_metrics.position)
                .offset(popup_metrics.offset)
                .child(div().opacity(model.presence.opacity()).child(menu));

            root = root.child(deferred(overlay).with_priority(1));
        }

        self.modifiers.apply(root, model)
    }
}

/// Where the popup opens from: the trigger's height, the gap to the popup and the popup panel.
pub(crate) struct SelectorAnchor {
    trigger_height: f32,
    offset_y: f32,
    panel: crate::controls::selector_list::SelectorItemsPanelLook,
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
    anchor: &SelectorAnchor,
    item_count: usize,
    viewport_size: impl Into<PopupSpace>,
) -> ResolvedSelectorPlacement {
    let space: PopupSpace = viewport_size.into();
    let trigger_bounds = trigger_bounds.unwrap_or_else(|| {
        Bounds::new(point(px(0.0), px(0.0)), Size { width: px(0.0), height: px(anchor.trigger_height) })
    });
    let menu_size = estimated_menu_size(anchor, item_count, trigger_bounds.size.width);
    let viewport_margin = px(anchor.panel.window_margin);
    let minimum_height = px((anchor.panel.padding * 2.0) + anchor.panel.item_height);

    if placement == SelectorPlacement::OverlayOnTrigger {
        // Covers the trigger and runs down to the window margin.
        let available = (space.viewport.height - viewport_margin - trigger_bounds.top()).max(minimum_height);
        let max_height = menu_size.height.min(available);
        return ResolvedSelectorPlacement {
            anchor: Anchor::TopLeft,
            position: trigger_bounds.origin,
            offset: point(px(0.0), px(0.0)),
            max_height,
            scrolling: menu_size.height > max_height,
            initial_scroll: px(0.0),
        };
    }

    let (side, flip) = match placement {
        SelectorPlacement::AboveStart => (PopupSide::Above, false),
        SelectorPlacement::Smart => (PopupSide::Below, true),
        _ => (PopupSide::Below, false),
    };
    let request = PopupPlacementRequest::new(trigger_bounds, side, space)
        .flip(flip)
        .gap(px(anchor.offset_y))
        .content_size(Some(menu_size))
        .window_margin(viewport_margin)
        .min_main_size(minimum_height);
    let resolved = if placement == SelectorPlacement::CenteredOnTrigger {
        request.resolve_centered()
    } else {
        request.resolve()
    };
    ResolvedSelectorPlacement {
        anchor: resolved.anchor,
        position: resolved.position,
        offset: resolved.offset,
        max_height: menu_size.height.min(resolved.max_size.height),
        scrolling: resolved.scrolling,
        initial_scroll: px(0.0),
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

fn estimated_menu_size(anchor: &SelectorAnchor, item_count: usize, trigger_width: Pixels) -> Size<Pixels> {
    let menu_min_width = px(anchor.panel.min_width);
    Size {
        width: if trigger_width > menu_min_width {
            trigger_width
        } else {
            menu_min_width
        },
        height: px(anchor.panel.padding * 2.0) + px(anchor.panel.item_height) * item_count,
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, point, px, size};

    use super::*;
    use crate::controls::selector::SelectorTriggerStyle;

    /// The SDK trigger's anchor at rest.
    fn look() -> SelectorAnchor {
        let theme = default_selector_theme();
        let resolved = super::super::styled::resolve_trigger(
            &theme.style(),
            SelectorTriggerStyle::Outline,
            Default::default(),
            1.0,
        );
        let root = resolved.root();
        SelectorAnchor {
            trigger_height: root.number("height"),
            offset_y: root.number("menu_offset_y"),
            panel: theme.items_panel(),
        }
    }

    /// The template draws the trigger straight from the resolved style (#433), through the shared face reader,
    /// with the values the `SelectorLook` compatibility type reads: at rest, hovered, pressed, disabled, focused,
    /// open and invalid.
    #[test]
    fn the_trigger_face_matches_the_look() {
        use crate::controls::button::face::FaceLook;
        use crate::controls::selector::{SelectorLook, SelectorVisualState};
        use crate::theme::InteractionState;
        let theme = default_selector_theme();
        let rest = InteractionState::default();
        for visual in [
            SelectorVisualState::default(),
            SelectorVisualState { interaction: InteractionState { hovered: true, ..rest }, ..Default::default() },
            SelectorVisualState { interaction: InteractionState { pressed: true, ..rest }, ..Default::default() },
            SelectorVisualState { interaction: InteractionState { disabled: true, ..rest }, ..Default::default() },
            SelectorVisualState { interaction: InteractionState { focused: true, ..rest }, ..Default::default() },
            SelectorVisualState { open: true, ..Default::default() },
            SelectorVisualState { invalid: true, ..Default::default() },
        ] {
            for style in [SelectorTriggerStyle::Outline, SelectorTriggerStyle::Ghost] {
                let resolved = super::super::styled::resolve_trigger(&theme.style(), style, visual, 1.0);
                let look = SelectorLook::from_resolved(&resolved, theme.items_panel(), false);
                let root = resolved.root();
                let face = FaceLook::from_part(root);
                let case = format!("{style:?} {visual:?}");
                assert_eq!(
                    (face.background, face.foreground),
                    (look.trigger_background, look.trigger_foreground),
                    "{case}"
                );
                assert_eq!(
                    (face.border, face.shadow.clone()),
                    (look.trigger_border, look.trigger_shadow.clone()),
                    "{case}"
                );
                assert_eq!(face.typography, look.trigger_typography, "{case}");
                assert_eq!(
                    (face.padding_x, face.padding_y, face.gap, face.height),
                    (look.trigger_padding_x, look.trigger_padding_y, look.trigger_gap, look.trigger_height),
                    "{case}"
                );
                assert_eq!(
                    (face.focus_border, face.opacity),
                    (look.trigger_focus_border, look.trigger_opacity),
                    "{case}"
                );
                assert_eq!(root.color("icon"), look.trigger_icon, "{case}");
                assert_eq!(
                    (root.number("radius"), root.number("icon_size")),
                    (look.trigger_radius, look.trigger_icon_size)
                );
            }
        }
        // Border width and focus geometry, read per state now, match the theme's metrics at rest.
        let resolved = super::super::styled::resolve_trigger(
            &theme.style(),
            SelectorTriggerStyle::Outline,
            Default::default(),
            1.0,
        );
        let face = FaceLook::from_part(resolved.root());
        let metrics = theme.metrics();
        assert_eq!(face.border_width, metrics.border_width);
        assert_eq!(face.focus.extent(), crate::infra::focus_adorner::FocusAdorner::new(metrics.focus).extent());
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
