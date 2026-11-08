//! A large, concrete collection with a richer item template for virtualization testing.
use xgpui_luma::color::gpui_bridge::from_hsla;
use std::{cell::Cell, ops::Range, sync::Arc};

use gpui::{Context, Div, Entity, Hsla, Render, SharedString, Window, div, hsla, prelude::*, px};
use xgpui_luma::controls::listbox::{
    ListBoxControl, ListBoxInput, ListBoxItemRenderModel, ListBoxState, ListBoxVirtualization, SelectionMode,
    SelectionPolicy,
};
use xgpui_luma::{hstack, vstack};
use xgpui_luma_color::ColorSwatch;
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::{
    VERTICAL_LIST_WIDTH,
    markup::listbox,
    presentation::{ExamplePresentation, SelectionMark},
};
use super::super::event_stream::ControlEventStream;

const ID: &str = "listbox-spectrum";
const ITEM_COUNT: u32 = 10_000;

#[derive(Default)]
struct RenderStats {
    total: usize,
    visible: Range<usize>,
    constructed: Range<usize>,
    template_calls: usize,
}

impl RenderStats {
    fn render(&self, look: &ShadcnLook) -> Div {
        let range_label = |range: &Range<usize>| {
            if range.is_empty() {
                "—".to_owned()
            } else {
                format!("{}–{}", range.start + 1, range.end)
            }
        };
        vstack! { gap=2.0; }
            .debug_selector(|| "spectrum-render-stats".to_owned())
            .typography_style(look.typography_scale(ShadcnTextSize::Xs))
            .text_color(look.chrome().muted_text)
            .child(format!("Collection: {} items", self.total))
            .child(format!("Visible: {} ({} rows)", range_label(&self.visible), self.visible.len()))
            .child(format!("Constructed: {} ({} rows)", range_label(&self.constructed), self.constructed.len()))
            .child(format!("Template calls: {} this render", self.template_calls))
            .child("Visible includes partial rows. Constructed includes the buffer.")
    }
}

struct SpectrumItem {
    id: u32,
    label: SharedString,
    detail: SharedString,
    colors: [Hsla; 5],
}

impl SpectrumItem {
    fn new(id: u32) -> Self {
        let hue = ((id - 1) % 360) as f32 / 360.0;
        let saturation = 0.55 + ((id - 1) / 360 % 5) as f32 * 0.1;
        Self {
            id,
            label: format!("Color {id:05}").into(),
            detail: format!("H {:.0}° · S {:.0}%", hue * 360.0, saturation * 100.0).into(),
            colors: [-0.08, -0.04, 0.0, 0.04, 0.08]
                .map(|offset| hsla((hue + offset).rem_euclid(1.0), saturation, 0.55, 1.0)),
        }
    }
}

/// Content only: the SDK and look supply selection, focus, hover, and scrolling.
fn spectrum_item_template(model: &ListBoxItemRenderModel<'_, SpectrumItem>, look: &ShadcnLook) -> Div {
    let item = model.item;
    let swatch =
        hstack! {}
            .w(px(50.0))
            .flex_shrink_0()
            .rounded(px(4.0))
            .overflow_hidden()
            .children(item.colors.map(|color| {
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .child(ColorSwatch::new(from_hsla(color)).height(px(30.0)).rounded(px(0.0)).bordered(false))
            }));
    hstack! { gap=6.0 align=center;
        SelectionMark { selected: model.selected },
        swatch,
        vstack! { gap=2.0;
            div().typography_style(look.typography_scale(ShadcnTextSize::Sm)).truncate().child(item.label.clone()),
            div().typography_style(look.typography_scale(ShadcnTextSize::Xs))
                .text_color(look.chrome().muted_text).truncate().child(item.detail.clone()),
        }.flex_1().min_w(px(0.0)),
    }
    .size_full()
    .px(px(8.0))
}

pub(super) struct SpectrumListExample {
    presentation: ExamplePresentation,
    list: ListBoxControl<Self, SpectrumItem, u32>,
    stats: RenderStats,
}

impl SpectrumListExample {
    pub(super) fn captured_motion(&self) -> (bool, xgpui_luma::controls::listbox::ListBoxMotion) {
        (self.list.is_motion_enabled(), self.list.motion_profile())
    }

    pub(super) fn new(look: Arc<ShadcnLook>, events: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        let mut state =
            ListBoxState::try_new((1..=ITEM_COUNT).map(SpectrumItem::new), |item| item.id, SelectionMode::Extended)
                .expect("spectrum items have unique numeric keys");
        state.set_selection_policy(super::selection_controls::DEFAULT_POLICY);
        Self {
            stats: RenderStats::default(),
            presentation: ExamplePresentation::new(look, "Spectrum", events),
            list: ListBoxControl::new(
                state,
                Self::handle_input,
                |key| (ID, *key).into(),
                |item| format!("{}, {}", item.label, item.detail).into(),
                cx,
            )
            .require_focus_for_scroll(true)
            .virtualization(ListBoxVirtualization::Uniform { overscan: 2 }),
        }
    }

    fn handle_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        let update = self.list.apply(input, cx);
        self.presentation.record(&update.events, cx);
    }

    pub(super) fn set_selection_policy(&mut self, policy: SelectionPolicy, cx: &mut Context<Self>) {
        let update = self.list.set_selection_policy(policy, cx);
        self.presentation.record(&update.events, cx);
    }

    pub(super) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.presentation.sync_look(look, cx);
    }
}

impl Render for SpectrumListExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let look = &self.presentation.look;
        let template_calls = Cell::new(0);
        let surface = listbox! { window, cx;
            id = ID;
            control = &mut self.list;
            look = look;
            aria_label = "Color spectrum, 10,000 items";
            width = VERTICAL_LIST_WIDTH;
            padding_x = 8.0;
            padding_y = 8.0;
            scroll_view! { vertical;
                visible_items = 5;
                vstack! {
                    gap = 4.0;
                    item_height = 56.0;
                    item_template = |model, _cx| {
                        template_calls.set(template_calls.get() + 1);
                        spectrum_item_template(model, look)
                    };
                }
            }
        };
        let rendered = self.list.render_parts().scroll.rendered_window();
        self.stats = RenderStats {
            total: self.list.state.snapshot().items().len(),
            visible: rendered.as_ref().map_or(0..0, |window| window.visible_range.clone()),
            constructed: rendered.map_or(0..0, |window| window.range),
            template_calls: template_calls.get(),
        };
        vstack! { gap=8.0;
            self.presentation.section(self.list.state.selected_keys().count(), surface),
            self.stats.render(look),
            div().typography_style(look.typography_scale(ShadcnTextSize::Xs))
                .text_color(look.chrome().muted_text)
                .child("10,000 items · 5 visible · Virtualized. Click to focus, then scroll or use Home / End. Shift extends selection."),
        }.w(px(VERTICAL_LIST_WIDTH)).flex_shrink_0()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{ScrollDelta, ScrollEvent, TestAppContext, TouchPhase, VisualTestContext, point};
    use super::*;

    fn settle(cx: &mut VisualTestContext) {
        for _ in 0..4 {
            cx.run_until_parked();
            if cx.update(|window, app| window.simulate_next_frame(app)) == 0 {
                break;
            }
        }
        cx.run_until_parked();
    }

    #[test]
    fn spectrum_example_virtualizes_rich_rows_and_reveals_both_ends() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(ShadcnLook::built_in());
            let events = cx.new(|cx| ControlEventStream::new(cx, look.clone(), "spectrum-test-events", ""));
            SpectrumListExample::new(look, events, cx)
        });
        cx.update(|window, _| window.activate_window());
        settle(cx);
        let surface = cx.debug_bounds("listbox-spectrum-surface").unwrap();
        let viewport = cx.debug_bounds("listbox-spectrum-viewport").unwrap();
        let first = cx.debug_bounds("listbox-spectrum-item-0").unwrap();
        assert_eq!(surface.size.width, px(VERTICAL_LIST_WIDTH));
        assert_eq!(viewport.size.height, px(296.0));
        assert_eq!(first.size.height, px(56.0));
        assert!(cx.debug_bounds("listbox-spectrum-item-9999").is_none());
        assert!(cx.debug_bounds("spectrum-render-stats").is_some());
        cx.update(|_, app| {
            let stats = &view.read(app).stats;
            assert_eq!(stats.total, 10_000);
            assert_eq!(stats.visible, 0..5);
            assert_eq!(stats.constructed, 0..7);
            assert_eq!(stats.template_calls, 7);
        });
        cx.simulate_click(point(first.left() + px(30.0), first.top() + px(25.0)), Default::default());
        cx.simulate_keystrokes("down");
        settle(cx);
        cx.update(|_, app| {
            let state = &view.read(app).list.state;
            assert_eq!(state.active_key(), Some(&2));
            assert_eq!(state.selected_keys().copied().collect::<Vec<_>>(), vec![1]);
        });
        cx.simulate_keystrokes("enter");
        settle(cx);
        cx.update(|_, app| {
            let state = &view.read(app).list.state;
            assert_eq!(state.selected_keys().copied().collect::<Vec<_>>(), vec![2]);
            assert!(state.visible_items().find(|item| item.key == 2).unwrap().state.selected);
        });
        let second = cx.debug_bounds("listbox-spectrum-item-1").unwrap();
        cx.simulate_click(second.center(), Default::default());
        settle(cx);
        cx.update(|_, app| assert_eq!(view.read(app).list.state.selected_keys().count(), 0));
        for (delta, visible, constructed) in [(-12_000.0, 200..205, 198..207), (-30.0, 200..206, 198..208)] {
            cx.simulate_event(ScrollEvent {
                position: point(first.left() + px(30.0), first.top() + px(25.0)),
                delta: ScrollDelta::Pixels(point(px(0.0), px(delta))),
                modifiers: Default::default(),
                source: gpui::ScrollSource::Pointer,
                touch_phase: TouchPhase::Moved,
            });
            settle(cx);
            cx.update(|_, app| {
                let stats = &view.read(app).stats;
                assert_eq!(stats.visible, visible);
                assert_eq!(stats.constructed, constructed);
                assert_eq!(stats.template_calls, constructed.len());
            });
        }
        cx.simulate_keystrokes("end");
        settle(cx);
        let last = cx.debug_bounds("listbox-spectrum-item-9999").unwrap();
        assert!(last.top() >= viewport.top() && last.bottom() <= viewport.bottom());
        assert!(cx.debug_bounds("listbox-spectrum-item-0").is_none());
        cx.update(|_, app| {
            let example = view.read(app);
            assert_eq!(example.list.state.snapshot().items().len(), 10_000);
            assert_eq!(example.list.state.active_key(), Some(&10_000));
            assert_eq!(example.stats.visible, 9_995..10_000);
            assert_eq!(example.stats.constructed, 9_993..10_000);
            assert_eq!(example.stats.template_calls, 7);
        });
        cx.simulate_keystrokes("home");
        settle(cx);
        assert_eq!(cx.debug_bounds("listbox-spectrum-item-0").unwrap().origin, first.origin);
        cx.update(|_, app| {
            let stats = &view.read(app).stats;
            assert_eq!(stats.visible, 0..5);
            assert_eq!(stats.constructed, 0..7);
            assert_eq!(stats.template_calls, 7);
        });
    }
}
