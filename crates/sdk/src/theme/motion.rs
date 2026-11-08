//! Look defaults use the SDK controls' own typed roles. This is configuration,
//! not a runtime: controls retain targets, accessibility, presence, and scheduling.

use crate::controls::{switch::SwitchMotion, tabs::TabsMotion};
use crate::motion::Transition;
use super::stylesheet::CommonStylesheet;

/// Vanilla SDK defaults, also the public starting point for looks. Each control
/// works without a look; adapters supply customized component profiles through
/// the SDK builder's `motion_defaults` seam.
#[derive(Clone, Debug)]
pub struct MotionDefaults {
    /// Default for standalone transition helpers, separate from control roles.
    pub transition: Transition,
    pub tabs_overlay: crate::motion::PresenceMotion,
    pub slide_panel: crate::controls::slide_panel::SlidePanelMotion,
    pub listbox: crate::controls::listbox::ListBoxMotion,
    pub scroll_container: crate::controls::scroll_container::ScrollContainerMotion,
    pub popover: crate::controls::popover_button::PopoverMotion,

    pub checkbox: crate::controls::checkbox::CheckboxMotion,
    pub radio: crate::controls::radio_button::RadioMotion,
    pub toggle: crate::controls::toggle::ToggleMotion,
    pub control_group: crate::controls::control_group::ControlGroupMotion,
    pub slider: crate::controls::slider::SliderMotion,
    pub progress: crate::controls::progress::ProgressMotion,
    pub stepper: crate::controls::stepper::StepperMotion,
    pub accordion: crate::controls::accordion::AccordionMotion,
    pub tree_view: crate::controls::tree_view::TreeViewMotion,
    pub sidebar: crate::controls::sidebar::SidebarMotion,
    pub split_view: crate::controls::split_view::SplitViewMotion,
    pub resizable_panels: crate::controls::resizable_panels::ResizablePanelsMotion,
    pub popup_menu: crate::controls::popup_menu::PopupMenuMotion,
    pub context_menu: crate::controls::context_menu::ContextMenuMotion,
    pub selector: crate::controls::selector::SelectorMotion,
    pub search_selector: crate::controls::search_selector::SearchSelectorMotion,
    pub autocomplete: crate::controls::autocomplete::AutocompleteMotion,
    pub combobox: crate::controls::combobox::ComboBoxMotion,
    pub table: crate::controls::table::TableMotion,

    pub switch: SwitchMotion,
    pub tabs: TabsMotion,
}

impl Default for MotionDefaults {
    fn default() -> Self {
        Self::vanilla().clone()
    }
}

impl MotionDefaults {
    /// Cached motion profiles constructed from the SDK-owned Rust defaults.
    /// Control profile defaults and looks share this baseline.
    pub fn vanilla() -> &'static Self {
        static DEFAULT: std::sync::OnceLock<MotionDefaults> = std::sync::OnceLock::new();
        DEFAULT.get_or_init(|| CommonStylesheet::sdk_default().motion.baseline_value())
    }

    /// Resolve the SDK stylesheet baseline, then typed role overrides.
    pub fn from_stylesheet(stylesheet: &CommonStylesheet) -> Self {
        let mut defaults = Self::default();
        stylesheet.motion.apply_to(&mut defaults);
        defaults
    }
}

impl CommonStylesheet {
    /// Resolve motion without depending on a look crate.
    ///
    /// ```
    /// use xgpui_luma::{controls::switch, theme::stylesheet::CommonStylesheet};
    /// let sheet = CommonStylesheet::parse(r#"
    /// [common.motion.switch.state]
    /// kind = "spring"
    /// stiffness = 300.0
    /// damping = 28.0
    /// "#)?;
    /// let builder = switch::new("notifications")
    ///     .motion_defaults(sheet.motion_defaults().switch);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn motion_defaults(&self) -> MotionDefaults {
        MotionDefaults::from_stylesheet(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::MotionState;
    use std::time::Duration;

    #[test]
    fn migrated_defaults_keep_existing_timing_and_stylesheet_precedence() {
        let stylesheet = CommonStylesheet::parse(
            "[common.motion.tabs.content]\nkind = 'timed'\nduration_ms = 125\neasing = 'out_cubic'",
        )
        .unwrap();
        let m = MotionDefaults::from_stylesheet(&stylesheet);
        let check = |transition: &Transition, ms, role| {
            let duration = Duration::from_millis(ms);
            let run = transition.start(MotionState::default(), 1.0);
            if ms > 0 {
                assert!(!run.sample(duration - Duration::from_nanos(1)).complete, "{role}");
            }
            assert!(run.sample(duration).complete, "{role}");
        };
        macro_rules! timed {
            ($ms:expr; $($role:expr),+ $(,)?) => { $(check(&$role, $ms, stringify!($role));)+ };
        }
        timed!(200;
            m.checkbox.state, m.radio.state, m.toggle.state, m.switch.state,
            m.control_group.selection, m.slider.thumb, m.progress.value, m.stepper.selection,
            m.accordion.expansion, m.tree_view.expansion, m.tree_view.position, m.sidebar.expansion,
            m.sidebar.popup.enter, m.sidebar.popup.exit, m.split_view.extent, m.resizable_panels.extent,
            m.popup_menu.disclosure, m.popup_menu.submenu, m.popup_menu.presence.enter, m.popup_menu.presence.exit,
            m.context_menu.disclosure, m.context_menu.presence.enter,
            m.context_menu.submenu.enter, m.selector.presence.enter, m.search_selector.disclosure,
            m.search_selector.presence.enter, m.autocomplete.presence.enter, m.combobox.presence.enter,
            m.popover.presence.enter, m.popover.presence.exit, m.tabs_overlay.enter, m.tabs_overlay.exit,
            m.tabs.indicator, m.tabs.disclosure, m.scroll_container.scroll, m.listbox.scroll,
        );
        timed!(0; m.context_menu.presence.exit, m.context_menu.submenu.exit, m.selector.presence.exit, m.search_selector.presence.exit,
            m.autocomplete.presence.exit, m.combobox.presence.exit);
        timed!(180; m.popup_menu.highlight);
        timed!(220; m.slide_panel.presence.enter, m.slide_panel.presence.exit);
        timed!(125; m.tabs.content);
        timed!(200; m.tabs.drop_layout, m.accordion.drop_layout, m.tree_view.drop_layout,
            m.sidebar.drop_layout, m.table.drop_layout, m.listbox.drop_layout);
        assert_eq!(m.progress.period, Duration::from_millis(1200));
    }
}
