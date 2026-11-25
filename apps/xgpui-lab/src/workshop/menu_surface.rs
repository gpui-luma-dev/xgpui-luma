//! SDK menus on the design surface page (issue #383). Floating Menu draws the menu panel every SDK menu
//! draws, open and inline, from `floating_menu.lss`. Context Menu (`context_menu.lss`) is the area a context
//! menu opens from, and Popup Menu (`popup_menu.lss`, which extends `button`) is its trigger; both take a
//! theme, so Design pins their picked states, and the menus they open keep the SDK's floating-menu look.
//! Each State matrix cell is a panel, target or trigger in that cell's states.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::context_menu::{
    ContextMenu, ContextMenuBuilder, ContextMenuEvent, ContextMenuLook, ContextMenuTheme, StyledContextMenuTheme,
    ThemedContextMenuTemplate,
};
use xgpui_luma::controls::floating_menu::{
    FloatingMenuClickHandler, FloatingMenuHoverHandler, FloatingMenuRenderModel, FloatingMenuTemplateHandlers,
    default_floating_menu_template, styled_floating_menu_look,
};
use xgpui_luma::infra::icon::DisclosureIcons;
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::infra::state::MenuPath;
use xgpui_luma::motion::overlay_presence::OverlayPresence;
use xgpui_luma::controls::popup_menu::{MenuSource, PopupMenu, PopupMenuBuilder, PopupMenuEvent, ThemedPopupMenuTemplate};
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::theme::{InteractionState, MetricTokens};

use super::selection_surface::popup_menu_theme;
use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The menu: items with ascenders and descenders, a submenu, a separator and a disabled item.
fn items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("duplicate").label("Duplicate page"),
        MenuItem::new("export")
            .label("Export")
            .submenu([MenuItem::new("pdf").label("PDF"), MenuItem::new("png").label("PNG graphic")]),
        MenuItem::separator("separator"),
        MenuItem::new("legacy").label("Legacy layout").enabled(false),
        MenuItem::new("copy").label("Copy styling"),
    ]
}

/// Matrix cells, made with the theme last bound and cleared when the page binds again.
#[derive(Default)]
struct Cells(RefCell<HashMap<SharedString, AnyView>>);

impl Cells {
    fn get(&self, id: &SharedString, make: impl FnOnce() -> AnyView) -> AnyView {
        self.0.borrow_mut().entry(id.clone()).or_insert_with(make).clone()
    }

    fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}

// --- Floating Menu -----------------------------------------------------------------------------------------
//
// The menu panel every SDK menu draws (context menus, popup menus, selector popups) from `floating_menu.lss`,
// drawn open, inline on the stage. Rows hover live in Test mode; Design pins `:hover` on the first row or `:disabled` on every row.

/// What the live menu panel reports: a row hovered or left, or chosen.
#[derive(Debug)]
enum MenuPanelEvent {
    Hover { item: SharedString, hovered: bool },
    Click { item: SharedString },
}

/// The menu panel drawn from `theme`, in the picked states if any, with live handlers if given.
fn menu_panel(
    id: &SharedString,
    theme: &Theme,
    pinned: Option<&StateSet>,
    handlers: Option<(Vec<FloatingMenuHoverHandler>, Vec<FloatingMenuClickHandler>)>,
) -> AnyElement {
    let disabled = pinned.is_some_and(|states| states.has("disabled"));
    let items: Vec<MenuItem> = items()
        .into_iter()
        .map(|item| {
            if disabled && !item.is_separator() {
                item.enabled(false)
            } else {
                item
            }
        })
        .collect();
    let active_path = pinned.is_some_and(|states| states.has("hover")).then_some(MenuPath::Root(0));
    let look = styled_floating_menu_look(&StyleBinding::new("floating_menu", ThemeSource::Fixed(theme.clone())));
    let icons = DisclosureIcons::default();
    let live = handlers.is_some();
    let (item_hovers, item_clicks) = handlers.unwrap_or_else(|| {
        let noop: Vec<FloatingMenuHoverHandler> =
            items.iter().map(|_| Box::new(|_: &bool, _: &mut Window, _: &mut App| {}) as _).collect();
        (noop, Vec::new())
    });
    let model = FloatingMenuRenderModel {
        id,
        items: &items,
        open_submenu: None,
        active_path,
        highlight: None,
        submenu_presence: OverlayPresence::new(true, false),
        look,
        disclosure_icons: &icons,
        submenu_transition: None,
    };
    // Only the live, unpinned panel follows the pointer; cells and Design draw just the picked states.
    let handlers = FloatingMenuTemplateHandlers {
        item_hovers,
        submenu_hovers: Vec::new(),
        controlled_hover: !live || pinned.is_some(),
        item_clicks,
    };
    default_floating_menu_template().render(&model, handlers).into_any_element()
}

/// Hosts the live menu panel, a view so the page can show and subscribe to it.
struct MenuPanel {
    id: SharedString,
    theme: Option<Theme>,
    pinned: Option<StateSet>,
    enabled: bool,
}

impl gpui::EventEmitter<MenuPanelEvent> for MenuPanel {}

impl Render for MenuPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(theme) = self.theme.clone() else {
            return div().into_any_element();
        };
        let items = items();
        let hovers = items
            .iter()
            .map(|item| {
                let item = item.id().clone();
                Box::new(cx.listener(move |this: &mut Self, hovered: &bool, _, cx| {
                    if this.enabled {
                        cx.emit(MenuPanelEvent::Hover { item: item.clone(), hovered: *hovered });
                    }
                })) as FloatingMenuHoverHandler
            })
            .collect();
        let clicks = items
            .iter()
            .filter(|item| !item.is_separator() && item.is_enabled() && item.submenu_items().is_empty())
            .map(|item| {
                let item = item.id().clone();
                Box::new(cx.listener(move |this: &mut Self, _: &gpui::ClickEvent, _, cx| {
                    if this.enabled {
                        cx.emit(MenuPanelEvent::Click { item: item.clone() });
                    }
                })) as FloatingMenuClickHandler
            })
            .collect();
        // With input off (Design) and nothing picked, the panel draws at rest, not following the pointer.
        let rest = StateSet::new(std::iter::empty::<&str>());
        let pinned = if self.enabled {
            self.pinned.as_ref()
        } else {
            self.pinned.as_ref().or(Some(&rest))
        };
        menu_panel(&self.id, &theme, pinned, Some((hovers, clicks)))
    }
}

pub(super) struct FloatingMenuSurface {
    panel: Entity<MenuPanel>,
}

impl FloatingMenuSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let panel =
            cx.new(|_| MenuPanel { id: "workshop-sdk-context-menu".into(), theme: None, pinned: None, enabled: true });
        Box::new(Self { panel })
    }
}

impl SurfaceControl for FloatingMenuSurface {
    fn title(&self) -> &'static str {
        "Floating Menu"
    }

    fn view(&self) -> AnyView {
        self.panel.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        self.panel.update(cx, |panel, cx| {
            panel.theme = Some(theme);
            panel.pinned = pinned.cloned();
            cx.notify();
        });
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.panel.update(cx, |panel, cx| {
            panel.enabled = enabled;
            cx.notify();
        });
    }

    fn template_states(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.panel, move |side: &mut ControlSide, _, event: &MenuPanelEvent, cx| {
            let (line, state) = match event {
                MenuPanelEvent::Hover { item, hovered } => {
                    (format!("Hover {{ item: {item}, hovered: {hovered} }}"), Some(("hover", *hovered)))
                }
                MenuPanelEvent::Click { item } => (format!("Click {{ item: {item} }}"), None),
            };
            on_event(side, SurfaceEvent { line, state }, cx);
        })
    }

    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, _: &mut App) -> AnyElement {
        menu_panel(&id, theme, Some(states), None)
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// --- Context Menu ------------------------------------------------------------------------------------------

struct PinnedContextMenuTheme {
    base: StyledContextMenuTheme,
    pinned: Option<StateSet>,
}

impl ContextMenuTheme for PinnedContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> ContextMenuLook {
        self.base.resolve(self.pinned.as_ref().map_or(state, StateSet::interaction))
    }

    fn metrics(&self) -> MetricTokens {
        self.base.metrics()
    }
}

fn context_template(theme: Theme, pinned: Option<StateSet>) -> Arc<ThemedContextMenuTemplate> {
    let menu: MenuSource = Arc::new(|| xgpui_luma::controls::floating_menu::default_floating_menu_theme().resolve());
    let base = StyledContextMenuTheme::new(ThemeSource::Fixed(theme), menu);
    Arc::new(ThemedContextMenuTemplate::new(Arc::new(PinnedContextMenuTheme { base, pinned })))
}

fn context_menu(id: impl Into<SharedString>) -> ContextMenuBuilder {
    ContextMenuBuilder::new(id).label("Right-click to copy styling").items(items())
}

pub(super) struct ContextMenuSurface {
    menu: Entity<ContextMenu>,
    cells: Cells,
}

impl ContextMenuSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Box::new(Self { menu: context_menu("workshop-sdk-context-menu").spawn(cx), cells: Cells::default() })
    }

    /// The SDK context menu, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn menu(&self) -> Entity<ContextMenu> {
        self.menu.clone()
    }
}

impl SurfaceControl for ContextMenuSurface {
    fn title(&self) -> &'static str {
        "Context Menu"
    }

    fn view(&self) -> AnyView {
        self.menu.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let template = context_template(theme, pinned.cloned());
        self.menu.update(cx, |menu, cx| menu.set_template(template, cx));
        self.cells.clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.menu.update(cx, |menu, cx| menu.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["focused"]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.menu, move |side: &mut ControlSide, _, event: &ContextMenuEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, ContextMenuEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                ContextMenuEvent::HoverChanged { hovered } => Some(("hover", *hovered)),
                ContextMenuEvent::FocusChanged { focused } => Some(("focused", *focused)),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let cell = self.cells.get(&id, || {
            context_menu(id.clone())
                .template(context_template(theme.clone(), Some(states.clone())))
                .spawn(cx)
                .into()
        });
        div().w(px(240.)).child(cell).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// --- Popup Menu --------------------------------------------------------------------------------------------

fn popup_template(theme: Theme, pinned: Option<StateSet>) -> Arc<ThemedPopupMenuTemplate> {
    Arc::new(ThemedPopupMenuTemplate::new(popup_menu_theme(theme, pinned)))
}

fn popup_menu(id: impl Into<SharedString>) -> PopupMenuBuilder {
    PopupMenuBuilder::new(id).label("Display options").items(items())
}

pub(super) struct PopupMenuSurface {
    menu: Entity<PopupMenu>,
    cells: Cells,
}

impl PopupMenuSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Box::new(Self { menu: popup_menu("workshop-sdk-popup-menu").spawn(cx), cells: Cells::default() })
    }

    /// The SDK popup menu, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn menu(&self) -> Entity<PopupMenu> {
        self.menu.clone()
    }
}

impl SurfaceControl for PopupMenuSurface {
    fn title(&self) -> &'static str {
        "Popup Menu"
    }

    fn view(&self) -> AnyView {
        self.menu.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let template = popup_template(theme, pinned.cloned());
        self.menu.update(cx, |menu, cx| menu.set_template(template, cx));
        self.cells.clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.menu.update(cx, |menu, cx| menu.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["focused"]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.menu, move |side: &mut ControlSide, _, event: &PopupMenuEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, PopupMenuEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                PopupMenuEvent::HoverChanged { hovered } => Some(("hover", *hovered)),
                PopupMenuEvent::FocusChanged { focused } => Some(("focused", *focused)),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let cell = self.cells.get(&id, || {
            popup_menu(id.clone())
                .template(popup_template(theme.clone(), Some(states.clone())))
                .spawn(cx)
                .into()
        });
        div().child(cell).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{Focusable, TestAppContext};

    use super::*;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_menu_pages_open_select_a_submenu_item_and_pin_their_states() {
        for (id, open_keys) in [("context-menu", "shift-f10"), ("popup-menu", "space")] {
            let mut app = TestAppContext::single();
            let (page, cx) = open_surface(&mut app, id);
            let side = cx.update(|_, cx| page.read(cx).side());
            cx.update(|window, cx| {
                let side = side.read(cx);
                let focus = if let Some(surface) = side.control_as::<ContextMenuSurface>() {
                    surface.menu().read(cx).focus_handle(cx)
                } else {
                    side.control_as::<PopupMenuSurface>().expect("a menu").menu().read(cx).focus_handle(cx)
                };
                focus.focus(window, cx);
            });
            cx.run_until_parked();
            cx.simulate_keystrokes(open_keys);
            cx.run_until_parked();
            cx.simulate_keystrokes("home down right enter");
            cx.run_until_parked();
            let text = cx.update(|_, cx| side.read(cx).events_text(cx));
            assert!(text.contains("pdf"), "{id} selects from the submenu: {text}");

            side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
            side.update(cx, |side, cx| side.choose_interaction("hover", cx));
            cx.run_until_parked();
            let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
            assert!(pinned.has("hover"), "{id}: {pinned:?}");

            side.update(cx, |side, cx| side.show_matrix(cx));
            cx.run_until_parked();
            assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "{id}'s matrix draws");
        }
    }

    #[test]
    fn the_floating_menu_page_draws_the_open_panel_and_pins_row_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "floating-menu");
        let side = cx.update(|_, cx| page.read(cx).side());
        assert!(cx.update(|_, cx| side.read(cx).control_as::<FloatingMenuSurface>().is_some()), "the panel page");

        // Test: hovering a row is logged and followed by the key; choosing one is logged.
        let row =
            cx.debug_bounds("workshop-surface-control").expect("the panel").origin + gpui::point(px(24.), px(16.));
        cx.simulate_mouse_move(row, None, gpui::Modifiers::none());
        cx.run_until_parked();
        cx.simulate_click(row, gpui::Modifiers::none());
        cx.run_until_parked();
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("Hover { item: duplicate, hovered: true }"), "{text}");
        assert!(text.contains("Click { item: duplicate }"), "{text}");

        // Design pins a row's :hover; the matrix draws panels.
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("hover", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("hover"), "{pinned:?}");
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "the matrix draws panels");
    }
}
