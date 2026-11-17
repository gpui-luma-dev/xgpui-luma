//! Native SDK collection controls. Row and cell rendering stays in the SDK.
use std::sync::Arc;
use gpui::{AnyView, Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{pager, table, tree_view};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};
use super::{
    event_stream::ControlEventStream,
    sample_layout::{card, page},
};

#[derive(Clone, Copy)]
pub(super) enum CollectionKind {
    PagingTable,
    ScrollingTable,
    TreeView,
}
impl CollectionKind {
    fn title(self) -> &'static str {
        match self {
            Self::PagingTable => "Paging Table",
            Self::ScrollingTable => "Scrolling Table",
            Self::TreeView => "Tree View",
        }
    }
    fn description(self) -> &'static str {
        match self {
            Self::PagingTable => "SDK PagingTableControl with its built-in table and pager.",
            Self::ScrollingTable => "SDK TableControl using its native smooth scrolling mode.",
            Self::TreeView => "Expand branches and select nodes in the SDK TreeViewControl.",
        }
    }
}
struct Row {
    id: usize,
    name: String,
    category: &'static str,
}
fn rows() -> Vec<Row> {
    (1..=48)
        .map(|id| Row { id, name: format!("Item {id:02}"), category: if id % 2 == 0 { "Document" } else { "Image" } })
        .collect()
}
fn table_builder(id: &'static str) -> table::TableBuilder<Row> {
    table::TableBuilder::new_typed(id).items(rows()).row_label(|row: &Row| row.name.clone()).grid_view([
        table::TableColumn::fixed("ID", 64., table::default_numeric_column_template(|row: &Row| row.id.to_string())),
        table::TableColumn::fill("Name", table::default_text_column_template(|row: &Row| row.name.clone())),
        table::TableColumn::fixed("Category", 140., table::default_text_column_template(|row: &Row| row.category)),
    ])
}
enum Control {
    Paging(table::PagingTable<Row>),
    Scrolling(table::Table<Row>),
    Tree(tree_view::TreeView<()>),
}
impl Control {
    fn view(&self) -> AnyView {
        match self {
            Self::Paging(v) => v.clone().into(),
            Self::Scrolling(v) => v.clone().into(),
            Self::Tree(v) => v.clone().into(),
        }
    }
    fn state(&self, cx: &gpui::App) -> crate::l10n::UiCopy {
        let table = match self {
            Self::Paging(v) => Some(v.read(cx).list()),
            Self::Scrolling(v) => Some(v),
            Self::Tree(_) => None,
        };
        let (page, count, mut selected) = if let Some(table) = table {
            let table = table.read(cx);
            (
                table.page_size().map(|_| (table.current_page() + 1, table.page_count())),
                table.items().len(),
                table.selected_items().iter().map(|row| row.id.to_string()).collect::<Vec<_>>(),
            )
        } else if let Self::Tree(tree) = self {
            (None, 0, tree.read(cx).selected_ids().iter().map(ToString::to_string).collect::<Vec<_>>())
        } else {
            (None, 0, Vec::new())
        };
        selected.sort();
        let tree = matches!(self, Self::Tree(_));
        crate::l10n::UiCopy::deferred(move |cx| {
            let selection = if selected.is_empty() {
                crate::l10n::UiCopy::state("none").resolve(cx).to_string()
            } else {
                cx.try_global::<xgpui_luma_l10n::Localizer>()
                    .and_then(|localizer| {
                        localizer.locale().fmt_list(&selected, xgpui_luma_l10n::formatting::ListKind::And).ok()
                    })
                    .unwrap_or_else(|| selected.join(", "))
            };
            match page {
                Some((page, pages)) => crate::l10n::messages::msg_lab_page_selected(
                    crate::l10n::number(&page.to_string(), cx).to_string(),
                    crate::l10n::number(&pages.to_string(), cx).to_string(),
                    selection,
                ),
                None if tree => crate::l10n::messages::msg_lab_selected(selection),
                None => crate::l10n::messages::msg_lab_rows_selected(
                    count as f64,
                    crate::l10n::number(&count.to_string(), cx).to_string(),
                    selection,
                ),
            }
        })
    }
}
pub(super) struct CollectionSample {
    kind: CollectionKind,
    look: Arc<Look>,
    control: Control,
    events: Entity<ControlEventStream>,
}
impl CollectionSample {
    pub(super) fn new(kind: CollectionKind, look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let control = match kind {
            CollectionKind::PagingTable => Control::Paging(
                table::PagingTableBuilder::new(
                    table_builder("workshop-sdk-paging-table").paged(8).visible_rows(8),
                    pager::PagerBuilder::new("workshop-sdk-table-pager").page_size_options([8, 16, 24]),
                )
                .spawn(cx),
            ),
            CollectionKind::ScrollingTable => Control::Scrolling(
                table_builder("workshop-sdk-scrolling-table")
                    .scroll_mode(table::TableScrollMode::ScrollSmooth)
                    .visible_rows(8)
                    .spawn(cx),
            ),
            CollectionKind::TreeView => Control::Tree(
                tree_view::TreeViewBuilder::new("workshop-sdk-tree")
                    .items([
                        tree_view::TreeNode::new("documents", "Documents", ())
                            .children([
                                tree_view::TreeNode::new("notes", "Notes.txt", ()),
                                tree_view::TreeNode::new("plans", "Plans", ()).children([
                                    tree_view::TreeNode::new("draft", "Draft.txt", ()),
                                    tree_view::TreeNode::new("final", "Final.txt", ()),
                                ]),
                            ])
                            .expanded(true),
                        tree_view::TreeNode::new("images", "Images", ()).children([tree_view::TreeNode::new(
                            "photo",
                            "Photo.png",
                            (),
                        )]),
                    ])
                    .spawn(cx),
            ),
        };
        match &control {
            Control::Paging(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &table::TableEvent, cx| this.table_event(event, cx))
                    .detach();
            }
            Control::Scrolling(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &table::TableEvent, cx| this.table_event(event, cx))
                    .detach();
            }
            Control::Tree(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &tree_view::TreeViewEvent<()>, cx| {
                    if matches!(
                        event,
                        tree_view::TreeViewEvent::RowHoverChanged { .. }
                            | tree_view::TreeViewEvent::FocusChanged { .. }
                    ) {
                        return;
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
                "workshop-collection-events",
                "Interact with the SDK collection to see its events.",
            )
        });
        Self { kind, look, control, events }
    }
    fn table_event(&mut self, event: &table::TableEvent, cx: &mut Context<Self>) {
        if matches!(event, table::TableEvent::RowHoverChanged { .. } | table::TableEvent::FocusChanged { .. }) {
            return;
        }
        self.record(event, cx);
    }
    fn record(&mut self, event: &impl std::fmt::Debug, cx: &mut Context<Self>) {
        let line = format!("{event:?}");
        self.events.update(cx, |events, cx| events.append_line(&line, cx));
        cx.notify();
    }
    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for CollectionSample {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let preview = if matches!(self.control, Control::Tree(_)) {
            div().w_full().h(px(280.)).child(self.control.view()).into_any_element()
        } else {
            self.control.view().into_any_element()
        };
        page(
            "workshop-collection-page",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(card(
                    &self.look,
                    "workshop-collection-example",
                    self.kind.title(),
                    self.kind.description(),
                    900.,
                    div().w_full().flex().flex_col().gap(px(16.)).child(preview).child(
                        div()
                            .typography_style(self.look.typography_scale(TextSize::Xs))
                            .text_color(self.look.chrome().muted_text)
                            .child(crate::l10n::ui(self.control.state(cx))),
                    ),
                ))
                .child(self.events.clone()),
        )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext, VisualTestContext};
    use crate::workshop::{Workshop, Sample};
    fn setup(app: &mut TestAppContext) -> (Entity<Workshop>, &mut VisualTestContext) {
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        })
    }
    fn select(workshop: &Entity<Workshop>, id: &'static str, cx: &mut VisualTestContext) -> Entity<CollectionSample> {
        workshop.update(cx, |v, cx| v.select(id, cx));
        cx.run_until_parked();
        cx.update(|_, cx| match &workshop.read(cx).samples[id] {
            Sample::Collection(v) => v.clone(),
            _ => panic!("missing native collection sample"),
        })
    }
    #[test]
    fn scrolling_table_uses_sdk_viewport_and_retains_selection() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        let sample = select(&workshop, "scrolling-table", cx);
        let table = cx.update(|_, cx| match &sample.read(cx).control {
            Control::Scrolling(v) => v.clone(),
            _ => unreachable!(),
        });
        cx.update(|window, cx| table.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("end space");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(table.read(cx).scroll_mode(), table::TableScrollMode::ScrollSmooth);
            assert_eq!(table.read(cx).active_index(), Some(47));
            assert_eq!(table.read(cx).selected_indices(), [47]);
            assert_eq!(sample.read(cx).control.state(cx).test_display(cx), "48 rows · Selected IDs: 48");
        });
        workshop.update(cx, |v, cx| {
            v.select("button", cx);
            v.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
        });
        let retained = select(&workshop, "scrolling-table", cx);
        assert_eq!(sample, retained);
        cx.update(|_, cx| assert_eq!(table.read(cx).selected_indices(), [47]));
    }
    #[test]
    fn paging_table_native_pager_stays_in_sync_with_keyboard_navigation() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        let sample = select(&workshop, "paging-table", cx);
        let (table, pager) = cx.update(|_, cx| match &sample.read(cx).control {
            Control::Paging(v) => {
                let v = v.read(cx);
                (v.list().clone(), v.pager().clone())
            }
            _ => unreachable!(),
        });
        cx.update(|window, cx| table.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("end space");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(table.read(cx).current_page(), 5);
            assert_eq!(pager.read(cx).current_page(), 5);
            assert_eq!(pager.read(cx).page_count(), 6);
            assert_eq!(table.read(cx).selected_indices(), [47]);
            assert_eq!(sample.read(cx).control.state(cx).test_display(cx), "Page 6 of 6 · Selected IDs: 48");
        });
        cx.simulate_keystrokes("home space");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(table.read(cx).current_page(), 0);
            assert_eq!(pager.read(cx).current_page(), 0);
        });
        select(&workshop, "tree-view", cx);
        select(&workshop, "paging-table", cx);
        cx.update(|_, cx| assert_eq!(table.read(cx).selected_indices(), [0]));
    }
    #[test]
    fn tree_view_native_keyboard_expansion_and_selection_are_retained() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        let sample = select(&workshop, "tree-view", cx);
        let tree = cx.update(|_, cx| match &sample.read(cx).control {
            Control::Tree(v) => v.clone(),
            _ => unreachable!(),
        });
        cx.update(|window, cx| tree.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("home left");
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!tree.read(cx).is_expanded(&"documents".into())));
        cx.simulate_keystrokes("right");
        cx.run_until_parked();
        cx.update(|_, cx| assert!(tree.read(cx).is_expanded(&"documents".into())));
        cx.simulate_keystrokes("end right end space");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert!(tree.read(cx).is_expanded(&"images".into()));
            assert!(tree.read(cx).selected_ids().contains("photo"));
            assert_eq!(sample.read(cx).control.state(cx).test_display(cx), "Selected: photo");
        });
        select(&workshop, "scrolling-table", cx);
        let retained = select(&workshop, "tree-view", cx);
        assert_eq!(sample, retained);
        cx.update(|_, cx| assert!(tree.read(cx).selected_ids().contains("photo")));
    }
    #[test]
    fn listbox_without_sdk_default_renderer_shows_message_instead_of_substitute() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        workshop.update(cx, |v, cx| v.select("listbox", cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-sdk-unavailable").is_some());
        cx.update(|_, cx| {
            assert_eq!(workshop.read(cx).selected, "listbox");
            assert!(!workshop.read(cx).samples.contains_key("listbox"));
        });
    }
}
