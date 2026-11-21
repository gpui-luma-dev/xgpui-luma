//! The `gpui` feature: `.apply()` and conversions. Run with `--features gpui`.
#![cfg(feature = "gpui")]

mod common;

use common::*;
use gpui::{AbsoluteLength, Hsla, Styled, div, px};
use xgpui_luma_style::{ApplyStyle, Color, PartId};

#[test]
fn colors_convert_to_gpui_hsla() {
    let red: Hsla = Color::srgb(1.0, 0.0, 0.0, 1.0).into();
    assert!((red.h - 0.0).abs() < 1e-3 && (red.s - 1.0).abs() < 1e-3 && (red.l - 0.5).abs() < 1e-3 && red.a == 1.0);
}

#[test]
fn apply_sets_core_properties_on_an_element() {
    let theme = shadcn_theme("light");
    let style = theme.resolve(&key(&theme, "checkbox", &[], &["checked"]));
    let mut el = div().apply(style.part(PartId::new("indicator")));
    let s = el.style();
    let expected: Hsla = style.part(PartId::new("indicator")).background().expect("background").into();
    match &s.background {
        Some(fill) => assert_eq!(fill.color().expect("solid fill"), expected.into()),
        None => panic!("no background"),
    }
    assert_eq!(s.border_color, Some(expected), "checked border follows the fill");
    assert_eq!(s.box_shadow.as_ref().map(Vec::len), Some(1), "Shadcn indicator shadow");

    let mut root = div().apply(style.root());
    let radius = root.style().corner_radii.top_left;
    assert_eq!(radius, Some(AbsoluteLength::Pixels(px(6.0))));
}
