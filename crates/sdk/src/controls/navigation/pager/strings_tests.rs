//! The built-in page indicator follows `LumaStrings`; an app formatter stays.
use super::*;
use crate::infra::strings::{set_luma_strings, tests::french};
use gpui::{Entity, TestAppContext, div, prelude::*};

struct Pair(Entity<PagerControl>, Entity<PagerControl>);
impl Render for Pair {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.0.clone()).child(self.1.clone())
    }
}

#[test]
fn page_indicator_follows_luma_strings_unless_the_app_set_it() {
    let mut app = TestAppContext::single();
    let (pair, cx) = app.add_window_view(|_, cx| {
        let built_in = PagerBuilder::new("built-in").page_count(5).current_page(1).spawn(cx);
        let app_text = PagerBuilder::new("app")
            .page_count(5)
            .current_page(1)
            .page_indicator_formatter(|page, pages| format!("{page}/{pages}").into())
            .spawn(cx);
        Pair(built_in, app_text)
    });
    cx.run_until_parked();
    cx.update(|_, app| assert_eq!(pair.read(app).0.read(app).page_indicator_for_test(), "Page 2 of 5"));
    cx.update(|_, app| set_luma_strings(french(), app));
    cx.run_until_parked();
    cx.update(|_, app| {
        assert_eq!(pair.read(app).0.read(app).page_indicator_for_test(), "Page 2 sur 5");
        assert_eq!(pair.read(app).1.read(app).page_indicator_for_test(), "2/5");
    });
    // A second replacement still applies: the installed indicator is not mistaken for app text.
    cx.update(|_, app| set_luma_strings(crate::infra::strings::LumaStrings::default(), app));
    cx.run_until_parked();
    cx.update(|_, app| assert_eq!(pair.read(app).0.read(app).page_indicator_for_test(), "Page 2 of 5"));
}
