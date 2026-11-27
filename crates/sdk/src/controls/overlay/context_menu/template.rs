use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use crate::style::{PartRead, StyleStates};
use std::sync::{Arc, OnceLock};

use gpui::{
    App, Bounds, ClickEvent, Div, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Stateful, Window, anchored,
    deferred, div, px, prelude::*, relative,
};

use super::ContextMenuRenderModel;
use crate::infra::popup_placement::{PopupPlacementRequest, PopupSide, PopupSpace};
use crate::controls::floating_menu::render_floating_menu_with_submenu_presence_and_transition;
use crate::controls::context_menu::{ContextMenuTheme, default_context_menu_theme};

pub type ContextMenuBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type ContextMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ContextMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type ContextMenuMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type ContextMenuMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct ContextMenuTemplateHandlers {
    pub target_bounds: ContextMenuBoundsHandler,
    pub target_aux_click: ContextMenuClickHandler,
    pub target_hover: ContextMenuHoverHandler,
    pub target_mouse_down: ContextMenuMouseDownHandler,
    pub target_mouse_up: ContextMenuMouseUpHandler,
    pub target_mouse_up_out: ContextMenuMouseUpHandler,
    pub root_mouse_down_out: ContextMenuMouseDownHandler,
    pub item_hovers: Vec<ContextMenuHoverHandler>,
    pub item_clicks: Vec<ContextMenuClickHandler>,
}

pub type ContextMenuTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &ContextMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait ContextMenuTemplate: Send + Sync {
    fn render(
        &self,
        model: &ContextMenuRenderModel<'_>,
        handlers: ContextMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

impl RenderModel for dyn ContextMenuTemplate {
    type Model<'a> = ContextMenuRenderModel<'a>;
}

pub struct ThemedContextMenuTemplate {
    theme: Arc<dyn ContextMenuTheme>,
    modifiers: RootModifiers<dyn ContextMenuTemplate>,
}

impl ThemedContextMenuTemplate {
    pub fn new(theme: Arc<dyn ContextMenuTheme>) -> Self {
        Self { theme, modifiers: RootModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ContextMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

pub fn default_context_menu_template() -> Arc<dyn ContextMenuTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ContextMenuTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedContextMenuTemplate::new(default_context_menu_theme())))
        .clone()
}

pub(super) fn modified_context_menu_template<F>(
    template: Arc<dyn ContextMenuTemplate>,
    modifier: F,
) -> Arc<dyn ContextMenuTemplate>
where
    F: Fn(Stateful<Div>, &ContextMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl ContextMenuTemplate for ModifiedTemplate<dyn ContextMenuTemplate> {
    fn render(
        &self,
        model: &ContextMenuRenderModel<'_>,
        handlers: ContextMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply(root, model)
    }
}

impl ContextMenuTemplate for ThemedContextMenuTemplate {
    fn render(
        &self,
        model: &ContextMenuRenderModel<'_>,
        handlers: ContextMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let ContextMenuTemplateHandlers {
            target_bounds,
            target_aux_click,
            target_hover,
            target_mouse_down,
            target_mouse_up,
            target_mouse_up_out,
            root_mouse_down_out,
            item_hovers,
            item_clicks,
        } = handlers;
        // Drawn straight from the resolved style (#433); the menu is the floating menu's look.
        let resolved = self.theme.style().resolve_states(StyleStates::new(model.state), 1.0);
        let style = resolved.root();
        let menu_look = self.theme.menu();
        let window_margin = px(style.number("window_margin"));
        let custom_content = model.target_content.map(|content| content(cx));
        let has_custom_content = custom_content.is_some();
        let mut target = div()
            .id("target")
            .relative()
            .flex()
            .on_hover(target_hover)
            .on_mouse_down(MouseButton::Right, target_mouse_down)
            .on_mouse_up(MouseButton::Right, target_mouse_up)
            .on_mouse_up_out(MouseButton::Right, target_mouse_up_out)
            .on_aux_click(target_aux_click);

        if let Some(content) = custom_content {
            target = target.items_start().justify_start().w_full().min_h(relative(1.0)).child(content);
        } else {
            let typography = style.typography(14.0, 20.0);
            target = target
                .cursor_pointer()
                .min_w(px(style.number("min_width")))
                .px(px(style.number("padding_x")))
                .py(px(style.number("padding_y")))
                .bg(style.color("background"))
                .text_color(style.color("foreground"))
                .border(px(style.number("border_width")))
                .border_color(style.color("border"))
                .rounded(px(style.number("radius")))
                .text_size(px(typography.size))
                .line_height(px(typography.line_height))
                .font_weight(typography.weight)
                .child(model.label.clone());
        }

        target = crate::style::fade(target, style.opacity());

        let mut root = div()
            .on_children_prepainted(move |bounds, window, cx| {
                if let Some(bounds) = bounds.first() {
                    target_bounds(bounds, window, cx);
                }
            })
            .id(model.id.clone())
            .relative()
            .child(target);
        if has_custom_content {
            root = root.w_full().min_h(relative(1.0));
        }

        if let Some(position) = model.menu_position {
            let menu = render_floating_menu_with_submenu_presence_and_transition(
                model.id,
                model.items,
                model.open_submenu,
                model.active_path,
                menu_look.clone(),
                item_hovers,
                item_clicks,
                model.submenu_presence.clone(),
                model.submenu_transition,
            );
            // At the pointer, opening upward when there is more room above (#433).
            let menu_size =
                gpui::size(px(menu_look.min_width), px(menu_look.padding * 2.0 + menu_look.rows_height(model.items)));
            let placement = PopupPlacementRequest::new(
                Bounds::new(position, gpui::Size::default()),
                PopupSide::Below,
                PopupSpace::of(window, cx),
            )
            .flip(true)
            .content_size(Some(menu_size))
            .window_margin(window_margin)
            .resolve();
            let overlay = anchored()
                .snap_to_window_with_margin(window_margin)
                .anchor(placement.anchor)
                .position(placement.position)
                // Dismiss relative to the menu surface. Its deferred bounds are
                // outside the target, so attaching this to the target closes
                // the menu on mouse-down before a menu item's click can run.
                .child(div().opacity(model.presence.opacity()).on_mouse_down_out(root_mouse_down_out).child(menu));

            root = root.child(deferred(overlay).with_priority(1));
        }

        self.modifiers.apply(root, model)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod resolved_style_tests {
    use gpui::{MouseMoveEvent, TestAppContext, point, px};

    use crate::controls::context_menu::{ContextMenu, ContextMenuLook, StyledContextMenuTheme};
    use crate::style::StyleStates;
    use crate::theme::InteractionState;

    use super::{ContextMenuTheme, ThemedContextMenuTemplate};

    /// The template draws the target straight from the resolved style (#433), with the values the
    /// `ContextMenuLook` compatibility type reads: the fill, stroke, corner radius and minimum width, at rest
    /// and disabled.
    #[test]
    fn the_target_draws_what_the_look_describes() {
        for enabled in [true, false] {
            let theme = StyledContextMenuTheme::sdk();
            let state = InteractionState { disabled: !enabled, ..Default::default() };
            let look = ContextMenuLook::from_resolved(
                &theme.style().resolve_states(StyleStates::new(state), 1.0),
                theme.menu(),
            );
            let mut app = TestAppContext::single();
            let (_, cx) = app.add_window_view(|_, cx| {
                ContextMenu::from_builder(
                    ContextMenu::new("parity")
                        .label("Target")
                        .enabled(enabled)
                        .template(std::sync::Arc::new(ThemedContextMenuTemplate::new(std::sync::Arc::new(theme)))),
                    cx,
                )
            });
            // Keep the pointer off the target so it draws at rest.
            cx.simulate_event(MouseMoveEvent { position: point(px(1000.0), px(400.0)), ..Default::default() });
            cx.run_until_parked();
            let scale = cx.update(|window, _| window.scale_factor());
            let quads = cx.update(|window, _| window.painted_quads().to_vec());
            let fill = quads.iter().find(|quad| quad.background == gpui::Background::from(look.target_background));
            let fill =
                fill.unwrap_or_else(|| panic!("the target is filled with the look's background (enabled {enabled})"));
            assert_eq!(fill.corner_radii.top_left, px(look.target_radius).scale(scale), "radius (enabled {enabled})");
            assert!(
                fill.bounds.size.width >= px(look.target_min_width).scale(scale),
                "minimum width (enabled {enabled})"
            );
            assert!(
                quads.iter().any(|quad| quad.border_color == look.target_border && quad.bounds == fill.bounds),
                "the target is stroked with the look's border (enabled {enabled})"
            );
        }
    }
}
