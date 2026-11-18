//! The default placeholder follows `LumaStrings`; app text stays fixed.
use super::*;
use crate::infra::strings::{set_luma_strings, tests::french};
use gpui::{TestAppContext, div, prelude::*};

struct Pair(Entity<AutocompleteControl>, Entity<AutocompleteControl>);
impl Render for Pair {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.0.clone()).child(self.1.clone())
    }
}

#[test]
fn placeholder_follows_luma_strings_unless_the_app_set_it() {
    let mut app = TestAppContext::single();
    let (pair, cx) = app.add_window_view(|_, cx| {
        let built_in = cx.new(|cx| AutocompleteControl::from_builder(AutocompleteBuilder::new("built-in", []), cx));
        let app_text = cx.new(|cx| {
            AutocompleteControl::from_builder(AutocompleteBuilder::new("app", []).placeholder("Find a city"), cx)
        });
        Pair(built_in, app_text)
    });
    cx.run_until_parked();
    cx.update(|_, app| {
        assert_eq!(pair.read(app).0.read(app).textfield.read(app).placeholder_for_test(), "Type to filter…");
    });
    cx.update(|_, app| set_luma_strings(french(), app));
    cx.run_until_parked();
    cx.update(|_, app| {
        let built_in = pair.read(app).0.read(app);
        assert_eq!(built_in.model.placeholder, "Filtrer…");
        assert_eq!(built_in.textfield.read(app).placeholder_for_test(), "Filtrer…");
        assert_eq!(pair.read(app).1.read(app).textfield.read(app).placeholder_for_test(), "Find a city");
    });
}
