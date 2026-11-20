//! The `.lss` button and toggle against the native Radix recipes, while the native button-family code
//! still exists (icon button, split button and control groups use it). Delete together with it.
use gpui::Hsla;
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::style::lss::{AttrValue, ControlId, StyleKey, Value, box_shadows};
use xgpui_luma::theme::{InteractionState, StandardBoxScale, ThemeMode};

use super::*;
use crate::button::{ButtonVariant, Paint, button_family_theme_with};
use crate::palette::{Accent, Gray};

fn close(a: Hsla, b: Hsla) -> bool {
    let (a, b) = (a.to_rgb(), b.to_rgb());
    [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
}

#[test]
fn button_matches_native_recipes() {
    let config = config();
    let st = |n: &str| config.state(n).unwrap();
    let at = |n: &str| config.attribute(n).unwrap();
    let variants = [
        (ButtonVariant::Solid, None),
        (ButtonVariant::Classic, Some("classic")),
        (ButtonVariant::Soft, Some("soft")),
        (ButtonVariant::Surface, Some("surface")),
        (ButtonVariant::Outline, Some("outline")),
        (ButtonVariant::Ghost, Some("ghost")),
        (ButtonVariant::GhostQuiet, Some("ghost_quiet")),
        (ButtonVariant::Page, Some("page")),
    ];
    for look in [
        Look::built_in(),
        Look::from_palettes(Accent::Lime, Gray::Auto, crate::button_layout::metric_tokens(), ThemeMode::Light),
    ] {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let theme = theme(&look).unwrap();
            for (size, sk) in [
                (crate::RadixSize::One, Some("1")),
                (crate::RadixSize::Two, None),
                (crate::RadixSize::Three, Some("3")),
            ] {
                let sized = look.with_size(size);
                for (variant, vk) in variants {
                    for (paint, tone, hc) in [
                        (Paint::accent(), None, false),
                        (Paint::gray(), Some("gray"), false),
                        (Paint::accent().high_contrast(), None, true),
                        (Paint::gray().high_contrast(), Some("gray"), true),
                    ] {
                        let native = button_family_theme_with(&sized, variant, paint);
                        let scale = StandardBoxScale::compute(&native.metrics(), 1.0);
                        for role in [
                            ButtonFamilyRole::Text,
                            ButtonFamilyRole::Icon,
                            ButtonFamilyRole::Toggle { selected: false },
                            ButtonFamilyRole::Toggle { selected: true },
                        ] {
                            for bits in 0..16u8 {
                                let s = InteractionState {
                                    hovered: bits & 1 != 0,
                                    pressed: bits & 2 != 0,
                                    focused: bits & 4 != 0,
                                    disabled: bits & 8 != 0,
                                    invalid: false,
                                };
                                let want =
                                    native.resolve_look(role, s, &scale, xgpui_luma::theme::FULL_RADIUS_PX).unwrap();
                                let toggle = matches!(role, ButtonFamilyRole::Toggle { .. });
                                let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
                                let key = StyleKey::new(ControlId::new(if toggle { "toggle" } else { "button" }))
                                    .attr_opt(at("variant"), vk.map(AttrValue::new))
                                    .attr_opt(at("size"), sk.map(AttrValue::new))
                                    .attr_opt(at("tone"), tone.map(AttrValue::new))
                                    .state_if(st("high_contrast"), hc)
                                    .state_if(st("hover"), s.hovered)
                                    .state_if(st("pressed"), s.pressed)
                                    .state_if(st("focused"), s.focused)
                                    .state_if(st("disabled"), s.disabled)
                                    .state_if(st("selected"), selected);
                                let r = theme.resolve(&key);
                                let root = r.root();
                                let case = format!("{mode:?} {size:?} {variant:?} {paint:?} {role:?} {s:?}");
                                let c = |got: Option<xgpui_luma::style::lss::Color>, want: Hsla, w: &str| {
                                    let g = got.map(Hsla::from).unwrap_or(gpui::transparent_black());
                                    assert!(
                                        close(g, want) || (g.a == 0.0 && want.a == 0.0),
                                        "{case} {w}: {got:?} vs {want:?}"
                                    )
                                };
                                c(root.background(), want.background, "bg");
                                c(root.foreground(), want.foreground, "fg");
                                c(root.border(), want.border.unwrap_or(gpui::transparent_black()), "border");
                                c(root.focus_border(), want.focus_border, "focus");
                                c(
                                    root.value("muted_foreground").and_then(Value::as_color),
                                    want.muted_foreground,
                                    "muted",
                                );
                                // Disabled buttons never paint their shadow, so it is compared only when enabled.
                                let sh =
                                    root.shadow().filter(|l| !l.is_empty() && !s.disabled).map(|l| box_shadows(&l));
                                let want_shadow = want.shadow.clone().filter(|_| !s.disabled);
                                assert_eq!(
                                    sh.as_ref().map(Vec::len),
                                    want_shadow.as_ref().map(Vec::len),
                                    "{case} shadow"
                                );
                                if let (Some(g), Some(w)) = (&sh, &want_shadow) {
                                    for (g, w) in g.iter().zip(w) {
                                        assert!(
                                            close(g.color, w.color)
                                                && g.offset == w.offset
                                                && g.blur_radius == w.blur_radius
                                                && g.spread_radius == w.spread_radius
                                                && g.inset == w.inset,
                                            "{case} shadow layer {g:?} vs {w:?}"
                                        );
                                    }
                                }
                                // The classic wash: fill shifted to its top and bottom, flipped when pressed.
                                let stop = |k: &str| root.value(k).and_then(Value::as_color).map(Hsla::from);
                                if variant == ButtonVariant::Classic && !s.disabled {
                                    let params = look.classic_params();
                                    let shift =
                                        |d| crate::button::shift_lightness_with_params(want.background, d, params);
                                    let (top, bottom) = (shift(params.fill_top_delta), shift(params.fill_bottom_delta));
                                    let (top, bottom) = if s.pressed { (bottom, top) } else { (top, bottom) };
                                    assert!(stop("background_start").is_some_and(|g| close(g, top)), "{case} wash top");
                                    assert!(
                                        stop("background_end").is_some_and(|g| close(g, bottom)),
                                        "{case} wash bottom"
                                    );
                                } else {
                                    assert!(stop("background_start").is_none(), "{case} no wash");
                                }
                                let n = |k: &str| root.value(k).and_then(Value::as_number).unwrap_or(-1.0);
                                let icon = matches!(role, ButtonFamilyRole::Icon);
                                assert_eq!(root.height(), Some(want.height), "{case} height");
                                assert_eq!(if icon { 0.0 } else { n("padding_x") }, want.padding_x, "{case} px");
                                assert_eq!(if icon { 0.0 } else { n("padding_y") }, want.padding_y, "{case} py");
                                assert_eq!(root.gap(), Some(want.gap), "{case} gap");
                                assert_eq!(
                                    if icon { n("icon_radius") } else { root.radius().unwrap() },
                                    want.radius,
                                    "{case} radius"
                                );
                                assert_eq!(
                                    if icon { n("icon_size") } else { n("text_icon_size") },
                                    want.icon_size,
                                    "{case} icon"
                                );
                                assert_eq!(root.font_size(), Some(want.typography.size), "{case} font");
                                assert_eq!(root.line_height(), Some(want.typography.line_height), "{case} lh");
                                assert_eq!(root.font_family(), Some(want.font_family.as_ref()), "{case} family");
                            }
                        }
                    }
                }
            }
        }
    }
}
