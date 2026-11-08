//! Display formatting for inspection source metadata.

use xgpui_luma::theme::InteractionLayer;
use crate::{ColorSource, MetricSource, TypographySource};

/// Full CSS custom-property form, e.g. `--accent/50`.
pub fn format_color_source(source: &ColorSource) -> String {
    match source {
        ColorSource::Transparent => "transparent".to_string(),
        ColorSource::CssVar { token } => format!("--{token}"),
        ColorSource::TokenAlpha { token, alpha_percent } => format!("--{token}/{alpha_percent}"),
        ColorSource::StateVar { base_token, layer } => {
            format!("--{}", state_var_token(base_token, *layer))
        }
        ColorSource::Algorithmic { base_token, layer } => {
            format!("--{} · algorithmic · {}", base_token, layer_label(*layer))
        }
        ColorSource::Derived { note } => note.clone(),
    }
}

/// CSS custom property that exists in the loaded theme file (grep target for the inspector).
pub fn format_inspect_css_key(source: &ColorSource) -> String {
    match source {
        ColorSource::Transparent => "transparent".to_string(),
        ColorSource::CssVar { token } => format!("--{token}"),
        ColorSource::TokenAlpha { token, alpha_percent } => format!("--{token}/{alpha_percent}"),
        ColorSource::StateVar { base_token, layer } => {
            format!("--{}", state_var_token(base_token, *layer))
        }
        ColorSource::Algorithmic { base_token, .. } => format!("--{base_token}"),
        ColorSource::Derived { note } => note.trim_start_matches("= ").to_string(),
    }
}

/// Primary source label for metric inspector rows (CSS token, scaffold path, or derivation).
pub fn format_inspect_metric_source(source: &MetricSource) -> String {
    match source {
        MetricSource::CssVar { token } => format!("--{token}"),
        MetricSource::Scaffold { path } => format!("scaffold · {path}"),
        MetricSource::Constant { label } => format!("constant · {label}"),
        MetricSource::Derived { note } => metric_derived_css_key(note).unwrap_or_else(|| note.clone()),
    }
}

/// Secondary provenance line for metrics when the source line alone is ambiguous.
pub fn format_inspect_metric_provenance(source: &MetricSource) -> Option<String> {
    match source {
        MetricSource::Derived { note } => Some(note.clone()),
        MetricSource::CssVar { token } => Some(format!("catalog · --{token}")),
        MetricSource::Scaffold { .. } | MetricSource::Constant { .. } => None,
    }
}

/// Primary source label for typography inspector rows.
pub fn format_inspect_typography_source(source: &TypographySource) -> String {
    match source {
        TypographySource::Constant { label } => label.clone(),
        TypographySource::Derived { note } => note.clone(),
        TypographySource::CssVar { token } => format!("--{token}"),
        TypographySource::Scaffold { path } => format!("scaffold · {path}"),
    }
}

/// Secondary provenance line for typography when the source is a catalog token.
pub fn format_inspect_typography_provenance(source: &TypographySource) -> Option<String> {
    match source {
        TypographySource::CssVar { token } => Some(format!("catalog · --{token}")),
        TypographySource::Scaffold { .. } | TypographySource::Constant { .. } => None,
        TypographySource::Derived { note } => Some(note.clone()),
    }
}

pub fn format_typography_px(value: f32) -> String {
    format_metric_px(value)
}

pub fn format_font_weight(value: gpui::FontWeight) -> String {
    format!("{}", value.0)
}

pub fn format_metric_px(value: f32) -> String {
    if (value - value.round()).abs() < f32::EPSILON {
        format!("{}px", value.round() as i32)
    } else {
        format!("{value}px")
    }
}

fn metric_derived_css_key(note: &str) -> Option<String> {
    note.split_whitespace()
        .find(|word| word.starts_with("--"))
        .map(|token| token.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '-').to_string())
}

/// How the resolved color relates to the css key (algorithmic state, inheritance, etc.).
pub fn format_inspect_provenance(source: &ColorSource) -> Option<String> {
    match source {
        ColorSource::Algorithmic { layer, .. } => Some(format!("algorithmic · {}", layer_label(*layer))),
        ColorSource::Derived { note } if note.starts_with("= ") => {
            Some(format!("inherits · {}", note.trim_start_matches("= ")))
        }
        ColorSource::Derived { note } => Some(note.clone()),
        ColorSource::StateVar { layer, .. } => Some(format!("catalog · {}", layer_label(*layer))),
        _ => None,
    }
}

/// Shadcn/Tailwind-style token reference for cross-checking theme CSS, e.g. `accent/50` or `primary-hover`.
pub fn format_css_style_ref(source: &ColorSource) -> String {
    match source {
        ColorSource::Transparent => "transparent".to_string(),
        ColorSource::CssVar { token } => token.clone(),
        ColorSource::TokenAlpha { token, alpha_percent } => format!("{token}/{alpha_percent}"),
        ColorSource::StateVar { base_token, layer } => state_var_token(base_token, *layer),
        ColorSource::Algorithmic { base_token, layer } => {
            format!("{} ({})", state_var_token(base_token, *layer), layer_label(*layer))
        }
        ColorSource::Derived { note } => note.clone(),
    }
}

fn state_var_token(base_token: &str, layer: InteractionLayer) -> String {
    match layer {
        InteractionLayer::Default => base_token.to_string(),
        InteractionLayer::Hovered => format!("{base_token}-hover"),
        InteractionLayer::Pressed => format!("{base_token}-pressed"),
        InteractionLayer::Disabled => format!("{base_token}-disabled"),
    }
}

fn layer_label(layer: InteractionLayer) -> &'static str {
    match layer {
        InteractionLayer::Default => "default",
        InteractionLayer::Hovered => "hover",
        InteractionLayer::Pressed => "pressed",
        InteractionLayer::Disabled => "disabled",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn format_inspect_css_key_points_at_catalog_token_for_algorithmic_states() {
        assert_eq!(
            format_inspect_css_key(&ColorSource::Algorithmic {
                base_token: "primary".into(),
                layer: InteractionLayer::Pressed,
            }),
            "--primary"
        );
        assert_eq!(
            format_inspect_provenance(&ColorSource::Algorithmic {
                base_token: "primary".into(),
                layer: InteractionLayer::Pressed,
            }),
            Some("algorithmic · pressed".to_string())
        );
        assert_eq!(
            format_inspect_css_key(&ColorSource::StateVar {
                base_token: "primary".into(),
                layer: InteractionLayer::Hovered,
            }),
            "--primary-hover"
        );
        assert_eq!(
            format_inspect_provenance(&ColorSource::StateVar {
                base_token: "primary".into(),
                layer: InteractionLayer::Hovered,
            }),
            Some("catalog · hover".to_string())
        );
        assert_eq!(format_inspect_css_key(&ColorSource::Derived { note: "= --primary".into() }), "--primary");
        assert_eq!(
            format_inspect_provenance(&ColorSource::Derived { note: "= --primary".into() }),
            Some("inherits · --primary".to_string())
        );
        assert!(format_inspect_provenance(&ColorSource::CssVar { token: "ring".into() }).is_none());
    }

    #[test]
    fn format_inspect_metric_source_uses_catalog_token_for_radius_derivation() {
        let source = MetricSource::Derived { note: "md = --radius − 2px".into() };
        assert_eq!(format_inspect_metric_source(&source), "--radius");
        assert_eq!(format_inspect_metric_provenance(&source), Some("md = --radius − 2px".to_string()));
        assert_eq!(
            format_inspect_metric_source(&MetricSource::Scaffold { path: "MetricTokens.control.md.padding_x".into() }),
            "scaffold · MetricTokens.control.md.padding_x"
        );
    }

    #[test]
    fn format_metric_px_rounds_whole_numbers() {
        assert_eq!(format_metric_px(14.0), "14px");
        assert_eq!(format_metric_px(1.5), "1.5px");
    }

    #[test]
    fn format_css_style_ref_uses_shorthand_notation() {
        assert_eq!(
            format_css_style_ref(&ColorSource::TokenAlpha { token: "accent".into(), alpha_percent: 50.0 }),
            "accent/50"
        );
        assert_eq!(format_css_style_ref(&ColorSource::CssVar { token: "primary".into() }), "primary");
        assert_eq!(
            format_css_style_ref(&ColorSource::StateVar {
                base_token: "secondary".into(),
                layer: InteractionLayer::Hovered,
            }),
            "secondary-hover"
        );
    }
}
