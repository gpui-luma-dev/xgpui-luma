//! Theme tokens for `xgpui-luma-style` styles (issue #282 spike).
//!
//! [`ThemeTokenSource`] answers token paths from a [`ThemeTokens`] snapshot.
//! [`tok`] holds typed handles for those paths.
//!
//! Paths:
//! - `palette.<role>`: any [`PaletteRole`](crate::theme::stylesheet::PaletteRole)
//!   name, read from the family's recipe palette;
//! - `metrics.border_width`, `metrics.disabled_opacity`,
//!   `metrics.focus.{gap,width,offset}`,
//!   `metrics.control.{radius,height,control_height,padding_x,padding_y,gap,icon_size}`;
//! - `typography.font.{sans,mono,serif}`;
//! - `typography.text.{body,label,caption,title,code}.{size,line_height,weight}`;
//! - `geometry.label_baseline`.

use gpui::Hsla;
use xgpui_luma_style::{Dyn, TokenSource};

use crate::theme::stylesheet::PaletteRole;
use crate::theme::{LumaPalette, LumaTextStyle, LumaTypography, MetricTokens, ThemeTokens};

/// Token lookup over a theme snapshot, with the palette recipes of one control
/// family applied.
pub struct ThemeTokenSource<'a> {
    tokens: &'a ThemeTokens,
    palette: LumaPalette<Hsla>,
    label_baseline: f32,
}

impl<'a> ThemeTokenSource<'a> {
    /// Tokens as the control `family` (e.g. `"checkbox"`) sees them.
    pub fn for_family(tokens: &'a ThemeTokens, family: &str) -> Self {
        Self {
            palette: tokens.stylesheet.recipes.palette(family, &tokens.palette),
            label_baseline: tokens.stylesheet.shared_geometry().label_baseline,
            tokens,
        }
    }
}

fn text_style(style: &LumaTextStyle, field: &str) -> Option<Dyn> {
    match field {
        "size" => Some(Dyn::Number(style.size)),
        "line_height" => Some(Dyn::Number(style.line_height)),
        "weight" => Some(Dyn::Number(style.weight.0)),
        _ => None,
    }
}

impl TokenSource for ThemeTokenSource<'_> {
    fn token(&self, path: &str) -> Option<Dyn> {
        if let Some(role) = path.strip_prefix("palette.") {
            return PaletteRole::from_path(role).map(|role| Dyn::Color(role.read(&self.palette)));
        }
        base_token(path, &self.tokens.metrics, &self.tokens.typography, self.label_baseline)
    }
}

/// The `metrics.*`, `typography.*` and `geometry.label_baseline` paths, for
/// token sources that hold these values outside a [`ThemeTokens`] (looks).
pub fn base_token(path: &str, metrics: &MetricTokens, typography: &LumaTypography, label_baseline: f32) -> Option<Dyn> {
    let number = |value: f32| Some(Dyn::Number(value));
    match path {
        "metrics.border_width" => number(metrics.border_width),
        "metrics.disabled_opacity" => number(metrics.disabled_opacity),
        "metrics.focus.gap" => number(metrics.focus.gap),
        "metrics.focus.width" => number(metrics.focus.width),
        "metrics.focus.offset" => number(metrics.focus.offset),
        "metrics.control.radius" => number(metrics.control.radius),
        "metrics.control.height" => number(metrics.control.height),
        "metrics.control.control_height" => number(metrics.control.control_height),
        "metrics.control.padding_x" => number(metrics.control.padding_x),
        "metrics.control.padding_y" => number(metrics.control.padding_y),
        "metrics.control.gap" => number(metrics.control.gap),
        "metrics.control.icon_size" => number(metrics.control.icon_size),
        "typography.font.sans" => Some(Dyn::Text(typography.font.sans.family.clone().into())),
        "typography.font.mono" => Some(Dyn::Text(typography.font.mono.family.clone().into())),
        "typography.font.serif" => Some(Dyn::Text(typography.font.serif.family.clone().into())),
        "geometry.label_baseline" => number(label_baseline),
        _ => {
            let (role, field) = path.strip_prefix("typography.text.")?.split_once('.')?;
            let text = &typography.text;
            let style = match role {
                "body" => &text.body,
                "label" => &text.label,
                "caption" => &text.caption,
                "title" => &text.title,
                "code" => &text.code,
                _ => return None,
            };
            text_style(style, field)
        }
    }
}

/// Typed token handles. Paths are checked by `every_handle_resolves` in the tests.
pub mod tok {
    macro_rules! tokens {
        ($($(#[$meta:meta])* $name:ident: $ty:ident = $path:literal;)+) => {
            $($(#[$meta])* pub const $name: xgpui_luma_style::Token<xgpui_luma_style::$ty> = xgpui_luma_style::Token::new($path);)+
            #[cfg(test)]
            pub(crate) const ALL: &[&str] = &[$($path),+];
        };
    }

    pub mod palette {
        tokens! {
            APP_FOREGROUND: Color = "palette.app.foreground";
            APP_MUTED_FOREGROUND: Color = "palette.app.muted_foreground";
            FOCUS_RING: Color = "palette.focus.ring";
            FORM_INPUT_BACKGROUND: Color = "palette.form.input.background";
            FORM_INPUT_BORDER: Color = "palette.form.input.border";
            STATE_HOVER_BACKGROUND: Color = "palette.state.hover.background";
            STATE_PRESSED_BACKGROUND: Color = "palette.state.pressed.background";
            STATE_SELECTED_BACKGROUND: Color = "palette.state.selected.background";
            STATE_SELECTED_FOREGROUND: Color = "palette.state.selected.foreground";
            STATE_DISABLED_BACKGROUND: Color = "palette.state.disabled.background";
            STATE_DISABLED_FOREGROUND: Color = "palette.state.disabled.foreground";
            BORDER_DEFAULT: Color = "palette.border.default";
            SURFACE_SUBTLE_BACKGROUND: Color = "palette.surface.subtle.background";
        }
    }

    pub mod metrics {
        tokens! {
            BORDER_WIDTH: Px = "metrics.border_width";
            DISABLED_OPACITY: Opacity = "metrics.disabled_opacity";
            RADIUS: Px = "metrics.control.radius";
            CONTROL_HEIGHT: Px = "metrics.control.control_height";
            /// The control row height (taller than `CONTROL_HEIGHT`).
            HEIGHT: Px = "metrics.control.height";
            GAP: Px = "metrics.control.gap";
            PADDING_X: Px = "metrics.control.padding_x";
            PADDING_Y: Px = "metrics.control.padding_y";
            ICON_SIZE: Px = "metrics.control.icon_size";
        }
    }

    pub mod typography {
        tokens! {
            SANS: FontFamily = "typography.font.sans";
            MONO: FontFamily = "typography.font.mono";
            BODY_SIZE: Px = "typography.text.body.size";
            BODY_LINE_HEIGHT: Px = "typography.text.body.line_height";
            LABEL_SIZE: Px = "typography.text.label.size";
            LABEL_LINE_HEIGHT: Px = "typography.text.label.line_height";
            LABEL_WEIGHT: Weight = "typography.text.label.weight";
        }
    }

    pub mod geometry {
        tokens! {
            LABEL_BASELINE: Px = "geometry.label_baseline";
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_handle_resolves() {
        for tokens in [ThemeTokens::light(), ThemeTokens::dark()] {
            let source = ThemeTokenSource::for_family(&tokens, "checkbox");
            for path in tok::palette::ALL
                .iter()
                .chain(tok::metrics::ALL)
                .chain(tok::typography::ALL)
                .chain(tok::geometry::ALL)
            {
                assert!(source.token(path).is_some(), "token `{path}` does not resolve");
            }
        }
    }

    #[test]
    fn unknown_paths_are_none() {
        let tokens = ThemeTokens::light();
        let source = ThemeTokenSource::for_family(&tokens, "checkbox");
        assert_eq!(source.token("palette.nope"), None);
        assert_eq!(source.token("typography.text.body.nope"), None);
        assert_eq!(source.token("metrics.nope"), None);
    }
}
