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
    use xgpui_luma::style::lss::{Color, Overrides, PartId, Prop, ShadowLayer};
    let clear = xgpui_luma::style::lss_color(gpui::transparent_black());
    let no_shadow: std::sync::Arc<[ShadowLayer]> = std::sync::Arc::from(Vec::new());
    let template = radix::button_cell_template::<D>(
        look,
        radix::ButtonVariant::GhostQuiet,
        radix::Paint::accent(),
        radix::ButtonSize::Two,
        radix::Radius::None,
    )
    .with_overrides(
        &Overrides::new()
            .set(PartId::ROOT, Prop::<Color>::new("background"), clear)
            .set(PartId::ROOT, Prop::<Color>::new("border"), clear)
            .set(PartId::ROOT, Prop::<std::sync::Arc<[ShadowLayer]>>::new("shadow"), no_shadow),
    );
    Button::new(id).typed(data).template(std::sync::Arc::new(template))
}
