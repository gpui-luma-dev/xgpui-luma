//! The SDK's styles in the Luma style language (issue #285).
//!
//! [`config`] is the one global configuration every stylesheet shares. [`sources`] lists the SDK's
//! `.lss` files: the token files and one file beside each migrated control. The config declares the
//! SDK's theme modes, `light` and `dark`; `theme/palette.lss` gives light values and `mode dark { … }`
//! overrides. [`default_theme`] is the SDK alone in light mode and [`dark_theme`] in dark mode. Looks
//! layer their own files over [`sources`] and may add modes with `Config::extend`.

use std::sync::{Arc, OnceLock};

#[cfg(test)]
pub(crate) mod fixture;

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
                    // Theme modes; `theme/palette.lss` holds light values and `mode dark` overrides. Looks
                    // add their own (e.g. a high-contrast mode) with `config().extend().modes(…)`.
                    .modes(&["light", "dark"])
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
        const SOURCE_PATHS: &[&str] = &[$($path,)+];
    };
}

sdk_sources! {
    "theme/tokens.lss",
    "theme/palette.lss", "controls/choice/checkbox/checkbox.lss", "controls/choice/radio_button/radio.lss",
        "controls/choice/switch/switch.lss",
        "controls/button/button.lss",
        "controls/button/trigger_button/trigger_button.lss",
        "controls/choice/toggle/toggle.lss",
        "controls/choice/control_group/control_group.lss",
        "controls/choice/radio_group/radio_group.lss",
        "controls/choice/toolbar/toolbar.lss",
        "controls/text/textfield/textfield.lss",
        "controls/text/textarea/textarea.lss",
        "controls/selection/selector/selector.lss",
        "controls/selection/autocomplete/autocomplete.lss",
        "controls/selection/combobox/combobox.lss",
        "controls/selection/search_selector/search_selector.lss",
        "controls/overlay/popover_button/popover.lss",
        "shell/title_bar.lss",
        "controls/overlay/floating_menu/floating_menu.lss",
        "controls/overlay/popup_menu/popup_menu.lss",
        "controls/overlay/context_menu/context_menu.lss",
        "controls/overlay/tooltip/tooltip.lss",
        "controls/overlay/overlay_window/overlay_window.lss",
        "controls/overlay/slide_panel/slide_panel.lss",
        "controls/range/slider/slider.lss",
        "controls/range/progress/progress.lss",
        "controls/navigation/stepper/stepper.lss",
        "controls/navigation/accordion/accordion.lss",
        "controls/navigation/pager/pager.lss",
        "controls/navigation/tabs/tabs.lss",
        "controls/navigation/sidebar/engine/sidebar.lss",
        "infra/drag_drop/drag_preview.lss",
        "controls/collection/tree_view/tree_view.lss",
        "controls/collection/table/table.lss",
        "controls/collection/listbox/listbox.lss",
        "controls/layout/scrollbar/scrollbar.lss",
        "controls/layout/dock_splitter/dock_splitter.lss",
        "controls/layout/resizable_panels/resizable_panels.lss",
        "controls/layout/split_view/split_view.lss",
        "controls/display/avatar.lss",
        "controls/display/badge.lss",
        "controls/display/card.lss",
        "controls/display/callout.lss",
        "controls/display/segmented.lss",
        "controls/display/color_chrome.lss",
        "controls/selection/selection_panel/selection_panel.lss",
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

/// The fade a part draws at in its resolved state: its `opacity` (`:disabled { opacity = … }`), or 1 when
/// unset. Templates apply this once instead of a separate disabled fade (#413).
pub(crate) fn part_opacity(part: xgpui_luma_style::PartView<'_>) -> f32 {
    part.get(xgpui_luma_style::props::OPACITY).map_or(1.0, |o| o.clamp(0.0, 1.0))
}

/// Applies a resolved part opacity ([`part_opacity`]) to `element`, leaving it untouched when opaque. The one
/// place templates apply a state fade.
pub(crate) fn fade<E: gpui::Styled>(element: E, opacity: f32) -> E {
    if opacity < 1.0 {
        element.opacity(opacity)
    } else {
        element
    }
}

/// Disabled parts that both fade and change their fill, which lets whatever is behind the control show through
/// a fill it doesn't normally have (#413). For every control in `theme`, every combination of the non-exclusive
/// SDK states is resolved with and without `disabled`. A part whose disabled `opacity` is below 1 must keep its
/// non-disabled background; a missing background and a transparent one count as the same. Returns one line per
/// problem, so looks can assert it is empty in their tests.
pub fn disabled_fade_problems(theme: &Theme) -> Vec<String> {
    use xgpui_luma_style::Value;
    const STATES: [&str; 7] = ["focused", "checked", "selected", "active", "expanded", "open", "invalid"];
    let fill = |part: xgpui_luma_style::PartView<'_>| {
        part.value("background").and_then(Value::as_color).filter(|color| color.alpha > 0.0)
    };
    let mut problems = Vec::new();
    for control in theme.stylesheet().controls() {
        for mask in 0u32..(1 << STATES.len()) {
            let states: Vec<&str> =
                STATES.iter().enumerate().filter(|(i, _)| mask & (1 << i) != 0).map(|(_, s)| *s).collect();
            let disabled_states: Vec<&str> = states.iter().copied().chain(["disabled"]).collect();
            let (Ok(resting), Ok(disabled)) = (
                theme.key_named(control.name(), &[], &states),
                theme.key_named(control.name(), &[], &disabled_states),
            ) else {
                continue;
            };
            let (resting, disabled) = (theme.resolve(&resting), theme.resolve(&disabled));
            for part in disabled.parts() {
                if part_opacity(disabled.part(part)) < 1.0 && fill(disabled.part(part)) != fill(resting.part(part)) {
                    problems.push(format!("{control} {part:?} {disabled_states:?}: fades and changes its fill"));
                }
            }
        }
    }
    problems
}

/// A number from a control's own properties in the SDK's `.lss` (light mode, no attributes or states),
/// for default trait paths of themes that supply only colors. Zero when the control or property is
/// missing.
pub(crate) fn sdk_number(control: &str, property: &str) -> f32 {
    let theme = default_theme();
    theme
        .key_named(control, &[], &[])
        .ok()
        .and_then(|key| theme.resolve(&key).root().value(property).and_then(xgpui_luma_style::Value::as_number))
        .unwrap_or(0.0)
}

/// A menu's offset from its trigger as a share of the control gap, from the SDK's `.lss`
/// (`menu_offset_y` over `gap`).
pub(crate) fn sdk_menu_offset_share(control: &str) -> f32 {
    let gap = sdk_number(control, "gap");
    if gap > 0.0 {
        sdk_number(control, "menu_offset_y") / gap
    } else {
        0.0
    }
}

/// The SDK alone in dark mode.
pub fn dark_theme() -> Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME
        .get_or_init(|| default_theme().with_mode("dark").unwrap_or_else(|d| panic!("the SDK dark theme binds:\n{d}")))
        .clone()
}

/// The SDK alone in light mode, with no native tokens.
pub fn default_theme() -> Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME
        .get_or_init(|| {
            Theme::bind(&stylesheet(), Arc::new(NoNativeTokens), "light")
                .unwrap_or_else(|d| panic!("the SDK theme binds:\n{d}"))
        })
        .clone()
}

/// The mode named after a scheme: `"light"` or `"dark"`.
pub fn scheme_name(mode: crate::theme::ThemeMode) -> &'static str {
    match mode {
        crate::theme::ThemeMode::Light => "light",
        crate::theme::ThemeMode::Dark => "dark",
    }
}

/// The light or dark scheme a declared mode comes down to through its bases (see
/// [`Config::scheme`]), e.g. light for a look's `hi_contrast` based on `light`. `None` when `config`
/// doesn't declare `mode` or it comes down to neither.
pub fn mode_scheme(config: &Config, mode: &str) -> Option<crate::theme::ThemeMode> {
    if !config.modes().any(|m| m == mode) {
        return None;
    }
    match config.scheme(mode) {
        "light" => Some(crate::theme::ThemeMode::Light),
        "dark" => Some(crate::theme::ThemeMode::Dark),
        _ => None,
    }
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

/// Where a control's styles come from: one theme, or a provider asked on every resolve (a look's live
/// theme, which follows its mode and edits).
#[derive(Clone)]
pub enum ThemeSource {
    Fixed(Theme),
    Live(Arc<dyn Fn() -> Theme + Send + Sync>),
}

impl ThemeSource {
    pub fn theme(&self) -> Theme {
        match self {
            Self::Fixed(theme) => theme.clone(),
            Self::Live(provider) => provider(),
        }
    }
}

/// One control's styles bound for rendering: the theme source, the control, and the attribute
/// values and states every instance has (a look's variant, size or tone). A control's theme adapter
/// adds its interaction states and resolves.
#[derive(Clone)]
pub struct StyleBinding {
    source: ThemeSource,
    control: &'static str,
    attrs: Vec<(xgpui_luma_style::AttrId, xgpui_luma_style::AttrValue)>,
    states: Vec<xgpui_luma_style::StateId>,
    overrides: xgpui_luma_style::Overrides,
}

impl StyleBinding {
    pub fn new(control: &'static str, source: ThemeSource) -> Self {
        Self {
            source,
            control,
            attrs: Vec::new(),
            states: Vec::new(),
            overrides: xgpui_luma_style::Overrides::new(),
        }
    }

    /// The control in the SDK's own theme.
    pub fn sdk(control: &'static str) -> Self {
        Self::new(control, ThemeSource::Fixed(default_theme()))
    }

    pub fn theme(&self) -> Theme {
        self.source.theme()
    }

    /// Shorthand for `attr("variant", variant)`.
    pub fn variant(self, variant: &str) -> Self {
        self.attr("variant", variant)
    }

    /// An attribute value for every instance. An attribute the configuration does not declare is ignored.
    pub fn attr(mut self, name: &str, value: &str) -> Self {
        if let Some(attr) = self.theme().stylesheet().config().attribute(name) {
            self.attrs.retain(|(a, _)| *a != attr);
            self.attrs.push((attr, xgpui_luma_style::AttrValue::named(value)));
        }
        self
    }

    /// A state that is always on or off. A state the configuration does not declare is ignored.
    pub fn state(mut self, name: &str, on: bool) -> Self {
        if let Some(state) = self.theme().stylesheet().config().state(name) {
            self.states.retain(|s| *s != state);
            if on {
                self.states.push(state);
            }
        }
        self
    }

    /// Values set on this instance.
    pub fn overrides(mut self, overrides: xgpui_luma_style::Overrides) -> Self {
        self.overrides = overrides;
        self
    }

    /// Resolve with these interaction states (by name) at `scale` device pixels per pixel.
    pub fn resolve(&self, states: &[(&str, bool)], scale: f32) -> Arc<xgpui_luma_style::Resolved> {
        self.resolve_for(&[], states, scale)
    }

    /// Resolve with extra attribute values for this call (a trigger style picked per render), then the
    /// interaction states. An attribute or value the configuration does not declare is ignored.
    pub fn resolve_for(
        &self,
        attrs: &[(&str, &str)],
        states: &[(&str, bool)],
        scale: f32,
    ) -> Arc<xgpui_luma_style::Resolved> {
        let theme = self.theme();
        let config = theme.stylesheet().config();
        let key = xgpui_luma_style::StyleKey::new(xgpui_luma_style::ControlId::new(self.control)).scale(scale);
        let key = self.states.iter().fold(key, |key, state| key.state(*state));
        let mut key = self.attrs.iter().fold(key, |key, (attr, value)| key.attr(*attr, *value));
        for (name, value) in attrs {
            if let Some(attr) = config.attribute(name) {
                key = key.attr(attr, xgpui_luma_style::AttrValue::named(value));
            }
        }
        for (name, on) in states {
            if let Some(state) = config.state(name) {
                key = key.state_if(state, *on);
            }
        }
        theme.resolve_with(&key, &self.overrides)
    }
}

#[cfg(test)]
mod tests {
    use gpui::Hsla;
    use xgpui_luma_style::Value;

    use super::*;

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

    /// The fixture stands in for the shipped token files: every control binds and checks clean
    /// against it, so it defines every token the controls read, with the right kinds.
    #[test]
    fn fixture_defines_every_token_the_controls_read() {
        for theme in fixture::themes() {
            for control in theme.stylesheet().controls() {
                let problems = theme.check(control);
                assert!(problems.is_empty(), "{control} in {}: {problems}", theme.mode());
            }
        }
    }

    /// The fixture's colors are all different, within a mode and across modes, so a part can only
    /// match a token's color by reading that token.
    #[test]
    fn fixture_colors_are_unique() {
        let text = include_str!("../tests/fixtures/colorful/palette.lss");
        let mut paths = Vec::new();
        let mut stack: Vec<&str> = Vec::new();
        for line in text.lines().map(str::trim).filter(|l| !l.starts_with("//")) {
            if let Some(name) = line.strip_suffix(" {") {
                stack.push(name);
            } else if line == "}" {
                stack.pop();
            } else if let Some((name, value)) = line.split_once(" = ")
                && value.starts_with("hsla(")
                && stack.first() == Some(&"tokens")
                && stack.get(1) == Some(&"palette")
            {
                paths.push(format!("{}.{}", stack[1..].join("."), name.trim()));
            }
        }
        assert!(paths.len() > 40, "the parser found the palette: {paths:?}");
        let mut seen: Vec<(String, gpui::Rgba)> = Vec::new();
        for theme in fixture::themes() {
            for path in &paths {
                let color =
                    fixture::token(&theme, path, "color").as_color().map(|c| Hsla::from(c).to_rgb()).expect("a color");
                let near = seen
                    .iter()
                    .find(|(_, c)| (c.r - color.r).abs() + (c.g - color.g).abs() + (c.b - color.b).abs() < 0.02);
                assert!(near.is_none(), "{path} in {} is too close to {near:?}", theme.mode());
                seen.push((format!("{} {path}", theme.mode()), color));
            }
        }
    }

    /// The default trait paths of themes that supply only colors read these SDK `.lss` properties; a
    /// renamed or missing property would read as zero.
    #[test]
    fn sdk_numbers_read_by_default_theme_paths_exist() {
        let theme = default_theme();
        for (control, property) in [
            ("textfield", "caret_width"),
            ("textfield", "caret_height_extra"),
            ("textarea", "caret_width"),
            ("textarea", "caret_height_extra"),
            ("popup_menu", "menu_offset_y"),
            ("popup_menu", "gap"),
            ("selector", "menu_offset_y"),
            ("selector", "gap"),
        ] {
            let resolved = theme.resolve(&theme.key_named(control, &[], &[]).expect("key"));
            assert!(resolved.root().value(property).is_some(), "{control}.{property}");
        }
        assert!(sdk_menu_offset_share("popup_menu") > 0.0 && sdk_menu_offset_share("selector") > 0.0);
    }

    /// `assert_reads` fails when a part reads a different token, so passing checks mean something.
    #[test]
    #[should_panic(expected = "reads palette.form.input.border")]
    fn assert_reads_rejects_the_wrong_token() {
        fixture::assert_reads(
            &fixture::light(),
            "checkbox",
            &[],
            "indicator",
            "background",
            "palette.form.input.border",
        );
    }

    /// The checkbox's parts read the palette tokens for each state, and its lengths read the metrics.
    #[test]
    fn checkbox_reads_its_tokens() {
        for theme in fixture::themes() {
            let reads = |states: &[&str], part: &str, prop: &str, token: &str| {
                fixture::assert_reads(&theme, "checkbox", states, part, prop, token)
            };
            reads(&[], "indicator", "background", "palette.form.input.background");
            reads(&[], "indicator", "border", "palette.form.input.border");
            reads(&["hover"], "indicator", "background", "palette.state.hover.background");
            reads(&["pressed", "hover"], "indicator", "background", "palette.state.pressed.background");
            reads(&["checked"], "indicator", "background", "palette.state.selected.background");
            reads(&["checked"], "indicator", "border", "palette.state.selected.background");
            reads(&["checked", "hover"], "indicator", "background", "palette.state.selected.background");
            reads(&["checked", "disabled"], "indicator", "background", "palette.state.disabled.background");
            reads(&[], "glyph", "foreground", "palette.app.foreground");
            reads(&["checked"], "glyph", "foreground", "palette.state.selected.foreground");
            reads(&["disabled"], "glyph", "foreground", "palette.state.disabled.foreground");
            reads(&[], "root", "focus_border", "palette.focus.ring");
            reads(&[], "root", "gap", "metrics.control.gap");
            reads(&[], "root", "focus_width", "metrics.focus.width");
        }
    }

    /// The slider and progress tracks are outlined in the default border, so they show on any surface; the
    /// scrollbar track reads the same border but draws none by default (#397).
    #[test]
    fn tracks_can_have_a_border() {
        for theme in fixture::themes() {
            for control in ["slider", "progress"] {
                fixture::assert_reads(&theme, control, &[], "track", "border", "palette.border.default");
            }
        }
        for control in ["slider", "progress"] {
            let binding = StyleBinding::sdk(control);
            let resolved = binding.resolve(&[], 1.0);
            let width = resolved.part(xgpui_luma_style::PartId::new("track")).value("border_width");
            assert!(width.and_then(xgpui_luma_style::Value::as_number).is_some_and(|w| w > 0.0), "{control}");
        }
        // The scrollbar track (its root) can be outlined too, but draws none by default.
        for theme in fixture::themes() {
            fixture::assert_reads(&theme, "scrollbar", &[], "root", "border", "palette.border.default");
        }
        let scrollbar = StyleBinding::sdk("scrollbar").resolve(&[], 1.0);
        assert_eq!(scrollbar.root().value("border_width").and_then(xgpui_luma_style::Value::as_number), Some(0.0));
    }

    /// A table row hovers like every other control, not in a chart color (#397).
    #[test]
    fn table_rows_hover_in_the_shared_hover_color() {
        for theme in fixture::themes() {
            fixture::assert_reads(&theme, "table", &["hover"], "row", "background", "palette.state.hover.background");
        }
    }

    /// An "on" toggle keeps light-on-dark text readable when hovered, and takes normal text on the pressed
    /// gray (#397): `button`'s :hover and :pressed only change the background.
    #[test]
    fn a_selected_toggle_keeps_readable_text_when_hovered_or_pressed() {
        for theme in fixture::themes() {
            let reads = |states: &[&str], prop: &str, token: &str| {
                fixture::assert_reads(&theme, "toggle", states, "root", prop, token)
            };
            reads(&["selected", "hover"], "background", "palette.state.selected.background");
            reads(&["selected", "hover"], "foreground", "palette.state.selected.foreground");
            reads(&["selected", "pressed", "hover"], "background", "palette.state.pressed.background");
            reads(&["selected", "pressed", "hover"], "foreground", "palette.app.foreground");
        }
    }

    /// Choice rows are as tall as their indicator plus 3px above and below (#397), not the shared
    /// control height, so stacked rows sit close.
    #[test]
    fn choice_rows_fit_their_indicator() {
        for control in ["checkbox", "radio"] {
            let (height, indicator) = (sdk_number(control, "height"), sdk_number(control, "indicator_size"));
            assert_eq!(height, indicator + 6.0, "{control}");
        }
    }

    /// The radio's parts read the palette tokens for each state, and its lengths read the metrics.
    #[test]
    fn radio_reads_its_tokens() {
        for theme in fixture::themes() {
            let reads = |states: &[&str], part: &str, prop: &str, token: &str| {
                fixture::assert_reads(&theme, "radio", states, part, prop, token)
            };
            reads(&[], "indicator", "background", "palette.form.input.background");
            reads(&["hover"], "indicator", "background", "palette.state.hover.background");
            reads(&["selected"], "indicator", "border", "palette.state.selected.background");
            reads(&["selected", "pressed"], "indicator", "border", "palette.state.pressed.background");
            reads(&["selected", "disabled"], "indicator", "border", "palette.form.input.border");
            reads(&["selected"], "dot", "foreground", "palette.state.selected.foreground");
            reads(&[], "root", "gap", "metrics.control.gap");
        }
    }

    /// The radio indicator is round: its radius follows its size, whatever the size is.
    #[test]
    fn radio_indicator_is_round() {
        for theme in fixture::themes() {
            let resolved = theme.resolve(&theme.key_named("radio", &[], &[]).expect("key"));
            let root = resolved.root();
            let length = |name: &str| root.value(name).and_then(Value::as_number);
            assert!(length("indicator_size").is_some_and(|s| s > 0.0));
            assert_eq!(length("indicator_radius"), length("indicator_size"));
        }
    }

    /// The switch's parts read the palette and elevation tokens for each state.
    #[test]
    fn switch_reads_its_tokens() {
        for theme in fixture::themes() {
            let reads = |states: &[&str], part: &str, prop: &str, token: &str| {
                fixture::assert_reads(&theme, "switch", states, part, prop, token)
            };
            reads(&[], "track", "background", "palette.form.input.background");
            reads(&[], "track", "shadow", "elevation.control");
            reads(&["checked"], "track", "background", "palette.state.selected.background");
            reads(&["checked"], "track", "border", "palette.state.selected.background");
            reads(&["checked", "disabled"], "track", "background", "palette.state.disabled.background");
            reads(&[], "thumb", "background", "palette.app.background");
            reads(&["checked"], "thumb", "background", "palette.state.selected.foreground");
            reads(&[], "root", "gap", "metrics.control.gap");
        }
    }

    /// Switch lengths snap to the pixel grid at every scale.
    #[test]
    fn switch_lengths_snap_at_every_scale() {
        for theme in fixture::themes() {
            for scale in [1.0, 1.25, 1.5, 2.0] {
                let key = theme.key_named("switch", &[], &[]).expect("key").scale(scale);
                let resolved = theme.resolve(&key);
                let root = resolved.root();
                for name in ["track_width", "track_height", "thumb_size"] {
                    let length = root.value(name).and_then(Value::as_number).expect("a length");
                    assert!(length > 0.0, "{name}@{scale}");
                    assert_eq!(length, crate::theme::snap_to_pixel(length, scale), "{name}@{scale} is snapped");
                }
            }
        }
    }

    /// The checkbox animates with the theme's default motion token.
    #[test]
    fn checkbox_transition_is_the_default_motion() {
        use crate::controls::checkbox::{CheckboxData, default_checkbox_template};
        let theme = default_theme();
        let Value::Motion(motion) = fixture::token(&theme, "motion.default", "motion") else {
            panic!("motion.default is a motion")
        };
        let expected = transition(&motion).description();
        for checked in [false, true] {
            let got = default_checkbox_template().transition(&CheckboxData::new(checked)).expect("transition");
            assert_eq!(got.description(), expected, "checked={checked}");
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

    /// The SDK keeps only behaviour variants (#287): a visual variant the trigger or style enums set
    /// draws the base in the SDK theme, in every state; the looks define the variants.
    #[test]
    fn sdk_draws_the_base_for_visual_variants() {
        let cases = [
            ("pager", "minimal"),
            ("popup_menu", "outline"),
            ("popup_menu", "ghost"),
            ("scrollbar", "soft"),
            ("selector", "outline"),
            ("selector", "ghost"),
            ("toolbar", "ghost"),
        ];
        for theme in [default_theme(), dark_theme()] {
            for (control, variant) in cases {
                let binding = StyleBinding::new(control, ThemeSource::Fixed(theme.clone()));
                for states in [
                    [("hover", false), ("disabled", false)],
                    [("hover", true), ("disabled", false)],
                    [("hover", false), ("disabled", true)],
                ] {
                    assert_eq!(
                        binding.resolve_for(&[("variant", variant)], &states, 1.0),
                        binding.resolve_for(&[], &states, 1.0),
                        "{} {control} [{variant}] {states:?}",
                        theme.mode()
                    );
                }
            }
        }
    }
}
