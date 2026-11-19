use super::*;
use gpui::TestAppContext;
use std::{cell::Cell, rc::Rc};

struct Owner {
    control: Switch,
    log: Vec<String>,
    _observer: Option<Subscription>,
}

#[test]
fn spring_state_interrupts_completes_and_obeys_animation_disable() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| Owner {
            control: new("spring").transition(Transition::spring(200.0, 18.0)).spawn(cx),
            log: Vec::new(),
            _observer: None,
        })
    });
    let control = app.read(|cx| owner.read(cx).control.clone());
    app.update(|cx| {
        control.update(cx, |control, cx| {
            control.transition.elapse_for_test(std::time::Duration::ZERO);
            control.set_data(true, cx);
            control.transition.elapse_for_test(std::time::Duration::from_millis(100));
            control.transition.sync();
            let interrupted = control.transition.state();
            assert!(interrupted.velocity > 0.0);
            control.set_data(false, cx);
            assert!((control.transition.state().value - interrupted.value).abs() < 0.0001);
            assert!((control.transition.state().velocity - interrupted.velocity).abs() < 0.0001);
            control.transition.elapse_for_test(std::time::Duration::from_secs(10));
            assert!(!control.transition.sync());
            assert_eq!(control.transition.progress(), 0.0);
            control.set_data(true, cx);
            control.set_motion_enabled(false, cx);
            assert_eq!(control.transition.progress(), 1.0);
            assert!(!control.transition.is_animating());
            control.set_data(false, cx);
            assert_eq!(control.transition.progress(), 0.0);
            control.set_motion_enabled(true, cx);
            control.set_data(true, cx);
            assert!(control.transition.is_animating());
        })
    });
}

fn activate(control: &Switch, cx: &mut App) {
    let button = control.read(cx).button.clone();
    button.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
}

#[test]
fn callbacks_are_additive_and_coexist_with_subscribers() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let control = new("choice")
                .on_event(cx, |owner: &mut Owner, event, _| {
                    if matches!(event, SwitchEvent::Change { .. }) {
                        owner.log.push("all".into());
                    }
                })
                .on_value_changed(cx, |owner, value, _| owner.log.push(format!("first:{value}")))
                .on_value_changed(cx, |owner, value, _| owner.log.push(format!("second:{value}")))
                .with_data(false)
                .spawn(cx);
            let observer = cx.subscribe(&control, |owner: &mut Owner, _, event: &SwitchEvent, _| {
                if matches!(event, SwitchEvent::Change { .. }) {
                    owner.log.push("observer".into());
                }
            });
            Owner { control, log: vec![], _observer: Some(observer) }
        })
    });
    let control = app.read(|cx| owner.read(cx).control.clone());
    app.update(|cx| activate(&control, cx));
    app.read(|cx| {
        assert!(*control.read(cx).data());
        let log = &owner.read(cx).log;
        assert_eq!(
            log.iter().filter(|s| s.as_str() != "observer").map(String::as_str).collect::<Vec<_>>(),
            ["all", "first:true", "second:true"]
        );
        assert_eq!(log.iter().filter(|s| s.as_str() == "observer").count(), 1);
    });
}

#[test]
fn callbacks_relay_state_and_can_update_emitter() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let control = new("choice")
                .on_value_changed(cx, |owner: &mut Owner, value, cx| {
                    owner.log.push(format!("change:{value}"));
                    owner.control.update(cx, |control, cx| control.set_enabled(false, cx));
                })
                .on_focus_changed(cx, |owner, value, _| owner.log.push(format!("focus:{value}")))
                .on_hover_changed(cx, |owner, value, _| owner.log.push(format!("hover:{value}")))
                .on_enabled_changed(cx, |owner, value, _| owner.log.push(format!("enabled:{value}")))
                .spawn(cx);
            Owner { control, log: vec![], _observer: None }
        })
    });
    let control = app.read(|cx| owner.read(cx).control.clone());
    app.update(|cx| {
        let button = control.read(cx).button.clone();
        button.update(cx, |_, cx| {
            cx.emit(ButtonEvent::FocusChanged { focused: true });
            cx.emit(ButtonEvent::HoverChanged { hovered: true });
        });
    });
    app.update(|cx| activate(&control, cx));
    app.update(|cx| control.update(cx, |control, cx| control.set_enabled(false, cx)));
    app.read(|cx| assert_eq!(owner.read(cx).log, ["focus:true", "hover:true", "change:true", "enabled:false"]));
}

#[test]
fn factory_callbacks_preserve_builder_handlers_and_set_data_is_silent() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let control = new("choice")
                .on_value_changed(cx, |owner: &mut Owner, _, _| owner.log.push("builder".into()))
                .spawn(cx)
                .on_value_changed(cx, |owner, _, _| owner.log.push("factory".into()));
            Owner { control, log: vec![], _observer: None }
        })
    });
    let control = app.read(|cx| owner.read(cx).control.clone());
    app.update(|cx| control.update(cx, |control, cx| control.set_data(true, cx)));
    app.read(|cx| assert!(owner.read(cx).log.is_empty()));
    app.update(|cx| activate(&control, cx));
    app.read(|cx| {
        assert!(!*control.read(cx).data());
        assert_eq!(owner.read(cx).log.len(), 2);
        assert!(owner.read(cx).log.iter().any(|s| s == "builder"));
        assert!(owner.read(cx).log.iter().any(|s| s == "factory"));
    });
}

#[test]
fn owner_is_weak_and_captures_release_with_control_or_builder() {
    let app = TestAppContext::single();
    let called = Rc::new(Cell::new(0));
    let capture = Rc::new(());
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let called = called.clone();
            let capture = capture.clone();
            let control = new("choice")
                .on_value_changed(cx, move |_: &mut Owner, _, _| {
                    let _keep = &capture;
                    called.set(called.get() + 1);
                })
                .spawn(cx);
            Owner { control, log: vec![], _observer: None }
        })
    });
    let control = app.read(|cx| owner.read(cx).control.clone());
    let weak_owner = owner.downgrade();
    drop(owner);
    app.update(|_| {});
    assert!(weak_owner.upgrade().is_none());
    app.update(|cx| activate(&control, cx));
    assert_eq!(called.get(), 0);
    assert_eq!(Rc::strong_count(&capture), 2);
    drop(control);
    app.update(|_| {});
    assert_eq!(Rc::strong_count(&capture), 1);
    app.update(|cx| {
        let owner = cx.new(|_| ());
        owner.update(cx, |_, cx| {
            let capture = capture.clone();
            let builder = new("abandoned").on_value_changed(cx, move |_, _, _| {
                let _keep = &capture;
            });
            drop(builder);
        });
    });
    assert_eq!(Rc::strong_count(&capture), 1);
}

#[test]
fn vanilla_sdk_defaults_and_stylesheet_work_without_a_look() {
    use crate::motion::MotionState;
    use std::time::Duration;
    // Timing comes from `.lss` (`transition`, 200 ms out-cubic by default), or this instance's override.
    let app = TestAppContext::single();
    app.update(|cx| {
        let vanilla = cx.new(|cx| new("vanilla").spawn(cx));
        let edited = cx.new(|cx| {
            new("edited")
                .transition(Transition::timed(Duration::from_millis(800), crate::motion::Easing::Linear))
                .spawn(cx)
        });
        let sample = |entity: &Entity<Switch>, elapsed| {
            entity.read(cx).read(cx).current_transition().start(MotionState::default(), 1.0).sample(elapsed)
        };
        assert!(!sample(&vanilla, Duration::from_millis(199)).complete);
        assert!(sample(&vanilla, Duration::from_millis(200)).complete);
        assert_eq!(sample(&edited, Duration::from_millis(200)).state.value, 0.25);
    });
}
