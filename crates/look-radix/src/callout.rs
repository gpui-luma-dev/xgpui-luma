//! Soft informational callout built from the current Radix accent scale.

use gpui::{Div, SharedString, div, prelude::*, px, svg};
use xgpui_luma::controls::button::ControlIcon;

use crate::{Look, ScaleFamily};

/// Paint a callout using accent 3 for its surface and accent 11 for text and icon.
/// Applications supply content and assets; spacing and colors belong to the look.
pub fn callout(look: &Look, text: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> Div {
    let foreground = look.resolve_step(ScaleFamily::Color, 11).hsla();
    let path = match icon.into() {
        ControlIcon::Lucide(icon) => SharedString::from(icon.asset_path()),
        ControlIcon::SvgPath(path) => path,
    };
    let geometry = resolve_geometry(look);
    div()
        .flex()
        .items_center()
        .gap(px(geometry.gap))
        .w_full()
        .rounded_lg()
        .p(px(geometry.padding))
        .bg(look.resolve_step(ScaleFamily::Color, 3).hsla())
        .child(div().flex_none().child(svg().path(path).size(px(geometry.icon_size)).text_color(foreground)))
        .child(div().text_size(px(geometry.font_size)).text_color(foreground).child(text.into()))
}

pub(crate) fn resolve_geometry(look: &Look) -> xgpui_luma::theme::stylesheet::CalloutGeometry {
    let metrics = look.metrics();
    let typography = xgpui_luma::theme::ThemeTokens::default().typography.text.label;
    look.common_stylesheet().callout.resolve_geometry(
        "",
        xgpui_luma::theme::stylesheet::CalloutGeometry {
            padding: metrics.spacing.s4,
            gap: metrics.control.md.gap,
            icon_size: metrics.control.md.icon_size,
            font_size: typography.size,
        },
    )
}

/// Native common geometry owned by this control.
pub(crate) fn native_callout_stylesheet() -> xgpui_luma::theme::stylesheet::CalloutStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    CalloutStylesheet {
        geometry: CalloutGeometryStylesheet {
            padding: Some(12.0),
            gap: Some(12.0),
            icon_size: Some(15.0),
            font_size: Some(14.0),
        },
        ..Default::default()
    }
}
