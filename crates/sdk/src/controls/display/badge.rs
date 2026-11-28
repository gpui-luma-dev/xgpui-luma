//! The badge read from `.lss` (issue #285): `badge.lss`.

use crate::style::PartRead;
use gpui::{Div, IntoElement, SharedString, div, prelude::*, px};

use crate::style::StyleBinding;

/// The badge's `.lss` values for one binding (the `badge` control, with the look's variant and size).
#[derive(Clone)]
pub struct StyledBadge {
    binding: StyleBinding,
}

impl StyledBadge {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyleBinding::sdk("badge"))
    }

    /// The badge's style for the binding's attributes and states (#433): the badge draws from its root.
    pub fn resolve(&self) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        self.binding.resolve(&[], 1.0)
    }
}

/// A badge drawn from `.lss`: a short label in the badge's colors and frame. Looks keep their own badge
/// elements; this is the SDK's.
pub struct Badge {
    label: SharedString,
    styled: StyledBadge,
}

impl Badge {
    /// A badge from the SDK's `.lss` alone.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), styled: StyledBadge::sdk() }
    }

    /// Draw from `styled` (a look's binding, variant or size) instead of the SDK alone.
    pub fn styled(mut self, styled: StyledBadge) -> Self {
        self.styled = styled;
        self
    }
}

impl IntoElement for Badge {
    type Element = Div;

    fn into_element(self) -> Div {
        let resolved = self.styled.resolve();
        let root = resolved.root();
        let (typography, font_family, border) =
            (root.typography(12.0, 16.0), root.font_family_name(), root.painted("border"));
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(root.number("gap")))
            .min_h(px(root.number("min_height")))
            .px(px(root.number("padding_x")))
            .py(px(root.number("padding_y")))
            .rounded(px(root.number("radius")))
            .bg(root.color("background"))
            .text_color(root.color("foreground"))
            .when(!font_family.is_empty(), |badge| badge.font_family(font_family))
            .font_weight(typography.weight)
            .text_size(px(typography.size))
            .line_height(px(typography.line_height))
            .when_some(border, |badge, border| badge.border(px(root.number("border_width"))).border_color(border))
            .child(self.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_badge_is_an_outlined_pill_with_label_text() {
        let resolved = StyledBadge::sdk().resolve();
        let look = resolved.root();
        assert_eq!(look.number("radius"), 999.0);
        assert!(
            look.painted("border").is_some_and(|border| border.a > 0.0),
            "an edge sets it apart from a subtle surface"
        );
        assert_eq!(look.number("min_height"), look.typography(12.0, 16.0).line_height + 2.0 * look.number("padding_y"));
        assert!(look.color("background").a > 0.0 && look.color("foreground").a > 0.0);
    }
}
