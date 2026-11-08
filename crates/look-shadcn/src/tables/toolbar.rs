//! Toolbar palette resolution shared by runtime and inspection.
use xgpui_luma::controls::toolbar::ToolbarVariant;
use xgpui_luma::theme::ThemeMode;
use crate::{ColorSource, LookContext, ResolvedColor, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct ToolbarColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub separator: ResolvedColor,
}

pub fn resolve_toolbar_colors(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
    variant: ToolbarVariant,
) -> ToolbarColorTable {
    let ctx = LookContext::new(mode, theme_mode, Default::default());
    let tokens = ctx.tokens;
    let separator = resolved_from_hsla(tokens.palette.border_default, ColorSource::CssVar { token: "border".into() });

    let (background, border) = match variant {
        ToolbarVariant::Ghost => (ResolvedColor::transparent(), ResolvedColor::transparent()),
        ToolbarVariant::Outline => {
            let background_value = if enabled {
                tokens.palette.muted_background
            } else {
                tokens.palette.disabled_background
            };
            let background_source = if enabled {
                ColorSource::CssVar { token: "muted".into() }
            } else {
                ColorSource::Derived { note: "muted · disabled".into() }
            };
            (
                resolved_from_hsla(background_value, background_source),
                resolved_from_hsla(tokens.palette.border_default, ColorSource::CssVar { token: "border".into() }),
            )
        }
    };

    ToolbarColorTable { background, border, separator }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}
