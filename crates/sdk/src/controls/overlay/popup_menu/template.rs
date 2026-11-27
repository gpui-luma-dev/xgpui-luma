use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Div, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Size, Stateful,
    Window, anchored, deferred, div, point, px, prelude::*,
};
use lucide_svg_static::Icon as LucideIcon;

use super::{PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTriggerModel};
use crate::controls::button::face::{Face, FaceLook, FaceShape};
use crate::infra::ElementExt;
use crate::infra::popup_placement::{PopupAlign, PopupPlacementRequest, PopupSide, PopupSpace, ResolvedPopupPlacement};
use crate::controls::floating_menu::render_floating_menu_with_submenu_hovers_and_icons_and_transition;
use crate::infra::icon::render_disclosure_icon;
use crate::controls::popup_menu::{PopupMenuTheme, default_popup_menu_theme};
use crate::style::PartRead;
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

impl RenderModel for dyn PopupMenuTemplate {
    type Model<'a> = PopupMenuRenderModel<'a>;
}

pub struct ThemedPopupMenuTemplate {
    theme: Arc<dyn PopupMenuTheme>,
    modifiers: RootModifiers<dyn PopupMenuTemplate>,
}

impl ThemedPopupMenuTemplate {
    pub fn new(theme: Arc<dyn PopupMenuTheme>) -> Self {
        Self { theme, modifiers: RootModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &PopupMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
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
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl PopupMenuTemplate for ModifiedTemplate<dyn PopupMenuTemplate> {
    fn render(
        &self,
        model: &PopupMenuRenderModel<'_>,
        handlers: PopupMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply(root, model)
    }
}

impl PopupMenuTemplate for ThemedPopupMenuTemplate {
    fn render(
        &self,
        model: &PopupMenuRenderModel<'_>,
        handlers: PopupMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
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
        // The outer split frame owns focus; each face owns its hover/pressed paint.
        // An open trigger draws hovered, like the selector and trigger button (#430); the disclosure face of a
        // split trigger is the one that opens.
        let state = if model.split {
            InteractionState { hovered: false, pressed: false, ..model.state }
        } else {
            InteractionState { hovered: model.state.hovered || model.open, ..model.state }
        };
        // Drawn straight from the resolved style (#433): the face through the shared face reader.
        let style = self.theme.style();
        let resolved = super::styled::resolve_trigger(&style, model.trigger_style, state, model.open, scale_factor);
        let root = resolved.root();
        let mut face_look = FaceLook::from_part(root);
        if model.without_elevation {
            face_look.shadow = None;
        }
        let trigger_icon = root.color_opt("icon").unwrap_or(face_look.foreground);
        let icon_size = root.number("icon_size");
        let anchor = MenuAnchor {
            trigger_height: face_look.height,
            offset_y: root.number("menu_offset_y"),
            menu: self.theme.menu(),
        };
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
        let face_content = (model.content)(&trigger_model, cx);
        // The trigger is a button face (#430): its fill, border, shadow, focus ring and fade come from one place.
        let face = Face {
            id: "trigger".into(),
            shape: if model.split {
                FaceShape::Bare
            } else if model.icon_only {
                FaceShape::Square
            } else {
                FaceShape::Padded
            },
            radius: model.trigger_radius_override.unwrap_or_else(|| root.number("radius")),
            full_width: model.full_width,
            min_height: !model.split && !model.icon_only,
            focused: model.state.focused && !model.state.disabled,
            disabled: model.state.disabled,
            elevation: !model.without_elevation,
            compact: false,
        };

        let mut trigger = if model.split {
            // The menu opens from the disclosure face; the action face never looks open.
            let primary = super::styled::resolve_trigger(
                &style,
                model.trigger_style,
                model.split_state.primary,
                false,
                scale_factor,
            );
            let primary = primary.root();
            let disclosure = super::styled::resolve_trigger(
                &style,
                model.trigger_style,
                InteractionState {
                    hovered: model.split_state.disclosure.hovered || model.open,
                    ..model.split_state.disclosure
                },
                model.open,
                scale_factor,
            );
            let disclosure = disclosure.root();
            let primary_foreground = primary.color("foreground");
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
                .bg(primary.color("background"))
                .h_full()
                .flex()
                .flex_1()
                .min_w(px(0.0))
                .items_center()
                .px(px(face_look.padding_x))
                .py(px(face_look.padding_y))
                .text_color(primary_foreground)
                .on_click(action_click)
                .child(face_content)
                .when_some(model.end_icon, |face, icon| {
                    face.gap(px(face_look.gap)).child(render_lucide_icon(icon, primary_foreground, icon_size))
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
                .bg(disclosure.color("background"))
                .flex()
                .items_center()
                .justify_center()
                .w(px(face_look.height))
                .h_full()
                .flex_shrink_0()
                .text_size(px(icon_size))
                .line_height(px(icon_size))
                .child(render_disclosure_icon(
                    model.disclosure_icons,
                    model.disclosure_progress,
                    disclosure.color_opt("icon").unwrap_or(disclosure.color("foreground")),
                    icon_size,
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
            if model.enabled {
                action_face = crate::style::fade(action_face, primary.opacity());
            }
            let faces = div()
                .flex()
                .size_full()
                .child(action_face)
                .child(
                    div()
                        .h_full()
                        .border_l(px(face_look.border_width))
                        .border_color(face_look.foreground)
                        .child(secondary),
                )
                .into_any_element();
            face.control(&face_look, faces, None)
        } else if model.icon_only {
            let icon_face = model
                .icon
                .map(|icon| render_lucide_icon(icon, face_look.foreground, icon_size))
                .unwrap_or(face_content);
            face.control(&face_look, icon_face, None)
                .text_size(px(icon_size))
                .line_height(px(icon_size))
                .on_click(trigger_click)
        } else {
            let trailing = if let Some(end_icon) = model.end_icon {
                render_lucide_icon(end_icon, trigger_icon, icon_size)
            } else {
                render_disclosure_icon(model.disclosure_icons, model.disclosure_progress, trigger_icon, icon_size)
            };
            face.control(&face_look, face_content, Some(trailing)).on_click(trigger_click)
        };
        trigger = trigger
            .on_prepaint(move |bounds, window, cx| trigger_bounds(&bounds, window, cx))
            .on_hover(trigger_hover)
            .on_mouse_down(MouseButton::Left, trigger_mouse_down)
            .on_mouse_up(MouseButton::Left, trigger_mouse_up)
            .on_mouse_up_out(MouseButton::Left, trigger_mouse_up_out);
        let trigger = face.root(trigger, &face_look);

        let mut root = div()
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
            .on_mouse_down_out(root_mouse_down_out)
            .child(trigger);

        if model.presence.should_paint() {
            let placement = resolve_popup_menu_placement(
                model.trigger_bounds,
                model.placement,
                &anchor,
                model.items,
                PopupSpace::of(window, cx),
            );
            let content_size = estimated_menu_size(
                &anchor,
                model.items,
                model.trigger_bounds.map(|bounds| bounds.size.width).unwrap_or(px(face_look.height)),
            );
            let offset = model.presence.adjust_offset(placement.offset, content_size);
            let menu = div().opacity(model.presence.opacity()).child(
                render_floating_menu_with_submenu_hovers_and_icons_and_transition(
                    model.id,
                    model.items,
                    model.open_submenu,
                    model.active_path,
                    anchor.menu.clone(),
                    item_hovers,
                    submenu_hovers,
                    item_clicks,
                    model.highlight,
                    crate::infra::icon::DisclosureIcons::default(),
                    model.submenu_transition,
                )
                // Taller than the space on its side, the menu scrolls (#433).
                .when(placement.scrolling, |menu| menu.max_h(placement.max_size.height).overflow_y_scroll()),
            );
            let overlay = anchored()
                .snap_to_window_with_margin(crate::controls::floating_menu::floating_menu_window_margin(&anchor.menu))
                .anchor(placement.anchor)
                .position(placement.position)
                .offset(offset)
                .child(menu);

            root = root.child(deferred(overlay).with_priority(1));
        }

        self.modifiers.apply(root, model)
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

/// Where the menu opens from: the trigger's height, the gap to the menu and the menu's style.
struct MenuAnchor {
    trigger_height: f32,
    offset_y: f32,
    menu: crate::style::StyleBinding,
}

fn estimated_menu_size(
    anchor: &MenuAnchor,
    items: &[crate::infra::menu_item::MenuItem],
    trigger_width: Pixels,
) -> Size<Pixels> {
    let menu = crate::controls::floating_menu::floating_menu_size(&anchor.menu, items);
    Size {
        width: if trigger_width > menu.width {
            trigger_width
        } else {
            menu.width
        },
        height: menu.height,
    }
}

fn resolve_popup_menu_placement(
    trigger_bounds: Option<Bounds<Pixels>>,
    placement: PopupMenuPlacement,
    anchor: &MenuAnchor,
    items: &[crate::infra::menu_item::MenuItem],
    viewport_size: impl Into<PopupSpace>,
) -> ResolvedPopupPlacement {
    let trigger_bounds = trigger_bounds.unwrap_or_else(|| {
        Bounds::new(point(px(0.0), px(0.0)), Size { width: px(0.0), height: px(anchor.trigger_height) })
    });
    let menu_size = estimated_menu_size(anchor, items, trigger_bounds.size.width);
    let (side, align, flip) = match placement {
        PopupMenuPlacement::Smart => (PopupSide::Below, PopupAlign::Start, true),
        PopupMenuPlacement::BelowStart | PopupMenuPlacement::CenteredOnTrigger => {
            (PopupSide::Below, PopupAlign::Start, false)
        }
        PopupMenuPlacement::AboveStart => (PopupSide::Above, PopupAlign::Start, false),
        PopupMenuPlacement::RightEnd => (PopupSide::End, PopupAlign::End, false),
    };
    let request = PopupPlacementRequest::new(trigger_bounds, side, viewport_size)
        .align(align)
        .flip(flip)
        .gap(px(anchor.offset_y))
        .content_size(Some(menu_size))
        .window_margin(crate::controls::floating_menu::floating_menu_window_margin(&anchor.menu));
    if placement == PopupMenuPlacement::CenteredOnTrigger {
        request.resolve_centered()
    } else {
        request.resolve()
    }
}

fn render_lucide_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    crate::infra::icon::lucide_icon(icon, color, size)
}

#[cfg(test)]
mod tests {
    use gpui::{Anchor, Bounds, point, px, size};

    use super::*;
    use crate::controls::popup_menu::PopupMenuTriggerStyle;
    use crate::theme::InteractionState;

    /// The SDK trigger'"'"'s anchor at rest.
    fn look() -> MenuAnchor {
        let theme = default_popup_menu_theme();
        let resolved = super::super::styled::resolve_trigger(
            &theme.style(),
            PopupMenuTriggerStyle::Outline,
            InteractionState::default(),
            false,
            1.0,
        );
        let root = resolved.root();
        MenuAnchor { trigger_height: root.number("height"), offset_y: root.number("menu_offset_y"), menu: theme.menu() }
    }

    /// The template draws the trigger straight from the resolved style (#433), through the shared face reader,
    /// with the values the `PopupMenuLook` compatibility type reads: at rest, hovered, pressed, disabled and open.
    #[test]
    fn the_trigger_face_matches_the_look() {
        let theme = default_popup_menu_theme();
        let rest = InteractionState::default();
        for (state, open) in [
            (rest, false),
            (InteractionState { hovered: true, ..rest }, false),
            (InteractionState { pressed: true, ..rest }, false),
            (InteractionState { disabled: true, ..rest }, false),
            (InteractionState { hovered: true, ..rest }, true),
        ] {
            let metrics = crate::controls::popup_menu::PopupMenuTriggerMetrics { open, ..Default::default() };
            let look = super::super::styled::look(
                &theme.style(),
                theme.menu(),
                PopupMenuTriggerStyle::Outline,
                metrics,
                state,
                1.0,
            );
            let resolved =
                super::super::styled::resolve_trigger(&theme.style(), PopupMenuTriggerStyle::Outline, state, open, 1.0);
            let root = resolved.root();
            let face = FaceLook::from_part(root);
            let case = format!("{state:?} open {open}");
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
            assert_eq!((face.focus_border, face.opacity), (look.trigger_focus_border, look.trigger_opacity), "{case}");
            assert_eq!(root.color_opt("icon").unwrap_or(face.foreground), look.trigger_icon, "{case}");
            assert_eq!(
                (root.number("radius"), root.number("icon_size")),
                (look.trigger_radius, look.trigger_icon_size)
            );
        }
        // Border width and focus geometry, read per state now, match the theme's metrics at rest.
        let resolved =
            super::super::styled::resolve_trigger(&theme.style(), PopupMenuTriggerStyle::Outline, rest, false, 1.0);
        let face = FaceLook::from_part(resolved.root());
        let metrics = theme.metrics();
        assert_eq!(face.border_width, metrics.border_width);
        assert_eq!(face.focus.extent(), crate::infra::focus_adorner::FocusAdorner::new(metrics.focus).extent());
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
        look.offset_y = 4.0;
        let items = [MenuItem::new("first"), MenuItem::separator("divider"), MenuItem::new("last")];
        let trigger = Bounds::new(point(px(12.0), px(200.0)), size(px(160.0), px(32.0)));
        // A divider is shorter than a row: the estimate is the menu's own size for these rows.
        let menu = crate::controls::floating_menu::floating_menu_size(&look.menu, &items);
        let size_of = |rows: &[MenuItem]| crate::controls::floating_menu::floating_menu_size(&look.menu, rows).height;
        let (two_rows, one_row) = (size_of(&[items[0].clone(), items[2].clone()]), size_of(&items[..1]));
        assert!(menu.height - two_rows < two_rows - one_row, "the divider adds less than a row");
        assert_eq!(estimated_menu_size(&look, &items, trigger.size.width).height, menu.height);
        // Room below for the menu with its short divider, but not if the divider were a full row: it opens below.
        let (row, divider) = (two_rows - one_row, menu.height - two_rows);
        let margin = crate::controls::floating_menu::floating_menu_window_margin(&look.menu);
        let height = trigger.bottom() + px(look.offset_y) + menu.height + margin + (row - divider) * 0.5;
        let placement = resolve_popup_menu_placement(
            Some(trigger),
            PopupMenuPlacement::Smart,
            &look,
            &items,
            size(px(320.0), height),
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
        assert_eq!(placement.offset, point(px(0.0), px(look.offset_y)));
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
        assert_eq!(placement.offset, point(px(0.0), -px(look.offset_y)));
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
        assert_eq!(placement.offset, point(px(look.offset_y), px(0.0)));
    }
}
