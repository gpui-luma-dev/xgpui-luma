// Shared by both look crates; all construction uses the headless GPUI context.
#[doc(hidden)]
#[macro_export]
// Avoid rustfmt width warnings when formatting nested macro bodies.
#[rustfmt::skip]
macro_rules! test_look_motion_contract {
    ($make_look:expr, $api:ident) => {
        #[test]
        fn motion_profiles_inherit_override_and_snapshot_across_control_families() {
            use gpui::AppContext;
            use xgpui_luma::motion::{Transition, Easing, MotionState};
            use std::time::Duration;
            let look = $make_look;
            look.motion(|m| {
                let transition = Transition::timed(Duration::from_secs(1), Easing::Linear);
                m.checkbox.state = transition.clone();
                m.radio.state = transition.clone();
                m.toggle.state = transition.clone();
                m.slider.thumb = transition.clone();
                m.progress.value = transition.clone();
                m.tree_view.expansion = transition.clone();
                m.popup_menu.highlight = transition.clone();
                m.popup_menu.presence.exit = transition.clone();
                m.context_menu.presence.enter = transition;
            });
            let app = gpui::TestAppContext::single();
            let values = app.update(|cx| {
                cx.new(|cx| {
                    let sample = |transition: &Transition| {
                        transition.start(MotionState::default(), 1.0).sample(Duration::from_millis(100)).state.value
                    };
                    let mut values = Vec::<f32>::new();
                    macro_rules! check {
                        ($builder:expr, $role:ident) => {{
                            let entity = $builder.look(&look).spawn(cx);
                            values.push(sample(&entity.read(cx).motion_profile().$role));
                        }};
                    }
                    check!($api::Checkbox::new("checkbox"), state);
                    check!($api::Radio::new("radio"), state);
                    check!($api::Toggle::new("toggle"), state);
                    check!($api::Slider::new("slider"), thumb);
                    check!($api::Progress::new("progress"), value);
                    check!($api::TreeView::<()>::new("tree"), expansion);
                    check!($api::PopupMenu::new("menu"), highlight);
                    let context = $api::ContextMenu::new("context").look(&look).spawn(cx);
                    values.push(sample(&context.read(cx).motion_profile().presence.enter));
                    let edited =
                        $api::Checkbox::new("edited").motion(|m| m.state = Transition::instant()).look(&look).spawn(cx);
                    assert_eq!(sample(&edited.read(cx).motion_profile().state), 1.0);
                    let menu = $api::PopupMenu::new("edited-menu")
                        .look(&look)
                        .motion(|m| m.presence.exit = Transition::instant())
                        .spawn(cx);
                    assert_eq!(sample(&menu.read(cx).motion_profile().presence.exit), 1.0);
                    assert_eq!(sample(&menu.read(cx).motion_profile().highlight), 0.1);
                    // Later look edits cannot rewrite an already-created entity.
                    look.motion(|m| m.context_menu.presence.enter = Transition::instant());
                    values.push(sample(&context.read(cx).motion_profile().presence.enter));
                    let new_context = $api::ContextMenu::new("new-context").look(&look).spawn(cx);
                    assert_eq!(sample(&new_context.read(cx).motion_profile().presence.enter), 1.0);
                    values
                })
            });
            app.read(|cx| assert!(values.read(cx).iter().all(|value| (*value - 0.1).abs() < 0.0001)));
        }
    };
}

/// Verify the same stylesheet/override contract through each look's public API.
#[doc(hidden)]
#[macro_export]
// Avoid rustfmt width warnings when formatting nested macro bodies.
#[rustfmt::skip]
macro_rules! test_stylesheet_motion_contract {
    ($make_look:expr, $api:ident) => {
        #[test]
        fn stylesheet_motion_reaches_controls_before_look_and_instance_overrides() {
            use gpui::AppContext;
            use xgpui_luma::motion::{Transition, Easing, MotionState};
            use std::time::Duration;
            let source = r#"
[common.motion.tabs.indicator]
kind = "spring"
stiffness = 300.0
damping = 28.0
[common.motion.tabs.content]
kind = "timed"
duration_ms = 800
easing = "linear"
"#;
            let look = ($make_look)(source);
            let vanilla = xgpui_luma::theme::MotionDefaults::vanilla();
            let inherited = look.motion_defaults();
            assert_eq!(inherited.switch.state.description(), vanilla.switch.state.description());
            assert_eq!(
                inherited.slide_panel.presence.enter.description(),
                vanilla.slide_panel.presence.enter.description()
            );
            assert_eq!(
                inherited.context_menu.presence.exit.description(),
                vanilla.context_menu.presence.exit.description()
            );
            let sample = |transition: &Transition| {
                transition.start(MotionState::default(), 1.0).sample(Duration::from_millis(200))
            };
            assert_eq!(sample(&look.motion_defaults().tabs.content).state.value, 0.25);
            look.motion(|m| m.tabs.content = Transition::timed(Duration::from_millis(400), Easing::Linear));
            let app = gpui::TestAppContext::single();
            app.update(|cx| {
                cx.new(|cx| {
                    let inherited = $api::Tabs::new("inherited").look(&look).spawn(cx);
                    let edited = $api::Tabs::new("edited")
                        .motion(|m| m.indicator = Transition::instant())
                        .look(&look)
                        .motion(move |m| {
                            m.content = xgpui_luma::motion::Transition::timed(
                                Duration::from_secs(1),
                                xgpui_luma::motion::Easing::OutCubic,
                            )
                        })
                        .motion(|m| m.content = Transition::instant())
                        .spawn(cx);
                    let profile = inherited.read(cx).motion_profile();
                    assert_eq!(sample(&profile.content).state.value, 0.5);
                    assert_eq!(sample(&profile.indicator), sample(&Transition::spring(300.0, 28.0)));
                    assert!(sample(&edited.read(cx).motion_profile().indicator).complete);
                    assert!(sample(&edited.read(cx).motion_profile().content).complete);
                    look.motion(|m| m.tabs.content = Transition::instant());
                    assert_eq!(sample(&inherited.read(cx).motion_profile().content).state.value, 0.5);
                })
            });
        }
    };
}
