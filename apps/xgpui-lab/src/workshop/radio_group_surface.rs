//! The SDK radio group on the design surface page (issue #383). Its rows are radio buttons, so the page
//! shows the radio's `.lss` (`radio`); the group's own `radio_group.lss` only lays the rows out and has
//! no states. Design mode pins the picked states on every row; `:selected` keeps the group's choice, and
//! without it no row is selected.

use std::cell::Cell;
use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::radio_button::{RadioButtonData, StyledRadioButtonTemplate, default_radio_button_template};
use xgpui_luma::controls::radio_group::{
    self, RadioGroup, RadioGroupEvent, RadioGroupGeometry, RadioGroupItem, RadioGroupLayout,
    radio_group_buttons_template_with,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::pinned_template::{Pin, PinnedTemplate};
use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The rows: id and label, with ascenders and descenders; the middle one starts selected.
const ROWS: [(&str, &str); 3] = [("daily", "Daily digest"), ("weekly", "Weekly summary"), ("quarterly", "Quarterly")];
const SELECTED: &str = "weekly";

pub(super) struct RadioGroupSurface {
    group: RadioGroup<RadioGroupItem>,
}

impl RadioGroupSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let group = radio_group::new("workshop-sdk-radio-group")
            .template(xgpui_luma::controls::radio_group::radio_group_buttons_template(
                default_radio_button_template(),
                RadioGroupLayout::Vertical,
            ))
            .items(ROWS.map(|(id, label)| RadioGroupItem::new(id).label(label)))
            .selected(SELECTED)
            .spawn(cx);
        Box::new(Self { group })
    }

    /// The SDK radio group, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn group(&self) -> RadioGroup<RadioGroupItem> {
        self.group.clone()
    }
}

/// The `radio_group` control in `theme`, for the rows' layout.
fn group_binding(theme: &Theme) -> StyleBinding {
    StyleBinding::new("radio_group", ThemeSource::Fixed(theme.clone()))
}

impl SurfaceControl for RadioGroupSurface {
    fn title(&self) -> &'static str {
        "Radio Group"
    }

    /// The group's own `radio_group.lss` lays the rows out (the gaps between them).
    fn composed(&self) -> &'static [&'static str] {
        &["radio_group"]
    }

    fn view(&self) -> AnyView {
        self.group.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        // Rows draw their dots settled (no fill animation in flight).
        let pin = pinned.map(|states| {
            let keep = states.has("selected");
            Pin::<RadioButtonData> {
                state: states.interaction(),
                data: Arc::new(move |live: &RadioButtonData| RadioButtonData::new(keep && live.selected)),
            }
        });
        let rows = PinnedTemplate::new(Arc::new(StyledRadioButtonTemplate::new(theme.clone())), pin);
        let template =
            radio_group_buttons_template_with(group_binding(&theme), Arc::new(rows), RadioGroupLayout::Vertical);
        let inner = self.group.read(cx).inner_group();
        inner.update(cx, |group, cx| group.set_template(template, cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.group.update(cx, |group, cx| group.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        default_radio_button_template().style_states()
    }

    /// `RadioGroupBuilder` has no way to make its rows invalid.
    fn unreachable_states(&self) -> &'static [&'static str] {
        &["invalid"]
    }

    /// Whether a row is selected.
    fn lasting_states(&self, cx: &App) -> Vec<(&'static str, bool)> {
        vec![("selected", self.group.read(cx).selected_id(cx).is_some())]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.group, move |side: &mut ControlSide, _, event: &RadioGroupEvent, cx| {
            let state = match event {
                RadioGroupEvent::Change { value } => Some(("selected", value.is_some())),
                RadioGroupEvent::FocusChanged { focused } => Some(("focused", *focused)),
                RadioGroupEvent::ItemHoverChanged { hovered, .. } => Some(("hover", *hovered)),
                RadioGroupEvent::ItemPressedChanged { pressed, .. } => Some(("pressed", *pressed)),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    /// The rows in `states`, the middle one selected when `:selected` is.
    fn cell(
        &self,
        theme: &Theme,
        id: SharedString,
        states: &StateSet,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let template = StyledRadioButtonTemplate::new(theme.clone());
        let gap = RadioGroupGeometry::from_binding(&group_binding(theme)).vertical_gap;
        let rows = ROWS.map(|(row, label)| {
            let model = ButtonRenderModel {
                id: format!("{id}-{row}").into(),
                data: RadioButtonData::new(states.has("selected") && row == SELECTED),
                content: Arc::new(move |_, _| div().child(label).into_any_element()),
                role: ButtonFamilyRole::Text,
                state: states.interaction(),
                radius_override: Cell::new(None),
                elevation: true,
                ..Default::default()
            };
            template.render(&model, window, cx)
        });
        div().flex().flex_col().gap(px(gap)).children(rows).into_any_element()
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
    fn the_radio_group_page_selects_and_pins_its_row_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "radio-group");
        let side = cx.update(|_, cx| page.read(cx).side());
        // Its own radio_group.lss (the rows' layout) is offered beside radio.lss.
        assert!(cx.debug_bounds("workshop-surface-stylesheets").is_some(), "a tab per composed stylesheet");
        let group = cx.update(|_, cx| side.read(cx).control_as::<RadioGroupSurface>().expect("a radio group").group());
        assert!(cx.update(|_, cx| side.read(cx).state_on("selected")), "a row starts selected");

        cx.update(|window, cx| group.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("end space");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(group.read(cx).selected_id(cx).as_deref(), Some("quarterly")));
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("Change"), "selection is logged: {text}");

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("pressed", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("pressed") && pinned.has("selected"), "{pinned:?}");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw the rows");
    }
}
