//! The paging table writes a selection summary only when given one.
use super::*;
use gpui::{TestAppContext, div};

struct Pair(PagingTable<u8>, PagingTable<u8>);
impl Render for Pair {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.0.clone()).child(self.1.clone())
    }
}

fn rows(id: &'static str) -> TableBuilder<u8> {
    TableBuilder::<u8>::new_typed(id)
        .items([1, 2, 3])
        .selection_mode(crate::controls::table::TableSelectionMode::Single)
        .selected_index(1)
}

#[test]
fn selection_summary_only_when_given() {
    let mut app = TestAppContext::single();
    let (pair, cx) = app.add_window_view(|_, cx| {
        let bare = PagingTableBuilder::new(rows("bare"), PagerBuilder::new("bare-pager")).spawn(cx);
        let summarized = PagingTableBuilder::new(rows("summary"), PagerBuilder::new("summary-pager"))
            .selection_summary(|selected, total| format!("{selected} sur {total}").into())
            .spawn(cx);
        Pair(bare, summarized)
    });
    cx.run_until_parked();
    cx.update(|_, app| {
        let bare = pair.read(app).0.read(app).pager.read(app);
        assert!(bare.info_text_for_test().is_none());
        assert!(bare.page_size_label_for_test().is_none());
        let summarized = pair.read(app).1.read(app).pager.read(app);
        assert_eq!(summarized.info_text_for_test().unwrap(), "1 sur 3");
    });
}
