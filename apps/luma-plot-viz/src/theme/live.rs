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
    button_family::{
        ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, ButtonFamilyTheme, DefaultButtonFamilyTheme,
    },
    radio_button::{DefaultRadioButtonTheme, RadioButtonPalette, RadioButtonTheme, RadioScale},
    scrollbar::{DefaultScrollbarTheme, ScrollbarLook, ScrollbarOrientation, ScrollbarStyle, ScrollbarTheme},
    selector::{
        DefaultSelectorTheme, SelectorLook, SelectorPalette, SelectorTheme, SelectorTriggerStyle, SelectorVisualState,
    },
    tabs::{DefaultTabsTheme, TabsItemLook, TabsListLook, TabsTheme},
    tooltip::{TooltipLook, TooltipTheme},
};
use xgpui_luma::theme::{ControlSize, InteractionState, MetricTokens, StandardBoxScale, ThemeMode, ThemeTokens};

struct Snapshot {
    tokens: ThemeTokens,
    button: DefaultButtonFamilyTheme,
    radio: DefaultRadioButtonTheme,
    scrollbar: DefaultScrollbarTheme,
    selector: DefaultSelectorTheme,
    tabs: DefaultTabsTheme,
    tooltip: TooltipLook,
}

impl Snapshot {
    fn new(tokens: ThemeTokens) -> Self {
        Self {
            button: DefaultButtonFamilyTheme::new(tokens.clone()),
            radio: DefaultRadioButtonTheme::new(tokens.clone()),
            scrollbar: DefaultScrollbarTheme::new(tokens.clone()),
            selector: DefaultSelectorTheme::new(tokens.clone()),
            tabs: DefaultTabsTheme::new(tokens.clone()),
            tooltip: TooltipLook::from_tokens(&tokens, "md"),
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
            light: Snapshot::new(ThemeTokens::light()),
            dark_snapshot: Snapshot::new(ThemeTokens::dark()),
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

impl ButtonFamilyTheme for AppTheme {
    fn resolve_look(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
        scale: &StandardBoxScale,
        pill_radius: f32,
    ) -> Option<ButtonFamilyLook> {
        self.current().button.resolve_look(role, size, state, scale, pill_radius)
    }

    fn resolve(&self, role: ButtonFamilyRole, size: ControlSize, state: InteractionState) -> ButtonFamilyPalette {
        self.current().button.resolve(role, size, state)
    }
    fn metrics(&self) -> MetricTokens {
        self.metrics()
    }
}

impl RadioButtonTheme for AppTheme {
    fn resolve(&self, checked: bool, state: InteractionState, size: ControlSize) -> RadioButtonPalette {
        self.current().radio.resolve(checked, state, size)
    }
    fn metrics(&self) -> MetricTokens {
        self.metrics()
    }
    fn scale(&self, size: ControlSize, scale_factor: f32) -> RadioScale {
        self.current().radio.scale(size, scale_factor)
    }
}

impl TabsTheme for AppTheme {
    fn resolve_drag(&self, size: ControlSize) -> xgpui_luma::infra::drag_drop::DragDropLook {
        self.current().tabs.resolve_drag(size)
    }

    fn resolve_list(&self, enabled: bool, size: ControlSize) -> TabsListLook {
        self.current().tabs.resolve_list(enabled, size)
    }
    fn resolve_item(&self, active: bool, state: InteractionState, size: ControlSize) -> TabsItemLook {
        self.current().tabs.resolve_item(active, state, size)
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
        size: ControlSize,
        scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        self.current().selector.resolve_look(style, state, size, scale, without_elevation)
    }
    fn resolve_visual_look(
        &self,
        style: SelectorTriggerStyle,
        state: SelectorVisualState,
        size: ControlSize,
        scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        self.current().selector.resolve_visual_look(style, state, size, scale, without_elevation)
    }
}

impl ScrollbarTheme for AppTheme {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: ScrollbarOrientation,
        size: ControlSize,
        style: ScrollbarStyle,
    ) -> ScrollbarLook {
        self.current().scrollbar.resolve(state, orientation, size, style)
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

    #[test]
    fn retained_control_themes_follow_mode_and_preserve_sdk_snapshots() {
        let theme = AppTheme::new();
        let buttons: Arc<dyn ButtonFamilyTheme> = Arc::new(theme.clone());
        let radios: Arc<dyn RadioButtonTheme> = Arc::new(theme.clone());
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
            let button = buttons.resolve(ButtonFamilyRole::Text, ControlSize::Md, state);
            let expected_button =
                DefaultButtonFamilyTheme::new(tokens.clone()).resolve(ButtonFamilyRole::Text, ControlSize::Md, state);
            assert_eq!(button.background, expected_button.background);
            let radio = radios.resolve(true, state, ControlSize::Md);
            assert_eq!(
                radio.dot_color,
                DefaultRadioButtonTheme::new(tokens.clone()).resolve(true, state, ControlSize::Md).dot_color
            );
            let tab = tabs.resolve_item(true, state, ControlSize::Md);
            assert_eq!(
                tab.label_color,
                DefaultTabsTheme::new(tokens.clone()).resolve_item(true, state, ControlSize::Md).label_color
            );
            let selector = selectors.resolve(SelectorTriggerStyle::default(), state, false);
            assert_eq!(
                selector.trigger_background,
                DefaultSelectorTheme::new(tokens.clone())
                    .resolve(SelectorTriggerStyle::default(), state, false)
                    .trigger_background
            );
            let scrollbar =
                scrollbars.resolve(state, ScrollbarOrientation::Horizontal, ControlSize::Md, ScrollbarStyle::Soft);
            assert_eq!(
                scrollbar.thumb_background,
                DefaultScrollbarTheme::new(tokens.clone())
                    .resolve(state, ScrollbarOrientation::Horizontal, ControlSize::Md, ScrollbarStyle::Soft)
                    .thumb_background
            );
            let tip = tooltip.resolve();
            assert_eq!(tip.background, TooltipLook::from_tokens(&tokens, "md").background);
            colors.push([
                button.foreground,
                radio.label_color,
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
