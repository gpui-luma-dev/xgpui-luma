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
    use xgpui_luma::style::PartRead;
    let resolved = callout_style(look).resolve();
    let style = resolved.root();
    div()
        .flex()
        .items_center()
        .gap(px(style.number("gap")))
        .w_full()
        .rounded(px(style.number("radius")))
        .p(px(style.number("padding_x")))
        .bg(style.color("background"))
        .child(
            div()
                .flex_none()
                .child(svg().path(path).size(px(style.number("icon_size"))).text_color(style.color("foreground"))),
        )
        .child(
            div()
                .text_size(px(style.typography(14.0, 20.0).size))
                .text_color(style.color("foreground"))
                .child(text.into()),
        )
}

/// The callout's `.lss` (`style/callout.lss`) for `look` (#433).
pub fn callout_style(look: &Look) -> xgpui_luma::controls::callout::StyledCallout {
    xgpui_luma::controls::callout::StyledCallout::new(xgpui_luma::style::StyleBinding::new(
        "callout",
        crate::textfield::live_source(look),
    ))
}
