//! Placeholders and "No matches" follow `LumaStrings`; app text stays fixed.
use super::*;
use crate::infra::strings::{set_luma_strings, tests::french};
use gpui::{TestAppContext, div, prelude::*};

struct Trio(Entity<SearchSelectorControl>, Entity<SearchSelectorControl>, Entity<SearchSelectorControl>);
impl Render for Trio {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.0.clone()).child(self.1.clone()).child(self.2.clone())
    }
}

#[test]
fn placeholders_follow_luma_strings_unless_the_app_set_them() {
    let mut app = TestAppContext::single();
    let (trio, cx) = app.add_window_view(|_, cx| {
        let built_in = cx.new(|cx| SearchSelectorControl::from_builder(SearchSelectorBuilder::new("built-in", []), cx));
        let builder_text = cx.new(|cx| {
            SearchSelectorControl::from_builder(
                SearchSelectorBuilder::new("builder", []).placeholder("Pick one").search_placeholder("Find"),
                cx,
            )
        });
        let setter_text =
            cx.new(|cx| SearchSelectorControl::from_builder(SearchSelectorBuilder::new("setter", []), cx));
        Trio(built_in, builder_text, setter_text)
    });
    cx.run_until_parked();
    trio.update(cx, |trio, cx| trio.2.update(cx, |control, cx| control.set_placeholder("Later", cx)));
    cx.update(|_, app| set_luma_strings(french(), app));
    cx.run_until_parked();
    cx.update(|_, app| {
        let built_in = trio.read(app).0.read(app);
        assert_eq!(built_in.model.placeholder, "Choisir…");
        assert_eq!(built_in.model.search_placeholder, "Rechercher dans la sélection");
        assert_eq!(built_in.popup_search_textfield.read(app).placeholder_for_test(), "Rechercher dans la sélection");
        let builder_text = trio.read(app).1.read(app);
        assert_eq!(builder_text.model.placeholder, "Pick one");
        assert_eq!(builder_text.popup_search_textfield.read(app).placeholder_for_test(), "Find");
        assert_eq!(trio.read(app).2.read(app).model.placeholder, "Later");
    });
}
