//! The popover's own panel, from `popover.lss` (#397): its `panel` part, drawn while `show_panel` is on. The
//! SDK leaves it off, so content brings its own chrome (a card, a menu); a look turns it on.

use crate::style::PartRead;
use gpui::{BoxShadow, Hsla};
use xgpui_luma_style::PartId;

use crate::style::StyleBinding;

/// The panel a popover draws around its content.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PopoverPanelLook {
    /// Whether the panel is drawn (`show_panel`); the rest is what it draws.
    pub shown: bool,
    pub background: Hsla,
    /// The content's text color, on the panel's fill.
    pub foreground: Hsla,
    pub border: Hsla,
    pub border_width: f32,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
}

impl PopoverPanelLook {
    /// The panel `binding` (a `popover` control) resolves to.
    pub fn from_binding(binding: &StyleBinding) -> Self {
        let resolved = binding.resolve(&[], 1.0);
        let panel = resolved.part(PartId::new("panel"));
        Self {
            shown: resolved.root().number("show_panel") != 0.0,
            background: panel.color("background"),
            foreground: panel.color("foreground"),
            border: panel.color("border"),
            border_width: panel.number("border_width"),
            shadow: panel.box_shadow(),
            radius: panel.number("radius"),
            padding: panel.number("padding"),
        }
    }

    /// The SDK's panel: off, with a floating surface's values for when a look turns it on.
    pub fn sdk() -> Self {
        static SDK: std::sync::OnceLock<PopoverPanelLook> = std::sync::OnceLock::new();
        SDK.get_or_init(|| Self::from_binding(&StyleBinding::sdk("popover"))).clone()
    }

    /// This look with the panel turned on or off.
    pub fn shown(self, shown: bool) -> Self {
        Self { shown, ..self }
    }

    /// `content` inside the panel this look draws; just `content` while the panel is off (`show_panel`). The
    /// popover draws its content through this, and a page can draw the panel inline with it.
    /// The height the panel adds around its content: padding and border on
    /// both edges, or nothing when the panel is off.
    pub fn chrome_height(&self) -> f32 {
        if self.shown {
            2.0 * (self.padding + self.border_width)
        } else {
            0.0
        }
    }

    pub fn wrap(&self, content: impl gpui::IntoElement) -> gpui::AnyElement {
        use gpui::{div, prelude::*, px};
        if !self.shown {
            return content.into_any_element();
        }
        div()
            .debug_selector(|| "popover-panel".into())
            .bg(self.background)
            .text_color(self.foreground)
            .border(px(self.border_width))
            .border_color(self.border)
            .rounded(px(self.radius))
            .shadow(self.shadow.clone())
            .p(px(self.padding))
            .child(content)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The SDK's popover panel is off, but its part is a real floating surface for a look to turn on (#397).
    #[test]
    fn the_sdk_panel_is_off_and_is_a_floating_surface() {
        let look = PopoverPanelLook::sdk();
        assert!(!look.shown, "off in the SDK");
        let menu = crate::controls::floating_menu::styled_floating_menu_look(&StyleBinding::sdk("floating_menu"));
        assert_eq!(
            (look.background, look.foreground, look.border),
            (menu.background, menu.foreground, menu.border),
            "a floating surface's colors"
        );
        assert!(look.background.a > 0.0 && !look.shadow.is_empty(), "{look:?}");
        assert!(look.border_width > 0.0 && look.radius > 0.0 && look.padding > 0.0, "{look:?}");
    }
}
