//! Serializable motion roles shared by vanilla SDK applications and looks.

use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::motion::{Easing, Transition};
use crate::theme::MotionDefaults;

/// A complete transition definition. Omission inherits; an authored definition
/// replaces the whole role. Custom runtime definitions remain available in Rust.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransitionSpec {
    Timed {
        duration_ms: u32,
        easing: Easing,
    },
    Spring {
        #[serde(deserialize_with = "positive_finite")]
        stiffness: f32,
        #[serde(deserialize_with = "positive_finite")]
        damping: f32,
    },
    Instant {},
}

fn positive_finite<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let value = f32::deserialize(deserializer)?;
    if !value.is_finite() || value <= 0.0 {
        return Err(serde::de::Error::custom("spring parameters must be positive and finite"));
    }
    Ok(value)
}

impl TransitionSpec {
    /// Compile stylesheet data into the same runtime value used by `.motion`.
    pub fn transition(&self) -> Transition {
        match *self {
            Self::Timed { duration_ms, easing } => {
                Transition::timed(Duration::from_millis(u64::from(duration_ms)), easing)
            }
            Self::Spring { stiffness, damping } => Transition::spring(stiffness, damping),
            Self::Instant {} => Transition::instant(),
        }
    }
}

// A single declaration supplies public typed fields and their overlay behavior.
// No string-keyed runtime dispatch or animation engine lives in a look.
macro_rules! profile {
    ($name:ident => $target:path { $($role:ident: $spec:ty),* $(,)? }) => {
        #[doc = "Optional stylesheet roles; omitted values retain the supplied defaults."]
        #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct $name { $(pub $role: $spec),* }
        impl $name {
            pub(crate) fn baseline_value(&self) -> $target {
                $target { $($role: self.$role.baseline_value()),* }
            }
            #[cfg(test)]
            pub(crate) fn is_complete(&self) -> bool {
                true $(&& self.$role.is_complete())*
            }
            /// Overlay authored roles onto an existing SDK profile.
            pub fn apply_to(&self, target: &mut $target) {
                $(self.$role.apply_to(&mut target.$role);)*
            }
        }
    };
}

impl TransitionSpec {
    fn apply_to(&self, target: &mut Transition) {
        *target = self.transition();
    }
}
trait ApplyTo<T> {
    fn apply_to(&self, target: &mut T);
    fn baseline_value(&self) -> T;
    #[cfg(test)]
    fn is_complete(&self) -> bool;
}
impl ApplyTo<Transition> for Option<TransitionSpec> {
    fn baseline_value(&self) -> Transition {
        self.as_ref().map(TransitionSpec::transition).unwrap_or_else(Transition::instant)
    }
    #[cfg(test)]
    fn is_complete(&self) -> bool {
        self.is_some()
    }
    fn apply_to(&self, target: &mut Transition) {
        if let Some(spec) = self {
            spec.apply_to(target);
        }
    }
}
impl ApplyTo<Duration> for Option<u32> {
    fn baseline_value(&self) -> Duration {
        self.map(|ms| Duration::from_millis(u64::from(ms))).unwrap_or_default()
    }
    #[cfg(test)]
    fn is_complete(&self) -> bool {
        self.is_some()
    }
    fn apply_to(&self, target: &mut Duration) {
        if let Some(ms) = self {
            *target = Duration::from_millis(u64::from(*ms));
        }
    }
}

profile!(PresenceMotionStylesheet => crate::motion::PresenceMotion {
    enter: Option<TransitionSpec>, exit: Option<TransitionSpec>,
});

profile!(SlidePanelMotionStylesheet => crate::controls::slide_panel::SlidePanelMotion {
    presence: PresenceMotionStylesheet,
});

profile!(ListBoxMotionStylesheet => crate::controls::listbox::ListBoxMotion {
    scroll: Option<TransitionSpec>,
    drop_layout: Option<TransitionSpec>,
});

profile!(ScrollContainerMotionStylesheet => crate::controls::scroll_container::ScrollContainerMotion {
    scroll: Option<TransitionSpec>,
});

profile!(PopoverMotionStylesheet => crate::controls::popover_button::PopoverMotion {
    presence: PresenceMotionStylesheet,
});

profile!(CheckboxMotionStylesheet => crate::controls::checkbox::CheckboxMotion {
    state: Option<TransitionSpec>,
});

profile!(RadioMotionStylesheet => crate::controls::radio_button::RadioMotion {
    state: Option<TransitionSpec>,
});

profile!(ToggleMotionStylesheet => crate::controls::toggle::ToggleMotion {
    state: Option<TransitionSpec>,
});

profile!(ControlGroupMotionStylesheet => crate::controls::control_group::ControlGroupMotion {
    selection: Option<TransitionSpec>,
});

profile!(SliderMotionStylesheet => crate::controls::slider::SliderMotion {
    thumb: Option<TransitionSpec>,
});

/// Progress has a settling value role and an independent looping period.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProgressMotionStylesheet {
    pub value: Option<TransitionSpec>,
    pub period_ms: Option<u32>,
}
impl ProgressMotionStylesheet {
    pub(crate) fn baseline_value(&self) -> crate::controls::progress::ProgressMotion {
        crate::controls::progress::ProgressMotion {
            value: self.value.baseline_value(),
            period: self.period_ms.baseline_value(),
        }
    }
    #[cfg(test)]
    pub(crate) fn is_complete(&self) -> bool {
        self.value.is_complete() && self.period_ms.is_complete()
    }
    pub fn apply_to(&self, target: &mut crate::controls::progress::ProgressMotion) {
        self.value.apply_to(&mut target.value);
        self.period_ms.apply_to(&mut target.period);
    }
}

profile!(StepperMotionStylesheet => crate::controls::stepper::StepperMotion {
    selection: Option<TransitionSpec>,
});

profile!(AccordionMotionStylesheet => crate::controls::accordion::AccordionMotion {
    expansion: Option<TransitionSpec>,
    drop_layout: Option<TransitionSpec>,
});

profile!(TreeViewMotionStylesheet => crate::controls::tree_view::TreeViewMotion {
    expansion: Option<TransitionSpec>,
    position: Option<TransitionSpec>,
    drop_layout: Option<TransitionSpec>,
});

profile!(SidebarMotionStylesheet => crate::controls::sidebar::SidebarMotion {
    expansion: Option<TransitionSpec>,
    popup: PresenceMotionStylesheet,
    drop_layout: Option<TransitionSpec>,
});

profile!(SplitViewMotionStylesheet => crate::controls::split_view::SplitViewMotion {
    extent: Option<TransitionSpec>,
});

profile!(ResizablePanelsMotionStylesheet => crate::controls::resizable_panels::ResizablePanelsMotion {
    extent: Option<TransitionSpec>,
});

profile!(PopupMenuMotionStylesheet => crate::controls::popup_menu::PopupMenuMotion {
    presence: PresenceMotionStylesheet,
    highlight: Option<TransitionSpec>,
    disclosure: Option<TransitionSpec>,
    submenu: Option<TransitionSpec>,
});

profile!(ContextMenuMotionStylesheet => crate::controls::context_menu::ContextMenuMotion {
    presence: PresenceMotionStylesheet,
    submenu: PresenceMotionStylesheet,
    disclosure: Option<TransitionSpec>,
});

profile!(SelectorMotionStylesheet => crate::controls::selector::SelectorMotion {
    presence: PresenceMotionStylesheet,
});

profile!(SearchSelectorMotionStylesheet => crate::controls::search_selector::SearchSelectorMotion {
    presence: PresenceMotionStylesheet,
    disclosure: Option<TransitionSpec>,
});

profile!(AutocompleteMotionStylesheet => crate::controls::autocomplete::AutocompleteMotion {
    presence: PresenceMotionStylesheet,
});

profile!(ComboBoxMotionStylesheet => crate::controls::combobox::ComboBoxMotion {
    presence: PresenceMotionStylesheet,
});

profile!(TableMotionStylesheet => crate::controls::table::TableMotion {
    drop_layout: Option<TransitionSpec>,
});

profile!(SwitchMotionStylesheet => crate::controls::switch::SwitchMotion {
    state: Option<TransitionSpec>,
});

profile!(TabsMotionStylesheet => crate::controls::tabs::TabsMotion {
    indicator: Option<TransitionSpec>,
    content: Option<TransitionSpec>,
    disclosure: Option<TransitionSpec>,
    drop_layout: Option<TransitionSpec>,
});

profile!(MotionStylesheet => MotionDefaults {
    transition: Option<TransitionSpec>,
    tabs_overlay: PresenceMotionStylesheet,
    slide_panel: SlidePanelMotionStylesheet,
    listbox: ListBoxMotionStylesheet,
    scroll_container: ScrollContainerMotionStylesheet,
    popover: PopoverMotionStylesheet,
    checkbox: CheckboxMotionStylesheet,
    radio: RadioMotionStylesheet,
    toggle: ToggleMotionStylesheet,
    control_group: ControlGroupMotionStylesheet,
    slider: SliderMotionStylesheet,
    progress: ProgressMotionStylesheet,
    stepper: StepperMotionStylesheet,
    accordion: AccordionMotionStylesheet,
    tree_view: TreeViewMotionStylesheet,
    sidebar: SidebarMotionStylesheet,
    split_view: SplitViewMotionStylesheet,
    resizable_panels: ResizablePanelsMotionStylesheet,
    popup_menu: PopupMenuMotionStylesheet,
    context_menu: ContextMenuMotionStylesheet,
    selector: SelectorMotionStylesheet,
    search_selector: SearchSelectorMotionStylesheet,
    autocomplete: AutocompleteMotionStylesheet,
    combobox: ComboBoxMotionStylesheet,
    table: TableMotionStylesheet,
    switch: SwitchMotionStylesheet,
    tabs: TabsMotionStylesheet,
});

impl MotionStylesheet {
    /// Native SDK motion definitions, preserving whole-role values.
    pub fn native_default() -> Self {
        let timed = TransitionSpec::Timed { duration_ms: 200, easing: Easing::OutCubic };
        let presence = PresenceMotionStylesheet { enter: Some(timed.clone()), exit: Some(timed.clone()) };
        let dismissal = PresenceMotionStylesheet { enter: Some(timed.clone()), exit: Some(TransitionSpec::Instant {}) };
        Self {
            transition: Some(timed.clone()),
            tabs_overlay: presence.clone(),
            slide_panel: SlidePanelMotionStylesheet {
                presence: PresenceMotionStylesheet {
                    enter: Some(TransitionSpec::Timed { duration_ms: 220, easing: Easing::OutCubic }),
                    exit: Some(TransitionSpec::Timed { duration_ms: 220, easing: Easing::OutCubic }),
                },
            },
            listbox: ListBoxMotionStylesheet { scroll: Some(timed.clone()), drop_layout: Some(timed.clone()) },
            scroll_container: ScrollContainerMotionStylesheet { scroll: Some(timed.clone()) },
            popover: PopoverMotionStylesheet { presence: presence.clone() },
            checkbox: CheckboxMotionStylesheet::native_default(),
            radio: RadioMotionStylesheet { state: Some(timed.clone()) },
            toggle: ToggleMotionStylesheet { state: Some(timed.clone()) },
            control_group: ControlGroupMotionStylesheet { selection: Some(timed.clone()) },
            slider: SliderMotionStylesheet::native_default(),
            progress: ProgressMotionStylesheet { value: Some(timed.clone()), period_ms: Some(1200) },
            stepper: StepperMotionStylesheet { selection: Some(timed.clone()) },
            accordion: AccordionMotionStylesheet { expansion: Some(timed.clone()), drop_layout: Some(timed.clone()) },
            tree_view: TreeViewMotionStylesheet {
                expansion: Some(timed.clone()),
                position: Some(timed.clone()),
                drop_layout: Some(timed.clone()),
            },
            sidebar: SidebarMotionStylesheet {
                expansion: Some(timed.clone()),
                popup: presence.clone(),
                drop_layout: Some(timed.clone()),
            },
            split_view: SplitViewMotionStylesheet { extent: Some(timed.clone()) },
            resizable_panels: ResizablePanelsMotionStylesheet { extent: Some(timed.clone()) },
            popup_menu: PopupMenuMotionStylesheet {
                presence,
                highlight: Some(TransitionSpec::Timed { duration_ms: 180, easing: Easing::OutCubic }),
                disclosure: Some(timed.clone()),
                submenu: Some(timed.clone()),
            },
            context_menu: ContextMenuMotionStylesheet {
                presence: dismissal.clone(),
                submenu: dismissal.clone(),
                disclosure: Some(timed.clone()),
            },
            selector: SelectorMotionStylesheet { presence: dismissal.clone() },
            search_selector: SearchSelectorMotionStylesheet {
                presence: dismissal.clone(),
                disclosure: Some(timed.clone()),
            },
            autocomplete: AutocompleteMotionStylesheet { presence: dismissal.clone() },
            combobox: ComboBoxMotionStylesheet { presence: dismissal },
            table: TableMotionStylesheet { drop_layout: Some(timed.clone()) },
            switch: SwitchMotionStylesheet::native_default(),
            tabs: TabsMotionStylesheet {
                indicator: Some(timed.clone()),
                content: Some(TransitionSpec::Instant {}),
                disclosure: Some(timed.clone()),
                drop_layout: Some(timed),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{motion::MotionState, theme::stylesheet::CommonStylesheet};

    #[test]
    fn embedded_sdk_stylesheet_defines_every_role_and_is_cached() {
        let sheet = CommonStylesheet::sdk_default().clone();
        assert!(sheet.motion.is_complete(), "SDK stylesheet must author every motion role");
        assert_eq!(&sheet, CommonStylesheet::sdk_default());
        assert!(std::ptr::eq(CommonStylesheet::sdk_default(), CommonStylesheet::sdk_default()));
        assert!(std::ptr::eq(MotionDefaults::vanilla(), MotionDefaults::vanilla()));
        let defaults = MotionDefaults::default();
        assert_eq!(defaults.transition.description(), Transition::default().description());
    }

    #[test]
    fn typed_roles_override_only_authored_defaults() {
        let sheet = CommonStylesheet::parse(
            r#"
[common.motion.tabs.indicator]
kind = "spring"
stiffness = 300.0
damping = 28.0
[common.motion.tabs.content]
kind = "instant"
[common.motion.switch.state]
kind = "timed"
duration_ms = 800
easing = "linear"
[common.motion.context_menu.presence.exit]
kind = "timed"
duration_ms = 400
easing = "out_cubic"
[common.motion.progress]
period_ms = 0
"#,
        )
        .unwrap();
        let defaults = sheet.motion_defaults();
        let run = |t: &Transition, ms| t.start(MotionState::default(), 1.0).sample(Duration::from_millis(ms));
        assert!(run(&defaults.tabs.content, 0).complete);
        assert_eq!(run(&defaults.switch.state, 200).state.value, 0.25);
        assert!(!run(&defaults.tabs.indicator, 200).complete);
        assert!(run(&defaults.tabs.indicator, 10_000).complete);
        assert!(!run(&defaults.tabs.drop_layout, 199).complete);
        assert!(run(&defaults.tabs.drop_layout, 200).complete);
        assert!(!run(&defaults.context_menu.presence.exit, 399).complete);
        assert!(run(&defaults.context_menu.presence.exit, 400).complete);
        assert!(run(&defaults.selector.presence.exit, 0).complete);
        assert_eq!(defaults.progress.period, Duration::ZERO);
    }

    #[test]
    fn in_out_cubic_stylesheet_round_trips_and_reaches_runtime() {
        let sheet = CommonStylesheet::parse(
            r#"
[common.motion.switch.state]
kind = "timed"
duration_ms = 1000
easing = "in_out_cubic"
"#,
        )
        .unwrap();
        let spec = sheet.motion.switch.state.as_ref().unwrap();
        let encoded = toml::to_string(spec).unwrap();
        assert!(encoded.contains("in_out_cubic"));
        assert_eq!(toml::from_str::<TransitionSpec>(&encoded).unwrap(), *spec);
        let transition = sheet.motion_defaults().switch.state;
        assert_eq!(
            transition.description(),
            crate::motion::TransitionDescription::Timed {
                duration: Duration::from_secs(1),
                easing: Easing::InOutCubic,
            }
        );
        let sample = transition.start(MotionState::default(), 1.0).sample(Duration::from_millis(250));
        assert_eq!(sample.state, MotionState { value: 0.0625, velocity: 0.75 });
        assert!(!sample.complete);
    }

    #[test]
    fn invalid_motion_is_rejected_at_parse_time() {
        for source in [
            "[common.motion.unknown.state]\nkind = 'instant'",
            "[common.motion.switch.unknown]\nkind = 'instant'",
            "[common.motion.switch.state]\nkind = 'unknown'",
            "[common.motion.switch.state]\nkind = 'timed'\nduration_ms = 20",
            "[common.motion.switch.state]\nkind = 'timed'\neasing = 'linear'",
            "[common.motion.switch.state]\nkind = 'timed'\nduration_ms = -1\neasing = 'linear'",
            "[common.motion.switch.state]\nkind = 'timed'\nduration_ms = 1.5\neasing = 'linear'",
            "[common.motion.switch.state]\nkind = 'timed'\nduration_ms = 4294967296\neasing = 'linear'",
            "[common.motion.switch.state]\nkind = 'timed'\nduration_ms = 20\neasing = 'bounce'",
            "[common.motion.switch.state]\nkind = 'timed'\nduration_ms = 20\neasing = 'linear'\ndamping = 20.0",
            "[common.motion.switch.state]\nkind = 'instant'\nduration_ms = 20",
            "[common.motion.switch.state]\nkind = 'spring'\nstiffness = 300.0",
            "[common.motion.switch.state]\nkind = 'spring'\nstiffness = 0.0\ndamping = 20.0",
            "[common.motion.switch.state]\nkind = 'spring'\nstiffness = 300.0\ndamping = -1.0",
            "[common.motion.switch.state]\nkind = 'spring'\nstiffness = nan\ndamping = 20.0",
            "[common.motion.switch.state]\nkind = 'spring'\nstiffness = 300.0\ndamping = inf",
            "[common.motion.progress]\nperiod_ms = -1",
            "[common.motion.progress]\nperiod_ms = 1.5",
        ] {
            assert!(CommonStylesheet::parse(source).is_err(), "accepted {source}");
        }
    }

    #[test]
    fn public_overlay_preserves_a_look_base_and_replaces_complete_roles() {
        let sheet = CommonStylesheet::parse(
            r#"
[common.motion.tabs.content]
kind = "timed"
duration_ms = 0
easing = "out_cubic"
"#,
        )
        .unwrap();
        let mut defaults = MotionDefaults::default();
        defaults.tabs.indicator = Transition::spring(300.0, 28.0);
        sheet.motion.apply_to(&mut defaults);
        assert!(defaults.tabs.content.start(MotionState::default(), 1.0).sample(Duration::ZERO).complete);
        let expected = Transition::spring(300.0, 28.0)
            .start(MotionState::default(), 1.0)
            .sample(Duration::from_millis(100));
        assert_eq!(
            defaults.tabs.indicator.start(MotionState::default(), 1.0).sample(Duration::from_millis(100)),
            expected
        );
    }
    #[test]
    fn every_stylesheet_role_reaches_its_sdk_profile() {
        macro_rules! check_roles {
            ($($family:ident { $($role:ident $(.$nested:ident)?),+ }),+ $(,)?) => {{
                let mut source = String::new();
                $($(source.push_str(&format!(
                    "[common.motion.{}.{}]\nkind = 'timed'\nduration_ms = 800\neasing = 'linear'\n",
                    stringify!($family), stringify!($role $(.$nested)?).replace(' ', ""),
                ));)+)+
                let defaults = CommonStylesheet::parse(&source).unwrap().motion_defaults();
                $($(assert_eq!(defaults.$family.$role $(.$nested)?
                    .start(MotionState::default(), 1.0)
                    .sample(Duration::from_millis(200)).state.value, 0.25,
                    "{}", stringify!($family.$role $(.$nested)?));)+)+
            }};
        }
        check_roles!(
            tabs_overlay { enter, exit }, slide_panel { presence.enter, presence.exit },
            listbox { scroll, drop_layout }, scroll_container { scroll },
            popover { presence.enter, presence.exit },
            checkbox { state }, radio { state }, toggle { state }, switch { state },
            control_group { selection }, slider { thumb }, progress { value }, stepper { selection },
            accordion { expansion, drop_layout }, tree_view { expansion, position, drop_layout },
            sidebar { expansion, popup.enter, popup.exit, drop_layout },
            split_view { extent }, resizable_panels { extent },
            popup_menu { presence.enter, presence.exit, highlight, disclosure, submenu },
            context_menu { presence.enter, presence.exit, submenu.enter, submenu.exit, disclosure },
            selector { presence.enter, presence.exit },
            search_selector { presence.enter, presence.exit, disclosure },
            autocomplete { presence.enter, presence.exit }, combobox { presence.enter, presence.exit },
            table { drop_layout }, tabs { indicator, content, disclosure, drop_layout },
        );
    }
}
