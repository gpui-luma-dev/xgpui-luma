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
    scrollbar::{ScrollbarTheme, StyledScrollbarTheme},
    selector::{SelectorTheme, StyledSelectorTheme},
    tabs::{StyledTabsTheme, TabsTheme},
    tooltip::TooltipTheme,
};
use xgpui_luma::style::ThemeSource;
use xgpui_luma::theme::{InteractionState, MetricTokens, ThemeMode, ThemeTokens};

struct Snapshot {
    tokens: ThemeTokens,
    scrollbar: StyledScrollbarTheme,
    selector: StyledSelectorTheme,
    tabs: StyledTabsTheme,
}

impl Snapshot {
    /// `style` is the SDK's `.lss` theme for the same mode as `tokens`.
    fn new(tokens: ThemeTokens, style: xgpui_luma::style::lss::Theme) -> Self {
        let binding = |control| xgpui_luma::style::StyleBinding::new(control, ThemeSource::Fixed(style.clone()));
        Self {
            scrollbar: StyledScrollbarTheme::new(binding("scrollbar"), tokens.metrics),
            tabs: StyledTabsTheme::new(binding("tabs")),
            selector: StyledSelectorTheme::new(
                ThemeSource::Fixed(style),
                Arc::new(|| xgpui_luma::style::StyleBinding::sdk("floating_menu")),
            ),
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

    fn style(&self) -> xgpui_luma::style::StyleBinding {
        self.current().tabs.style()
    }
    fn font_family(&self) -> SharedString {
        self.current().tabs.font_family()
    }
}

impl SelectorTheme for AppTheme {
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        self.current().selector.style()
    }
    fn menu(&self) -> xgpui_luma::style::StyleBinding {
        self.current().selector.menu()
    }
    fn metrics(&self) -> MetricTokens {
        self.metrics()
    }
}

impl ScrollbarTheme for AppTheme {
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        self.current().scrollbar.style()
    }
    fn metrics(&self) -> MetricTokens {
        self.metrics()
    }
}

impl TooltipTheme for AppTheme {
    /// The SDK tooltip in the current mode.
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        xgpui_luma::style::StyleBinding::new("tooltip", ThemeSource::Fixed(self.style_theme()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::controls::selector::SelectorTriggerStyle;
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
            let label = |binding: &xgpui_luma::style::StyleBinding| {
                use xgpui_luma::style::PartRead;
                xgpui_luma::controls::tabs::resolve_tab(binding, true, state)
                    .part(xgpui_luma::style::lss::PartId::new("item"))
                    .color("foreground")
            };
            assert_eq!(label(&tabs.style()), label(&StyledTabsTheme::new(sdk("tabs")).style()));
            let selector = xgpui_luma::controls::selector::resolve_selector_trigger(
                &selectors.style(),
                SelectorTriggerStyle::default(),
                xgpui_luma::controls::selector::SelectorVisualState { interaction: state, ..Default::default() },
                1.0,
            );
            let selector_background = {
                use xgpui_luma::style::PartRead;
                selector.root().color("background")
            };
            // The selector trigger is the SDK's base `.lss` trigger for the mode: the outline variant is a
            // look's, not the SDK's (#287).
            let expected = tokens.palette.surface.subtle.background.to_rgb();
            let got = selector_background.to_rgb();
            assert!((got.r - expected.r).abs() < 2e-3, "selector follows the mode: {got:?} != {expected:?}");
            let thumb = |binding: &xgpui_luma::style::StyleBinding| {
                use xgpui_luma::controls::scrollbar::{ScrollbarStyle, resolve_scrollbar};
                use xgpui_luma::style::PartRead;
                resolve_scrollbar(binding, state, ScrollbarStyle::Soft).root().color("thumb")
            };
            let scrollbar_thumb = thumb(&scrollbars.style());
            assert_eq!(scrollbar_thumb, thumb(&StyledScrollbarTheme::new(sdk("scrollbar"), tokens.metrics).style()));
            let tip = {
                use xgpui_luma::style::PartRead;
                xgpui_luma::controls::tooltip::resolve_tooltip(&tooltip.style()).root().color("background")
            };
            let expected =
                tokens.stylesheet.recipes.palette("tooltip", &tokens.palette).surface.floating.background.to_rgb();
            let got = tip.to_rgb();
            assert!((got.r - expected.r).abs() < 2e-3, "tooltip follows the mode: {got:?} != {expected:?}");
            colors.push([
                button.foreground,
                radio_label,
                label(&tabs.style()),
                selector_background,
                scrollbar_thumb,
                tip,
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
