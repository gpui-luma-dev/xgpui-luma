//! The SDK ships no words of its own: bare controls start empty (the pager
//! uses a wordless "2 / 5"), and text reaches them only through their inputs.
use crate::controls::{accordion, autocomplete, combobox, pager, search_selector};

#[test]
fn bare_controls_carry_no_english() {
    assert!(combobox::new("combo", []).model.placeholder.is_empty());
    assert!(autocomplete::new("auto", []).model.placeholder.is_empty());
    let search = search_selector::new("search", []);
    assert!(search.model.placeholder.is_empty());
    assert!(search.model.search_placeholder.is_empty());
    assert!(search.model.no_matches.is_empty());
    assert!(accordion::new("accordion").model.drag_handle_label.is_empty());
    assert_eq!((pager::PagerTemplateParameters::default().page_indicator_formatter)(2, 5), "2 / 5");
}

#[test]
fn text_inputs_reach_the_controls() {
    assert_eq!(combobox::new("combo", []).placeholder("Choisir…").model.placeholder, "Choisir…");
    assert_eq!(autocomplete::new("auto", []).placeholder("Filtrer…").model.placeholder, "Filtrer…");
    let search = search_selector::new("search", [])
        .placeholder("Choisir…")
        .search_placeholder("Rechercher")
        .no_matches("Aucun");
    assert_eq!(search.model.placeholder, "Choisir…");
    assert_eq!(search.model.search_placeholder, "Rechercher");
    assert_eq!(search.model.no_matches, "Aucun");
    assert_eq!(accordion::new("accordion").drag_handle_label("Glisser").model.drag_handle_label, "Glisser");
}
