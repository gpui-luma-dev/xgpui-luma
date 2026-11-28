pub(crate) mod palette;
mod mode;
mod plot;
pub(crate) use mode::{id as mode_id, mode_for_id};
pub(crate) use plot::SdkCartesianStyle;

use std::sync::Arc;

use gpui::{Styled, px};
use xgpui_luma::controls::frame::FrameBuilder;
use xgpui_luma::style::lss::{Overrides, PartId, Prop};
use xgpui_luma::style::{StyleBinding, lss_color};
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
    FrameBuilder::new("plot-viz-scroll-surface").style_source(move |_| {
        let background = lss_color(look.tokens().palette.app.background);
        StyleBinding::sdk("frame").overrides(Overrides::new().set(PartId::ROOT, Prop::new("background"), background))
    })
}

/// General decorated frame; content owns its own rendering and presentation.
pub(crate) fn panel_frame(look: &AppTheme, id: &'static str) -> FrameBuilder {
    let look = look.clone();
    FrameBuilder::new(id).style_source(move |_| {
        let (root, palette) = (PartId::ROOT, &look.tokens().palette);
        StyleBinding::sdk("frame").overrides(
            Overrides::new()
                .set(root, Prop::new("background"), lss_color(palette.surface.panel.background))
                .set(root, Prop::new("border"), lss_color(palette.border.default))
                .set(root, Prop::<f32>::new("border_width"), 1.0)
                .set(root, Prop::<f32>::new("radius"), 8.0),
        )
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
