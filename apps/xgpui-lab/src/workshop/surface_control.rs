//! The control a Workshop design surface page shows (issue #376): everything the page needs that
//! differs from one control to the next, behind one trait. The page itself (stage, State matrix,
//! states and events, `.lss` inspector) is the same for every control.

#[cfg(all(test, feature = "test-support"))]
use std::any::Any;
use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::theme::InteractionState;

use super::surface_page::ControlSide;
use crate::theme::Look;

/// A set of `.lss` states by name, e.g. `{checked, hover}`: what the page picks in Design mode and
/// draws in each State matrix cell. Names are the stylesheet config's.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct StateSet(Vec<String>);

impl StateSet {
    pub(super) fn new<S: Into<String>>(names: impl IntoIterator<Item = S>) -> Self {
        let mut names: Vec<String> = names.into_iter().map(Into::into).collect();
        names.sort();
        names.dedup();
        Self(names)
    }

    pub(super) fn has(&self, name: &str) -> bool {
        self.0.iter().any(|n| n == name)
    }

    /// The SDK interaction states the set names (the config's `hover`, `pressed`, `focused`, `disabled`
    /// and `invalid`), for a template's render model.
    pub(super) fn interaction(&self) -> InteractionState {
        InteractionState {
            hovered: self.has("hover"),
            pressed: self.has("pressed"),
            focused: self.has("focused"),
            disabled: self.has("disabled"),
            invalid: self.has("invalid"),
        }
    }
}

/// One of the live control's events, as the page logs and follows it.
pub(super) struct SurfaceEvent {
    /// The line the events panel shows.
    pub line: String,
    /// The `.lss` state the event turns on or off, if any (e.g. `("checked", true)`).
    pub state: Option<(&'static str, bool)>,
}

/// Called with each of the live control's events (see [`SurfaceControl::subscribe`]).
pub(super) type OnSurfaceEvent = fn(&mut ControlSide, SurfaceEvent, &mut Context<ControlSide>);

/// A control the design surface page can show.
pub(super) trait SurfaceControl {
    /// The page title, e.g. `Checkbox`.
    fn title(&self) -> &'static str;

    /// The live control, for the Control stage.
    fn view(&self) -> AnyView;

    /// Draw the live control from `theme` (already scaled by the stage zoom) and, in Design mode, with
    /// `pinned` in place of its live states.
    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App);

    /// Let the live control take input (Test mode) or not (Design mode).
    fn set_enabled(&self, enabled: bool, cx: &mut App);

    /// The `.lss` states the control's template sets in its key, by name, including any no rule uses
    /// (e.g. `focused`, for the focus ring).
    fn template_states(&self) -> Vec<&'static str>;

    /// The control's own word for an `.lss` state, where it differs from the state's name: a switch is
    /// `on`, not `checked`. The page's design controls (state switches, State matrix, live states) show
    /// it; the `.lss` inspector keeps the file's names.
    fn state_label(&self, _state: &str) -> Option<&'static str> {
        None
    }

    /// States the template reads that the live control can never reach: shown, never settable.
    fn unreachable_states(&self) -> &'static [&'static str] {
        &[]
    }

    /// The live control's lasting states as it is now (e.g. a checkbox's `checked`), which the key
    /// follows in Test mode. Transient states (hover, press, focus) are not lasting. Controls without
    /// one have none.
    fn lasting_states(&self, _cx: &App) -> Vec<(&'static str, bool)> {
        Vec::new()
    }

    /// Follow the live control's events.
    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription;

    /// The control's template from `theme`, drawn bare for `states`: one State matrix cell.
    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, window: &mut Window, cx: &mut App)
    -> AnyElement;

    /// Other `.lss` controls the live control composes (e.g. a radio group's `radio_group` layout beside
    /// its `radio` rows), each shown in its own inspector beside the page's. Most controls compose none.
    fn composed(&self) -> &'static [&'static str] {
        &[]
    }

    /// The width the stage gives the live control at actual size, for a control with no width of its own
    /// (a slider, a progress bar); it scales with the zoom. `None`: the control sizes itself.
    fn stage_width(&self) -> Option<f32> {
        None
    }

    /// The height the stage gives the live control at actual size, like [`Self::stage_width`]. The stage
    /// shows a resize handle on each axis that has a size, so the control can be dragged to another.
    fn stage_height(&self) -> Option<f32> {
        None
    }

    /// The live control's own status text (e.g. an autocomplete's search status), shown in Test mode's
    /// lower panel rather than on the stage. Most controls have none.
    fn status(&self) -> Option<SharedString> {
        None
    }

    /// Settings of the live control that aren't `.lss` states (e.g. a toolbar's focus strategy), in the
    /// page's look, shown on the page's lower panel. Most controls have none.
    fn options(&self) -> Option<AnyElement> {
        None
    }

    /// The control as `Any`, so tests can reach its concrete type.
    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn Any;
}

/// Makes a page's control, with the page's look (for any [`SurfaceControl::options`]) and the context of
/// the control side that owns it.
pub(super) type MakeSurfaceControl = fn(&Arc<Look>, &mut Context<ControlSide>) -> Box<dyn SurfaceControl>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_state_set_is_unordered_and_maps_interaction_states() {
        assert_eq!(StateSet::new(["hover", "checked"]), StateSet::new(["checked", "hover", "hover"]));
        let states = StateSet::new(["checked", "focused", "pressed"]);
        assert!(states.has("checked") && !states.has("hover"));
        assert_eq!(
            states.interaction(),
            InteractionState { focused: true, pressed: true, ..Default::default() },
            "checked is data, not an interaction state"
        );
    }
}
