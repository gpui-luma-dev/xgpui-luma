//! The default placeholder follows `LumaStrings`; app text stays fixed.
use super::*;
use crate::infra::strings::{set_luma_strings, tests::french};
use gpui::{TestAppContext, div, prelude::*};

struct Pair(Entity<ComboBoxControl>, Entity<ComboBoxControl>);
impl Render for Pair {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.0.clone()).child(self.1.clone())
    }
}

#[test]
fn placeholder_follows_luma_strings_unless_the_app_set_it() {
    let mut app = TestAppContext::single();
    let (pair, cx) = app.add_window_view(|_, cx| {
        let built_in = cx.new(|cx| ComboBoxControl::from_builder(ComboBoxBuilder::new("built-in", []), cx));
        let app_text = cx.new(|cx| ComboBoxControl::from_builder(ComboBoxBuilder::new("app", []).placeholder(""), cx));
        Pair(built_in, app_text)
    });
    cx.run_until_parked();
    cx.update(|_, app| {
        let built_in = pair.read(app).0.read(app);
        assert_eq!(built_in.model.placeholder, "Select…");
        assert_eq!(built_in.textfield.read(app).placeholder_for_test(), "Select…");
    });
    cx.update(|_, app| set_luma_strings(french(), app));
    cx.run_until_parked();
    cx.update(|_, app| {
        let built_in = pair.read(app).0.read(app);
        assert_eq!(built_in.model.placeholder, "Choisir…");
        assert_eq!(built_in.textfield.read(app).placeholder_for_test(), "Choisir…");
        let app_text = pair.read(app).1.read(app);
        assert_eq!(app_text.model.placeholder, "");
        assert_eq!(app_text.textfield.read(app).placeholder_for_test(), "");
    });
}
