use super::*;
use std::sync::Arc;
use gpui::{TestAppContext, px};
use crate::{
    controls::progress::{LinearProgressTemplate, ProgressOrientation, default_progress_theme},
    infra::locale::{LayoutDirection, LocaleContext, set_locale_context},
};

#[test]
fn default_horizontal_fill_and_thumb_follow_live_locale_without_changing_value() {
    let mut app = TestAppContext::single();
    let (progress, cx) = app.add_window_view(|_, cx| {
        ProgressControl::from_builder(
            ProgressBuilder::new("localized-progress")
                .value(25.)
                .show_thumb(true)
                .motion_enabled(false)
                .template(Arc::new(LinearProgressTemplate::new(default_progress_theme()))),
            cx,
        )
    });
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        let track = cx.debug_bounds("progress-track").unwrap();
        let fill = cx.debug_bounds("progress-fill").unwrap();
        let thumb = cx.debug_bounds("progress-thumb").unwrap();
        assert!((fill.size.width - track.size.width * 0.25).abs() < px(1.));
        if direction == LayoutDirection::Rtl {
            assert_eq!(fill.right(), track.right());
            assert!((thumb.center().x - fill.left()).abs() < px(1.));
        } else {
            assert_eq!(fill.left(), track.left());
            assert!((thumb.center().x - fill.right()).abs() < px(1.));
        }
        cx.update(|_, cx| {
            let view = progress.read(cx);
            assert_eq!(view.value(), 25.);
            assert_eq!(view.render_model().percentage, 0.25);
        });
    }
    // An override equal to the current default must still stop locale adoption.
    progress.update(cx, |view, cx| view.set_direction(ProgressDirection::LeftToRight, cx));
    cx.update(|_, cx| set_locale_context(LocaleContext { direction: LayoutDirection::Rtl, ..Default::default() }, cx));
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(progress.read(cx).direction(), ProgressDirection::LeftToRight));
}

#[test]
fn installed_locale_is_used_at_construction_but_explicit_and_vertical_directions_win() {
    for explicit in [
        None,
        Some(ProgressDirection::LeftToRight),
        Some(ProgressDirection::RightToLeft),
        Some(ProgressDirection::BottomToTop),
        Some(ProgressDirection::TopToBottom),
    ] {
        let mut app = TestAppContext::single();
        app.update(|cx| {
            set_locale_context(LocaleContext { direction: LayoutDirection::Rtl, ..Default::default() }, cx)
        });
        let (progress, cx) = app.add_window_view(|_, cx| {
            let mut builder = ProgressBuilder::new("initial").value(40.).motion_enabled(false);
            if let Some(direction) = explicit {
                builder = builder.direction(direction);
            }
            ProgressControl::from_builder(builder, cx)
        });
        cx.update(|_, cx| {
            assert_eq!(progress.read(cx).direction(), explicit.unwrap_or(ProgressDirection::RightToLeft))
        });
        cx.update(|_, cx| set_locale_context(LocaleContext::default(), cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(progress.read(cx).direction(), explicit.unwrap_or(ProgressDirection::LeftToRight))
        });
    }
    let mut app = TestAppContext::single();
    app.update(|cx| set_locale_context(LocaleContext { direction: LayoutDirection::Rtl, ..Default::default() }, cx));
    let (progress, cx) = app.add_window_view(|_, cx| {
        ProgressControl::from_builder(ProgressBuilder::new("vertical").orientation(ProgressOrientation::Vertical), cx)
    });
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(progress.read(cx).direction(), ProgressDirection::BottomToTop));
}
