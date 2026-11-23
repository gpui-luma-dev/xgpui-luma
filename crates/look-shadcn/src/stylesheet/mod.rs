pub(crate) mod config;
mod find;
mod metadata;
mod resolve;
mod selector;
mod size_presets;

pub fn embedded_stylesheet() -> &'static StylesheetConfig {
    embedded()
}

pub use config::{ButtonElevationRule, FloatingMenuSurfaceElevationRule, LayeredElevationRule, StylesheetConfig};
pub use find::*;
pub use metadata::*;
pub use resolve::{
    resolve_button_color_rule, resolve_button_metrics_rule, resolve_sidebar_container_color_rule,
    resolve_sidebar_metrics, resolve_progress_metrics, resolve_color_ref, resolve_stepper_metrics,
    resolve_scrollbar_color_rule, resolve_scrollbar_metrics, resolve_slider_metrics, resolve_stylesheet_shadow_token,
    resolve_layered_elevation_shadow, resolve_stylesheet_metric, resolve_typography_rule, ResolvedFields,
};

use std::sync::OnceLock;

fn embedded() -> &'static StylesheetConfig {
    static STYLESHEET: OnceLock<StylesheetConfig> = OnceLock::new();
    STYLESHEET.get_or_init(|| {
        let mut config = StylesheetConfig::native_default();
        define_shadcn_slider(&mut config);
        config
    })
}

/// Construct the complete Shadcn common overrides using native definitions.
pub fn native_shadcn_common_stylesheet() -> xgpui_luma::theme::stylesheet::CommonStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    let mut motion = CommonStylesheet::sdk_default().motion.clone();
    motion.tabs.content =
        Some(TransitionSpec::Timed { duration_ms: 300, easing: xgpui_luma::motion::Easing::OutCubic });
    let mut common = CommonStylesheet {
        motion,
        slider: native_shadcn_common_slider(),
        tabs: crate::controls::tabs::native_tabs_stylesheet(),
        button: crate::controls::button::native_button_stylesheet(),
        toggle: crate::controls::toggle::native_toggle_stylesheet(),
        scrollbar: crate::controls::scrollbar::native_scrollbar_stylesheet(),
        stepper: crate::controls::stepper::native_stepper_stylesheet(),
        tooltip: crate::tooltip::native_tooltip_stylesheet(),
        sidebar: crate::controls::sidebar::native_sidebar_stylesheet(),
        ..Default::default()
    };
    size_presets::apply_size_presets(&mut common);
    common.with_sdk_defaults()
}

pub use crate::controls::slider::{native_shadcn_common_slider, native_shadcn_slider, define_shadcn_slider};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::BadgeVariant;
    use xgpui_luma::theme::{InteractionLayer, ThemeMode};

    #[test]
    fn enabled_controls_find_matching_rules() {
        let stylesheet = embedded_stylesheet();
        assert!(find_badge_color_rule(stylesheet, BadgeVariant::Default, ThemeMode::Dark).is_some());
        assert!(find_progress_color_rule(stylesheet, true).is_some());
        assert!(find_split_view_color_rule(stylesheet, false).is_some());
        assert!(find_control_group_list_color_rule(stylesheet, true).is_some());
        assert!(find_textfield_color_rule(stylesheet, "input", true, false, ThemeMode::Light).is_some());
        assert!(find_textfield_color_rule(stylesheet, "primary", true, false, ThemeMode::Light).is_some());
        assert!(find_textfield_elevation_rule(stylesheet, crate::controls::ShadcnTextFieldStyle::Primary).is_some());
        assert!(find_autocomplete_chrome_color_rule(stylesheet).is_some());
        assert!(find_slider_color_rule(stylesheet, "primary", InteractionLayer::Hovered).is_some());
        assert!(find_slider_color_rule(stylesheet, "secondary", InteractionLayer::Default).is_some());
    }
}
