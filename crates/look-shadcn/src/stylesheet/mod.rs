pub(crate) mod config;
mod find;
mod resolve;
mod selector;
mod size_presets;

pub fn embedded_stylesheet() -> &'static StylesheetConfig {
    embedded()
}

pub use config::StylesheetConfig;
pub use find::*;
pub use resolve::{
    resolve_button_metrics_rule, resolve_sidebar_container_color_rule, resolve_sidebar_metrics, resolve_color_ref,
    resolve_stylesheet_metric, resolve_typography_rule, ResolvedFields,
};

use std::sync::OnceLock;

fn embedded() -> &'static StylesheetConfig {
    static STYLESHEET: OnceLock<StylesheetConfig> = OnceLock::new();
    STYLESHEET.get_or_init(|| {
        let mut config = StylesheetConfig::native_default();
        // Shadcn's common slider geometry over the SDK baseline (what `define_shadcn_slider` applied).
        native_shadcn_common_slider().apply_to(&mut config.common.slider);
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
        button: crate::controls::button::native_button_stylesheet(),
        tooltip: crate::tooltip::native_tooltip_stylesheet(),
        sidebar: crate::controls::sidebar::native_sidebar_stylesheet(),
        ..Default::default()
    };
    size_presets::apply_size_presets(&mut common);
    common.with_sdk_defaults()
}

pub use crate::controls::slider::native_shadcn_common_slider;
