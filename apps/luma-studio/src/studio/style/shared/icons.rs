use gpui::{AnyElement, Hsla, IntoElement, div, prelude::*, px};
use lucide_svg_static::Icon as LucideIcon;

pub(crate) fn render_lucide_icon(icon: LucideIcon, color: Hsla, size: f32) -> AnyElement {
    div()
        .text_size(px(size))
        .line_height(px(size))
        .child(xgpui_luma::infra::icon::lucide_icon(icon, color, size))
        .into_any_element()
}
