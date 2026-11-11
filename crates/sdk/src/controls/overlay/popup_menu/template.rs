use std::sync::{Arc, OnceLock};

use gpui::{
    Anchor, AnyElement, App, Bounds, ClickEvent, Div, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Point, Size,
    Stateful, Window, anchored, deferred, div, point, px, prelude::*,
};
use lucide_svg_static::Icon as LucideIcon;

use super::{PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTriggerModel};
use crate::controls::button_family::button_family_effective_border;
use crate::controls::floating_menu::render_floating_menu_with_submenu_hovers_and_icons_and_transition;
use crate::infra::icon::render_disclosure_icon;
use crate::controls::popup_menu::{PopupMenuLook, PopupMenuTheme, PopupMenuTriggerMetrics, default_popup_menu_theme};
use crate::theme::InteractionState;

pub type PopupMenuBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type PopupMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type PopupMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type PopupMenuMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type PopupMenuMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

/// Optional per-face input handlers; omitted by static template previews.
pub struct SplitFaceHandlers {
    pub primary_hover: PopupMenuHoverHandler,
    pub disclosure_hover: PopupMenuHoverHandler,
    pub primary_down: PopupMenuMouseDownHandler,
    pub disclosure_down: PopupMenuMouseDownHandler,
}

pub struct PopupMenuTemplateHandlers {
    pub split_faces: Option<SplitFaceHandlers>,
    pub trigger_bounds: PopupMenuBoundsHandler,
    pub action_click: PopupMenuClickHandler,
    pub trigger_click: PopupMenuClickHandler,
    pub trigger_hover: PopupMenuHoverHandler,
    pub trigger_mouse_down: PopupMenuMouseDownHandler,
    pub trigger_mouse_up: PopupMenuMouseUpHandler,
    pub trigger_mouse_up_out: PopupMenuMouseUpHandler,
    pub root_mouse_down_out: PopupMenuMouseDownHandler,
    pub item_hovers: Vec<PopupMenuHoverHandler>,
    pub submenu_hovers: Vec<Vec<PopupMenuHoverHandler>>,
    pub item_clicks: Vec<PopupMenuClickHandler>,
}

pub type PopupMenuTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &PopupMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait PopupMenuTemplate: Send + Sync {
    fn render(
        &self,
        model: &PopupMenuRenderModel<'_>,
        handlers: PopupMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedPopupMenuTemplate {
    theme: Arc<dyn PopupMenuTheme>,
    modifiers: Vec<PopupMenuTemplateModifier>,
}

impl ThemedPopupMenuTemplate {
    pub fn new(theme: Arc<dyn PopupMenuTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &PopupMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &PopupMenuRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

struct ModifiedPopupMenuTemplate {
    base: Arc<dyn PopupMenuTemplate>,
    modifiers: Vec<PopupMenuTemplateModifier>,
}

impl ModifiedPopupMenuTemplate {
    fn new(base: Arc<dyn PopupMenuTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: PopupMenuTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &PopupMenuRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

pub fn default_popup_menu_template() -> Arc<dyn PopupMenuTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn PopupMenuTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedPopupMenuTemplate::new(default_popup_menu_theme()))).clone()
}

pub(super) fn modified_popup_menu_template<F>(
    template: Arc<dyn PopupMenuTemplate>,
    modifier: F,
) -> Arc<dyn PopupMenuTemplate>
where
    F: Fn(Stateful<Div>, &PopupMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedPopupMenuTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl PopupMenuTemplate for ModifiedPopupMenuTemplate {
    fn render(
        &self,
        model: &PopupMenuRenderModel<'_>,
        handlers: PopupMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }
}

impl PopupMenuTemplate for ThemedPopupMenuTemplate {
    fn render(
        &self,
        model: &PopupMenuRenderModel<'_>,
        handlers: PopupMenuTemplateHandlers,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let PopupMenuTemplateHandlers {
            split_faces,
            trigger_bounds,
            action_click,
            trigger_click,
            trigger_hover,
            trigger_mouse_down,
            trigger_mouse_up,
            trigger_mouse_up_out,
            root_mouse_down_out,
            item_hovers,
            submenu_hovers,
            item_clicks,
        } = handlers;
        let scale_factor = window.scale_factor();
        let metrics = PopupMenuTriggerMetrics {
            size: model.trigger_size,
            menu_size: model.menu_size,
            without_elevation: model.without_elevation,
            icon_only: model.icon_only,
            trigger_radius_override: model.trigger_radius_override,
        };
        // The outer split frame owns focus; each face owns its hover/pressed paint.
        let state = if model.split {
            InteractionState { hovered: false, pressed: false, ..model.state }
        } else {
            model.state
        };
        let mut look = self.theme.resolve_look(model.trigger_style, metrics, state, scale_factor, _cx);
        let focused = model.state.focused && !model.state.disabled;
        let border = button_family_effective_border(look.trigger_border);
        let focus = crate::infra::focus_adorner::FocusAdorner::new(self.theme.metrics().focus);
        let focus_extent = focus.extent();
        let trigger_model = PopupMenuTriggerModel {
            id: model.id.clone(),
            label: model.label.clone(),
            open: model.open,
            enabled: if model.split {
                !model.split_state.primary.disabled
            } else {
                model.enabled
            },
            state: if model.split {
                model.split_state.primary
            } else {
                model.state
            },
        };
        let face = (model.content)(&trigger_model, _cx);
        let mut trigger = div()
            .id("trigger")
            .relative()
            .flex()
            .items_center()
            .cursor_pointer()
            .on_hover(trigger_hover)
            .on_mouse_down(MouseButton::Left, trigger_mouse_down)
            .on_mouse_up(MouseButton::Left, trigger_mouse_up)
            .on_mouse_up_out(MouseButton::Left, trigger_mouse_up_out)
            .bg(look.trigger_background)
            .text_color(look.trigger_foreground)
            .rounded(px(look.trigger_radius));

        if model.split {
            let primary_look =
                self.theme.resolve_look(model.trigger_style, metrics, model.split_state.primary, scale_factor, _cx);
            let disclosure_look =
                self.theme
                    .resolve_look(model.trigger_style, metrics, model.split_state.disclosure, scale_factor, _cx);
            let mut action_face = div()
                .id("action-face")
                .role(gpui::Role::Button)
                .aria_label(model.label.clone())
                .aria_keyshortcuts("Enter Space")
                .a11y_synthetic_children({
                    let enabled = !model.split_state.primary.disabled;
                    move |builder| split_face_accessibility(builder.parent_node(), enabled, false)
                })
                .when(model.split_state.primary.disabled, |face| {
                    face.aria_description("Primary action unavailable; press Down Arrow for menu options")
                })
                .bg(primary_look.trigger_background)
                .h_full()
                .flex()
                .flex_1()
                .min_w(px(0.0))
                .items_center()
                .px(px(look.trigger_padding_x))
                .py(px(look.trigger_padding_y))
                .text_color(primary_look.trigger_foreground)
                .text_size(px(look.trigger_typography.size))
                .line_height(px(look.trigger_typography.line_height))
                .font_weight(look.trigger_typography.weight)
                .on_click(action_click)
                .child(face)
                .when_some(model.end_icon, |face, icon| {
                    face.gap(px(look.trigger_gap)).child(render_lucide_icon(
                        icon,
                        primary_look.trigger_foreground,
                        look.trigger_icon_size,
                    ))
                });
            let mut secondary = div()
                .id("secondary-face")
                .role(gpui::Role::Button)
                .aria_label(format!("{} menu", model.label))
                .aria_keyshortcuts("ArrowDown ArrowUp")
                .aria_expanded(model.open)
                .a11y_synthetic_children({
                    let enabled = model.enabled;
                    move |builder| split_face_accessibility(builder.parent_node(), enabled, true)
                })
                .bg(disclosure_look.trigger_background)
                .flex()
                .items_center()
                .justify_center()
                .w(px(look.trigger_height))
                .h_full()
                .flex_shrink_0()
                .text_size(px(look.trigger_icon_size))
                .line_height(px(look.trigger_icon_size))
                .child(render_disclosure_icon(
                    model.disclosure_icons,
                    model.disclosure_progress,
                    disclosure_look.trigger_foreground,
                    look.trigger_icon_size,
                ))
                .on_click(trigger_click);
            if let Some(faces) = split_faces {
                action_face =
                    action_face.on_hover(faces.primary_hover).on_mouse_down(MouseButton::Left, faces.primary_down);
                secondary =
                    secondary.on_hover(faces.disclosure_hover).on_mouse_down(MouseButton::Left, faces.disclosure_down);
            }
            #[cfg(any(test, feature = "test-support"))]
            {
                let primary_id = model.id.clone();
                let disclosure_id = model.id.clone();
                action_face = action_face.debug_selector(move || format!("{primary_id}/primary"));
                secondary = secondary.debug_selector(move || format!("{disclosure_id}/disclosure"));
            }
            if model.split_state.primary.disabled && model.enabled {
                action_face = action_face.opacity(self.theme.metrics().disabled_opacity);
            }
            trigger = trigger
                .overflow_hidden()
                .border(px(self.theme.metrics().border_width.default))
                .border_color(border)
                .h(px(look.trigger_height))
                .child(action_face)
                .child(
                    div()
                        .h_full()
                        .border_l(px(self.theme.metrics().border_width.default))
                        .border_color(look.trigger_foreground)
                        .child(secondary),
                );
        } else if model.icon_only {
            let icon_face = model
                .icon
                .map(|icon| render_lucide_icon(icon, look.trigger_foreground, look.trigger_icon_size))
                .unwrap_or(face);
            trigger = trigger
                .justify_center()
                .size(px(look.trigger_height))
                .text_size(px(look.trigger_icon_size))
                .line_height(px(look.trigger_icon_size))
                .child(icon_face)
                .on_click(trigger_click);
        } else {
            trigger = trigger
                .justify_between()
                .gap(px(look.trigger_gap))
                .px(px(look.trigger_padding_x))
                .py(px(look.trigger_padding_y))
                .min_h(px(look.trigger_height))
                .text_size(px(look.trigger_typography.size))
                .line_height(px(look.trigger_typography.line_height))
                .font_weight(look.trigger_typography.weight)
                .child(face)
                .child(if let Some(end_icon) = model.end_icon {
                    render_lucide_icon(end_icon, look.trigger_foreground, look.trigger_icon_size)
                } else {
                    render_disclosure_icon(
                        model.disclosure_icons,
                        model.disclosure_progress,
                        look.trigger_foreground,
                        look.trigger_icon_size,
                    )
                })
                .on_click(trigger_click);

            if model.full_width {
                trigger = trigger.w_full();
            }
        }

        if model.full_width {
            trigger = trigger.w_full();
        }

        if !model.split {
            trigger = trigger.border(px(self.theme.metrics().border_width.default)).border_color(border);
        }

        if let Some(shadows) = look.trigger_shadow.take().filter(|shadows| !shadows.is_empty()) {
            trigger = trigger.shadow(shadows);
        }

        if model.state.disabled {
            trigger = trigger.opacity(self.theme.metrics().disabled_opacity);
        }

        let mut root = div()
            .on_children_prepainted(move |bounds, window, cx| {
                if let Some(bounds) = bounds.first() {
                    trigger_bounds(bounds, window, cx);
                }
            })
            .id(model.id.clone())
            .flex()
            .flex_col()
            .items_stretch()
            .relative()
            .when(model.split, |root| {
                root.role(gpui::Role::Group).aria_label(model.label.clone()).aria_description(
                    "Enter or Space activates the primary action; Down or Up Arrow opens the menu; Escape dismisses",
                )
            })
            .on_mouse_down_out(root_mouse_down_out);

        let oversize_extent = focus_extent;
        if oversize_extent > 0.0 {
            root = root.p(px(oversize_extent)).child(trigger);
        } else {
            root = root.child(trigger);
        }
        root = root.child(focus.overlay(look.trigger_focus_border, focused, look.trigger_radius, false));

        if model.presence.should_paint() {
            let placement = resolve_popup_menu_placement(
                model.trigger_bounds,
                model.placement,
                &look,
                model.items,
                window.viewport_size(),
            );
            let content_size = estimated_menu_size(
                &look,
                model.items,
                model.trigger_bounds.map(|bounds| bounds.size.width).unwrap_or(px(look.trigger_height)),
            );
            let offset = model.presence.adjust_offset(placement.offset, content_size);
            let menu = div().opacity(model.presence.opacity()).child(
                render_floating_menu_with_submenu_hovers_and_icons_and_transition(
                    model.id,
                    model.items,
                    model.open_submenu,
                    model.active_path,
                    look.floating_menu.clone(),
                    item_hovers,
                    submenu_hovers,
                    item_clicks,
                    model.highlight,
                    crate::infra::icon::DisclosureIcons::default(),
                    model.submenu_transition,
                ),
            );
            let overlay = anchored()
                .snap_to_window_with_margin(px(look.floating_menu.window_margin))
                .anchor(placement.anchor)
                .position(placement.position)
                .offset(offset)
                .child(menu);

            root = root.child(deferred(overlay).with_priority(1));
        }

        self.apply_modifiers(root, model)
    }
}

// GPUI exposes disabled/has-popup through AccessKit's node API.
fn split_face_accessibility(node: &mut gpui::accesskit::Node, enabled: bool, disclosure: bool) {
    if !enabled {
        node.set_disabled();
    }
    if disclosure {
        node.set_has_popup(gpui::accesskit::HasPopup::Menu);
    }
}

#[derive(Clone, Copy, Debug)]
struct ResolvedPopupMenuPlacement {
    anchor: Anchor,
    position: Point<Pixels>,
    offset: Point<Pixels>,
}

fn estimated_menu_size(
    look: &PopupMenuLook,
    items: &[crate::infra::menu_item::MenuItem],
    trigger_width: Pixels,
) -> Size<Pixels> {
    let menu_min_width = px(look.floating_menu.min_width);
    Size {
        width: if trigger_width > menu_min_width {
            trigger_width
        } else {
            menu_min_width
        },
        height: px(look.floating_menu.padding * 2.0) + px(look.floating_menu.rows_height(items)),
    }
}

fn resolve_popup_menu_placement(
    trigger_bounds: Option<Bounds<Pixels>>,
    placement: PopupMenuPlacement,
    look: &PopupMenuLook,
    items: &[crate::infra::menu_item::MenuItem],
    viewport_size: Size<Pixels>,
) -> ResolvedPopupMenuPlacement {
    let trigger_bounds = trigger_bounds.unwrap_or_else(|| {
        Bounds::new(point(px(0.0), px(0.0)), Size { width: px(0.0), height: px(look.trigger_height) })
    });
    let menu_size = estimated_menu_size(look, items, trigger_bounds.size.width);
    let offset_y = px(look.menu_offset_y);
    let resolved = match placement {
        PopupMenuPlacement::Smart => {
            let viewport_bottom = viewport_size.height - px(look.floating_menu.window_margin);
            if trigger_bounds.bottom() + offset_y + menu_size.height <= viewport_bottom {
                PopupMenuPlacement::BelowStart
            } else {
                PopupMenuPlacement::AboveStart
            }
        }
        placement => placement,
    };

    match resolved {
        PopupMenuPlacement::Smart | PopupMenuPlacement::BelowStart => ResolvedPopupMenuPlacement {
            anchor: Anchor::TopLeft,
            position: point(trigger_bounds.left(), trigger_bounds.bottom()),
            offset: point(px(0.0), offset_y),
        },
        PopupMenuPlacement::AboveStart => ResolvedPopupMenuPlacement {
            anchor: Anchor::BottomLeft,
            position: point(trigger_bounds.left(), trigger_bounds.top()),
            offset: point(px(0.0), -offset_y),
        },
        PopupMenuPlacement::RightEnd => ResolvedPopupMenuPlacement {
            anchor: Anchor::BottomLeft,
            position: point(trigger_bounds.right(), trigger_bounds.bottom()),
            offset: point(offset_y, px(0.0)),
        },
        PopupMenuPlacement::CenteredOnTrigger => ResolvedPopupMenuPlacement {
            anchor: Anchor::TopLeft,
            position: trigger_bounds.center(),
            offset: point(-(menu_size.width * 0.5), -(menu_size.height * 0.5)),
        },
    }
}

fn render_lucide_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    crate::infra::icon::lucide_icon(icon, color, size)
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, point, px, size};

    use super::*;
    use crate::controls::popup_menu::{PopupMenuTriggerMetrics, PopupMenuTriggerStyle, compose_popup_menu_look};
    use crate::theme::{ControlSize, InteractionState, StandardBoxScale};

    fn look() -> PopupMenuLook {
        let theme = default_popup_menu_theme();
        compose_popup_menu_look(
            &theme.resolve(
                PopupMenuTriggerStyle::Outline,
                PopupMenuTriggerMetrics::default(),
                InteractionState::default(),
            ),
            &StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), 1.0),
        )
    }

    #[test]
    fn split_faces_expose_independent_disabled_and_menu_semantics() {
        let mut primary = gpui::accesskit::Node::new(gpui::Role::Button);
        split_face_accessibility(&mut primary, false, false);
        assert!(primary.is_disabled());
        assert_eq!(primary.has_popup(), None);
        let mut disclosure = gpui::accesskit::Node::new(gpui::Role::Button);
        split_face_accessibility(&mut disclosure, true, true);
        assert!(!disclosure.is_disabled());
        assert_eq!(disclosure.has_popup(), Some(gpui::accesskit::HasPopup::Menu));
        split_face_accessibility(&mut disclosure, false, true);
        assert!(disclosure.is_disabled());
    }

    #[test]
    fn smart_placement_accounts_for_short_separator_rows() {
        use crate::infra::menu_item::MenuItem;
        let mut look = look();
        look.floating_menu.padding = 4.0;
        look.floating_menu.item_height = 30.0;
        look.floating_menu.separator_thickness = 2.0;
        look.floating_menu.separator_spacing = 3.0;
        look.menu_offset_y = 4.0;
        let items = [MenuItem::new("first"), MenuItem::separator("divider"), MenuItem::new("last")];
        let trigger = Bounds::new(point(px(12.0), px(200.0)), size(px(160.0), px(32.0)));
        assert_eq!(estimated_menu_size(&look, &items, trigger.size.width).height, px(76.0));
        let placement = resolve_popup_menu_placement(
            Some(trigger),
            PopupMenuPlacement::Smart,
            &look,
            &items,
            size(px(320.0), px(320.0)),
        );
        assert_eq!(placement.anchor, Anchor::TopLeft);
    }

    #[test]
    fn smart_placement_uses_below_when_it_fits() {
        let look = look();
        let trigger = Bounds::new(point(px(12.0), px(80.0)), size(px(160.0), px(32.0)));
        let placement = resolve_popup_menu_placement(
            Some(trigger),
            PopupMenuPlacement::Smart,
            &look,
            &vec![crate::infra::menu_item::MenuItem::new("item"); 3],
            size(px(320.0), px(360.0)),
        );

        assert_eq!(placement.anchor, Anchor::TopLeft);
        assert_eq!(placement.position, point(trigger.left(), trigger.bottom()));
        assert_eq!(placement.offset, point(px(0.0), px(look.menu_offset_y)));
    }

    #[test]
    fn smart_placement_uses_above_when_below_is_constrained() {
        let look = look();
        let trigger = Bounds::new(point(px(12.0), px(300.0)), size(px(160.0), px(32.0)));
        let placement = resolve_popup_menu_placement(
            Some(trigger),
            PopupMenuPlacement::Smart,
            &look,
            &vec![crate::infra::menu_item::MenuItem::new("item"); 4],
            size(px(320.0), px(360.0)),
        );

        assert_eq!(placement.anchor, Anchor::BottomLeft);
        assert_eq!(placement.position, point(trigger.left(), trigger.top()));
        assert_eq!(placement.offset, point(px(0.0), -px(look.menu_offset_y)));
    }

    #[test]
    fn explicit_above_ignores_available_space() {
        let look = look();
        let trigger = Bounds::new(point(px(12.0), px(80.0)), size(px(160.0), px(32.0)));
        let placement = resolve_popup_menu_placement(
            Some(trigger),
            PopupMenuPlacement::AboveStart,
            &look,
            &vec![crate::infra::menu_item::MenuItem::new("item"); 3],
            size(px(320.0), px(360.0)),
        );

        assert_eq!(placement.anchor, Anchor::BottomLeft);
        assert_eq!(placement.position, point(trigger.left(), trigger.top()));
    }

    #[test]
    fn centered_placement_anchors_to_trigger_center() {
        let look = look();
        let trigger = Bounds::new(point(px(40.0), px(80.0)), size(px(160.0), px(32.0)));
        let menu_size =
            estimated_menu_size(&look, &vec![crate::infra::menu_item::MenuItem::new("item"); 5], trigger.size.width);
        let placement = resolve_popup_menu_placement(
            Some(trigger),
            PopupMenuPlacement::CenteredOnTrigger,
            &look,
            &vec![crate::infra::menu_item::MenuItem::new("item"); 5],
            size(px(320.0), px(360.0)),
        );

        assert_eq!(placement.anchor, Anchor::TopLeft);
        assert_eq!(placement.position, trigger.center());
        assert_eq!(placement.offset, point(-(menu_size.width * 0.5), -(menu_size.height * 0.5)));
    }

    #[test]
    fn right_end_placement_opens_beside_trigger_bottom_aligned() {
        let look = look();
        let trigger = Bounds::new(point(px(12.0), px(280.0)), size(px(220.0), px(48.0)));
        let placement = resolve_popup_menu_placement(
            Some(trigger),
            PopupMenuPlacement::RightEnd,
            &look,
            &vec![crate::infra::menu_item::MenuItem::new("item"); 4],
            size(px(800.0), px(600.0)),
        );

        assert_eq!(placement.anchor, Anchor::BottomLeft);
        assert_eq!(placement.position, point(trigger.right(), trigger.bottom()));
        assert_eq!(placement.offset, point(px(look.menu_offset_y), px(0.0)));
    }
}
