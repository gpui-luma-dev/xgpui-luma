//! Tooltip options exercised against one retained preview.
use xgpui_luma::controls::textfield::TextFieldEventExt;
use xgpui_luma::controls::slider::SliderEventExt;
use xgpui_luma::controls::switch::SwitchEventExt;
use std::{sync::Arc, time::Duration};
use gpui::{AnyElement, FontWeight, Context, Entity, IntoElement, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::{
    controls::{
        button::{Button},
        slider::{Slider, SliderEvent},
        switch::{Switch},
        textfield::{TextField, TextFieldEvent},
        tooltip::{Tooltip, TooltipPlacement, TooltipDismissal, bubble},
    },
    prelude::{HasPresenter, TooltipEntityExt},
};
use xgpui_luma_look_radix::{self as radix, Look, SemanticRole};
use crate::assets::{icon_named, react_icon};

const DEFAULT_TEXT: &str = "This is a theme-aware tooltip";
const DEFAULT_HINT: &str = "⌘S";

pub struct TooltipPlayground {
    look: Arc<Look>,
    delay_ms: u64,
    clicks: u64,
    once: bool,
    hide_delay_ms: u64,
    duration_ms: u64,
    below: bool,
    permanent: bool,
    enabled: bool,
    text: String,
    shortcut: String,
    custom: bool,
    text_input: TextField,
    shortcut_input: TextField,
    reset_text: Entity<Button>,
    reset_shortcut: Entity<Button>,
    custom_content: Switch,
    preview: Entity<Button>,
    delay: Slider,
    hide_delay: Slider,
    duration: Slider,
    placement: Switch,
    dismissal: Switch,
    enable: Switch,
    show_once: Switch,
    rearm: Entity<Button>,
    reset_all: Entity<Button>,

    #[cfg(all(test, feature = "test-support"))]
    render_probe: Option<xgpui_luma::controls::tooltip::TooltipTemplate>,
}

impl TooltipPlayground {
    pub fn new(look: &Arc<Look>, cx: &mut Context<Self>) -> Self {
        let preview = radix::Button::new("tooltip-preview")
            .look(look)
            .label("Test button · 0 clicks")
            .with_template_modifier(|root, _| root.flex_none().debug_selector(|| "tooltip-preview".into()))
            .spawn(cx)
            .tooltip(Tooltip::new(DEFAULT_TEXT).shortcut(DEFAULT_HINT), cx);
        let text_input = radix::TextField::new("tooltip-text")
            .look(look)
            .value(DEFAULT_TEXT)
            .with_template_modifier(|root, _| root.flex_none().debug_selector(|| "tooltip-text".into()))
            .spawn(cx);
        let shortcut_input = radix::TextField::new("tooltip-shortcut")
            .look(look)
            .value(DEFAULT_HINT)
            .with_template_modifier(|root, _| root.flex_none().debug_selector(|| "tooltip-shortcut".into()))
            .spawn(cx);
        let reset_text = reset_button("tooltip-reset-text", "Reset tooltip text", look, cx);
        let reset_shortcut = reset_button("tooltip-reset-shortcut", "Restore sample hint", look, cx);
        let custom_content = radix::Switch::new("tooltip-custom").look(look).label("Use custom content").spawn(cx);
        let delay = radix::Slider::new("tooltip-delay")
            .look(look)
            .range(0.0..2000.0)
            .step(50.0)
            .value(500.0)
            .with_template_modifier(|root, _| root.flex_none().debug_selector(|| "tooltip-delay".into()))
            .spawn(cx);
        let hide_delay = radix::Slider::new("tooltip-hide-delay")
            .look(look)
            .range(0.0..1000.0)
            .step(10.0)
            .value(180.0)
            .with_template_modifier(|root, _| root.flex_none().debug_selector(|| "tooltip-hide-delay".into()))
            .spawn(cx);
        let duration = radix::Slider::new("tooltip-duration")
            .look(look)
            .range(0.0..10000.0)
            .step(250.0)
            .value(0.0)
            .with_template_modifier(|root, _| root.flex_none().debug_selector(|| "tooltip-duration".into()))
            .spawn(cx);
        let placement = radix::Switch::new("tooltip-placement").look(look).label("Below target").spawn(cx);
        let dismissal = radix::Switch::new("tooltip-dismissal").look(look).label("Keep dismissed").spawn(cx);
        let enable = radix::Switch::new("tooltip-enable").look(look).label("Enabled").checked(true).spawn(cx);
        let show_once = radix::Switch::new("tooltip-once").look(look).label("Show once").spawn(cx);
        let rearm = radix::Button::new("tooltip-rearm")
            .look(look)
            .soft()
            .label("Restart test")
            .with_template_modifier(|root, _| root.flex_none().debug_selector(|| "tooltip-rearm".into()))
            .spawn(cx);
        let reset_all = radix::Button::new("tooltip-reset-all")
            .look(look)
            .soft()
            .label("Reset properties")
            .with_template_modifier(|root, _| root.flex_none().debug_selector(|| "tooltip-reset-all".into()))
            .spawn(cx);

        reset_all.clone().on_click(cx, |this, cx| {
            this.reset_properties(cx);
        });
        reset_text.clone().on_click(cx, |this, cx| {
            this.text = DEFAULT_TEXT.into();
            this.text_input.update(cx, |field, cx| field.set_value(DEFAULT_TEXT, cx));
            this.apply(cx);
        });
        reset_shortcut.clone().on_click(cx, |this, cx| {
            this.shortcut = DEFAULT_HINT.into();
            this.shortcut_input.update(cx, |field, cx| field.set_value(DEFAULT_HINT, cx));
            this.apply(cx);
        });
        text_input.clone().on_event(cx, |this, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                this.text = value.clone();
                this.apply(cx);
            }
        });
        shortcut_input.clone().on_event(cx, |this, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                this.shortcut = value.clone();
                this.apply(cx);
            }
        });
        custom_content.clone().on_value_changed(cx, |this, on, cx| {
            this.custom = on;
            this.apply(cx);
        });
        preview.clone().on_click(cx, |this, cx| {
            this.clicks = this.clicks.saturating_add(1);
            this.preview.update(cx, |button, cx| {
                button.set_label(
                    format!("Test button · {} {}", this.clicks, if this.clicks == 1 { "click" } else { "clicks" }),
                    cx,
                )
            });
            cx.notify();
        });
        delay.clone().on_event(cx, |this, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } = event {
                this.delay_ms = value.round() as u64;
                this.apply(cx);
            }
        });
        hide_delay.clone().on_event(cx, |this, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } = event {
                this.hide_delay_ms = value.round() as u64;
                this.apply(cx);
            }
        });
        duration.clone().on_event(cx, |this, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } = event {
                this.duration_ms = value.round() as u64;
                this.apply(cx);
            }
        });
        placement.clone().on_value_changed(cx, |this, on, cx| {
            this.below = on;
            this.apply(cx);
        });
        dismissal.clone().on_value_changed(cx, |this, on, cx| {
            this.permanent = on;
            this.apply(cx);
        });
        enable.clone().on_value_changed(cx, |this, on, cx| {
            this.enabled = on;
            this.apply(cx);
        });
        show_once.clone().on_value_changed(cx, |this, on, cx| {
            this.once = on;
            this.apply(cx);
        });
        rearm.clone().on_click(cx, |this, cx| {
            this.apply(cx);
        });
        Self {
            look: Arc::clone(look),
            delay_ms: 500,
            clicks: 0,
            once: false,
            hide_delay_ms: 180,
            duration_ms: 0,
            below: false,
            permanent: false,
            enabled: true,
            text: DEFAULT_TEXT.into(),
            shortcut: DEFAULT_HINT.into(),
            custom: false,
            text_input,
            shortcut_input,
            reset_text,
            reset_shortcut,
            custom_content,
            preview,
            delay,
            hide_delay,
            duration,
            placement,
            dismissal,
            enable,
            show_once,
            rearm,
            reset_all,

            #[cfg(all(test, feature = "test-support"))]
            render_probe: None,
        }
    }

    fn reset_properties(&mut self, cx: &mut Context<Self>) {
        self.text = DEFAULT_TEXT.into();
        self.shortcut = DEFAULT_HINT.into();
        self.delay_ms = 500;
        self.hide_delay_ms = 180;
        self.duration_ms = 0;
        self.below = false;
        self.permanent = false;
        self.enabled = true;
        self.once = false;
        self.custom = false;
        self.text_input.update(cx, |field, cx| field.set_value(DEFAULT_TEXT, cx));
        self.shortcut_input.update(cx, |field, cx| field.set_value(DEFAULT_HINT, cx));
        self.delay.update(cx, |slider, cx| slider.set_value(500.0, cx));
        self.hide_delay.update(cx, |slider, cx| slider.set_value(180.0, cx));
        self.duration.update(cx, |slider, cx| slider.set_value(0.0, cx));
        for (toggle, selected) in [
            (&self.placement, false),
            (&self.dismissal, false),
            (&self.enable, true),
            (&self.show_once, false),
            (&self.custom_content, false),
        ] {
            toggle.update(cx, |toggle, cx| toggle.set_data(selected, cx));
        }
        self.apply(cx);
    }

    fn configuration(&self) -> Tooltip {
        let mut tooltip = Tooltip::new(self.text.clone())
            .delay(Duration::from_millis(self.delay_ms))
            .hide_delay(Duration::from_millis(self.hide_delay_ms))
            .duration((self.duration_ms > 0).then(|| Duration::from_millis(self.duration_ms)))
            .placement(if self.below {
                TooltipPlacement::Below
            } else {
                TooltipPlacement::Above
            })
            .dismissal(if self.permanent {
                TooltipDismissal::Permanently
            } else {
                TooltipDismissal::UntilLeave
            })
            .enabled(self.enabled);
        if self.once {
            tooltip = tooltip.show_once();
        }
        if !self.shortcut.is_empty() {
            tooltip = tooltip.shortcut(self.shortcut.clone());
        }
        if self.custom {
            tooltip = tooltip.template(|model| {
                let mut model = model.clone();
                model.text = format!("Custom help\n{}", model.text).into();
                bubble(&model)
            });
        }
        #[cfg(all(test, feature = "test-support"))]
        if let Some(probe) = self.render_probe.clone() {
            tooltip = tooltip.template(move |model| probe(model));
        }
        tooltip
    }

    fn apply(&self, cx: &mut Context<Self>) {
        self.preview.clone().tooltip(self.configuration(), cx);
        cx.notify();
    }
}

fn reset_button<M: 'static>(id: &'static str, help: &'static str, look: &Look, cx: &mut Context<M>) -> Entity<Button> {
    radix::Button::new(id)
        .look(look)
        .ghost_quiet()
        .role(xgpui_luma::controls::button_family::ButtonFamilyRole::Icon)
        .content(|model, _| {
            icon_named("reset")
                .map(|icon| react_icon(icon, model.look.foreground, 15.0))
                .unwrap_or_else(|| div().into_any_element())
        })
        .with_template_modifier(move |root, _| root.flex_none().debug_selector(move || id.into()))
        .spawn(cx)
        .help(help, cx)
}

// Passive layout helpers; all interactive controls come from the Radix factories.
fn note(look: &Look, text: impl Into<gpui::SharedString>) -> AnyElement {
    div()
        .text_size(px(12.0))
        .line_height(px(17.0))
        .text_color(look.resolve_role(SemanticRole::MutedForeground).hsla())
        .child(text.into())
        .into_any_element()
}

fn property(
    look: &Look,
    title: impl Into<gpui::SharedString>,
    explanation: &str,
    control: impl IntoElement,
) -> AnyElement {
    div()
        .w_full()
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(div().text_size(px(13.0)).font_weight(FontWeight::SEMIBOLD).child(title.into()))
        .child(control)
        .child(note(look, explanation))
        .into_any_element()
}

fn panel(look: &Look, title: &str, description: &str, content: impl IntoElement) -> AnyElement {
    radix::Card::new(look)
        .size(radix::CardSize::Two)
        .child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(20.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(5.0))
                        .child(div().text_size(px(16.0)).font_weight(FontWeight::SEMIBOLD).child(title.to_owned()))
                        .child(note(look, description)),
                )
                .child(content),
        )
        .into_element()
        .w_full()
        .flex_none()
        .into_any_element()
}

fn switch_control(switch: Switch, selector: &'static str) -> AnyElement {
    div()
        .self_start()
        .flex_none()
        .flex()
        .items_start()
        .debug_selector(move || selector.into())
        .child(switch)
        .into_any_element()
}

impl Render for TooltipPlayground {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let look = &self.look;
        let wide = f32::from(window.viewport_size().width) >= 1050.0;
        let ready = if !self.enabled {
            "Disabled — enable the tooltip to test it."
        } else if self.text.is_empty() {
            "No message — enter text to show a tooltip."
        } else if self.once {
            "Show once is on — restart before each new presentation."
        } else if self.permanent {
            "Keep dismissed is on — restart after Escape."
        } else {
            "Ready — hover the test button to begin."
        };
        let target = panel(
            look,
            "Test target",
            "The properties configure the tooltip on this button.",
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(note(look, ready))
                .child(div().w_full().h(px(80.0)).flex().items_center().justify_center().child(self.preview.clone()))
                .child(
                    div()
                        .debug_selector(|| "tooltip-exit-area".into())
                        .w_full()
                        .h(px(48.0))
                        .rounded(px(6.0))
                        .bg(look.resolve_role(SemanticRole::Background).hsla())
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(note(look, "Move the pointer here to test hiding")),
                ),
        );
        let actions = div()
            .w_full()
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div().flex().items_start().gap(px(12.0)).child(self.rearm.clone()).child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(note(look, "Clears dismissal and show-once history. Keeps properties and click count.")),
                ),
            )
            .child(
                div().flex().items_start().gap(px(12.0)).child(self.reset_all.clone()).child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(note(look, "Restores default properties and restarts. Keeps click count.")),
                ),
            );
        let text_row = div()
            .w_full()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(div().flex_1().min_w_0().child(self.text_input.clone()))
            .child(self.reset_text.clone());
        let hint_row = div()
            .w_full()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(div().flex_1().min_w_0().child(self.shortcut_input.clone()))
            .child(self.reset_shortcut.clone());
        let content = panel(look, "Content", "Edit these values, then hover the test button to see the result.",
            div().flex().flex_col().gap(px(24.0))
                .child(property(look, "Tooltip message", "This text appears inside the button's tooltip. Empty text disables the bubble. The reset icon restores the original message.", text_row))
                .child(property(look, "Shortcut / hint", "The sample value ⌘S appears beside the message in the test button's tooltip. Edit it to change that display; delete it to hide the hint. It does not bind a key. Reset restores ⌘S.", hint_row))
                .child(property(look, "Custom content", "On adds a Custom help heading to the bubble. Off uses the standard tooltip renderer. Both use the current theme.", switch_control(self.custom_content.clone(), "tooltip-custom"))));
        let timing = panel(look, "Timing", "All values are milliseconds. 1,000 ms is one second.",
            div().flex().flex_col().gap(px(24.0))
                .child(property(look, format!("Show delay · {} ms", self.delay_ms), "How long the pointer must stay over the button before help appears. 0 shows immediately. Keyboard focus always opens immediately.", self.delay.clone()))
                .child(property(look, format!("Hide delay · {} ms", self.hide_delay_ms), "How long help remains after leaving both button and bubble. Try 1,000 ms, then move to the exit area. Click and Esc close immediately regardless of this value.", self.hide_delay.clone()))
                .child(property(look, if self.duration_ms == 0 { "Visible duration · unlimited".into() } else { format!("Visible duration · {} ms", self.duration_ms) }, "Maximum time visible, starting when the bubble appears. 0 means unlimited. With a limit, stay hovered to see it expire; leave and return to show it again.", self.duration.clone())));
        let behavior = panel(look, "Behavior", "Switches apply immediately. A filled track means the option is on.",
            div().flex().flex_col().gap(px(24.0))
                .child(property(look, "Tooltip enabled", "Off prevents the bubble from appearing. The test button still accepts clicks.", switch_control(self.enable.clone(), "tooltip-enable")))
                .child(property(look, "Preferred placement", "Off prefers above the button; on prefers below. The bubble flips when the preferred side has insufficient room.", switch_control(self.placement.clone(), "tooltip-placement")))
                .child(property(look, "Escape dismissal", "Off: Esc hides help until you leave the target and return. On: Esc keeps help dismissed until Restart test or a property change.", switch_control(self.dismissal.clone(), "tooltip-dismissal")))
                .child(property(look, "Show once", "On allows one actual presentation per test. Leaving before the show delay does not consume it. Use Restart test to allow another presentation.", switch_control(self.show_once.clone(), "tooltip-once"))));
        let properties = div()
            .id("tooltip-properties")
            .debug_selector(|| "tooltip-properties".into())
            .w_full()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .children([content, timing, behavior]);
        let left = div()
            .debug_selector(|| "tooltip-test-column".into())
            .flex()
            .flex_col()
            .gap(px(16.0))
            .when(wide, |column| column.w(px(360.0)).flex_none())
            .when(!wide, |column| column.w_full().flex_none())
            .child(target)
            .child(actions);
        let right = div()
            .id("tooltip-property-scroll")
            .flex()
            .flex_col()
            .when(wide, |column| column.flex_1().min_w_0().h_full().overflow_y_scroll())
            .when(!wide, |column| column.w_full().flex_none())
            .child(properties);
        div().w_full().flex_none().flex().flex_col().gap(px(24.0))
            .text_color(look.resolve_role(SemanticRole::Foreground).hsla())
            .child(div().flex().flex_col().gap(px(6.0))
                .child(div().text_size(px(24.0)).font_weight(FontWeight::SEMIBOLD).child("Tooltip test bench"))
                .child(note(look, "Configure the properties, then exercise the test button. Each property change applies immediately and starts a fresh tooltip test.")))
            .child(div().w_full().flex().gap(px(24.0))
                .when(wide, |layout| layout.h(px((f32::from(window.viewport_size().height) - 300.0).max(460.0))).items_stretch())
                .when(!wide, |layout| layout.flex_col().flex_none())
                .child(left).child(right))
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use gpui::{MouseMoveEvent, TestAppContext, point};

    struct Page {
        playground: Entity<TooltipPlayground>,
    }
    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            // Match the app's flex column and scroll viewport, not an isolated button.
            div().size_full().flex().flex_col().child(div().h(px(80.0)).flex_none()).child(
                div()
                    .id("test-scroll")
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .overflow_y_scroll()
                    .child(div().w_full().flex().flex_col().px(px(32.0)).pt(px(20.0)).child(self.playground.clone())),
            )
        }
    }

    #[test]
    fn standard_window_separates_target_and_properties_and_resets_defaults() {
        let mut app = TestAppContext::single();
        let look = Arc::new(Look::built_in());
        let (page, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Page { playground: cx.new(|cx| TooltipPlayground::new(&look, cx)) }
        });
        cx.simulate_resize(gpui::size(px(1300.0), px(800.0)));
        cx.run_until_parked();
        let target = cx.debug_bounds("tooltip-test-column").expect("target column");
        let properties = cx.debug_bounds("tooltip-properties").expect("properties column");
        let preview = cx.debug_bounds("tooltip-preview").expect("visible test button");
        assert!(target.origin.x + target.size.width < properties.origin.x);
        assert!(preview.origin.y + preview.size.height < px(800.0));
        let playground = cx.update(|_, app| page.read(app).playground.clone());
        cx.update(|_, app| {
            playground.update(app, |state, cx| {
                state.text = "Changed".into();
                state.shortcut = "Ctrl+S".into();
                state.delay_ms = 1500;
                state.hide_delay_ms = 900;
                state.duration_ms = 5000;
                state.enabled = false;
                state.below = true;
                state.permanent = true;
                state.once = true;
                state.custom = true;
                state.clicks = 3;
                state.apply(cx);
            })
        });
        cx.run_until_parked();
        let reset = cx.debug_bounds("tooltip-reset-all").expect("reset action");
        cx.simulate_click(reset.center(), Default::default());
        cx.run_until_parked();
        cx.update(|_, app| {
            let state = playground.read(app);
            assert_eq!(state.text, DEFAULT_TEXT);
            assert_eq!(state.shortcut, DEFAULT_HINT);
            assert_eq!((state.delay_ms, state.hide_delay_ms, state.duration_ms), (500, 180, 0));
            assert!(state.enabled);
            assert!(!state.below && !state.permanent && !state.once && !state.custom);
            assert_eq!(state.clicks, 3);
            assert_eq!(state.text_input.read(app).value(), DEFAULT_TEXT);
            assert_eq!(state.shortcut_input.read(app).value(), DEFAULT_HINT);
            assert!(*state.enable.read(app).data());
            assert!(!*state.show_once.read(app).data());
        });
    }

    #[test]
    fn field_edits_and_resets_reach_the_rendered_tooltip() {
        let mut app = TestAppContext::single();
        let look = Arc::new(Look::built_in());
        let (page, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            Page { playground: cx.new(|cx| TooltipPlayground::new(&look, cx)) }
        });
        cx.simulate_resize(gpui::size(px(1300.0), px(2600.0)));
        cx.run_until_parked();
        let playground = cx.update(|_, app| page.read(app).playground.clone());
        let painted = Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = painted.clone();
        cx.update(|_, app| {
            playground.update(app, |state, _| {
                state.render_probe = Some(Arc::new(move |model| {
                    log.lock()
                        .unwrap()
                        .push((model.text.to_string(), model.shortcut.as_ref().map(ToString::to_string)));
                    bubble(model)
                }));
            })
        });
        for (selector, value) in [("tooltip-text", "Changed help"), ("tooltip-shortcut", "Ctrl+S")] {
            let bounds = cx.debug_bounds(selector).unwrap();
            cx.simulate_click(bounds.center(), Default::default());
            cx.simulate_keystrokes("cmd-a");
            cx.simulate_input(value);
            cx.run_until_parked();
        }
        let hover = |cx: &mut gpui::VisualTestContext| {
            let bounds = cx.debug_bounds("tooltip-preview").unwrap();
            cx.simulate_event(MouseMoveEvent { position: bounds.center(), ..Default::default() });
            cx.run_until_parked();
            cx.executor().advance_clock(Duration::from_millis(500));
            cx.run_until_parked();
        };
        hover(cx);
        assert_eq!(painted.lock().unwrap().last(), Some(&("Changed help".into(), Some("Ctrl+S".into()))));
        let bounds = cx.debug_bounds("tooltip-text").unwrap();
        cx.simulate_click(bounds.center(), Default::default());
        cx.simulate_keystrokes("cmd-a backspace");
        cx.run_until_parked();
        let count = painted.lock().unwrap().len();
        hover(cx);
        assert_eq!(painted.lock().unwrap().len(), count);
        for (selector, expected_shortcut) in [
            ("tooltip-reset-text", Some("Ctrl+S".to_owned())),
            ("tooltip-reset-shortcut", Some(DEFAULT_HINT.to_owned())),
        ] {
            let bounds = cx.debug_bounds(selector).unwrap();
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
            hover(cx);
            assert_eq!(painted.lock().unwrap().last(), Some(&(DEFAULT_TEXT.into(), expected_shortcut)));
        }
    }

    #[test]
    fn preview_and_option_buttons_dispatch_in_scroll_page() {
        let mut app = TestAppContext::single();
        let look = Arc::new(Look::built_in());
        let (page, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            Page { playground: cx.new(|cx| TooltipPlayground::new(&look, cx)) }
        });
        cx.simulate_resize(gpui::size(px(1300.0), px(2600.0)));
        cx.run_until_parked();
        let playground = cx.update(|_, app| page.read(app).playground.clone());
        for clicks in 1..=2 {
            let bounds = cx.debug_bounds("tooltip-preview").expect("painted preview");
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
            assert_eq!(cx.update(|_, app| playground.read(app).clicks), clicks);
        }
        for (selector, value) in [("tooltip-text", "Updated help"), ("tooltip-shortcut", "Ctrl+S")] {
            let bounds = cx.debug_bounds(selector).expect("painted text input");
            cx.simulate_click(bounds.center(), Default::default());
            cx.simulate_keystrokes("cmd-a");
            cx.simulate_input(value);
            cx.run_until_parked();
        }
        cx.update(|_, app| {
            let state = playground.read(app);
            assert_eq!(state.text, "Updated help");
            assert_eq!(state.shortcut, "Ctrl+S");
        });
        for selector in ["tooltip-reset-text", "tooltip-reset-shortcut"] {
            let bounds = cx.debug_bounds(selector).expect("painted reset icon");
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
        }
        cx.update(|_, app| {
            let state = playground.read(app);
            assert_eq!(state.text, DEFAULT_TEXT);
            assert_eq!(state.shortcut, DEFAULT_HINT);
            assert_eq!(state.text_input.read(app).value(), DEFAULT_TEXT);
            assert_eq!(state.shortcut_input.read(app).value(), DEFAULT_HINT);
        });
        let custom_bounds = cx.debug_bounds("tooltip-custom").expect("painted custom switch");
        cx.simulate_click(custom_bounds.center(), Default::default());
        cx.run_until_parked();
        assert!(cx.update(|_, app| playground.read(app).custom));
        let preview_bounds = cx.debug_bounds("tooltip-preview").expect("painted preview");
        assert!(preview_bounds.size.width > px(0.0));
        assert!(preview_bounds.size.height > px(0.0));
        let delay_bounds = cx.debug_bounds("tooltip-delay").expect("painted delay slider");
        cx.simulate_click(
            point(delay_bounds.origin.x + delay_bounds.size.width * 0.75, delay_bounds.center().y),
            Default::default(),
        );
        cx.run_until_parked();
        let delay_ms = cx.update(|_, app| playground.read(app).delay_ms);
        assert!(delay_ms > 500 && delay_ms <= 2000);
        assert_eq!(delay_ms % 50, 0);
        let hide_bounds = cx.debug_bounds("tooltip-hide-delay").expect("painted hide delay slider");
        cx.simulate_click(hide_bounds.center(), Default::default());
        cx.run_until_parked();
        assert!(cx.update(|_, app| playground.read(app).hide_delay_ms) > 180);
        let duration_bounds = cx.debug_bounds("tooltip-duration").expect("painted duration slider");
        cx.simulate_click(duration_bounds.center(), Default::default());
        cx.run_until_parked();
        let duration_ms = cx.update(|_, app| playground.read(app).duration_ms);
        assert!(duration_ms > 0 && duration_ms <= 10000);
        assert_eq!(duration_ms % 250, 0);
        for selector in ["tooltip-placement", "tooltip-dismissal", "tooltip-enable"] {
            let bounds = cx.debug_bounds(selector).expect("painted option");
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
        }
        cx.update(|_, app| {
            let state = playground.read(app);
            assert!(state.below && state.permanent && !state.enabled);
            assert!(*state.placement.read(app).data());
            assert!(*state.dismissal.read(app).data());
            assert!(!*state.enable.read(app).data());
        });
        // Restore ordinary Escape/re-entry behavior for the lifecycle checks.
        for selector in ["tooltip-placement", "tooltip-dismissal", "tooltip-enable"] {
            let bounds = cx.debug_bounds(selector).expect("painted option");
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
        }
        let once_bounds = cx.debug_bounds("tooltip-once").expect("painted once button");
        cx.simulate_click(once_bounds.center(), Default::default());
        cx.run_until_parked();
        assert!(cx.update(|_, app| playground.read(app).once));
        assert!(cx.update(|_, app| *playground.read(app).show_once.read(app).data()));
        let rearm_bounds = cx.debug_bounds("tooltip-rearm").expect("painted rearm button");
        cx.simulate_click(rearm_bounds.center(), Default::default());
        cx.run_until_parked();

        // Observe a real painted bubble through the same target and scroll layout.
        let rendered = Arc::new(AtomicUsize::new(0));
        let count = rendered.clone();
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = events.clone();
        cx.update(|_, app| {
            playground.update(app, |state, _| state.once = false);
            playground.read(app).preview.clone().tooltip(
                playground
                    .read(app)
                    .configuration()
                    .on_event(move |event| log.lock().unwrap().push(event))
                    .template(move |model| {
                        count.fetch_add(1, Ordering::Relaxed);
                        bubble(model)
                    }),
                app,
            );
        });
        cx.run_until_parked();
        cx.simulate_event(MouseMoveEvent { position: preview_bounds.center(), ..Default::default() });
        cx.run_until_parked();
        assert_eq!(rendered.load(Ordering::Relaxed), 0);
        cx.executor().advance_clock(Duration::from_millis(delay_ms));
        cx.run_until_parked();
        assert!(rendered.load(Ordering::Relaxed) > 0);
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        let dismissed = rendered.load(Ordering::Relaxed);
        cx.simulate_event(MouseMoveEvent { position: preview_bounds.center(), ..Default::default() });
        cx.executor().advance_clock(Duration::from_millis(delay_ms));
        cx.run_until_parked();
        assert_eq!(rendered.load(Ordering::Relaxed), dismissed);
        cx.simulate_event(MouseMoveEvent { position: point(px(700.0), px(500.0)), ..Default::default() });
        cx.run_until_parked();
        cx.simulate_event(MouseMoveEvent { position: preview_bounds.center(), ..Default::default() });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(delay_ms));
        cx.run_until_parked();
        assert!(rendered.load(Ordering::Relaxed) > dismissed);
        use xgpui_luma::controls::tooltip::TooltipEvent;
        cx.executor().advance_clock(Duration::from_millis(duration_ms - 1));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Shown));
        cx.executor().advance_clock(Duration::from_millis(1));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Hidden));
        let expired_count = rendered.load(Ordering::Relaxed);
        cx.simulate_event(MouseMoveEvent { position: preview_bounds.center(), ..Default::default() });
        cx.executor().advance_clock(Duration::from_millis(delay_ms));
        cx.run_until_parked();
        assert_eq!(rendered.load(Ordering::Relaxed), expired_count);
        cx.simulate_event(MouseMoveEvent { position: point(px(700.0), px(700.0)), ..Default::default() });
        cx.run_until_parked();
        cx.simulate_event(MouseMoveEvent { position: preview_bounds.center(), ..Default::default() });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(delay_ms));
        cx.run_until_parked();
        assert!(rendered.load(Ordering::Relaxed) > expired_count);
        let bounds = cx.debug_bounds("tooltip-duration").expect("painted duration slider");
        cx.simulate_click(point(bounds.origin.x + px(1.0), bounds.center().y), Default::default());
        cx.run_until_parked();
        assert_eq!(cx.update(|_, app| playground.read(app).duration_ms), 0);
    }
}
