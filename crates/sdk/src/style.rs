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
    (shared: [$($path:literal),+ $(,)?], light: $light:literal, dark: $dark:literal $(,)?) => {
        fn shared_sources() -> Vec<Source> {
            vec![$(Source::new(concat!("sdk/", $path), include_str!($path))),+]
        }

        /// The SDK's style files, for the `sdk` layer: the shared files and the light palette.
        pub fn sources() -> Vec<Source> {
            let mut sources = shared_sources();
            sources.push(Source::new(concat!("sdk/", $light), include_str!($light)));
            sources
        }

        /// The SDK's files with the dark palette instead of the light one.
        pub fn dark_sources() -> Vec<Source> {
            let mut sources = shared_sources();
            sources.push(Source::new(concat!("sdk/", $dark), include_str!($dark)));
            sources
        }

        #[cfg(test)]
        const SOURCE_PATHS: &[&str] = &[$($path,)+ $light, $dark];
    };
}

sdk_sources! {
    shared: ["theme/tokens.lss", "controls/choice/checkbox/checkbox.lss", "controls/choice/radio_button/radio.lss",
        "controls/choice/switch/switch.lss",
        "controls/button/button.lss",
        "controls/choice/toggle/toggle.lss",
    ],
    light: "theme/palette.lss",
    dark: "theme/palette-dark.lss",
}

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

/// The SDK's files with the dark palette, compiled on their own.
pub fn dark_stylesheet() -> Stylesheet {
    static SHEET: OnceLock<Stylesheet> = OnceLock::new();
    SHEET
        .get_or_init(|| {
            Stylesheet::builder(config())
                .layer("sdk", dark_sources())
                .compile()
                .unwrap_or_else(|d| panic!("the SDK dark style files compile:\n{d}"))
        })
        .clone()
}

/// The SDK alone with its dark palette (another SDK theme, not a mode).
pub fn dark_theme() -> Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME
        .get_or_init(|| {
            Theme::bind(&dark_stylesheet(), Arc::new(NoNativeTokens), "")
                .unwrap_or_else(|d| panic!("the SDK dark theme binds:\n{d}"))
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
    use xgpui_luma_style::{ControlId, PartId, StyleKey, Value};

    use super::*;
    use crate::theme::ThemeTokens;

    fn close(a: Hsla, b: Hsla) -> bool {
        let (a, b) = (a.to_rgb(), b.to_rgb());
        [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
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
        for theme in [default_theme(), dark_theme()] {
            for control in theme.stylesheet().controls() {
                let problems = theme.check(control);
                assert!(problems.is_empty(), "{control}: {problems}");
            }
        }
    }

    #[test]
    fn palettes_match_theme_tokens_light_and_dark() {
        tokens_match(ThemeTokens::light(), default_theme());
        tokens_match(ThemeTokens::dark(), dark_theme());
    }

    fn tokens_match(native: ThemeTokens, theme: Theme) {
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
            let e = &native.elevation;
            let shadows = [
                ("elevation.control", &e.control),
                ("elevation.thumb", &e.thumb),
                ("elevation.menu", &e.menu),
                ("elevation.popover", &e.popover),
                ("elevation.panel", &e.panel),
                ("elevation.dialog", &e.dialog),
                ("elevation.overlay_window", &e.overlay_window),
                ("elevation.drag_preview", &e.drag_preview),
            ];
            let decls: String = colors
                .iter()
                .enumerate()
                .map(|(i, (path, _))| format!("  c{i}: color = {path}\n"))
                .chain(shadows.iter().enumerate().map(|(i, (path, _))| format!("  s{i}: shadow = {path}\n")))
                .collect();
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
            for (i, (path, expected)) in shadows.iter().enumerate() {
                let got = resolved.root().value(&format!("s{i}")).cloned();
                let Some(Value::Shadow(layers)) = got else {
                    panic!("{path}: {got:?}")
                };
                assert_eq!(layers.len(), expected.layers.len(), "{path}");
                for (g, w) in layers.iter().zip(&expected.layers) {
                    assert!(close(Hsla::from(g.color), w.color), "{path}: {g:?} != {w:?}");
                    assert_eq!(
                        (g.x, g.y, g.blur, g.spread, g.inset),
                        (w.offset_x, w.offset_y, w.blur, w.spread, w.inset),
                        "{path}"
                    );
                }
            }
            std::collections::HashMap::new()
        };
        assert!(values.is_empty());
    }

    /// The SDK checkbox's `.lss` values (they matched `DefaultCheckboxTheme` when it was deleted).
    #[test]
    fn checkbox_values_follow_the_sdk_tokens() {
        let palette = ThemeTokens::light().palette;
        let theme = default_theme();
        let config = config();
        let state = |name: &str| config.state(name).expect("state");
        let checkbox = ControlId::new("checkbox");
        let resolve =
            |states: &[&str]| theme.resolve(&states.iter().fold(StyleKey::new(checkbox), |key, s| key.state(state(s))));
        let background = |states: &[&str]| resolve(states).part(PartId::new("indicator")).background().map(Hsla::from);
        let expect = |got: Option<Hsla>, want: Hsla, what: &str| {
            assert!(got.is_some_and(|g| close(g, want)), "{what}: {got:?} != {want:?}");
        };
        expect(background(&[]), palette.form.input.background, "unchecked");
        expect(background(&["hover"]), palette.state.hover.background, "hover");
        expect(background(&["pressed", "hover"]), palette.state.pressed.background, "pressed");
        expect(background(&["checked"]), palette.state.selected.background, "checked");
        expect(background(&["checked", "disabled"]), palette.state.disabled.background, "disabled");
        let rest = resolve(&[]);
        let root = rest.root();
        let length = |name: &str| root.value(name).and_then(Value::as_number);
        assert_eq!(length("indicator_size"), Some(18.0));
        assert_eq!(length("glyph_size"), Some(12.0));
        assert_eq!(length("indicator_radius"), Some(4.0));
        assert_eq!((root.height(), root.gap()), (Some(36.0), Some(8.0)));
    }

    /// The SDK radio's `.lss` values in both SDK themes (they matched `DefaultRadioButtonTheme` when it
    /// was deleted).
    #[test]
    fn radio_values_follow_the_sdk_tokens() {
        for (tokens, theme) in [(ThemeTokens::light(), default_theme()), (ThemeTokens::dark(), dark_theme())] {
            let palette = tokens.palette;
            let config = config();
            let state = |name: &str| config.state(name).expect("state");
            let resolve = |states: &[&str]| {
                theme.resolve(&states.iter().fold(StyleKey::new(ControlId::new("radio")), |key, s| key.state(state(s))))
            };
            let color = |states: &[&str], part: &'static str, border: bool| {
                let resolved = resolve(states);
                let part = resolved.part(PartId::new(part));
                if border {
                    part.border()
                } else {
                    part.background().or(part.foreground())
                }
                .map(Hsla::from)
            };
            let expect = |got: Option<Hsla>, want: Hsla, what: &str| {
                assert!(got.is_some_and(|g| close(g, want)), "{what}: {got:?} != {want:?}");
            };
            expect(color(&[], "indicator", false), palette.form.input.background, "unselected");
            expect(color(&["hover"], "indicator", false), palette.state.hover.background, "hover");
            expect(color(&["selected"], "indicator", true), palette.state.selected.background, "selected ring");
            expect(
                color(&["selected", "pressed"], "indicator", true),
                palette.state.pressed.background,
                "pressed ring",
            );
            expect(color(&["selected", "disabled"], "indicator", true), palette.form.input.border, "disabled ring");
            expect(color(&["selected"], "dot", false), palette.state.selected.foreground, "dot");
            let rest = resolve(&[]);
            let root = rest.root();
            let length = |name: &str| root.value(name).and_then(Value::as_number);
            assert_eq!((length("indicator_size"), length("dot_size")), (Some(18.0), Some(9.0)));
            assert_eq!(length("indicator_radius"), Some(18.0), "round");
        }
    }

    /// The SDK switch's `.lss` values in both SDK themes (they matched `DefaultSwitchTheme` when it was
    /// deleted).
    #[test]
    fn switch_values_follow_the_sdk_tokens() {
        for (tokens, theme) in [(ThemeTokens::light(), default_theme()), (ThemeTokens::dark(), dark_theme())] {
            let palette = tokens.palette;
            let config = config();
            let state = |name: &str| config.state(name).expect("state");
            let resolve = |states: &[&str], scale: f32| {
                let key = states.iter().fold(StyleKey::new(ControlId::new("switch")), |key, s| key.state(state(s)));
                theme.resolve(&key.scale(scale))
            };
            let color = |states: &[&str], part: &'static str, border: bool| {
                let resolved = resolve(states, 1.0);
                let part = resolved.part(PartId::new(part));
                if border {
                    part.border()
                } else {
                    part.background().or(part.foreground())
                }
                .map(Hsla::from)
            };
            let expect = |got: Option<Hsla>, want: Hsla, what: &str| {
                assert!(got.is_some_and(|g| close(g, want)), "{what}: {got:?} != {want:?}");
            };
            expect(color(&[], "track", false), palette.form.input.background, "off track");
            expect(color(&["checked"], "track", false), palette.state.selected.background, "on track");
            expect(color(&["checked"], "track", true), palette.state.selected.background, "on track border");
            expect(color(&["checked", "disabled"], "track", false), palette.state.disabled.background, "disabled");
            expect(color(&[], "thumb", false), palette.app.background, "off thumb");
            expect(color(&["checked"], "thumb", false), palette.state.selected.foreground, "on thumb");
            for scale in [1.0, 1.25, 1.5, 2.0] {
                let resolved = resolve(&[], scale);
                let root = resolved.root();
                let length = |name: &str| root.value(name).and_then(Value::as_number);
                let snap = |v: f32| crate::theme::snap_to_pixel(v, scale);
                assert_eq!(length("track_width"), Some(snap(40.0)), "@{scale}");
                assert_eq!(length("track_height"), Some(snap(18.0)), "@{scale}");
                assert_eq!(length("thumb_size"), Some(snap(14.0)), "@{scale}");
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
}
