//! The popover's own panel, from `popover.lss` (#397): its `panel` part, drawn while `show_panel` is on. The
//! SDK leaves it off, so content brings its own chrome (a card, a menu); a look turns it on.

use std::sync::Arc;

use crate::style::PartRead;
use xgpui_luma_style::{Overrides, PartId, Prop, Resolved};

use crate::style::StyleBinding;

/// The popover panel's style for `binding` (a `popover` control) (#433): the popover draws its panel from the
/// `panel` part, while the root's `show_panel` is on.
pub fn resolve_popover_panel(binding: &StyleBinding) -> Arc<Resolved> {
    binding.resolve(&[], 1.0)
}

/// `binding` with its panel turned on or off (`show_panel`).
pub fn show_popover_panel(binding: StyleBinding, shown: bool) -> StyleBinding {
    let show = Overrides::new().set(PartId::ROOT, Prop::<f32>::new("show_panel"), if shown { 1.0 } else { 0.0 });
    binding.with_overrides(&show)
}

/// Whether `resolved` (see [`resolve_popover_panel`]) draws its panel.
pub fn popover_panel_shown(resolved: &Resolved) -> bool {
    resolved.root().number("show_panel") != 0.0
}

/// The height the panel `resolved` describes adds around its content: padding and border on both edges, or
/// nothing when the panel is off.
pub fn popover_panel_chrome_height(resolved: &Resolved) -> f32 {
    if !popover_panel_shown(resolved) {
        return 0.0;
    }
    let panel = resolved.part(PartId::new("panel"));
    2.0 * (panel.number("padding") + panel.number("border_width"))
}

/// `content` inside the panel `resolved` describes; just `content` while the panel is off (`show_panel`). The
/// popover draws its content through this, and a page can draw the panel inline with it.
pub fn wrap_popover_panel(resolved: &Resolved, content: impl gpui::IntoElement) -> gpui::AnyElement {
    use gpui::{div, prelude::*, px};
    if !popover_panel_shown(resolved) {
        return content.into_any_element();
    }
    let panel = resolved.part(PartId::new("panel"));
    div()
        .debug_selector(|| "popover-panel".into())
        .bg(panel.color("background"))
        .text_color(panel.color("foreground"))
        .border(px(panel.number("border_width")))
        .border_color(panel.color("border"))
        .rounded(px(panel.number("radius")))
        .shadow(panel.box_shadow())
        .p(px(panel.number("padding")))
        .child(content)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The SDK's popover panel is off, but its part is a real floating surface for a look to turn on (#397).
    #[test]
    fn the_sdk_panel_is_off_and_is_a_floating_surface() {
        let binding = StyleBinding::sdk("popover");
        let resolved = resolve_popover_panel(&binding);
        assert!(!popover_panel_shown(&resolved), "off in the SDK");
        assert!(popover_panel_shown(&resolve_popover_panel(&show_popover_panel(binding, true))));
        let panel = resolved.part(PartId::new("panel"));
        let menu_style = StyleBinding::sdk("floating_menu").resolve(&[], 1.0);
        let menu = menu_style.root();
        assert_eq!(
            (panel.color("background"), panel.color("foreground"), panel.color("border")),
            (menu.color("background"), menu.color("foreground"), menu.color("border")),
            "a floating surface's colors"
        );
        assert!(panel.color("background").a > 0.0 && !panel.box_shadow().is_empty());
        assert!(panel.number("border_width") > 0.0 && panel.number("radius") > 0.0 && panel.number("padding") > 0.0);
    }
}
