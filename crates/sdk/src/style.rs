//! The SDK's styles in the Luma style language (issue #285).
//!
//! [`config`] is the one global configuration every stylesheet shares. [`sources`] lists the SDK's
//! `.lss` files: the token file and one file beside each migrated control. [`default_theme`] is the
//! SDK alone, bound with no native tokens and no mode (the SDK has no light/dark; another SDK theme
//! is another token file). Looks layer their own files over [`sources`].

use std::sync::{Arc, OnceLock};

pub use xgpui_luma_style as lss;
use xgpui_luma_style::{Config, NoNativeTokens, SnapPolicy, Source, Stylesheet, Theme};

/// The SDK's states, exclusive interaction group, generic attributes, layers and snapping. Looks extend
/// it with their own attributes and states via [`Config::extend`].
pub fn config() -> Arc<Config> {
    static CONFIG: OnceLock<Arc<Config>> = OnceLock::new();
    CONFIG
        .get_or_init(|| {
            Arc::new(
                Config::builder()
                    .layers(&["sdk", "look", "app"])
                    .states(&[
                        "hover", "pressed", "focused", "disabled", "checked", "selected", "active", "expanded", "open",
                        "invalid",
                    ])
                    .exclusive(&["disabled", "pressed", "hover"])
                    // Generic axes with no values: looks write the values (`size sm`, `size 2`) in their
                    // files and add their own axes with `config().extend()` (Radix `tone`, `radius`).
                    .attributes(&["variant", "size"])
                    .snap(SnapPolicy::Lengths)
                    .build()
                    .expect("the SDK style configuration is valid"),
            )
        })
        .clone()
}

macro_rules! sdk_sources {
    ($($path:literal),+ $(,)?) => {
        /// The SDK's style files, for the `sdk` layer.
        pub fn sources() -> Vec<Source> {
            vec![$(Source::new(concat!("sdk/", $path), include_str!($path))),+]
        }

        #[cfg(test)]
        const SOURCE_PATHS: &[&str] = &[$($path),+];
    };
}

sdk_sources!["theme/tokens.lss", "controls/choice/checkbox/checkbox.lss"];

/// The SDK's files compiled on their own.
pub fn stylesheet() -> Stylesheet {
    static SHEET: OnceLock<Stylesheet> = OnceLock::new();
    SHEET
        .get_or_init(|| {
            Stylesheet::builder(config())
                .layer("sdk", sources())
                .compile()
                .unwrap_or_else(|d| panic!("the SDK style files compile:\n{d}"))
        })
        .clone()
}

/// The SDK alone: no native tokens, no mode.
pub fn default_theme() -> Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME
        .get_or_init(|| {
            Theme::bind(&stylesheet(), Arc::new(NoNativeTokens), "")
                .unwrap_or_else(|d| panic!("the SDK theme binds:\n{d}"))
        })
        .clone()
}

/// The SDK motion runtime's transition for a `transition` property value.
pub fn transition(motion: &xgpui_luma_style::Motion) -> crate::motion::Transition {
    use crate::motion::{Easing, Transition};
    use xgpui_luma_style::{Easing as LssEasing, Motion};
    match *motion {
        Motion::Instant => Transition::instant(),
        Motion::Timed { duration_ms, easing } => {
            let easing = match easing {
                LssEasing::Linear => Easing::Linear,
                LssEasing::InQuad => Easing::InQuad,
                LssEasing::OutQuad => Easing::OutQuad,
                LssEasing::InOutQuad => Easing::InOutQuad,
                LssEasing::InCubic => Easing::InCubic,
                LssEasing::OutCubic => Easing::OutCubic,
                LssEasing::InOutCubic => Easing::InOutCubic,
            };
            // Whole nanoseconds, so equal files give equal transitions (an equal transition does not
            // restart a running animation).
            let nanos = (f64::from(duration_ms.max(0.0)) * 1e6).round() as u64;
            Transition::timed(std::time::Duration::from_nanos(nanos), easing)
        }
        Motion::Spring { stiffness, damping } => Transition::spring(stiffness, damping),
    }
}

#[cfg(test)]
mod tests {
    use gpui::Hsla;
    use xgpui_luma_style::{Color, ControlId, PartId, Value};

    use super::*;
    use crate::controls::checkbox::{CheckboxTheme, DefaultCheckboxTheme};
    use crate::theme::{InteractionState, ThemeTokens};

    fn close(a: Hsla, b: Hsla) -> bool {
        let (a, b) = (a.to_rgb(), b.to_rgb());
        [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
    }

    fn hsla(c: Option<Color>) -> Option<Hsla> {
        c.map(Hsla::from)
    }

    #[test]
    fn sources_list_every_lss_file() {
        fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).expect("readable").flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, root, out);
                } else if path.extension().is_some_and(|e| e == "lss") {
                    out.push(path.strip_prefix(root).expect("under src").to_string_lossy().replace('\\', "/"));
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut found = Vec::new();
        walk(&root, &root, &mut found);
        for file in &found {
            assert!(SOURCE_PATHS.contains(&file.as_str()), "{file} exists but is not listed in style::sources()");
        }
    }

    #[test]
    fn every_axis_combination_resolves_without_problems() {
        let theme = default_theme();
        for control in theme.stylesheet().controls() {
            let problems = theme.check(control);
            assert!(problems.is_empty(), "{control}: {problems}");
        }
    }

    #[test]
    fn tokens_match_theme_tokens_light() {
        let native = ThemeTokens::light();
        let theme = default_theme();
        let values: std::collections::HashMap<String, Value> = {
            // Read each token by resolving it through a throwaway file: a property per token.
            let p = &native.palette;
            let colors: Vec<(&str, Hsla)> = vec![
                ("palette.app.background", p.app.background),
                ("palette.app.foreground", p.app.foreground),
                ("palette.app.muted_foreground", p.app.muted_foreground),
                ("palette.surface.panel.background", p.surface.panel.background),
                ("palette.surface.panel.foreground", p.surface.panel.foreground),
                ("palette.surface.panel.border", p.surface.panel.border),
                ("palette.surface.floating.background", p.surface.floating.background),
                ("palette.surface.floating.border", p.surface.floating.border),
                ("palette.surface.subtle.background", p.surface.subtle.background),
                ("palette.surface.subtle.foreground", p.surface.subtle.foreground),
                ("palette.state.hover.background", p.state.hover.background),
                ("palette.state.hover.foreground", p.state.hover.foreground),
                ("palette.state.pressed.background", p.state.pressed.background),
                ("palette.state.selected.background", p.state.selected.background),
                ("palette.state.selected.foreground", p.state.selected.foreground),
                ("palette.state.disabled.background", p.state.disabled.background),
                ("palette.state.disabled.foreground", p.state.disabled.foreground),
                ("palette.form.input.background", p.form.input.background),
                ("palette.form.input.foreground", p.form.input.foreground),
                ("palette.form.input.border", p.form.input.border),
                ("palette.form.input.placeholder", p.form.input.placeholder),
                ("palette.focus.ring", p.focus.ring),
                ("palette.border.default", p.border.default),
                ("palette.border.strong", p.border.strong),
            ];
            let decls: String =
                colors.iter().enumerate().map(|(i, (path, _))| format!("  c{i}: color = {path}\n")).collect();
            let sheet = theme
                .stylesheet()
                .with_source("sdk", Source::new("probe.lss", format!("probe {{\n{decls}}}")))
                .expect("probe compiles");
            let probe = Theme::bind(&sheet, Arc::new(NoNativeTokens), "").expect("binds");
            let resolved = probe.resolve(&xgpui_luma_style::StyleKey::new(ControlId::new("probe")));
            for (i, (path, expected)) in colors.iter().enumerate() {
                let got = resolved.root().value(&format!("c{i}")).and_then(Value::as_color).map(Hsla::from);
                assert!(got.is_some_and(|g| close(g, *expected)), "{path}: {got:?} != {expected:?}");
            }
            std::collections::HashMap::new()
        };
        assert!(values.is_empty());
    }

    #[test]
    fn checkbox_matches_the_native_theme() {
        let tokens = ThemeTokens::light();
        let native = DefaultCheckboxTheme::new(tokens.clone());
        let theme = default_theme();
        let config = config();
        let state = |name: &str| config.state(name).expect("state");
        let checkbox = ControlId::new("checkbox");
        let (indicator, glyph, label) = (PartId::new("indicator"), PartId::new("glyph"), PartId::new("label"));
        let sheet = theme.stylesheet();
        let length = |name: &str| sheet.prop::<f32>(checkbox, name).expect("declared");

        for bits in 0..32u8 {
            let s = InteractionState {
                hovered: bits & 1 != 0,
                pressed: bits & 2 != 0,
                focused: bits & 4 != 0,
                disabled: bits & 8 != 0,
                invalid: bits & 16 != 0,
            };
            for checked in [false, true] {
                for scale in [1.0, 1.25, 1.5, 2.0] {
                    let key = xgpui_luma_style::StyleKey::new(checkbox)
                        .state_if(state("hover"), s.hovered)
                        .state_if(state("pressed"), s.pressed)
                        .state_if(state("focused"), s.focused)
                        .state_if(state("disabled"), s.disabled)
                        .state_if(state("invalid"), s.invalid)
                        .state_if(state("checked"), checked)
                        .scale(scale);
                    let case = format!("{s:?} checked={checked} @{scale}x");
                    let style = theme.resolve(&key);
                    let palette = native.resolve(checked, s);
                    let metrics = native.scale(scale);

                    let color = |part: PartId, got: Option<Color>, want: Hsla, what: &str| {
                        let got = hsla(got).unwrap_or_else(|| panic!("{case}: {what} on {part} is unset"));
                        assert!(close(got, want), "{case}: {what}: {got:?} != {want:?}");
                    };
                    color(
                        indicator,
                        style.part(indicator).background(),
                        palette.indicator_background,
                        "indicator background",
                    );
                    color(indicator, style.part(indicator).border(), palette.indicator_border, "indicator border");
                    color(glyph, style.part(glyph).foreground(), palette.checkmark_color, "checkmark");
                    color(label, style.part(label).foreground(), palette.label_color, "label");
                    color(PartId::ROOT, style.root().focus_border(), palette.focus_border, "focus border");
                    assert_eq!(style.root().background(), None, "{case}: control background");
                    assert_eq!(style.root().border(), None, "{case}: control border");
                    assert_eq!(style.part(indicator).shadow(), None, "{case}: indicator shadow");

                    let root = style.root();
                    assert_eq!(
                        root.get(length("indicator_size")),
                        Some(metrics.indicator_size),
                        "{case}: indicator_size"
                    );
                    assert_eq!(root.get(length("glyph_size")), Some(metrics.glyph_size), "{case}: glyph_size");
                    assert_eq!(
                        root.get(length("indicator_radius")),
                        Some(metrics.indicator_radius),
                        "{case}: indicator_radius"
                    );
                    assert_eq!(
                        root.get(length("label_baseline_shift")),
                        Some(metrics.label_baseline_shift),
                        "{case}: baseline"
                    );
                    assert_eq!(root.height(), Some(metrics.height), "{case}: height");
                    assert_eq!(root.gap(), Some(metrics.gap), "{case}: gap");
                    assert_eq!(root.radius(), Some(metrics.control_radius), "{case}: control radius");
                    let number = |name: &str| root.value(name).and_then(Value::as_number);
                    assert_eq!(number("disabled_opacity"), Some(metrics.disabled_opacity), "{case}: disabled opacity");
                    assert_eq!(number("border_width"), Some(tokens.metrics.border_width), "{case}: border width");

                    let text = style.part(label);
                    assert_eq!(text.font_size(), Some(palette.label_typography.size), "{case}: font size");
                    let lh = text.line_height().expect("line height");
                    assert!((lh - palette.label_typography.line_height).abs() < 1e-4, "{case}: line height {lh}");
                    assert_eq!(
                        number_of(&text, "font_weight"),
                        Some(palette.label_typography.weight.0),
                        "{case}: weight"
                    );
                    assert_eq!(text.font_family(), Some(palette.label_font_family.as_ref()), "{case}: font family");
                }
            }
        }
    }

    #[test]
    fn checkbox_transition_matches_the_former_checkbox_motion() {
        use crate::controls::checkbox::{CheckboxData, default_checkbox_template};
        use crate::motion::{Easing, TransitionDescription};
        // What `CheckboxMotion::state` was in `MotionDefaults::vanilla()`.
        let expected =
            TransitionDescription::Timed { duration: std::time::Duration::from_millis(200), easing: Easing::OutCubic };
        for checked in [false, true] {
            let transition = default_checkbox_template().transition(&CheckboxData::new(checked)).expect("transition");
            assert_eq!(transition.description(), expected, "checked={checked}");
        }
    }

    #[test]
    fn transitions_convert_to_the_motion_runtime() {
        use crate::motion::{Easing, TransitionDescription};
        use std::time::Duration;
        use xgpui_luma_style::{Easing as E, Motion};
        assert_eq!(transition(&Motion::Instant).description(), TransitionDescription::Instant);
        assert_eq!(
            transition(&Motion::Spring { stiffness: 300.0, damping: 28.0 }).description(),
            TransitionDescription::Spring { stiffness: 300.0, damping: 28.0 }
        );
        for (lss, sdk) in [
            (E::Linear, Easing::Linear),
            (E::InQuad, Easing::InQuad),
            (E::OutQuad, Easing::OutQuad),
            (E::InOutQuad, Easing::InOutQuad),
            (E::InCubic, Easing::InCubic),
            (E::OutCubic, Easing::OutCubic),
            (E::InOutCubic, Easing::InOutCubic),
        ] {
            assert_eq!(
                transition(&Motion::Timed { duration_ms: 150.0, easing: lss }).description(),
                TransitionDescription::Timed { duration: Duration::from_millis(150), easing: sdk }
            );
        }
    }

    #[test]
    fn new_easings_run_from_zero_to_one() {
        use crate::motion::{Easing, MotionState, Transition};
        use std::time::Duration;
        for easing in [Easing::InQuad, Easing::OutQuad, Easing::InOutQuad, Easing::InCubic] {
            let run = Transition::timed(Duration::from_millis(100), easing).start(MotionState::default(), 1.0);
            let mid = run.sample(Duration::from_millis(50)).state.value;
            assert!(mid > 0.0 && mid < 1.0, "{easing:?} midpoint {mid}");
            let end = run.sample(Duration::from_millis(100));
            assert!(end.complete && (end.state.value - 1.0).abs() < 1e-6, "{easing:?}");
        }
    }

    fn number_of(part: &xgpui_luma_style::PartView<'_>, name: &str) -> Option<f32> {
        part.value(name).and_then(Value::as_number)
    }
}
