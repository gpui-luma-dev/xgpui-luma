//! The paging table's selection summary follows `LumaStrings`.
use super::*;
use crate::infra::strings::{set_luma_strings, tests::french};
use gpui::TestAppContext;

#[test]
fn selection_summary_follows_luma_strings() {
    let mut app = TestAppContext::single();
    let (table, cx) = app.add_window_view(|_, cx| {
        let builder = PagingTableBuilder::new(TableBuilder::<()>::new_typed("rows"), PagerBuilder::new("pager"));
        let list = builder.list_builder.spawn(cx);
        let pager = builder.pager_builder.spawn(cx);
        PagingTableControl::new(list, pager, cx)
    });
    cx.run_until_parked();
    let pager = cx.update(|_, app| table.read(app).pager.clone());
    cx.update(|_, app| assert_eq!(pager.read(app).info_text_for_test().unwrap(), "0 of 0 row(s) selected."));
    cx.update(|_, app| set_luma_strings(french(), app));
    cx.run_until_parked();
    cx.update(|_, app| assert_eq!(pager.read(app).info_text_for_test().unwrap(), "0 sur 0 ligne(s)"));
}
