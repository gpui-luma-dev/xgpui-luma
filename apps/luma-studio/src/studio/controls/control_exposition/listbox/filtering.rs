//! Host-owned query/sort, SDK-owned projection and selection.
use xgpui_luma::controls::textfield::TextFieldEventExt;
use xgpui_luma::controls::selector::SelectorEventExt;
use std::sync::Arc;
use gpui::{Context, Div, Entity, Render, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::{
    listbox::{
        ListBoxControl, ListBoxInput, ListBoxItemRenderModel, ListBoxState, ListBoxVirtualization, SelectionMode,
        SelectionPolicy,
    },
    selector::{Selector, SelectorEvent, SelectorItem},
    textfield::{TextField, TextFieldEvent},
};
use xgpui_luma::{hstack, vstack};
use xgpui_luma_look_shadcn::{self as shadcn, LumaTypographyExt, ShadcnLook, ShadcnTextSize};
use super::{
    VERTICAL_LIST_WIDTH,
    markup::listbox,
    presentation::{ExamplePresentation, SelectionMark},
};
use super::super::event_stream::ControlEventStream;

const ID: &str = "listbox-filtering";

struct Document {
    id: usize,
    name: SharedString,
    kind: &'static str,
}

impl Document {
    fn samples() -> impl Iterator<Item = Self> {
        [
            ("Release notes", "Writing"),
            ("Color palette", "Design"),
            ("API reference", "Engineering"),
            ("Welcome guide", "Writing"),
            ("Icon library", "Design"),
            ("Keyboard shortcuts", "Engineering"),
            ("Layout studies", "Design"),
            ("Testing checklist", "Engineering"),
            ("Getting started", "Writing"),
            ("Typography", "Design"),
            ("Migration guide", "Writing"),
            ("Performance notes", "Engineering"),
        ]
        .into_iter()
        .enumerate()
        .map(|(id, (name, kind))| Self { id, name: name.into(), kind })
    }

    fn matches(&self, query: &str) -> bool {
        self.name.to_lowercase().contains(query) || self.kind.to_lowercase().contains(query)
    }
}

fn document_template(model: &ListBoxItemRenderModel<'_, Document>, look: &ShadcnLook) -> Div {
    hstack! { gap=6.0 align=center;
        SelectionMark { selected: model.selected },
        vstack! { gap=2.0;
            div().typography_style(look.typography_scale(ShadcnTextSize::Sm)).truncate().child(model.item.name.clone()),
            div().typography_style(look.typography_scale(ShadcnTextSize::Xs))
                .text_color(look.chrome().muted_text).child(model.item.kind),
        }.flex_1().min_w(px(0.0)),
    }
    .size_full()
    .px(px(8.0))
}

pub(super) struct FilteringExample {
    presentation: ExamplePresentation,
    list: ListBoxControl<Self, Document, usize>,
    search: TextField,
    sort: Entity<Selector>,
    query: String,
    sort_order: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl FilteringExample {
    pub(super) fn captured_motion(&self) -> (bool, xgpui_luma::controls::listbox::ListBoxMotion) {
        (self.list.is_motion_enabled(), self.list.motion_profile())
    }

    pub(super) fn new(look: Arc<ShadcnLook>, events: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        let search = shadcn::TextField::new("listbox-filter-query")
            .look(&look)
            .placeholder("Filter title or category…")
            .full_width(true)
            .spawn(cx);
        let sort = shadcn::Selector::new("listbox-filter-sort")
            .look(&look)
            .label("Display order")
            .items([
                SelectorItem::new("source").label("Source order"),
                SelectorItem::new("ascending").label("Title A–Z"),
                SelectorItem::new("descending").label("Title Z–A"),
            ])
            .selected_id("source")
            .spawn(cx);
        let subscriptions = {
            search.clone().on_event(cx, |this, event, cx| {
                if let TextFieldEvent::Change { value } = event {
                    this.query = value.trim().to_lowercase();
                    this.update_projection(cx);
                }
            });
            sort.clone().on_event(cx, |this, event, cx| {
                if let SelectorEvent::Change { item_id, .. } = event {
                    this.sort_order = item_id.clone();
                    this.update_projection(cx);
                }
            });
            Vec::new()
        };
        let mut state = ListBoxState::try_new(Document::samples(), |item| item.id, SelectionMode::Extended)
            .expect("sample documents have unique keys");
        state.set_selection_policy(super::selection_controls::DEFAULT_POLICY);
        Self {
            presentation: ExamplePresentation::new(look, "Filtering and sorting", events),
            list: ListBoxControl::new(state, Self::handle_input, |key| (ID, *key).into(), |item| item.name.clone(), cx)
                .require_focus_for_scroll(true)
                .virtualization(ListBoxVirtualization::Uniform { overscan: 2 }),
            search,
            sort,
            query: String::new(),
            sort_order: "source".into(),
            _subscriptions: subscriptions,
        }
    }

    fn update_projection(&mut self, cx: &mut Context<Self>) {
        let mut matches: Vec<_> =
            self.list.state.snapshot().items().iter().filter(|item| item.matches(&self.query)).collect();
        match self.sort_order.as_ref() {
            "ascending" => matches.sort_by(|a, b| a.name.cmp(&b.name)),
            "descending" => matches.sort_by(|a, b| b.name.cmp(&a.name)),
            _ => {}
        }
        let keys: Vec<_> = matches.into_iter().map(|item| item.id).collect();
        match self.list.set_projection(keys, cx) {
            Ok(update) => self.presentation.record(&update.events, cx),
            Err(error) => self.presentation.record_message(&error.to_string(), cx),
        }
    }

    fn handle_input(&mut self, input: ListBoxInput<usize>, _: &mut Window, cx: &mut Context<Self>) {
        let update = self.list.apply(input, cx);
        self.presentation.record(&update.events, cx);
    }

    pub(super) fn set_selection_policy(&mut self, policy: SelectionPolicy, cx: &mut Context<Self>) {
        let update = self.list.set_selection_policy(policy, cx);
        self.presentation.record(&update.events, cx);
    }

    pub(super) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.presentation.sync_look(look, cx);
        self.search.update(cx, |_, cx| cx.notify());
        self.sort.update(cx, |_, cx| cx.notify());
    }
}

impl Render for FilteringExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let look = &self.presentation.look;
        let surface = listbox! { window, cx;
            id = ID;
            control = &mut self.list;
            look = look;
            aria_label = "Filtered documents";
            width = VERTICAL_LIST_WIDTH;
            padding_x = 8.0;
            padding_y = 8.0;
            scroll_view! { vertical;
                visible_items = 5;
                vstack! {
                    gap = 4.0;
                    item_height = 56.0;
                    item_template = |model, _cx| document_template(model, look);
                }
            }
        };
        let state = &self.list.state;
        let visible = state.visible_items().len();
        let selected = state.selected_keys().count();
        let hidden = state.selected_keys().filter(|key| state.visible_index(key).is_none()).count();
        let summary = if visible == 0 {
            format!("No matches · {} source documents", state.snapshot().items().len())
        } else {
            format!("Showing {visible} of {} documents", state.snapshot().items().len())
        };
        vstack! { gap=8.0;
            self.search.clone(), self.sort.clone(),
            self.presentation.section(selected, surface),
            div().debug_selector(|| "filtering-summary".to_owned()).child(summary),
            div().child(format!("Selected: {selected} ({hidden} hidden)")),
            div().child("Try “design”. Hidden selections remain checked when you clear the filter. Shift selects in displayed order."),
        }.w(px(VERTICAL_LIST_WIDTH)).flex_shrink_0()
            .typography_style(look.typography_scale(ShadcnTextSize::Xs)).text_color(look.chrome().muted_text)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{TestAppContext, VisualTestContext};

    fn settle(cx: &mut VisualTestContext) {
        for _ in 0..8 {
            cx.run_until_parked();
            if cx.update(|window, app| window.simulate_next_frame(app)) == 0 {
                break;
            }
        }
        cx.run_until_parked();
    }

    #[test]
    fn filtering_fixture_subscriptions_preserve_selection_and_source_data() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(ShadcnLook::built_in());
            let events = cx.new(|cx| ControlEventStream::new(cx, look.clone(), "filter-events", ""));
            FilteringExample::new(look, events, cx)
        });
        settle(cx);
        assert_eq!(cx.debug_bounds("listbox-filtering-surface").unwrap().size.width, px(250.0));
        assert_eq!(cx.debug_bounds("listbox-filtering-viewport").unwrap().size.height, px(296.0));
        cx.update(|window, _| window.activate_window());
        let first = cx.debug_bounds("listbox-filtering-item-0").unwrap();
        cx.simulate_click(first.center(), Default::default());
        settle(cx);
        cx.update(|_, app| assert_eq!(view.read(app).list.state.selected_key(), Some(&0)));
        cx.simulate_click(first.center(), Default::default());
        settle(cx);
        cx.update(|_, app| assert_eq!(view.read(app).list.state.selected_keys().count(), 0));
        cx.update(|_, app| {
            view.update(app, |this, cx| {
                this.list.apply(ListBoxInput::Select(0), cx);
            });
            view.read(app).search.clone().update(app, |field, cx| {
                field.set_value("DESIGN", cx);
                cx.emit(TextFieldEvent::Change { value: "DESIGN".into() });
            });
        });
        settle(cx);
        cx.update(|_, app| {
            let state = &view.read(app).list.state;
            assert_eq!(state.visible_items().map(|item| item.key).collect::<Vec<_>>(), vec![1, 4, 6, 9]);
            assert_eq!(state.selected_key(), Some(&0));
            assert_eq!(state.visible_index(&0), None);
            assert_eq!(state.snapshot().items().len(), 12);
            view.read(app).sort.clone().update(app, |_, cx| {
                cx.emit(SelectorEvent::Change { item_id: "descending".into(), label: "Title Z–A".into() })
            });
        });
        settle(cx);
        cx.update(|_, app| {
            assert_eq!(
                view.read(app).list.state.visible_items().map(|item| item.key).collect::<Vec<_>>(),
                vec![9, 6, 4, 1]
            );
            view.read(app).search.clone().update(app, |field, cx| {
                field.set_value("no matches here", cx);
                cx.emit(TextFieldEvent::Change { value: "no matches here".into() });
            });
        });
        settle(cx);
        assert!(cx.debug_bounds("listbox-filtering-item-0").is_none());
        cx.update(|_, app| {
            let state = &view.read(app).list.state;
            assert_eq!(state.visible_items().len(), 0);
            assert_eq!(state.active_key(), None);
            assert_eq!(state.selected_key(), Some(&0));
            view.read(app).search.clone().update(app, |field, cx| {
                field.set_value("", cx);
                cx.emit(TextFieldEvent::Change { value: "".into() });
            });
        });
        settle(cx);
        cx.update(|_, app| {
            let state = &view.read(app).list.state;
            assert_eq!(state.visible_items().len(), 12);
            assert!(state.visible_items().find(|item| item.key == 0).unwrap().state.selected);
        });
    }
}
