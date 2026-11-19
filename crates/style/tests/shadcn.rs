//! The Shadcn look: CSS catalog tokens, light/dark modes, size-scoped metrics, variants, `extends`, `:=`.

mod common;

use common::*;
use xgpui_luma_style::{Color, Easing, Motion};

fn css(mode: &str, name: &str) -> Color {
    ShadcnCatalog::new().color(mode, name)
}

#[test]
fn fixtures_resolve_without_problems_in_both_modes() {
    for mode in ["light", "dark"] {
        let theme = shadcn_theme(mode);
        for control in ["checkbox", "button", "popup_menu"] {
            for variant in ["secondary", "outline", "ghost", "content_only"] {
                for size in ["sm", "md", "lg"] {
                    for states in
                        [&[][..], &["hover"], &["pressed"], &["checked"], &["checked", "hover"], &["disabled"]]
                    {
                        let key = theme.key(control).attr("variant", variant).attr("size", size).states(states);
                        theme.resolve(&key);
                    }
                }
            }
            theme.resolve(&theme.key(control));
        }
        assert!(theme.diagnostics().is_empty(), "{mode}: {}", theme.diagnostics());
    }
}

#[test]
fn checkbox_reads_the_css_catalog() {
    let theme = shadcn_theme("light");
    let style = |states: &[&str]| theme.resolve(&theme.key("checkbox").states(states));

    // outline_layer.default = first(css.card, css.background)
    assert_color(style(&[]).part("indicator").background(), css("light", "card"), "resting");
    assert_color(style(&["checked"]).part("indicator").background(), css("light", "primary"), "checked");
    let hover = ShadcnCatalog::layer(css("light", "primary"), "hover", "light");
    assert_color(style(&["checked", "hover"]).part("indicator").background(), hover, "checked + hover");
    assert_color(style(&["checked"]).part("glyph").foreground(), css("light", "primary_foreground"), "checkmark");

    // The SDK geometry still applies under the look.
    assert_eq!(style(&[]).root().length("indicator_size"), Some(18.0));
    // `shadow = none` when disabled leaves the shadow unset.
    assert_eq!(style(&[]).part("indicator").shadow("shadow").map(<[_]>::len), Some(1));
    assert_eq!(style(&["disabled"]).part("indicator").get("shadow"), None);
}

#[test]
fn size_scoped_tokens_follow_the_instance_size() {
    let theme = shadcn_theme("light");
    // size sm: indicator = metrics.control.control_height (28 for sm) × 0.45 = 12.6 → 13 at 1×.
    let sm = theme.resolve(&theme.key("checkbox").attr("size", "sm"));
    assert_eq!(sm.root().length("indicator_size"), Some(13.0));
    assert_eq!(sm.root().height(), Some(28.0));
    // md uses the SDK formula with the md height (36).
    let md = theme.resolve(&theme.key("checkbox").attr("size", "md"));
    assert_eq!(md.root().length("indicator_size"), Some(18.0));
    // css.spacing is absent, so the `spacing` constant falls back to 4: padding_x (md) = 4 × 3.5.
    let button = theme.resolve(&theme.key("popup_menu").attr("size", "md"));
    assert_eq!(button.root().length("padding_x"), Some(14.0));
}

#[test]
fn base_states_still_apply_inside_variants() {
    let theme = shadcn_theme("light");
    // secondary sets only its resting and hover/pressed colors; the base :disabled applies to it.
    let key = theme.key("button").attr("variant", "secondary");
    assert_color(theme.resolve(&key).root().background(), css("light", "secondary"), "secondary");
    assert_color(
        theme.resolve(&key.clone().state("disabled")).root().background(),
        css("light", "muted"),
        "secondary disabled",
    );

    // ghost extends outline (checkbox): outline's checked colors apply to ghost.
    let ghost = theme.key("checkbox").attr("variant", "ghost").state("checked");
    assert_color(theme.resolve(&ghost).part("indicator").background(), css("light", "foreground"), "ghost checked");
    let why = theme.explain(&ghost, "indicator", "background");
    assert_eq!(why.winner.expect("winner").scope, "variant outline");
}

#[test]
fn owning_assignments_replace_every_state() {
    let theme = shadcn_theme("light");
    for states in [&[][..], &["hover"], &["pressed"], &["disabled"]] {
        let key = theme.key("button").attr("variant", "content_only").states(states);
        let style = theme.resolve(&key);
        assert_color(style.root().background(), Color::TRANSPARENT, "content_only background");
        assert_color(style.root().foreground(), css("light", "foreground"), "content_only foreground");
    }
    let why = theme.explain(&theme.key("button").attr("variant", "content_only").state("hover"), "root", "background");
    assert_eq!(why.owned_by.expect("owned").scope, "variant content_only");
}

#[test]
fn modes_pick_mode_specific_tokens() {
    let light = shadcn_theme("light");
    let dark = shadcn_theme("dark");
    let key = |t: &xgpui_luma_style::Theme| t.key("button").attr("variant", "outline");
    assert_color(light.resolve(&key(&light)).root().background(), css("light", "background"), "light outline");
    let input = css("dark", "input");
    assert_color(dark.resolve(&key(&dark)).root().background(), input.with_alpha(0.3), "dark outline");
    // Native tokens switch with the mode too.
    assert_color(
        dark.resolve(&dark.key("checkbox").state("checked")).part("indicator").background(),
        css("dark", "primary"),
        "dark checked",
    );
}

#[test]
fn popup_trigger_paints_like_the_button_including_its_variant() {
    let theme = shadcn_theme("light");
    for variant in [None, Some("secondary"), Some("outline"), Some("ghost")] {
        for states in [&[][..], &["hover"], &["disabled"]] {
            let mut trigger = theme.key("popup_menu").states(states);
            let mut button = theme.key("button").states(states);
            if let Some(v) = variant {
                trigger = trigger.attr("variant", v);
                button = button.attr("variant", v);
            }
            let t = theme.resolve(&trigger).part("trigger").background();
            let b = theme.resolve(&button).root().background();
            assert_eq!(t, b, "{variant:?} {states:?}");
        }
    }
    let why = theme.explain(&theme.key("popup_menu").attr("variant", "outline"), "trigger", "background");
    assert_eq!(why.winner.expect("winner").via.as_deref(), Some("button.root"));
    // Parts the look does not touch keep the SDK's motion.
    let open = theme.resolve(&theme.key("popup_menu").state("open"));
    assert_eq!(open.part("menu").transition(), Some(Motion::Timed { duration_ms: 200.0, easing: Easing::OutCubic }));
}

#[test]
fn look_fonts_fall_back_when_the_catalog_has_none() {
    let theme = shadcn_theme("light");
    let style = theme.resolve(&theme.key("checkbox"));
    assert_eq!(style.part("label").string("font_family"), Some("Geist"));
}
