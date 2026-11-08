//! Bare icon chrome: no face, border or shadow in any interaction state.

use gpui::SharedString;
use xgpui_luma::controls::button::{Button, ButtonBuilder};
use xgpui_luma_look_radix::{self as radix, Look};

/// Bind the inner SDK button face, rather than only clearing the outer wrapper.
pub(crate) fn ghost_no_hover<D: Clone + 'static>(
    id: impl Into<SharedString>,
    look: &Look,
    data: D,
) -> ButtonBuilder<D> {
    let geometry = radix::button_look_for(
        look,
        radix::ButtonVariant::GhostQuiet,
        radix::Paint::accent(),
        radix::ButtonSize::Two,
        radix::Radius::None,
    );
    Button::new(id)
        .typed(data)
        .template(radix::button_template(look, radix::ButtonVariant::GhostQuiet, radix::Paint::accent()))
        .with_look(move |model| {
            let mut look = geometry(model);
            look.background = gpui::transparent_black();
            look.border = None;
            look.shadow = None;
            look
        })
}
