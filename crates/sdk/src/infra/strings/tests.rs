use std::sync::Arc;

use gpui::TestAppContext;

use super::{LumaStrings, set_luma_strings};

/// A complete non-English snapshot for control tests.
pub(crate) fn french() -> LumaStrings {
    LumaStrings {
        select_placeholder: "Choisir…".into(),
        selection_search: "Rechercher dans la sélection".into(),
        filter_placeholder: "Filtrer…".into(),
        no_matches: "Aucun résultat".into(),
        rows_per_page: "Lignes par page".into(),
        disabled: "Désactivé".into(),
        page_of: Arc::new(|page, pages| format!("Page {page} sur {pages}").into()),
        rows_selected: Arc::new(|selected, total| format!("{selected} sur {total} ligne(s)").into()),
    }
}

#[test]
fn defaults_are_the_sdk_english_text_on_main() {
    let strings = LumaStrings::default();
    assert_eq!(strings.select_placeholder, "Select…");
    assert_eq!(strings.selection_search, "Selection search");
    assert_eq!(strings.filter_placeholder, "Type to filter…");
    assert_eq!(strings.no_matches, "No matches");
    assert_eq!(strings.rows_per_page, "Rows per page");
    assert_eq!(strings.disabled, "Disabled");
    assert_eq!((strings.page_of)(2, 5), "Page 2 of 5");
    assert_eq!((strings.rows_selected)(1, 3), "1 of 3 row(s) selected.");
}

#[test]
fn current_falls_back_without_installing_and_reflects_replacement() {
    let app = TestAppContext::single();
    app.update(|cx| {
        assert_eq!(LumaStrings::current(cx).no_matches, "No matches");
        assert!(cx.try_global::<LumaStrings>().is_none());
        set_luma_strings(french(), cx);
        assert_eq!(LumaStrings::current(cx).no_matches, "Aucun résultat");
        assert_eq!((LumaStrings::current(cx).page_of)(2, 5), "Page 2 sur 5");
    });
}
