//! The SDK tabs on the design surface page (issue #383). Its `.lss` (`tabs`) styles the list and each
//! tab: hover, pressed, focused, disabled, and `selected` for the active tab. Design mode pins the picked
//! states on every tab; `:selected` keeps the active tab, and without it no tab is active.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Div, Entity, SharedString, Stateful, Subscription, Window, prelude::*};
use xgpui_luma::controls::tabs::{
    ControlFocusState, StyledTabsTheme, Tabs, TabsBuilder, TabsEvent, TabsItem, TabsItemState, TabsRenderItem,
    TabsRenderModel, TabsTemplate, TabsTemplateHandlers, TabsTriggerKind, TabsWidthMode, ThemedTabsTemplate,
};
use xgpui_luma::infra::icon::DisclosureIcons;
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The tabs: id, label (with ascenders and descenders) and whether the tab is enabled.
const TABS: [(&str, &str, bool); 4] = [
    ("typography", "Typography", true),
    ("legacy", "Legacy", false),
    ("spacing", "Spacing", true),
    ("display", "Display", true),
];
const ACTIVE: &str = "typography";

/// The `.lss` states a tab reads.
const STATES: [&str; 5] = ["hover", "pressed", "focused", "selected", "disabled"];

pub(super) struct TabsSurface {
    tabs: Entity<Tabs>,
}

impl TabsSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let tabs = TabsBuilder::new("workshop-sdk-tabs")
            .items(TABS.map(|(id, label, enabled)| TabsItem::new(id).label(label).enabled(enabled)))
            .active(ACTIVE)
            .spawn(cx);
        Box::new(Self { tabs })
    }

    /// The SDK tabs, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn tabs(&self) -> Entity<Tabs> {
        self.tabs.clone()
    }
}

/// The tabs template for the `tabs` control in `theme`.
fn template(theme: Theme) -> ThemedTabsTemplate {
    let binding = StyleBinding::new("tabs", ThemeSource::Fixed(theme));
    ThemedTabsTemplate::new(Arc::new(StyledTabsTheme::new(binding)))
}

/// `model` with every tab in the picked states; `:selected` keeps the active tab.
fn pinned_model<'a>(model: &TabsRenderModel<'a>, states: &StateSet) -> TabsRenderModel<'a> {
    let (keep, disabled, focused) = (states.has("selected"), states.has("disabled"), states.has("focused"));
    TabsRenderModel {
        drag_drop: model.drag_drop.clone(),
        id: model.id,
        width_mode: model.width_mode,
        items: model
            .items
            .iter()
            .map(|item| {
                let active = keep && item.active;
                TabsRenderItem {
                    id: item.id,
                    label: item.label,
                    trigger_kind: item.trigger_kind,
                    leading_accessory: item.leading_accessory,
                    trailing_accessory: item.trailing_accessory,
                    active,
                    enabled: !disabled,
                    state: TabsItemState {
                        disabled,
                        hovered: states.has("hover"),
                        pressed: states.has("pressed"),
                        selected: active,
                        active: focused,
                        focus_visible: focused,
                    },
                    disclosure_progress: item.disclosure_progress,
                }
            })
            .collect(),
        active_id: model.active_id.filter(|_| keep),
        enabled: !disabled,
        focus: ControlFocusState { focused, focus_visible: focused },
        indicator: model.indicator.filter(|_| keep),
        indicator_motion: model.indicator_motion.filter(|_| keep),
        disclosure_icons: model.disclosure_icons,
    }
}

/// Draws the base template with the picked states (Design mode), or as the tabs are.
struct PinnedTabsTemplate {
    base: ThemedTabsTemplate,
    pinned: Option<StateSet>,
}

impl TabsTemplate for PinnedTabsTemplate {
    fn render(
        &self,
        model: &TabsRenderModel<'_>,
        handlers: TabsTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        match &self.pinned {
            Some(states) => self.base.render(&pinned_model(model, states), handlers, window, cx),
            None => self.base.render(model, handlers, window, cx),
        }
    }
}

/// Handlers that do nothing, one per tab, for a State matrix cell.
fn inert_handlers(tabs: usize) -> TabsTemplateHandlers {
    TabsTemplateHandlers {
        item_bounds: (0..tabs).map(|_| Box::new(|_: &_, _: &mut Window, _: &mut App| {}) as _).collect(),
        item_hovers: (0..tabs).map(|_| Box::new(|_: &bool, _: &mut Window, _: &mut App| {}) as _).collect(),
        item_mouse_downs: (0..tabs).map(|_| Box::new(|_: &_, _: &mut Window, _: &mut App| {}) as _).collect(),
        item_mouse_ups: (0..tabs).map(|_| Box::new(|_: &_, _: &mut Window, _: &mut App| {}) as _).collect(),
        item_mouse_up_outs: (0..tabs).map(|_| Box::new(|_: &_, _: &mut Window, _: &mut App| {}) as _).collect(),
        item_clicks: (0..tabs).map(|_| Box::new(|_: &_, _: &mut Window, _: &mut App| {}) as _).collect(),
    }
}

impl SurfaceControl for TabsSurface {
    fn title(&self) -> &'static str {
        "Tabs Navigation"
    }

    fn view(&self) -> AnyView {
        self.tabs.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let template = PinnedTabsTemplate { base: template(theme), pinned: pinned.cloned() };
        self.tabs.update(cx, |tabs, cx| tabs.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.tabs.update(cx, |tabs, cx| tabs.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        STATES.to_vec()
    }

    /// Whether a tab is active.
    fn lasting_states(&self, cx: &App) -> Vec<(&'static str, bool)> {
        vec![("selected", self.tabs.read(cx).active_id().is_some())]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.tabs, move |side: &mut ControlSide, _, event: &TabsEvent, cx| {
            // Layout reports, not something the person did.
            if matches!(event, TabsEvent::ItemBoundsChanged { .. }) {
                return;
            }
            let state = match event {
                TabsEvent::Change { .. } => Some(("selected", true)),
                TabsEvent::FocusChanged { focused } => Some(("focused", *focused)),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    /// The tabs in `states`, the first one active when `:selected` is. Cells draw no active indicator: it
    /// needs the tabs' measured bounds, which only the live control has.
    fn cell(
        &self,
        theme: &Theme,
        id: SharedString,
        states: &StateSet,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let tabs: Vec<(SharedString, SharedString)> =
            TABS.iter().map(|(id, label, _)| (SharedString::from(*id), SharedString::from(*label))).collect();
        let icons = DisclosureIcons::default();
        let active = SharedString::from(ACTIVE);
        let model = TabsRenderModel {
            drag_drop: None,
            id: &id,
            width_mode: TabsWidthMode::default(),
            items: tabs
                .iter()
                .map(|(tab, label)| TabsRenderItem {
                    id: tab,
                    label,
                    trigger_kind: TabsTriggerKind::Select,
                    leading_accessory: None,
                    trailing_accessory: None,
                    active: *tab == active,
                    enabled: true,
                    state: TabsItemState::default(),
                    disclosure_progress: 0.,
                })
                .collect(),
            active_id: Some(&active),
            enabled: true,
            focus: ControlFocusState::default(),
            indicator: None,
            indicator_motion: None,
            disclosure_icons: &icons,
        };
        let model = pinned_model(&model, states);
        template(theme.clone()).render(&model, inert_handlers(tabs.len()), window, cx).into_any_element()
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
    fn the_tabs_page_skips_disabled_tabs_and_pins_its_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "tabs-navigation");
        let side = cx.update(|_, cx| page.read(cx).side());
        let tabs = cx.update(|_, cx| side.read(cx).control_as::<TabsSurface>().expect("tabs").tabs());
        assert!(cx.update(|_, cx| side.read(cx).state_on("selected")), "a tab starts active");

        cx.update(|window, cx| tabs.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("right");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(tabs.read(cx).active_id().map(|id| id.as_ref()), Some("spacing")));
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("Change"), "tab changes are logged: {text}");

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("hover", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("hover") && pinned.has("selected"), "{pinned:?}");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw tabs");
    }
}
