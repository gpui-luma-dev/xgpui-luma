//! Shared mode selection for retained SDK controls and chart providers.
//!
//! Only the mode is mutable. Both SDK snapshots and their default themes are
//! prepared once; controls delegate to the selected snapshot at resolve time.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use gpui::SharedString;
use xgpui_luma::controls::{
    button::StyledButtonTemplate,
    radio_button::{RadioButtonData, StyledRadioButtonTemplate},
    scrollbar::{ScrollbarLook, ScrollbarOrientation, ScrollbarStyle, ScrollbarTheme, StyledScrollbarTheme},
    selector::{
        SelectorLook, SelectorPalette, SelectorTheme, SelectorTriggerStyle, SelectorVisualState, StyledSelectorTheme,
    },
    tabs::{StyledTabsTheme, TabsItemLook, TabsListLook, TabsTheme},
    tooltip::{TooltipLook, TooltipTheme},
};
use xgpui_luma::style::ThemeSource;
use xgpui_luma::theme::{InteractionState, MetricTokens, StandardBoxScale, ThemeMode, ThemeTokens};

struct Snapshot {
    tokens: ThemeTokens,
    scrollbar: StyledScrollbarTheme,
    selector: StyledSelectorTheme,
    tabs: StyledTabsTheme,
    tooltip: TooltipLook,
}

impl Snapshot {
    /// `style` is the SDK's `.lss` theme for the same mode as `tokens`.
    fn new(tokens: ThemeTokens, style: xgpui_luma::style::lss::Theme) -> Self {
        let panel_tokens = tokens.clone();
        let binding = |control| xgpui_luma::style::StyleBinding::new(control, ThemeSource::Fixed(style.clone()));
        Self {
            scrollbar: StyledScrollbarTheme::new(binding("scrollbar"), tokens.metrics),
            tabs: StyledTabsTheme::new(binding("tabs")),
            selector: StyledSelectorTheme::new(
                ThemeSource::Fixed(style),
                Arc::new(move || xgpui_luma::controls::selector_list::default_selector_items_panel_look(&panel_tokens)),
            ),
            tooltip: TooltipLook::from_tokens(&tokens, None),
            tokens,
        }
    }
}

struct State {
    dark: AtomicBool,
    light: Snapshot,
    dark_snapshot: Snapshot,
}

/// Clones share the mode, including clones retained by tooltip closures.
#[derive(Clone)]
pub(crate) struct AppTheme(Arc<State>);

impl AppTheme {
    pub(crate) fn new() -> Self {
        Self(Arc::new(State {
            dark: AtomicBool::new(true),
            light: Snapshot::new(ThemeTokens::light(), xgpui_luma::style::default_theme()),
            dark_snapshot: Snapshot::new(ThemeTokens::dark(), xgpui_luma::style::dark_theme()),
        }))
    }

    pub(crate) fn mode(&self) -> ThemeMode {
        if self.0.dark.load(Ordering::Relaxed) {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        }
    }

    pub(crate) fn set_mode(&self, mode: ThemeMode) {
        self.0.dark.store(mode == ThemeMode::Dark, Ordering::Relaxed);
    }

    fn current(&self) -> &Snapshot {
        match self.mode() {
            ThemeMode::Light => &self.0.light,
            ThemeMode::Dark => &self.0.dark_snapshot,
        }
    }

    pub(crate) fn tokens(&self) -> &ThemeTokens {
        &self.current().tokens
    }
    pub(crate) fn metrics(&self) -> MetricTokens {
        self.tokens().metrics
    }
}

impl AppTheme {
    /// The SDK's `.lss` theme for the current mode: its light or its dark palette.
    pub(crate) fn style_theme(&self) -> xgpui_luma::style::lss::Theme {
        match self.mode() {
            ThemeMode::Light => xgpui_luma::style::default_theme(),
            ThemeMode::Dark => xgpui_luma::style::dark_theme(),
        }
    }

    /// The SDK button drawn from `.lss`, following the mode.
    pub(crate) fn button_template<D: xgpui_luma::controls::button::ButtonData>(
        &self,
    ) -> Arc<dyn xgpui_luma::controls::button::ButtonTemplate<D>> {
        let theme = self.clone();
        Arc::new(StyledButtonTemplate::<D>::live(Arc::new(move || theme.style_theme())))
    }

    /// The SDK radio drawn from `.lss`, following the mode.
    pub(crate) fn radio_template(&self) -> Arc<dyn xgpui_luma::controls::button::ButtonTemplate<RadioButtonData>> {
        let theme = self.clone();
        Arc::new(StyledRadioButtonTemplate::live(Arc::new(move || theme.style_theme())))
    }
}

impl TabsTheme for AppTheme {
    fn resolve_drag(&self) -> xgpui_luma::infra::drag_drop::DragDropLook {
        self.current().tabs.resolve_drag()
    }

    fn resolve_list(&self, enabled: bool) -> TabsListLook {
        self.current().tabs.resolve_list(enabled)
    }
    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsItemLook {
        self.current().tabs.resolve_item(active, state)
    }
    fn font_family(&self) -> SharedString {
        self.current().tabs.font_family()
    }
}

impl SelectorTheme for AppTheme {
    fn resolve(
        &self,
        style: SelectorTriggerStyle,
        state: InteractionState,
        without_elevation: bool,
    ) -> SelectorPalette {
        self.current().selector.resolve(style, state, without_elevation)
    }
    fn metrics(&self) -> MetricTokens {
        self.metrics()
    }
    fn resolve_look(
        &self,
        style: SelectorTriggerStyle,
        state: InteractionState,
        scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        self.current().selector.resolve_look(style, state, scale, without_elevation)
    }
    fn resolve_visual_look(
        &self,
        style: SelectorTriggerStyle,
        state: SelectorVisualState,
        scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        self.current().selector.resolve_visual_look(style, state, scale, without_elevation)
    }
}

impl ScrollbarTheme for AppTheme {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: ScrollbarOrientation,
        style: ScrollbarStyle,
    ) -> ScrollbarLook {
        self.current().scrollbar.resolve(state, orientation, style)
    }
    fn metrics(&self) -> MetricTokens {
        self.metrics()
    }
}

impl TooltipTheme for AppTheme {
    fn resolve(&self) -> TooltipLook {
        self.current().tooltip.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::controls::button_family::ButtonFamilyRole;

    #[test]
    fn retained_control_themes_follow_mode_and_preserve_sdk_snapshots() {
        let theme = AppTheme::new();
        let tabs: Arc<dyn TabsTheme> = Arc::new(theme.clone());
        let selectors: Arc<dyn SelectorTheme> = Arc::new(theme.clone());
        let scrollbars: Arc<dyn ScrollbarTheme> = Arc::new(theme.clone());
        let tooltip: Arc<dyn TooltipTheme> = Arc::new(theme.clone());
        let dark_snapshot = theme.tokens().clone();
        let independent = AppTheme::new();
        let mut colors = Vec::new();
        for mode in [ThemeMode::Dark, ThemeMode::Light, ThemeMode::Dark] {
            theme.set_mode(mode);
            let tokens = match mode {
                ThemeMode::Light => ThemeTokens::light(),
                ThemeMode::Dark => ThemeTokens::dark(),
            };
            let state = InteractionState::default();
            let button = StyledButtonTemplate::<()>::live(Arc::new({
                let theme = theme.clone();
                move || theme.style_theme()
            }))
            .look(ButtonFamilyRole::Text, state, 1.0);
            let style = theme.style_theme();
            let sdk = |control| xgpui_luma::style::StyleBinding::new(control, ThemeSource::Fixed(style.clone()));
            let expected_button =
                StyledButtonTemplate::<()>::new(style.clone()).look(ButtonFamilyRole::Text, state, 1.0);
            assert_eq!(button.background, expected_button.background);
            let radio_label = theme
                .style_theme()
                .resolve(&xgpui_luma::style::lss::StyleKey::new(xgpui_luma::style::lss::ControlId::new("radio")))
                .part(xgpui_luma::style::lss::PartId::new("label"))
                .foreground()
                .map(gpui::Hsla::from)
                .expect("radio label");
            let expected = tokens.palette.app.foreground.to_rgb();
            let got = radio_label.to_rgb();
            assert!((got.r - expected.r).abs() < 2e-3, "radio label follows the mode: {got:?} != {expected:?}");
            let tab = tabs.resolve_item(true, state);
            assert_eq!(tab.label_color, StyledTabsTheme::new(sdk("tabs")).resolve_item(true, state).label_color);
            let selector = selectors.resolve(SelectorTriggerStyle::default(), state, false);
            // The selector trigger is the SDK's base `.lss` trigger for the mode: the outline variant is a
            // look's, not the SDK's (#287).
            let expected = tokens.palette.surface.subtle.background.to_rgb();
            let got = selector.trigger_background.to_rgb();
            assert!((got.r - expected.r).abs() < 2e-3, "selector follows the mode: {got:?} != {expected:?}");
            let scrollbar = scrollbars.resolve(state, ScrollbarOrientation::Horizontal, ScrollbarStyle::Soft);
            assert_eq!(
                scrollbar.thumb_background,
                StyledScrollbarTheme::new(sdk("scrollbar"), tokens.metrics)
                    .resolve(state, ScrollbarOrientation::Horizontal, ScrollbarStyle::Soft)
                    .thumb_background
            );
            let tip = tooltip.resolve();
            assert_eq!(tip.background, TooltipLook::from_tokens(&tokens, None).background);
            colors.push([
                button.foreground,
                radio_label,
                tab.label_color,
                selector.trigger_background,
                scrollbar.thumb_background,
                tip.background,
            ]);
        }
        for i in 0..colors[0].len() {
            assert_ne!(colors[0][i], colors[1][i], "control {i} retained stale colors");
        }
        assert_eq!(colors[0], colors[2]);
        assert_eq!(dark_snapshot.palette.app.background, theme.tokens().palette.app.background);
        assert_eq!(independent.mode(), ThemeMode::Dark);
    }
}
