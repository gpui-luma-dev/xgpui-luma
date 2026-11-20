//! The `.lss` button and toggle against the native Shadcn looks, while the native button-family code still
//! exists (icon button, split button and control groups use it). Delete together with the native code.
use gpui::Hsla;
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::style::lss::{AttrValue, ControlId, StyleKey, Value, box_shadows};
use xgpui_luma::theme::{InteractionState, ThemeMode};

use super::*;
use crate::ShadcnSize;
use crate::controls::button::button_look;

fn close(a: Hsla, b: Hsla) -> bool {
    let (a, b) = (a.to_rgb(), b.to_rgb());
    [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
}

#[test]
fn button_matches_native() {
    let config = xgpui_luma::style::config();
    let st = |n: &str| config.state(n).unwrap();
    let (va, sa) = (config.attribute("variant").unwrap(), config.attribute("size").unwrap());
    for css in [
        crate::FALLBACK_CSS.to_string(),
        include_str!("../../tests/fixtures/tweakcn/retro-arcade.css").to_string(),
        include_str!("../../tests/fixtures/tweakcn/astrovista.css").to_string(),
    ] {
        let look = ShadcnLook::from_css_str(&css).unwrap();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let theme = theme(&look).unwrap();
            let tokens = look.mode_tokens();
            for (style, variant) in [
                (ShadcnButtonStyle::Primary, None),
                (ShadcnButtonStyle::Secondary, Some("secondary")),
                (ShadcnButtonStyle::Outline, Some("outline")),
                (ShadcnButtonStyle::Ghost, Some("ghost")),
                (ShadcnButtonStyle::ContentOnly, Some("content_only")),
            ] {
                for (size, sk) in [(ShadcnSize::Sm, Some("sm")), (ShadcnSize::Md, None), (ShadcnSize::Lg, Some("lg"))] {
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
                            let want = button_look(&tokens, mode, style, role, size, s);
                            let toggle = matches!(role, ButtonFamilyRole::Toggle { .. });
                            let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
                            let key = StyleKey::new(ControlId::new(if toggle { "toggle" } else { "button" }))
                                .attr_opt(va, variant.map(AttrValue::new))
                                .attr_opt(sa, sk.map(AttrValue::new))
                                .state_if(st("hover"), s.hovered)
                                .state_if(st("pressed"), s.pressed)
                                .state_if(st("focused"), s.focused)
                                .state_if(st("disabled"), s.disabled)
                                .state_if(st("selected"), selected);
                            let r = theme.resolve(&key);
                            let root = r.root();
                            let case = format!("{mode:?} {style:?} {size:?} {role:?} {s:?}");
                            let c = |got: Option<xgpui_luma::style::lss::Color>, want: Hsla, w: &str| {
                                assert!(
                                    got.map(Hsla::from).is_some_and(|g| close(g, want)),
                                    "{case} {w}: {got:?} vs {want:?}"
                                )
                            };
                            c(root.background(), want.background, "bg");
                            c(root.foreground(), want.foreground, "fg");
                            match want.border {
                                Some(b) => c(root.border(), b, "border"),
                                None => assert!(
                                    root.border().is_none_or(|b| b.to_srgb()[3] == 0.0),
                                    "{case} border {:?}",
                                    root.border()
                                ),
                            }
                            c(root.focus_border(), want.focus_border, "focus");
                            c(root.value("muted_foreground").and_then(Value::as_color), want.muted_foreground, "muted");
                            let sh = root.shadow().filter(|l| !l.is_empty()).map(|l| box_shadows(&l));
                            assert_eq!(sh.as_ref().map(Vec::len), want.shadow.as_ref().map(Vec::len), "{case} shadow");
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
                            assert!(
                                (root.line_height().unwrap() - want.typography.line_height).abs() < 1e-3,
                                "{case} lh"
                            );
                            assert_eq!(root.font_family(), Some(want.font_family.as_ref()), "{case} family");
                        }
                    }
                }
            }
        }
    }
}
