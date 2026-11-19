//! The Shadcn look: CSS catalog tokens, light/dark modes, size-scoped metrics, variants, `extends`, `:=`.

mod common;

use common::*;
use xgpui_luma_style::{Color, Easing, Motion, PartId};

const INDICATOR: PartId = PartId::new("indicator");
const GLYPH: PartId = PartId::new("glyph");
const LABEL: PartId = PartId::new("label");

fn css(mode: &str, name: &str) -> Color {
    ShadcnCatalog::new().color(mode, name)
}

#[test]
fn every_axis_combination_resolves_without_problems_in_both_modes() {
    let theme = shadcn_theme("light");
    for control in ["checkbox", "button", "popup_menu"] {
        let id = theme.stylesheet().control(control).expect("declared");
        let problems = theme.check(id);
        assert!(problems.is_empty(), "{control}: {problems}");
    }
    assert_eq!(theme.stylesheet().modes(), ["dark", "light"]);
}

#[test]
fn checkbox_reads_the_css_catalog() {
    let theme = shadcn_theme("light");
    let style = |states: &[&str]| theme.resolve(&key(&theme, "checkbox", &[], states));

    // outline_layer.default = first(css.card, css.background)
    assert_color(style(&[]).part(INDICATOR).background(), css("light", "card"), "resting");
    assert_color(style(&["checked"]).part(INDICATOR).background(), css("light", "primary"), "checked");
    let hover = ShadcnCatalog::layer(css("light", "primary"), "hover", "light");
    assert_color(style(&["checked", "hover"]).part(INDICATOR).background(), hover, "checked + hover");
    assert_color(style(&["checked"]).part(GLYPH).foreground(), css("light", "primary_foreground"), "checkmark");

    // The SDK geometry still applies under the look.
    assert_eq!(style(&[]).root().value("indicator_size").and_then(|v| v.as_number()), Some(18.0));
    // `shadow = none` when disabled leaves the shadow unset.
    assert_eq!(style(&[]).part(INDICATOR).shadow().map(<[_]>::len), Some(1));
    assert_eq!(style(&["disabled"]).part(INDICATOR).shadow(), None);
}

#[test]
fn size_scoped_tokens_follow_the_instance_size() {
    let theme = shadcn_theme("light");
    let size = |s: &str| theme.resolve(&key(&theme, "checkbox", &[("size", s)], &[]));
    // size sm: indicator = metrics.control.control_height (28 for sm) × 0.45 = 12.6 → 13 at 1×.
    assert_eq!(size("sm").root().value("indicator_size").and_then(|v| v.as_number()), Some(13.0));
    assert_eq!(size("sm").root().height(), Some(28.0));
    // md uses the SDK formula with the md height (36).
    assert_eq!(size("md").root().value("indicator_size").and_then(|v| v.as_number()), Some(18.0));
    // css.spacing is absent, so the `spacing` constant falls back to 4: padding_x (md) = 4 × 3.5.
    let menu = theme.resolve(&key(&theme, "popup_menu", &[("size", "md")], &[]));
    assert_eq!(menu.root().value("padding_x").and_then(|v| v.as_number()), Some(14.0));
}

#[test]
fn base_states_still_apply_inside_variants() {
    let theme = shadcn_theme("light");
    // secondary sets only its resting and hover/pressed colors; the base :disabled applies to it.
    let secondary = theme.resolve(&key(&theme, "button", &[("variant", "secondary")], &[]));
    assert_color(secondary.root().background(), css("light", "secondary"), "secondary");
    let disabled = theme.resolve(&key(&theme, "button", &[("variant", "secondary")], &["disabled"]));
    assert_color(disabled.root().background(), css("light", "muted"), "secondary disabled");

    // ghost extends outline (checkbox): outline's checked colors apply to ghost.
    let ghost = key(&theme, "checkbox", &[("variant", "ghost")], &["checked"]);
    assert_color(theme.resolve(&ghost).part(INDICATOR).background(), css("light", "foreground"), "ghost checked");
    assert_eq!(theme.explain(&ghost, INDICATOR, "background").winner.expect("winner").scope, "variant outline");
}

#[test]
fn owning_assignments_replace_every_state() {
    let theme = shadcn_theme("light");
    for states in [&[][..], &["hover"], &["pressed"], &["disabled"]] {
        let style = theme.resolve(&key(&theme, "button", &[("variant", "content_only")], states));
        assert_color(style.root().background(), Color::TRANSPARENT, "content_only background");
        assert_color(style.root().foreground(), css("light", "foreground"), "content_only foreground");
    }
    let hovered = key(&theme, "button", &[("variant", "content_only")], &["hover"]);
    let why = theme.explain(&hovered, PartId::ROOT, "background");
    assert_eq!(why.owned_by.expect("owned").scope, "variant content_only");
}

#[test]
fn modes_pick_mode_specific_tokens() {
    let light = shadcn_theme("light");
    let dark = light.with_mode("dark").expect("dark exists");
    let outline = |t: &xgpui_luma_style::Theme| t.resolve(&key(t, "button", &[("variant", "outline")], &[]));
    assert_color(outline(&light).root().background(), css("light", "background"), "light outline");
    assert_color(outline(&dark).root().background(), css("dark", "input").with_alpha(0.3), "dark outline");
    // Native tokens switch with the mode too.
    let checked = dark.resolve(&key(&dark, "checkbox", &[], &["checked"]));
    assert_color(checked.part(INDICATOR).background(), css("dark", "primary"), "dark checked");
}

#[test]
fn popup_trigger_paints_like_the_button_including_its_variant() {
    let theme = shadcn_theme("light");
    let trigger = PartId::new("trigger");
    for variant in [None, Some("secondary"), Some("outline"), Some("ghost")] {
        for states in [&[][..], &["hover"], &["disabled"]] {
            let attrs: Vec<(&str, &str)> = variant.map(|v| ("variant", v)).into_iter().collect();
            let t = theme.resolve(&key(&theme, "popup_menu", &attrs, states)).part(trigger).background();
            let b = theme.resolve(&key(&theme, "button", &attrs, states)).root().background();
            assert_eq!(t, b, "{variant:?} {states:?}");
        }
    }
    let why = theme.explain(&key(&theme, "popup_menu", &[("variant", "outline")], &[]), trigger, "background");
    assert_eq!(why.winner.expect("winner").via.as_deref(), Some("button.root"));
    // Parts the look does not touch keep the SDK's motion.
    let open = theme.resolve(&key(&theme, "popup_menu", &[], &["open"]));
    let timed = Motion::Timed { duration_ms: 200.0, easing: Easing::OutCubic };
    assert_eq!(open.part(PartId::new("menu")).transition(), Some(timed));
}

#[test]
fn look_fonts_come_from_the_catalog() {
    let theme = shadcn_theme("light");
    let style = theme.resolve(&key(&theme, "checkbox", &[], &[]));
    assert_eq!(style.part(LABEL).font_family(), Some("Geist"));
}
