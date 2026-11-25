//! SDK menus on the design surface page (issue #383): Context Menu (`context_menu.lss`, its target) and
//! Popup Menu (`popup_menu.lss`, which extends `button`, its trigger). Both take a theme, so Design mode pins
//! the picked states on the target or trigger. The menus they open keep the SDK's floating-menu look. Each
//! State matrix cell is a small target or trigger of its own, drawn in that cell's states.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::context_menu::{
    ContextMenu, ContextMenuBuilder, ContextMenuEvent, ContextMenuLook, ContextMenuTheme, MenuItem,
    StyledContextMenuTheme, ThemedContextMenuTemplate,
};
use xgpui_luma::controls::popup_menu::{MenuSource, PopupMenu, PopupMenuBuilder, PopupMenuEvent, ThemedPopupMenuTemplate};
use xgpui_luma::style::ThemeSource;
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

    /// `PopupMenuBuilder` has no way to make the trigger invalid.
    fn unreachable_states(&self) -> &'static [&'static str] {
        &["invalid"]
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
}
