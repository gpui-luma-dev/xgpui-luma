//! Standard Radix radio groups backed by SDK selection and keyboard navigation.

use std::sync::Arc;
use gpui::{Context, SharedString, prelude::*};
use xgpui_luma::controls::control_group::{ControlGroupItem, ControlGroupItemLike};
use xgpui_luma::controls::radio_group::{
    RadioGroupBuilder, RadioGroupLayout, RadioGroupTemplate, radio_group_buttons_template,
};
use crate::{Look, Paint, RadioSize, RadioVariant};
use crate::look::resolve_look;

/// Standard (non-segmented) radio template.
pub fn radio_group_template<T: ControlGroupItemLike + 'static>(
    look: &Look,
    variant: RadioVariant,
    paint: Paint,
    size: RadioSize,
    layout: RadioGroupLayout,
) -> RadioGroupTemplate<T> {
    let radio = crate::radio::styled_radio_template(look, variant, paint, size).with_modifier(|element, model| {
        let selector = model.id.to_string();
        element.debug_selector(move || selector.clone())
    });
    radio_group_buttons_template(Arc::new(radio), layout)
}

/// Look-owned single-selection group. Events and state are owned by the SDK.
pub struct RadioGroup<T: ControlGroupItemLike + 'static = ControlGroupItem> {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::radio_group::RadioGroupEvent>,

    look: Option<Look>,
    variant: RadioVariant,
    paint: Paint,
    size: RadioSize,
    layout: RadioGroupLayout,
    builder: RadioGroupBuilder<T>,
}

impl<T: ControlGroupItemLike + 'static> RadioGroup<T> {
    xgpui_luma::forward_motion!(xgpui_luma::controls::control_group::ControlGroupMotion, builder builder);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(enabled);
        self
    }

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::radio_group::RadioGroupEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `RadioGroupEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::radio_group::RadioGroupEvent::Change { value } = event {
                callback(owner, value, cx);
            }
        })
    }

    /// Handle `RadioGroupEvent::Activate` through the existing event stream.
    pub fn on_activate<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::radio_group::RadioGroupEvent::Activate { value } = event {
                callback(owner, value, cx);
            }
        })
    }

    /// Handle `RadioGroupEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::radio_group::RadioGroupEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `RadioGroupEvent::ItemFocused` through the existing event stream.
    pub fn on_item_focused<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::radio_group::RadioGroupEvent::ItemFocused { item_id } = event {
                callback(owner, item_id, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            look: None,
            variant: RadioVariant::default(),
            paint: Paint::accent(),
            size: RadioSize::default(),
            layout: RadioGroupLayout::Vertical,
            builder: xgpui_luma::controls::radio_group::new(id).selection_follows_active(true),
        }
    }
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }
    pub fn variant(mut self, variant: RadioVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn paint(mut self, paint: Paint) -> Self {
        self.paint = paint;
        self
    }
    pub fn size(mut self, size: RadioSize) -> Self {
        self.size = size;
        self
    }
    pub fn horizontal(mut self) -> Self {
        self.layout = RadioGroupLayout::Horizontal;
        self.builder = self.builder.horizontal();
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.builder = self.builder.items(items);
        self
    }
    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.selected(id);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }
    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> xgpui_luma::controls::radio_group::RadioGroup<T> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
            self.builder
                .stylesheet(&look.sized_stylesheet(look.size()))
                .motion_defaults(look.motion_defaults().control_group)
                .template(radio_group_template(&look, self.variant, self.paint, self.size, self.layout))
                .spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_radio_group_stylesheet() -> xgpui_luma::theme::stylesheet::RadioGroupStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    RadioGroupStylesheet {
        // The radio template already applies disabled opacity to the whole row.
        geometry: RadioGroupGeometryStylesheet { disabled_opacity: Some(1.0), ..Default::default() },
        sizes: xgpui_luma::theme::ControlSizes::from([(
            "horizontal".into(),
            RadioGroupGeometryStylesheet { gap: Some(8.0), ..Default::default() },
        )]),
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Render, TestAppContext, Window, div, px};
    use xgpui_luma::controls::control_group::{ControlGroupItemRenderModel, make_control_group_item_template};
    use xgpui_luma::controls::radio_group::RadioGroup as SdkRadioGroup;

    struct Harness {
        group: SdkRadioGroup<ControlGroupItem>,
        changes: usize,
        focused: bool,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(self.group.clone())
        }
    }

    #[test]
    fn sdk_radio_rows_keep_selection_keyboard_callbacks_and_custom_content() {
        for horizontal in [false, true] {
            let mut app = TestAppContext::single();
            let (view, cx) = app.add_window_view(|window, cx| {
                window.activate_window();
                xgpui_luma::focus::bind_default_focus_keys(cx);
                xgpui_luma::key_handling::bind_default_control_keys(cx);
                let mut builder = RadioGroup::new("radios")
                    .items([
                        ControlGroupItem::new("a").label("Alpha"),
                        ControlGroupItem::new("b").label("Beta"),
                        ControlGroupItem::new("c").label("Disabled").enabled(false),
                    ])
                    .selected("a")
                    .on_value_changed(cx, |owner: &mut Harness, _, _| owner.changes += 1)
                    .on_focus_changed(cx, |owner: &mut Harness, focused, _| owner.focused = focused);
                if horizontal {
                    builder = builder.horizontal();
                }
                Harness { group: builder.spawn(cx), changes: 0, focused: false }
            });
            cx.simulate_resize(gpui::size(px(600.0), px(400.0)));
            cx.run_until_parked();
            let a = cx.debug_bounds("radios-a").unwrap();
            let b = cx.debug_bounds("radios-b").unwrap();
            if horizontal {
                assert!(b.left() > a.left());
                assert!(
                    f32::from(a.size.width - b.size.width).abs() <= 1.0,
                    "horizontal rows share available width, allowing pixel rounding"
                );
            } else {
                assert!(b.top() > a.top());
            }
            let click = |cx: &mut gpui::VisualTestContext, selector: &'static str| {
                let bounds = cx.debug_bounds(selector).unwrap();
                cx.simulate_click(bounds.center(), Default::default());
                cx.run_until_parked();
            };
            click(cx, "radios-b");
            click(cx, "radios-c");
            cx.update(|_, app| {
                let view = view.read(app);
                assert_eq!(view.group.read(app).selected_id(app), Some("b".into()));
                assert_eq!(view.changes, 1);
                assert!(view.focused);
            });
            cx.simulate_keystrokes(if horizontal { "right" } else { "down" });
            cx.run_until_parked();
            cx.update(|_, app| {
                assert_eq!(view.read(app).group.read(app).selected_id(app), Some("a".into()));
                assert_eq!(view.read(app).changes, 2);
            });
            let group = cx.update(|_, app| view.read(app).group.read(app).inner_group());
            group.update(cx, |group, cx| {
                group.set_item_template(
                    Some(make_control_group_item_template(
                        |item: &ControlGroupItemRenderModel<'_, ControlGroupItem>, _, _| {
                            let selector = format!("custom-{}", item.item.id());
                            div()
                                .id(item.item.id().clone())
                                .debug_selector(move || selector.clone())
                                .child(item.item.label().clone())
                        },
                    )),
                    cx,
                );
            });
            cx.run_until_parked();
            click(cx, "custom-b");
            cx.update(|_, app| {
                assert_eq!(group.read(app).selected_id(), Some(&"b".into()));
                assert_eq!(view.read(app).changes, 3);
            });
            group.update(cx, |group, cx| group.set_enabled(false, cx));
            cx.run_until_parked();
            click(cx, "custom-a");
            cx.simulate_keystrokes("space");
            cx.run_until_parked();
            cx.update(|_, app| {
                assert_eq!(group.read(app).selected_id(), Some(&"b".into()));
                assert_eq!(view.read(app).changes, 3);
            });
            cx.update(|window, _| {
                use xgpui_luma::style::lss::{ControlId, PartId, StyleKey, Value};
                let theme = Look::built_in().style_theme();
                let disabled = crate::style::config().state("disabled").expect("state");
                let resolved =
                    theme.resolve(&StyleKey::new(ControlId::new("radio")).state(disabled).scale(window.scale_factor()));
                let number = |name: &str| resolved.root().value(name).and_then(Value::as_number).expect("number");
                let mut expected =
                    gpui::Hsla::from(resolved.part(PartId::new("indicator")).background().expect("background"));
                expected.a *= number("disabled_opacity");
                let size = number("indicator_size");
                assert!(
                    window.painted_quads().iter().any(|quad| {
                        quad.background.as_solid().is_some_and(|c| {
                            let (c, e) = (c.to_rgb(), expected.to_rgb());
                            (c.r - e.r).abs() < 2e-3 && (c.a - e.a).abs() < 2e-3
                        }) && quad.bounds.size.width == px(size).scale(window.scale_factor())
                    }),
                    "disabled indicator opacity is applied once"
                );
            });
        }
    }

    #[test]
    fn bound_radio_size_controls_label_typography_even_with_sdk_default_model_size() {
        let look = Look::built_in();
        use xgpui_luma::style::lss::{AttrValue, ControlId, PartId, StyleKey};
        let size_attr = crate::style::config().attribute("size").expect("size");
        // Each size has its own label text, larger with the size (values come from `radio.lss`).
        let mut previous = (0.0, 0.0);
        for size in RadioSize::ALL {
            let key = StyleKey::new(ControlId::new("radio")).attr(size_attr, AttrValue::named(size.as_str()));
            let resolved = look.style_theme().resolve(&key);
            let label = resolved.part(PartId::new("label"));
            let text = (label.font_size().expect("a font size"), label.line_height().expect("a line height"));
            assert!(text.0 > previous.0 && text.1 > previous.1, "{size:?}: {text:?} after {previous:?}");
            previous = text;
        }
    }
}
