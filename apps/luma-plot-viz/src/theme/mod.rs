pub(crate) mod palette;
mod mode;
mod plot;
pub(crate) use mode::{id as mode_id, mode_for_id};
pub(crate) use plot::SdkCartesianStyle;

use std::sync::Arc;

use gpui::{Edges, Styled, px};
use xgpui_luma::controls::frame::{FrameBuilder, FrameLook};
use xgpui_luma::theme::{LumaChrome, LumaTextStyle, LumaTypography};
mod live;
pub(crate) use live::AppTheme;

pub(crate) fn default_theme() -> Arc<AppTheme> {
    Arc::new(AppTheme::new())
}

pub(crate) fn chrome(look: &AppTheme) -> LumaChrome {
    let palette = &look.tokens().palette;
    LumaChrome {
        app_background: palette.app.background,
        content_background: palette.app.background,
        panel_background: palette.surface.panel.background,
        border: palette.border.default,
        title_text: palette.app.foreground,
        body_text: palette.app.foreground,
        muted_text: palette.app.muted_foreground,
    }
}

/// Fixed host surface for the app's scroll viewport.
pub(crate) fn scroll_surface(look: &AppTheme) -> FrameBuilder {
    let look = look.clone();
    FrameBuilder::new("plot-viz-scroll-surface")
        .look_provider(move |_| FrameLook { background: look.tokens().palette.app.background, ..FrameLook::default() })
}

/// General decorated frame; content owns its own rendering and presentation.
pub(crate) fn panel_frame(look: &AppTheme, id: &'static str) -> FrameBuilder {
    let look = look.clone();
    FrameBuilder::new(id).look_provider(move |_| FrameLook {
        background: look.tokens().palette.surface.panel.background,
        border_color: look.tokens().palette.border.default,
        border_widths: Edges::all(px(1.0)),
        radius: px(8.0),
        ..FrameLook::default()
    })
}

pub(crate) fn typography() -> LumaTypography {
    LumaTypography::default()
}

pub(crate) trait TypographyExt: Styled + Sized {
    fn typography_style(self, style: LumaTextStyle) -> Self {
        self.text_size(px(style.size)).line_height(px(style.line_height)).font_weight(style.weight)
    }
}

impl<T: Styled> TypographyExt for T {}

pub(crate) mod points;

pub(crate) mod pie;

pub(crate) mod radar;

pub(crate) mod radial;
