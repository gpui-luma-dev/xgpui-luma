//! Segmented presentation for the SDK radio group; selection stays SDK-owned.

use std::sync::Arc;
use gpui::{IntoElement, div, prelude::*, px};
use xgpui_luma::controls::control_group::{ControlGroupItemHandlerExt, ControlGroupItemLike};
use xgpui_luma::controls::radio_group::RadioGroupTemplate;
use crate::{Look, ScaleFamily};
use xgpui_luma::infra::locale::{LocaleContext, LogicalStyled};

/// Horizontal radio-group template with a shared track and selected segment.
/// Item templates supply content only; colors and geometry belong to the look.
pub fn segmented_radio_template<T: ControlGroupItemLike + 'static>(look: &Look) -> RadioGroupTemplate<T> {
    let look = look.clone();
    Arc::new(move |model, handlers, window, cx| {
        let geometry = resolve_geometry(&look);
        let gray = |step| look.resolve_step(ScaleFamily::Gray, step).hsla();
        let rows = model
            .items
            .iter()
            .zip(handlers.into_item_handlers())
            .map(|(item, handlers)| {
                let fill = if item.state.pressed {
                    6
                } else if item.selected {
                    5
                } else if item.state.hovered {
                    4
                } else {
                    3
                };
                let border = if item.state.focus_visible {
                    look.resolve_step(ScaleFamily::Color, 8).hsla()
                } else {
                    gray(if item.selected { 6 } else { fill })
                };
                let content = match model.item_template {
                    Some(template) => template(item, window, cx),
                    None => div().child(item.item.label().clone()).into_any_element(),
                };
                div()
                    .id(format!("{}-{}", model.id, item.item.id()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_1()
                    .h(px(geometry.height))
                    .px(px(geometry.padding_x))
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(border)
                    .bg(gray(fill))
                    .text_size(px(geometry.font_size))
                    .line_height(px(geometry.line_height))
                    .text_color(gray(if item.enabled { 12 } else { 8 }))
                    .when(item.enabled, |row| row.cursor_pointer())
                    .child(content)
                    .control_group_item_handlers(handlers)
            })
            .collect::<Vec<_>>();
        div()
            .id(model.id.clone())
            .flex()
            .flex_row_logical(LocaleContext::current(cx).direction)
            .items_center()
            .rounded(px(5.0))
            .bg(gray(3))
            .children(rows)
    })
}

pub(crate) fn resolve_geometry(look: &Look) -> xgpui_luma::theme::stylesheet::SegmentedGeometry {
    let control = look.metrics().control.md;
    let typography = xgpui_luma::theme::ThemeTokens::default().typography.text.label;
    look.common_stylesheet().segmented.resolve_geometry(
        "2",
        xgpui_luma::theme::stylesheet::SegmentedGeometry {
            height: control.height,
            padding_x: control.padding_x,
            font_size: typography.size,
            line_height: typography.line_height,
        },
    )
}

/// Native common geometry owned by this control.
pub(crate) fn native_segmented_stylesheet() -> xgpui_luma::theme::stylesheet::SegmentedStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    SegmentedStylesheet {
        sizes: xgpui_luma::theme::ControlSizes::from([(
            "2".into(),
            SegmentedGeometryStylesheet {
                height: Some(32.0),
                padding_x: Some(12.0),
                font_size: Some(14.0),
                line_height: Some(20.0),
            },
        )]),
        ..Default::default()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod locale_tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};
    use gpui::{Context, Focusable, Render, TestAppContext, Window};
    use xgpui_luma::{
        controls::radio_group::{self, RadioGroup, RadioGroupItem, RadioGroupEvent},
        infra::locale::{LayoutDirection, LocaleContext, set_locale_context},
    };
    struct Harness {
        group: RadioGroup<RadioGroupItem>,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(400.)).child(self.group.clone())
        }
    }
    #[test]
    fn segmented_visual_order_matches_arrows_and_keeps_selection_on_locale_refresh() {
        let mut app = TestAppContext::single();
        let (host, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            let group = radio_group::new("rtl-segments")
                .horizontal()
                .active_descendant()
                .selection_follows_active(true)
                .items([
                    RadioGroupItem::new("first"),
                    RadioGroupItem::new("disabled").enabled(false),
                    RadioGroupItem::new("middle"),
                    RadioGroupItem::new("last"),
                ])
                .selected("middle")
                .active("middle")
                .motion_enabled(false)
                .with_item_template(|item, _, _| {
                    let index = item.index;
                    div().debug_selector(move || format!("segment-label-{index}")).child(item.item.label().clone())
                })
                .template(segmented_radio_template(&Look::built_in()))
                .spawn(cx);
            Harness { group }
        });
        cx.run_until_parked();
        let group = cx.update(|_, cx| host.read(cx).group.clone());
        cx.update(|window, cx| group.read(cx).focus_handle(cx).focus(window, cx));
        let changes = Rc::new(Cell::new(0));
        let observed = changes.clone();
        let _sub = cx.update(|_, cx| {
            cx.subscribe(&group, move |_, event: &RadioGroupEvent, _| {
                if matches!(event, RadioGroupEvent::Change { .. }) {
                    observed.set(observed.get() + 1);
                }
            })
        });
        for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr, LayoutDirection::Rtl] {
            group.update(cx, |view, cx| {
                view.set_selected("middle", cx);
                view.set_active("middle", cx);
            });
            let before = changes.get();
            cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
            cx.run_until_parked();
            let first = cx.debug_bounds("segment-label-0").unwrap();
            let last = cx.debug_bounds("segment-label-3").unwrap();
            assert_eq!(first.left() < last.left(), direction == LayoutDirection::Ltr);
            assert_eq!(changes.get(), before);
            cx.update(|window, cx| {
                assert_eq!(group.read(cx).selected_id(cx).as_deref(), Some("middle"));
                assert!(group.read(cx).focus_handle(cx).is_focused(window));
            });
            cx.simulate_keystrokes("right");
            cx.run_until_parked();
            cx.update(|_, cx| {
                assert_eq!(
                    group.read(cx).selected_id(cx).as_deref(),
                    Some(if direction == LayoutDirection::Rtl {
                        "first"
                    } else {
                        "last"
                    })
                )
            });
            cx.simulate_keystrokes("left");
            cx.run_until_parked();
            cx.update(|_, cx| assert_eq!(group.read(cx).selected_id(cx).as_deref(), Some("middle")));
        }
        let disabled = cx.debug_bounds("segment-label-1").unwrap();
        cx.simulate_click(disabled.center(), Default::default());
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(group.read(cx).selected_id(cx).as_deref(), Some("middle")));
        let last = cx.debug_bounds("segment-label-3").unwrap();
        cx.simulate_click(last.center(), Default::default());
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(group.read(cx).selected_id(cx).as_deref(), Some("last")));
    }
}
