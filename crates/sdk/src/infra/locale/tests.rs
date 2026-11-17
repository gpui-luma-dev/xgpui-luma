use super::*;
use gpui::{AbsoluteLength, DefiniteLength, Length, Styled, div, px, relative, rems};

#[test]
fn compatibility_defaults_and_exact_integer_display() {
    let context = LocaleContext::default();
    assert_eq!(context.direction, LayoutDirection::Ltr);
    assert_eq!(context.language, None);
    assert_eq!(context.strings.cancel, "Cancel");
    assert_eq!(context.strings.clear, "Clear");
    assert_eq!(context.strings.first, "First");
    assert_eq!(context.strings.previous, "Previous");
    assert_eq!(context.strings.next, "Next");
    assert_eq!(context.strings.last, "Last");
    assert_eq!(context.strings.no_matches, "No matches");
    assert_eq!(context.strings.select_placeholder, "Select…");
    assert_eq!(context.strings.search_placeholder, "Search…");
    assert_eq!(context.strings.selection_search, "Selection search");
    assert_eq!(context.numbers.integer(u64::MAX), "18446744073709551615");
    assert_eq!(context.numbers.number(-1.5), "-1.5");
    assert_eq!(context.numbers.percent(0.125), "12.5%");
    assert_eq!(context.numbers.range(-2., 3.), "-2–3");
    assert!(!context.numbers.number(f64::NAN).is_empty());
    assert!(!context.numbers.number(f64::INFINITY).is_empty());
}

#[test]
fn chrome_and_data_share_language_but_have_different_base_direction() {
    let context = LocaleContext {
        direction: LayoutDirection::Rtl,
        language: Some("ar-EG-u-nu-latn".into()),
        ..Default::default()
    };
    let ui = context.paragraph_options();
    let data = context.data_paragraph_options();
    assert_eq!(ui.direction, TextDirection::RightToLeft);
    assert_eq!(data.direction, TextDirection::Auto);
    assert_eq!(ui.language.as_deref(), Some("ar-EG-u-nu-latn"));
    assert_eq!(ui.language, data.language);
    assert_eq!(LocaleContext::default().paragraph_options().direction, TextDirection::LeftToRight);
}

#[test]
fn range_hook_uses_the_custom_number_formatter() {
    struct Custom;
    impl NumberFormat for Custom {
        fn integer(&self, value: u64) -> SharedString {
            format!("n{value}").into()
        }
        fn number(&self, value: f64) -> SharedString {
            format!("n{value}").into()
        }
    }
    assert_eq!(Custom.range(1., 2.), "n1–n2");
    assert_eq!(Custom.percent(0.25), "n25%");
}

#[test]
fn logical_styles_preserve_units_and_block_axis_properties() {
    let mut style = div()
        .pt(px(90.))
        .pb(px(91.))
        .ps(rems(1.), LayoutDirection::Rtl)
        .pe(relative(0.25), LayoutDirection::Rtl)
        .ms(Length::Auto, LayoutDirection::Rtl)
        .me(px(-5.), LayoutDirection::Rtl)
        .start(relative(0.5), LayoutDirection::Rtl)
        .end(px(7.), LayoutDirection::Rtl);
    let style = style.style();
    assert_eq!(style.padding.top, Some(px(90.).into()));
    assert_eq!(style.padding.bottom, Some(px(91.).into()));
    assert_eq!(style.padding.right, Some(DefiniteLength::Absolute(rems(1.).into())));
    assert_eq!(style.padding.left, Some(relative(0.25)));
    assert_eq!(style.margin.right, Some(Length::Auto));
    assert_eq!(style.margin.left, Some(px(-5.).into()));
    assert_eq!(style.inset.right, Some(relative(0.5).into()));
    assert_eq!(style.inset.left, Some(px(7.).into()));
}

#[test]
fn logical_corners_and_borders_map_to_all_four_physical_corners() {
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
        let mut element = div()
            .border_s(px(1.), direction)
            .border_e(rems(2.), direction)
            .rounded_ss(px(3.), direction)
            .rounded_se(px(4.), direction)
            .rounded_es(px(5.), direction)
            .rounded_ee(px(6.), direction);
        let style = element.style();
        let (left, right, tl, tr, bl, br) = match direction {
            LayoutDirection::Ltr => (px(1.).into(), rems(2.).into(), 3., 4., 5., 6.),
            LayoutDirection::Rtl => (rems(2.).into(), px(1.).into(), 4., 3., 6., 5.),
        };
        assert_eq!(style.border_widths.left, Some(left));
        assert_eq!(style.border_widths.right, Some(right));
        assert_eq!(style.corner_radii.top_left, Some(AbsoluteLength::from(px(tl))));
        assert_eq!(style.corner_radii.top_right, Some(AbsoluteLength::from(px(tr))));
        assert_eq!(style.corner_radii.bottom_left, Some(AbsoluteLength::from(px(bl))));
        assert_eq!(style.corner_radii.bottom_right, Some(AbsoluteLength::from(px(br))));
    }
}

#[cfg(feature = "test-support")]
mod headless {
    use super::*;
    use gpui::{Context, Render, Window, TestAppContext, prelude::*};

    struct Probe;
    impl Render for Probe {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let direction = LocaleContext::current(cx).direction;
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .id("row")
                        .debug_selector(|| "locale-row".into())
                        .flex()
                        .relative()
                        .w(px(300.))
                        .h(px(60.))
                        .flex_row_logical(direction)
                        .ps(px(10.), direction)
                        .pe(px(20.), direction)
                        .child(div().debug_selector(|| "leading".into()).w(px(30.)).h(px(20.)).ms(px(5.), direction))
                        .child(div().debug_selector(|| "trailing".into()).w(px(40.)).h(px(20.))),
                )
                .child(
                    div().relative().w(px(300.)).h(px(50.)).child(
                        div()
                            .debug_selector(|| "positioned".into())
                            .absolute()
                            .start(px(15.), direction)
                            .w(px(20.))
                            .h(px(10.)),
                    ),
                )
        }
    }

    #[test]
    fn fallback_is_non_mutating_and_published_context_reflows_logical_layout() {
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, _| Probe);
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert!(cx.try_global::<LocaleContext>().is_none());
            assert_eq!(LocaleContext::current(cx).direction, LayoutDirection::Ltr);
            assert_eq!((LocaleContext::current(cx).strings.page_of)(2, 5, cx), "Page 2 of 5");
            assert!(cx.try_global::<LocaleContext>().is_none());
        });
        let leading = cx.debug_bounds("leading").unwrap();
        let trailing = cx.debug_bounds("trailing").unwrap();
        let positioned = cx.debug_bounds("positioned").unwrap();
        assert_eq!(leading.left(), px(15.));
        assert_eq!(trailing.left(), px(45.));
        assert_eq!(positioned.left(), px(15.));
        cx.update(|_, cx| {
            set_locale_context(
                LocaleContext { direction: LayoutDirection::Rtl, language: Some("ar".into()), ..Default::default() },
                cx,
            )
        });
        cx.run_until_parked();
        let leading = cx.debug_bounds("leading").unwrap();
        let trailing = cx.debug_bounds("trailing").unwrap();
        let positioned = cx.debug_bounds("positioned").unwrap();
        assert_eq!(leading.right(), px(285.));
        assert_eq!(trailing.right(), px(255.));
        assert_eq!(positioned.right(), px(285.));
        assert!(leading.left() > trailing.left());
    }

    #[test]
    fn translated_strings_formatters_and_language_are_published_together() {
        let app = TestAppContext::single();
        app.update(|cx| {
            let mut strings = SdkStrings::default();
            strings.next = "Weiter".into();
            strings.page_of = Arc::new(|page, pages, cx| {
                format!(
                    "Seite {page} von {pages} ({})",
                    LocaleContext::current(cx).language.as_deref().unwrap_or("none")
                )
                .into()
            });
            set_locale_context(
                LocaleContext { language: Some("de".into()), strings: Arc::new(strings), ..Default::default() },
                cx,
            );
            let snapshot = LocaleContext::current(cx);
            assert_eq!(snapshot.strings.next, "Weiter");
            assert_eq!((snapshot.strings.page_of)(2, 8, cx), "Seite 2 von 8 (de)");
        });
    }
}
