//! Headless regression coverage for shared group renderers and their extension points.
use std::sync::Arc;

use gpui::{Context, Render, TestAppContext, Window, div, prelude::*, px};
use xgpui_luma::controls::control_group::*;
use crate::*;

fn items() -> Vec<ControlGroupItem> {
    ["a", "b", "c", "d"]
        .into_iter()
        .map(|id| ControlGroupItem::new(id).label(id).enabled(id != "c"))
        .collect()
}

struct GroupHarness {
    group: ControlGroup<ControlGroupItem>,
    events: Vec<ControlGroupEvent>,
}
impl GroupHarness {
    fn new(kind: usize, horizontal: bool, custom: usize, cx: &mut Context<Self>) -> Self {
        xgpui_luma::focus::bind_default_focus_keys(cx);
        xgpui_luma::key_handling::bind_default_control_keys(cx);
        let look = Look::built_in();
        let group = match kind {
            0 => {
                let mut builder = CheckboxGroup::new("group").look(&look).items(items()).selected_ids(["a"]);
                if horizontal {
                    builder = builder.horizontal();
                }
                builder.spawn(cx)
            }
            1 => {
                let mut builder = RadioGroup::new("group").look(&look).items(items()).selected("a");
                if horizontal {
                    builder = builder.horizontal();
                }
                builder.spawn(cx).read(cx).inner_group()
            }
            4 => {
                use xgpui_luma::controls::radio_group::{self, RadioGroupLayout, radio_group_buttons_template};
                let layout = if horizontal {
                    RadioGroupLayout::Horizontal
                } else {
                    RadioGroupLayout::Vertical
                };
                let mut builder =
                    radio_group::new("group").items(items()).selected("a").template(radio_group_buttons_template(
                        radio_template_for(&look, RadioVariant::default(), Paint::accent(), RadioSize::default()),
                        layout,
                    ));
                if horizontal {
                    builder = builder.horizontal();
                }
                builder.spawn(cx).read(cx).inner_group()
            }
            _ => {
                let mut builder = if kind == 2 {
                    control_group_builder(&look, "group", xgpui_luma::theme::ControlSize::Md)
                } else {
                    icon_group_builder(&look, "group", xgpui_luma::theme::ControlSize::Md)
                }
                .items(items())
                .selected_ids(["a"]);
                builder = if horizontal {
                    builder.horizontal()
                } else {
                    builder.vertical()
                };
                builder.spawn(cx)
            }
        };
        group.update(cx, |group, cx| {
            if custom > 0 {
                group.set_item_template(
                    Some(make_control_group_item_template(
                        |item: &ControlGroupItemRenderModel<'_, ControlGroupItem>, _, _| {
                            let id = format!("content-{}", item.item.id());
                            div()
                                .id(item.item.id().clone())
                                .debug_selector(move || id.clone())
                                .child(item.item.label().clone())
                        },
                    )),
                    cx,
                );
            }
            if custom == 2 {
                group.set_item_element_template(
                    Some(Arc::new(|item, custom, window, cx| {
                        let id = format!("override-{}", item.item.id());
                        div()
                            .id(item.item.id().clone())
                            .debug_selector(move || id.clone())
                            .w(px(40.0))
                            .h(px(30.0))
                            .child(custom.expect("content override retained")(item, window, cx))
                    })),
                    cx,
                );
            }
        });
        let group = group.on_event(cx, |owner, event, _| owner.events.push(event.clone()));
        Self { group, events: Vec::new() }
    }
}
impl Render for GroupHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.group.clone())
    }
}

#[test]
fn shared_groups_preserve_input_callbacks_templates_layout_and_identity() {
    for kind in 0..5 {
        for horizontal in [false, true] {
            for custom in 0..3 {
                let mut app = TestAppContext::single();
                let (view, cx) = app.add_window_view(|window, cx| {
                    window.activate_window();
                    GroupHarness::new(kind, horizontal, custom, cx)
                });
                cx.simulate_resize(gpui::size(px(600.0), px(600.0)));
                cx.run_until_parked();
                let selector = |id: &str| {
                    let index = match id {
                        "a" => 0,
                        "b" => 1,
                        "c" => 2,
                        "d" => 3,
                        _ => unreachable!(),
                    };
                    if custom == 2 {
                        ["override-a", "override-b", "override-c", "override-d"][index]
                    } else {
                        ["group-a", "group-b", "group-c", "group-d"][index]
                    }
                };
                let a = cx.debug_bounds(selector("a")).expect("first row");
                let b = cx.debug_bounds(selector("b")).expect("second row");
                if horizontal {
                    assert!(b.origin.x > a.origin.x);
                } else {
                    assert!(b.origin.y > a.origin.y);
                }
                if custom > 0 {
                    assert!(cx.debug_bounds("content-b").is_some());
                }
                if custom == 2 {
                    assert!(cx.debug_bounds("group-b").is_none(), "element override wins");
                }
                let click = |cx: &mut gpui::VisualTestContext, id: &str| {
                    let bounds = cx.debug_bounds(selector(id)).unwrap();
                    cx.simulate_click(bounds.center(), Default::default());
                    cx.run_until_parked();
                };
                click(cx, "b");
                click(cx, "c");
                cx.update(|_, app| {
                    let harness = view.read(app);
                    assert_eq!(harness.group.read(app).active_id().unwrap().as_ref(), "b");
                    assert_eq!(
                        harness.events.iter().filter(|e| matches!(e, ControlGroupEvent::Change { .. })).count(),
                        1
                    );
                    assert_eq!(
                        harness.events.iter().filter(|e| matches!(e, ControlGroupEvent::Activate { .. })).count(),
                        1
                    );
                    assert!(
                        harness.events.iter().any(|e| matches!(e, ControlGroupEvent::FocusChanged { focused: true }))
                    );
                });
                cx.simulate_keystrokes(if horizontal { "right" } else { "down" });
                cx.run_until_parked();
                cx.update(|_, app| assert_eq!(view.read(app).group.read(app).active_id().unwrap().as_ref(), "d"));
                cx.simulate_keystrokes("space");
                cx.run_until_parked();
                cx.update(|_, app| assert!(view.read(app).group.read(app).selected_ids().iter().any(|id| id == "d")));
                // Reordering keeps selection and event targets tied to IDs, not indices.
                let group = cx.update(|_, app| view.read(app).group.clone());
                group.update(cx, |group, cx| group.set_items(items().into_iter().rev(), cx));
                cx.run_until_parked();
                click(cx, "a");
                cx.update(|_, app| {
                    assert_eq!(group.read(app).active_id().unwrap().as_ref(), "a");
                    assert_eq!(group.read(app).selected_ids().iter().any(|id| id == "a"), kind != 0);
                });
                group.update(cx, |group, cx| group.set_enabled(false, cx));
                cx.run_until_parked();
                let before = cx.update(|_, app| group.read(app).selected_ids().to_vec());
                click(cx, "b");
                cx.simulate_keystrokes("space");
                cx.run_until_parked();
                cx.update(|_, app| assert_eq!(group.read(app).selected_ids(), before));
            }
        }
    }
}

#[test]
fn shared_button_group_chrome_preserves_size_radius_disabled_palette_and_custom_foreground() {
    use xgpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale, ThemeMode};
    use xgpui_luma::controls::button_family::{ButtonFamilyRole, compose_button_family_look};
    struct Chrome {
        look: Look,
        size: ControlSize,
        icon: bool,
        selected: bool,
        disabled: bool,
        focused: bool,
    }
    impl Render for Chrome {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let stylesheet = self.look.common_stylesheet();
            let id = "chrome".into();
            let item = ControlGroupItem::new("item").label("Item");
            let model = ControlGroupItemRenderModel {
                group_id: &id,
                stylesheet: &stylesheet,
                layout: ControlGroupLayout::Vertical,
                item: &item,
                index: 0,
                sibling_count: 1,
                selected: self.selected,
                selection_progress: if self.selected { 1.0 } else { 0.0 },
                active: self.focused,
                enabled: !self.disabled,
                state: ControlGroupItemState {
                    disabled: self.disabled,
                    active: self.focused,
                    focus_visible: self.focused,
                    selected: self.selected,
                    ..Default::default()
                },
                selection_mode: ControlSelectionMode::SingleAllowNone,
                foreground: None,
            };
            let expected = button_family_theme(&self.look, ButtonVariant::Ghost)
                .resolve(
                    ButtonFamilyRole::Toggle { selected: self.selected },
                    self.size,
                    InteractionState { focused: self.focused, disabled: self.disabled, ..Default::default() },
                )
                .foreground;
            let custom = make_control_group_item_template(
                move |item: &ControlGroupItemRenderModel<'_, ControlGroupItem>, _, _| {
                    assert_eq!(item.foreground, Some(expected));
                    div().w(px(10.0)).h(px(10.0))
                },
            );
            let template = if self.icon {
                icon_group_item_template(&self.look, self.size)
            } else {
                control_group_item_template(&self.look, self.size)
            };
            div().flex().items_start().child(template(&model, Some(&custom), window, cx))
        }
    }
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            for icon in [false, true] {
                let look = Look::built_in();
                look.set_mode(mode);
                let scale = StandardBoxScale::compute(size, &look.metrics(), 1.0);
                let paint_look = look.clone();
                let mut app = TestAppContext::single();
                let (view, cx) = app.add_window_view(|_, _| Chrome {
                    look: paint_look,
                    size,
                    icon,
                    selected: false,
                    disabled: false,
                    focused: false,
                });
                let mut initial = None;
                for (selected, disabled, focused) in
                    [(false, false, false), (true, false, false), (true, false, true), (true, true, true)]
                {
                    view.update(cx, |view, cx| {
                        view.selected = selected;
                        view.disabled = disabled;
                        view.focused = focused;
                        cx.notify();
                    });
                    cx.run_until_parked();
                    let bounds = cx.debug_bounds("chrome-item").unwrap();
                    let state = InteractionState { disabled, focused, ..Default::default() };
                    let theme = button_family_theme(&look, ButtonVariant::Ghost);
                    let role = ButtonFamilyRole::Toggle { selected };
                    let palette = theme.resolve_look(role, size, state, &scale, scale.radius).unwrap_or_else(|| {
                        compose_button_family_look(&theme.resolve(role, size, state), role, &scale, scale.radius)
                    });
                    assert_eq!(bounds.size.height, px(palette.height));
                    if icon {
                        assert_eq!(bounds.size.width, px(palette.height));
                    }
                    if let Some(initial) = initial {
                        assert_eq!(bounds, initial, "state must not alter hit bounds");
                    }
                    initial = Some(bounds);
                    cx.update(|window, _| {
                        let quads = window.painted_quads();
                        if palette.background.a > 0.0 || palette.border.is_some_and(|border| border.a > 0.0) {
                            let face = quads
                                .iter()
                                .find(|q| q.background.as_solid() == Some(palette.background))
                                .expect("unchanged Radix background, including disabled alpha");
                            if let Some(border) = palette.border.filter(|border| border.a > 0.0) {
                                assert!(quads.iter().any(|q| q.border_color == border), "unchanged Radix border");
                            }
                            let radius = if icon {
                                look.metrics().radius.pill.min(palette.height / 2.0)
                            } else {
                                scale.radius
                            };
                            assert_eq!(face.corner_radii.top_left, px(radius).scale(window.scale_factor()));
                        }
                        assert_eq!(
                            quads
                                .iter()
                                .any(|q| q.border_color == palette.focus_border && palette.focus_border.a > 0.0),
                            focused && !disabled
                        );
                    });
                }
            }
        }
    }
}

#[test]
fn choice_group_rows_preserve_indicator_geometry_and_spacing() {
    use xgpui_luma::theme::{InteractionState, ThemeMode};
    struct Choices {
        group: ControlGroup<ControlGroupItem>,
    }
    impl Render for Choices {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().flex().items_start().child(self.group.clone())
        }
    }
    let verify = |template: ControlGroupTemplate<ControlGroupItem>,
                  horizontal: bool,
                  color: gpui::Hsla,
                  diameter: f32,
                  radius: f32| {
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, cx| {
            let mut builder = ControlGroupBuilder::new("choice")
                .items(items())
                .selected_ids(["a"])
                .stylesheet(&Look::built_in().common_stylesheet())
                .template(template);
            if horizontal {
                builder = builder.horizontal();
            }
            Choices { group: builder.spawn(cx) }
        });
        cx.simulate_mouse_move(gpui::point(px(1000.0), px(1000.0)), None, Default::default());
        cx.run_until_parked();
        let a = cx.debug_bounds("choice-a").unwrap();
        let b = cx.debug_bounds("choice-b").unwrap();
        if horizontal {
            assert_eq!(b.left() - a.right(), px(8.0));
            assert_eq!(a.top(), b.top());
        } else {
            assert_eq!(b.top() - a.bottom(), px(8.0));
            assert_eq!(a.left(), b.left());
        }
        cx.update(|window, _| {
            let quads = window.painted_quads();
            let indicator = quads
                .iter()
                .find(|q| {
                    q.background.as_solid() == Some(color)
                        && q.bounds.size.width == px(diameter).scale(window.scale_factor())
                        && q.bounds.size.height == px(diameter).scale(window.scale_factor())
                })
                .expect("selected indicator retains its Radix diameter and palette");
            assert_eq!(indicator.corner_radii.top_left, px(radius).scale(window.scale_factor()));
        });
    };
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let look = Look::built_in();
        look.set_mode(mode);
        for horizontal in [false, true] {
            for size in CheckboxSize::ALL {
                for radius in [Radius::None, Radius::Small, Radius::Medium, Radius::Large, Radius::Full] {
                    for variant in [CheckboxVariant::Classic, CheckboxVariant::Surface, CheckboxVariant::Soft] {
                        let theme = checkbox_theme_for(&look, variant, Paint::accent(), size, radius);
                        let palette = theme.resolve(true, InteractionState::default(), size.control_size());
                        let scale = checkbox_scale_for(size, radius);
                        verify(
                            checkbox_group_template(&look, variant, Paint::accent(), size, radius),
                            horizontal,
                            palette.indicator_background,
                            scale.indicator_size,
                            scale.indicator_radius,
                        );
                    }
                }
            }
            for size in RadioSize::ALL {
                for variant in [RadioVariant::Classic, RadioVariant::Surface, RadioVariant::Soft] {
                    let theme = radio_theme_for(&look, variant, Paint::accent(), size);
                    let palette = theme.resolve(true, InteractionState::default(), size.control_size());
                    let scale = radio_scale_for(size);
                    let layout = if horizontal {
                        xgpui_luma::controls::radio_group::RadioGroupLayout::Horizontal
                    } else {
                        xgpui_luma::controls::radio_group::RadioGroupLayout::Vertical
                    };
                    verify(
                        radio_group_template(&look, variant, Paint::accent(), size, layout),
                        horizontal,
                        palette.indicator_background,
                        scale.indicator_size,
                        scale.indicator_size / 2.0,
                    );
                }
            }
        }
    }
}
