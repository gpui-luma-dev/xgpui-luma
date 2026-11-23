//! Soft informational callout built from the current Radix accent scale.

use gpui::{Div, SharedString, div, prelude::*, px, svg};
use xgpui_luma::controls::button::ControlIcon;

use crate::Look;

/// Paint a callout using accent 3 for its surface and accent 11 for text and icon.
/// Applications supply content and assets; spacing and colors belong to the look.
pub fn callout(look: &Look, text: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> Div {
    let path = match icon.into() {
        ControlIcon::Lucide(icon) => SharedString::from(icon.asset_path()),
        ControlIcon::SvgPath(path) => path,
    };
    let style = callout_look(look);
    div()
        .flex()
        .items_center()
        .gap(px(style.gap))
        .w_full()
        .rounded(px(style.radius))
        .p(px(style.padding))
        .bg(style.background)
        .child(div().flex_none().child(svg().path(path).size(px(style.icon_size)).text_color(style.foreground)))
        .child(div().text_size(px(style.font_size)).text_color(style.foreground).child(text.into()))
}

/// The callout from `.lss` (`style/callout.lss`).
pub fn callout_look(look: &Look) -> xgpui_luma::controls::callout::CalloutLook {
    xgpui_luma::controls::callout::StyledCallout::new(xgpui_luma::style::StyleBinding::new(
        "callout",
        crate::textfield::live_source(look),
    ))
    .look()
}
