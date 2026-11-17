//! Shared headless look contract. No platform application is launched.
use std::sync::{Arc, Mutex};
use gpui::{Context, Entity, Focusable, IntoElement, Render, TestAppContext, Window, div, prelude::*, px};
use super::{LayoutDirection, LocaleContext, SdkStrings, set_locale_context};
use crate::controls::{
    button::ButtonTemplate,
    checkbox::{self, CheckboxData},
    selector::{Selector, SelectorItem, SelectorTemplate},
};
use crate::infra::presenter::HasPresenter;

struct Harness {
    check: checkbox::Checkbox,
    select: Entity<Selector>,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(600.)).flex().flex_col().gap(px(8.)).child(self.check.clone()).child(self.select.clone())
    }
}

/// Exercise the same pseudo-locale geometry/default-copy contract for each look.
/// Uses supplied look templates, not SDK fallback chrome.
pub fn assert_look_conformance(
    checkbox_template: Arc<dyn ButtonTemplate<CheckboxData>>,
    selector_template: Arc<dyn SelectorTemplate<SelectorItem>>,
) {
    let labels = Arc::new(Mutex::new(Vec::new()));
    let capture = labels.clone();
    let mut app = TestAppContext::single();
    let (host, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        Harness {
            check: checkbox::new("locale-check")
                .label("[Expanded checkbox label for pseudo locale]")
                .with_data(true)
                .template(checkbox_template)
                .motion_enabled(false)
                .spawn(cx),
            select: Selector::new("locale-select")
                .template(selector_template)
                .items([SelectorItem::new("first").label("First choice")])
                .motion_enabled(false)
                .with_template_modifier(move |root, model| {
                    capture.lock().unwrap().push(model.label.to_string());
                    root.debug_selector(|| "conformance-selector".into())
                })
                .spawn(cx),
        }
    });
    cx.run_until_parked();
    let (check, select) = cx.update(|window, cx| {
        let host = host.read(cx);
        let check = host.check.clone();
        let select = host.select.clone();
        select.read(cx).focus_handle(cx).focus(window, cx);
        (check, select)
    });
    for (tag, direction, prompt) in [
        ("en-XA", LayoutDirection::Ltr, "[Šéļéçţ å çĥöîçé — expanded placeholder]"),
        ("ar-XB", LayoutDirection::Rtl, "اختيار عنصر من القائمة"),
        ("en", LayoutDirection::Ltr, "Select…"),
    ] {
        cx.update(|_, cx| {
            set_locale_context(
                LocaleContext {
                    direction,
                    language: Some(tag.into()),
                    strings: Arc::new(SdkStrings { select_placeholder: prompt.into(), ..Default::default() }),
                    ..Default::default()
                },
                cx,
            )
        });
        cx.run_until_parked();
        assert_eq!(labels.lock().unwrap().last().unwrap(), prompt);
        let indicator = cx.debug_bounds("checkbox-indicator").unwrap();
        let label = cx.debug_bounds("checkbox-label").unwrap();
        assert_eq!(indicator.left() < label.left(), direction == LayoutDirection::Ltr);
        let caret = cx.debug_bounds("selector-trigger-caret").unwrap();
        let label = cx.debug_bounds("selector-trigger-label").unwrap();
        let shell = cx.debug_bounds("conformance-selector").unwrap();
        assert_eq!(label.left() < caret.left(), direction == LayoutDirection::Ltr);
        assert!(label.left() >= shell.left() && label.right() <= shell.right());
        assert!(caret.left() >= shell.left() && caret.right() <= shell.right());
        cx.update(|window, cx| {
            assert!(*check.read(cx).data());
            assert!(select.read(cx).selected_id().is_none());
            assert!(select.read(cx).focus_handle(cx).is_focused(window));
        });
    }
    select.update(cx, |select, cx| select.set_label("Application override", cx));
    cx.update(|_, cx| set_locale_context(LocaleContext::default(), cx));
    cx.run_until_parked();
    assert_eq!(labels.lock().unwrap().last().unwrap(), "Application override");
}

/// Run the shared headless localization contract with look-owned templates.
#[macro_export]
macro_rules! l10n_conformance {
    ($checkbox:expr, $selector:expr) => {
        #[test]
        fn l10n_look_conformance() {
            $crate::infra::locale::conformance::assert_look_conformance($checkbox, $selector);
        }
    };
}
