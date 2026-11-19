//! How SDK templates would use the crate: handles looked up once, keys built without allocation each
//! frame, typed reads, and per-instance overrides (API topics 1, 2 and 5).

mod common;

use common::*;
use xgpui_luma_style::{
    AttrId, AttrValue, Color, ControlId, Motion, Overrides, PartId, Prop, StateId, StateSet, StyleKey, Theme,
    ValueSource, props,
};

/// What the SDK's checkbox module would hold in a `OnceLock`, built once from the stylesheet.
struct CheckboxHandles {
    control: ControlId,
    indicator: PartId,
    glyph: PartId,
    indicator_size: Prop<f32>,
    glyph_size: Prop<f32>,
    checked: StateId,
    hover: StateId,
    disabled: StateId,
    variant: AttrId,
}

impl CheckboxHandles {
    fn new(theme: &Theme) -> Self {
        let sheet = theme.stylesheet();
        let config = sheet.config();
        let control = sheet.control("checkbox").expect("the SDK declares checkbox");
        Self {
            control,
            indicator: sheet.part(control, "indicator").expect("part"),
            glyph: sheet.part(control, "glyph").expect("part"),
            indicator_size: sheet.prop(control, "indicator_size").expect("declared length"),
            glyph_size: sheet.prop(control, "glyph_size").expect("declared length"),
            checked: config.state("checked").expect("state"),
            hover: config.state("hover").expect("state"),
            disabled: config.state("disabled").expect("state"),
            variant: config.attribute("variant").expect("attribute"),
        }
    }
}

/// The model a checkbox template receives each frame.
struct Model {
    checked: bool,
    hovered: bool,
    disabled: bool,
    variant: Option<AttrValue>,
    scale: f32,
}

fn key_for(h: &CheckboxHandles, m: &Model) -> StyleKey {
    StyleKey::new(h.control)
        .state_if(h.checked, m.checked)
        .state_if(h.hover, m.hovered)
        .state_if(h.disabled, m.disabled)
        .attr_opt(h.variant, m.variant)
        .scale(m.scale)
}

#[test]
fn a_template_reads_typed_values_through_handles() {
    let theme = radix_theme("light");
    let h = CheckboxHandles::new(&theme);
    let model = Model { checked: true, hovered: true, disabled: false, variant: None, scale: 2.0 };
    let style = theme.resolve(&key_for(&h, &model));

    let indicator = style.part(h.indicator);
    assert_color(indicator.get(props::BACKGROUND), radix_step("accent", 10, "light"), "checked + hover");
    // No size attribute: the SDK formula applies (36 × 0.5), snapped at 2×.
    assert_eq!(style.root().get(h.indicator_size), Some(18.0));
    assert!(style.root().get(h.glyph_size).is_some());
    assert!(style.part(h.glyph).get(props::FOREGROUND).is_some());
    assert_eq!(style.root().get(props::TRANSITION).map(|m| matches!(m, Motion::Timed { .. })), Some(true));
}

#[test]
fn keys_are_small_copyable_values_and_cache_hits_are_shared() {
    let theme = sdk_theme();
    let h = CheckboxHandles::new(&theme);
    let model = Model { checked: false, hovered: false, disabled: false, variant: None, scale: 1.0 };
    let a = key_for(&h, &model);
    let b = a; // Copy
    assert_eq!(a, b);
    let first = theme.resolve(&a);
    let second = theme.resolve(&b);
    assert!(std::sync::Arc::ptr_eq(&first, &second), "the second resolve is a cache hit");
    // State sets combine like bit flags.
    let set: StateSet = [h.checked, h.hover].into_iter().collect();
    assert!(set.contains(h.checked) && set.contains(h.hover) && !set.contains(h.disabled));
}

#[test]
fn handles_fail_at_startup_when_files_disagree() {
    let theme = sdk_theme();
    let sheet = theme.stylesheet();
    let checkbox = sheet.control("checkbox").expect("declared");
    // A renamed property, or a handle of the wrong type, is caught when the SDK starts.
    assert!(sheet.prop::<f32>(checkbox, "indicator_sise").is_err());
    assert!(sheet.prop::<Color>(checkbox, "indicator_size").is_err());
    assert!(sheet.part(checkbox, "indicatr").is_none());
    assert!(sheet.control("chekbox").is_none());
}

#[test]
fn instance_overrides_win_in_every_state_and_dependents_follow() {
    let theme = sdk_theme();
    let h = CheckboxHandles::new(&theme);
    let green = Color::oklch(0.6, 0.2, 145.0, 1.0);
    let overrides = Overrides::new().set(h.indicator, props::BACKGROUND, green).set(
        PartId::ROOT,
        props::TRANSITION,
        Motion::Instant,
    );

    for (checked, disabled) in [(false, false), (true, false), (true, true)] {
        let model = Model { checked, hovered: false, disabled, variant: None, scale: 1.0 };
        let style = theme.resolve_with(&key_for(&h, &model), &overrides);
        assert_color(style.part(h.indicator).background(), green, "override in every state");
        assert_eq!(style.root().transition(), Some(Motion::Instant));
        if checked && !disabled {
            // The SDK rule `border = background` follows the overridden background.
            assert_color(style.part(h.indicator).border(), green, "border follows");
        }
    }

    let model = Model { checked: true, hovered: false, disabled: false, variant: None, scale: 1.0 };
    let why = theme.explain_with(&key_for(&h, &model), &overrides, h.indicator, "background");
    assert_eq!(why.source, ValueSource::InstanceOverride);
    assert!(!why.overridden.is_empty(), "the rules it replaced are still listed");
}

#[test]
fn token_overrides_follow_mode_changes() {
    let light = shadcn_theme("light");
    let dark = light.with_mode("dark").expect("dark");
    let h = CheckboxHandles::new(&light);
    let overrides = Overrides::new().set_token(h.indicator, props::BACKGROUND, "palette.focus.ring");
    let model = Model { checked: false, hovered: false, disabled: false, variant: None, scale: 1.0 };
    let in_light = light.resolve_with(&key_for(&h, &model), &overrides).part(h.indicator).background();
    let in_dark = dark.resolve_with(&key_for(&h, &model), &overrides).part(h.indicator).background();
    assert_color(in_light, ShadcnCatalog::new().color("light", "ring"), "light ring");
    assert_color(in_dark, ShadcnCatalog::new().color("dark", "ring"), "dark ring");
}

#[test]
fn variants_come_from_builders_as_interned_values() {
    let theme = radix_theme("light");
    let h = CheckboxHandles::new(&theme);
    // A look builder's `.variant("soft")` interns once; the template passes the value through.
    let soft = AttrValue::named("soft");
    let model = Model { checked: true, hovered: false, disabled: false, variant: Some(soft), scale: 1.0 };
    let style = theme.resolve(&key_for(&h, &model));
    assert_color(style.part(h.indicator).background(), radix_step("accent", 5, "light"), "soft");
}

#[test]
fn equal_override_sets_share_cached_results() {
    let theme = sdk_theme();
    let h = CheckboxHandles::new(&theme);
    let model = Model { checked: false, hovered: false, disabled: false, variant: None, scale: 1.0 };
    let k = key_for(&h, &model);
    // Built twice, in a different order: same contents, same cached result.
    let a = Overrides::new().set(PartId::ROOT, props::GAP, 4.0).set(h.indicator, props::RADIUS, 2.0);
    let b = Overrides::new().set(h.indicator, props::RADIUS, 2.0).set(PartId::ROOT, props::GAP, 4.0);
    assert_eq!(a, b);
    assert_eq!(a.id(), b.id());
    assert!(std::sync::Arc::ptr_eq(&theme.resolve_with(&k, &a), &theme.resolve_with(&k, &b)));
    // An empty override set is the same as none.
    assert!(std::sync::Arc::ptr_eq(&theme.resolve_with(&k, &Overrides::new()), &theme.resolve(&k)));
    // Different contents are different entries.
    let c = Overrides::new().set(PartId::ROOT, props::GAP, 4.5);
    assert_ne!(a, c);
    assert_eq!(theme.resolve_with(&k, &c).root().gap(), Some(5.0)); // 4.5 snapped at 1×
}

#[test]
fn overrides_for_unknown_parts_or_properties_are_reported_and_ignored() {
    let theme = sdk_theme();
    let h = CheckboxHandles::new(&theme);
    let model = Model { checked: false, hovered: false, disabled: false, variant: None, scale: 1.0 };
    let misspelled: Prop<f32> = Prop::new("indicator_sise");
    let overrides =
        Overrides::new().set(PartId::ROOT, misspelled, 3.0).set(PartId::new("indicatr"), props::RADIUS, 2.0);
    let style = theme.resolve_with(&key_for(&h, &model), &overrides);
    assert_eq!(style.root().value("indicator_sise"), None);
    let d = theme.diagnostics();
    assert!(
        d.mentions("instance override: \"indicator_sise\" is not a core or declared property of checkbox — did you mean \"indicator_size\"?"),
        "{d}"
    );
    assert!(
        d.mentions("instance override: unknown part \"indicatr\" for checkbox — did you mean \"indicator\"?"),
        "{d}"
    );
}

#[test]
fn override_problems_have_no_file_location() {
    let theme = sdk_theme();
    let h = CheckboxHandles::new(&theme);
    let model = Model { checked: false, hovered: false, disabled: false, variant: None, scale: 1.0 };
    let overrides = Overrides::new().set_token(h.indicator, props::BACKGROUND, "palette.nope");
    theme.resolve_with(&key_for(&h, &model), &overrides);
    let d = theme.diagnostics();
    let problem = d.iter().find(|x| x.message.contains("palette.nope")).expect("reported");
    assert_eq!(problem.source, None, "not blamed on a style file: {problem}");
    assert!(problem.message.starts_with("instance override for indicator.background: "), "{}", problem.message);
}
