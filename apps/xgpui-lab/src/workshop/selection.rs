//! Native SDK selection controls, without app-owned item or control templates.
use std::sync::Arc;
use gpui::{AnyView, Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{
    autocomplete, combobox, search_selector, selector, selection_panel, split_button, context_menu::MenuItem,
};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};
use super::{
    event_stream::ControlEventStream,
    sample_layout::{card, page},
};

#[derive(Clone, Copy)]
pub(super) enum SelectionKind {
    Autocomplete,
    ComboBox,
    SearchSelector,
    Selector,
    Panel,
    SplitButton,
}
impl SelectionKind {
    fn title(self) -> &'static str {
        match self {
            Self::Autocomplete => "Autocomplete TextBox",
            Self::ComboBox => "ComboBox",
            Self::SearchSelector => "SearchSelector",
            Self::Selector => "Selector",
            Self::Panel => "Selection Panel",
            Self::SplitButton => "Split Button",
        }
    }
    fn description(self) -> &'static str {
        match self {
            Self::Autocomplete => "SDK Autocomplete. Type to filter suggestions; arrow keys and Enter choose an item.",
            Self::ComboBox => "SDK ComboBox. Type to filter or open the option list and choose an item.",
            Self::SearchSelector => "SDK SearchSelector. Open the popup, then search and select an item.",
            Self::Selector => "SDK Selector. Open the popup and choose an item with the pointer or keyboard.",
            Self::Panel => {
                "SDK SelectionPanel. An always-visible option list with native selection and keyboard navigation."
            }
            Self::SplitButton => {
                "SDK SplitButton. Enter or Space activates Apply; Down opens the independent disclosure menu."
            }
        }
    }
}
const ITEMS: [(&str, &str, bool); 5] = [
    ("apple", "Apple", true),
    ("apricot", "Apricot", true),
    ("unavailable", "Unavailable", false),
    ("banana", "Banana", true),
    ("cherry", "Cherry", true),
];
fn filter_items() -> Vec<autocomplete::SelectionItem> {
    ITEMS
        .iter()
        .map(|&(id, label, enabled)| autocomplete::SelectionItem::new(id, label).enabled(enabled))
        .collect()
}
enum Control {
    Autocomplete(autocomplete::Autocomplete),
    ComboBox(combobox::ComboBox),
    SearchSelector(search_selector::SearchSelector),
    Selector(Entity<selector::Selector>),
    Panel(Entity<selection_panel::SelectionPanelControl>),
    SplitButton(Entity<split_button::SplitButton>),
}
impl Control {
    fn view(&self) -> AnyView {
        match self {
            Self::Autocomplete(v) => v.clone().into(),
            Self::ComboBox(v) => v.clone().into(),
            Self::SearchSelector(v) => v.clone().into(),
            Self::Selector(v) => v.clone().into(),
            Self::Panel(v) => v.clone().into(),
            Self::SplitButton(v) => v.clone().into(),
        }
    }
}
pub(super) struct SelectionSample {
    kind: SelectionKind,
    look: Arc<Look>,
    control: Control,
    query: String,
    selected: Option<String>,
    action_count: usize,
    events: Entity<ControlEventStream>,
}
impl SelectionSample {
    pub(super) fn new(kind: SelectionKind, look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let control = match kind {
            SelectionKind::Autocomplete => Control::Autocomplete(
                autocomplete::AutocompleteBuilder::new("workshop-sdk-autocomplete", filter_items())
                    .placeholder(super::sdk_text::FILTER_PLACEHOLDER)
                    .spawn(cx),
            ),
            SelectionKind::ComboBox => Control::ComboBox(
                combobox::ComboBoxBuilder::new("workshop-sdk-combobox", filter_items())
                    .placeholder(super::sdk_text::SELECT_PLACEHOLDER)
                    .spawn(cx),
            ),
            SelectionKind::SearchSelector => Control::SearchSelector(
                search_selector::SearchSelectorBuilder::new("workshop-sdk-search-selector", filter_items())
                    .placeholder(super::sdk_text::SELECT_PLACEHOLDER)
                    .search_placeholder(super::sdk_text::SELECTION_SEARCH)
                    .no_matches(super::sdk_text::NO_MATCHES)
                    .spawn(cx),
            ),
            SelectionKind::Selector => Control::Selector(
                selector::SelectorBuilder::new("workshop-sdk-selector")
                    .label("Select fruit")
                    .items(
                        ITEMS
                            .iter()
                            .map(|&(id, label, enabled)| selector::SelectorItem::new(id).label(label).enabled(enabled)),
                    )
                    .spawn(cx),
            ),
            SelectionKind::Panel => Control::Panel(
                selection_panel::SelectionPanelBuilder::new("workshop-sdk-selection-panel")
                    .items(ITEMS.iter().map(|&(id, label, enabled)| {
                        selection_panel::SelectionPanelItem::new(id).label(label).enabled(enabled)
                    }))
                    .spawn(cx),
            ),
            SelectionKind::SplitButton => Control::SplitButton(
                split_button::SplitButtonBuilder::new("workshop-sdk-split-button")
                    .text(super::sdk_text::split_button())
                    .label("Apply")
                    .items(ITEMS.iter().map(|&(id, label, enabled)| MenuItem::new(id).label(label).enabled(enabled)))
                    .spawn(cx),
            ),
        };
        match &control {
            Control::Autocomplete(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &autocomplete::AutocompleteEvent, cx| {
                    match event {
                        autocomplete::AutocompleteEvent::Change { query } => this.query = query.to_string(),
                        autocomplete::AutocompleteEvent::Select { item_id, .. }
                        | autocomplete::AutocompleteEvent::Complete { item_id, .. } => {
                            this.selected = Some(item_id.to_string())
                        }
                        autocomplete::AutocompleteEvent::Clear => {
                            this.query.clear();
                            this.selected = None;
                        }
                        autocomplete::AutocompleteEvent::FocusChanged { .. } => return,
                        _ => {}
                    }
                    this.record(event, cx);
                })
                .detach();
            }
            Control::ComboBox(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &combobox::ComboBoxEvent, cx| {
                    match event {
                        combobox::ComboBoxEvent::Change { query } => this.query = query.to_string(),
                        combobox::ComboBoxEvent::Select { item_id, .. }
                        | combobox::ComboBoxEvent::Complete { item_id, .. } => {
                            this.selected = Some(item_id.to_string())
                        }
                        combobox::ComboBoxEvent::Clear => {
                            this.query.clear();
                            this.selected = None;
                        }
                        combobox::ComboBoxEvent::FocusChanged { .. } => return,
                        _ => {}
                    }
                    this.record(event, cx);
                })
                .detach();
            }
            Control::SearchSelector(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &search_selector::SearchSelectorEvent, cx| {
                    match event {
                        search_selector::SearchSelectorEvent::Change { query } => this.query = query.to_string(),
                        search_selector::SearchSelectorEvent::Select { item_id, .. }
                        | search_selector::SearchSelectorEvent::Complete { item_id, .. } => {
                            this.selected = Some(item_id.to_string())
                        }
                        search_selector::SearchSelectorEvent::Clear => {
                            this.query.clear();
                            this.selected = None;
                        }
                        search_selector::SearchSelectorEvent::FocusChanged { .. } => return,
                        _ => {}
                    }
                    this.record(event, cx);
                })
                .detach();
            }
            Control::Selector(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &selector::SelectorEvent, cx| {
                    match event {
                        selector::SelectorEvent::Change { item_id, .. } => this.selected = Some(item_id.to_string()),
                        selector::SelectorEvent::FocusChanged { .. } => return,
                        _ => {}
                    }
                    this.record(event, cx);
                })
                .detach();
            }
            Control::Panel(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &selection_panel::SelectionPanelEvent, cx| {
                    match event {
                        selection_panel::SelectionPanelEvent::ActivateRow { item_id, .. } => {
                            this.selected = Some(item_id.to_string())
                        }
                        selection_panel::SelectionPanelEvent::HoverChanged { .. }
                        | selection_panel::SelectionPanelEvent::FocusChanged { .. } => return,
                        _ => {}
                    }
                    this.record(event, cx);
                })
                .detach();
            }
            Control::SplitButton(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &split_button::SplitButtonEvent, cx| {
                    match event {
                        split_button::SplitButtonEvent::ActionClick => this.action_count += 1,
                        split_button::SplitButtonEvent::Select { item_id, .. } => {
                            this.selected = Some(item_id.to_string())
                        }
                        split_button::SplitButtonEvent::HoverChanged { .. }
                        | split_button::SplitButtonEvent::FocusChanged { .. } => return,
                        _ => {}
                    }
                    this.record(event, cx);
                })
                .detach();
            }
        }
        let events = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "workshop-selection-events",
                "Interact with the native SDK selection control to see its events.",
            )
        });
        Self { kind, look, control, query: String::new(), selected: None, action_count: 0, events }
    }
    fn record(&mut self, event: &impl std::fmt::Debug, cx: &mut Context<Self>) {
        self.events.update(cx, |events, cx| events.append_line(&format!("{event:?}"), cx));
        cx.notify();
    }
    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for SelectionSample {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let state = match self.kind {
            SelectionKind::Autocomplete | SelectionKind::ComboBox | SelectionKind::SearchSelector => {
                format!("Last query: {:?} · Last selection: {}", self.query, self.selected.as_deref().unwrap_or("none"))
            }
            SelectionKind::SplitButton => format!(
                "Apply actions: {} · Last menu selection: {}",
                self.action_count,
                self.selected.as_deref().unwrap_or("none")
            ),
            _ => format!("Selected: {}", self.selected.as_deref().unwrap_or("none")),
        };
        page(
            "workshop-selection-page",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(card(
                    &self.look,
                    "workshop-selection-example",
                    self.kind.title(),
                    self.kind.description(),
                    700.,
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(16.))
                        .child(
                            div()
                                .debug_selector(|| "workshop-selection-viewport".into())
                                .w_full()
                                .child(self.control.view()),
                        )
                        .child(
                            div()
                                .typography_style(self.look.typography_scale(TextSize::Xs))
                                .text_color(self.look.chrome().muted_text)
                                .child(state),
                        ),
                ))
                .child(self.events.clone()),
        )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext, VisualTestContext, point};
    use crate::workshop::{Sample, Workshop};
    fn setup(app: &mut TestAppContext) -> (Entity<Workshop>, &mut VisualTestContext) {
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            // Workshop is mounted without the app's enclosing LumaFocus scope.
            cx.bind_keys([gpui::KeyBinding::new("escape", xgpui_luma::focus::EscapeFocus, None)]);
        });
        app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        })
    }
    fn select(workshop: &Entity<Workshop>, id: &'static str, cx: &mut VisualTestContext) -> Entity<SelectionSample> {
        workshop.update(cx, |v, cx| v.select(id, cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let Sample::Selection(v) = &workshop.read(cx).samples[id] else {
                panic!("missing SDK selection sample")
            };
            v.clone()
        })
    }
    fn assert_retained(
        workshop: &Entity<Workshop>,
        sample: &Entity<SelectionSample>,
        id: &'static str,
        expected: &str,
        cx: &mut VisualTestContext,
    ) {
        workshop.update(cx, |v, cx| {
            v.select("button", cx);
            v.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
        });
        assert_eq!(select(workshop, id, cx), *sample);
        cx.update(|_, cx| assert_eq!(sample.read(cx).selected.as_deref(), Some(expected)));
    }
    #[test]
    fn sdk_filterable_controls_search_commit_and_retain_selection() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        for id in ["autocomplete-textfield", "combobox", "search-selector"] {
            let sample = select(&workshop, id, cx);
            let bounds = cx.debug_bounds("workshop-selection-viewport").unwrap();
            cx.simulate_click(point(bounds.left() + px(25.), bounds.bottom() - px(15.)), Default::default());
            cx.run_until_parked();
            cx.simulate_input("ban");
            cx.run_until_parked();
            cx.update(|_, cx| assert_eq!(sample.read(cx).query, "ban", "{id} must use native search input"));
            cx.simulate_keystrokes("enter");
            cx.run_until_parked();
            cx.update(|_, cx| {
                assert_eq!(sample.read(cx).selected.as_deref(), Some("banana"), "{id} must commit its filtered result")
            });
            assert_retained(&workshop, &sample, id, "banana", cx);
        }
    }
    #[test]
    fn sdk_selector_and_panel_respect_disabled_rows_and_retain_native_selection() {
        for (id, keys) in
            [("popup-selector", "space home down down enter"), ("selection-panel", "home down down enter")]
        {
            let mut app = TestAppContext::single();
            let (workshop, cx) = setup(&mut app);
            let sample = select(&workshop, id, cx);
            cx.update(|window, cx| {
                let focus = match &sample.read(cx).control {
                    Control::Selector(v) => v.read(cx).focus_handle(cx),
                    Control::Panel(v) => v.read(cx).focus_handle(cx),
                    _ => unreachable!(),
                };
                focus.focus(window, cx);
            });
            cx.run_until_parked();
            if id == "selection-panel" {
                let bounds = cx.debug_bounds("workshop-selection-viewport").unwrap();
                cx.simulate_click(point(bounds.left() + px(25.), bounds.top() + px(15.)), Default::default());
            }
            cx.update(|window, cx| {
                if let Control::Panel(v) = &sample.read(cx).control {
                    assert!(v.read(cx).focus_handle(cx).is_focused(window), "native panel must own keyboard focus");
                }
            });
            cx.simulate_keystrokes(keys);
            cx.run_until_parked();
            if id == "selection-panel" {
                // The SDK panel allows a disabled row to become active, but rejects activation.
                cx.update(|_, cx| assert_eq!(sample.read(cx).selected.as_deref(), Some("apple")));
                cx.simulate_keystrokes("down enter");
                cx.run_until_parked();
            }
            cx.update(|_, cx| {
                assert_eq!(sample.read(cx).selected.as_deref(), Some("banana"), "{id} must select an enabled row")
            });
            assert_retained(&workshop, &sample, id, "banana", cx);
            cx.update(|_, cx| match &sample.read(cx).control {
                Control::Selector(v) => assert_eq!(v.read(cx).selected_id().map(|id| id.as_ref()), Some("banana")),
                Control::Panel(_) => {}
                _ => unreachable!(),
            });
            if id == "selection-panel" {
                cx.update(|window, cx| {
                    let Control::Panel(v) = &sample.read(cx).control else {
                        unreachable!()
                    };
                    v.read(cx).focus_handle(cx).focus(window, cx);
                });
                cx.run_until_parked();
                cx.simulate_keystrokes("up up enter");
                cx.run_until_parked();
                cx.update(|_, cx| assert_eq!(sample.read(cx).selected.as_deref(), Some("apricot")));
            }
        }
    }
    #[test]
    fn sdk_split_button_keeps_primary_action_and_disclosure_selection_independent() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        let sample = select(&workshop, "split-button", cx);
        cx.update(|window, cx| {
            let Control::SplitButton(v) = &sample.read(cx).control else {
                unreachable!()
            };
            v.read(cx).focus_handle(cx).focus(window, cx);
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("space down end enter");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(sample.read(cx).action_count, 1);
            assert_eq!(sample.read(cx).selected.as_deref(), Some("cherry"));
        });
        cx.simulate_keystrokes("down escape");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(sample.read(cx).action_count, 1));
        assert_retained(&workshop, &sample, "split-button", "cherry", cx);
        cx.update(|_, cx| assert_eq!(sample.read(cx).action_count, 1));
    }
}
