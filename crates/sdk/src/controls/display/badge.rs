//! The badge read from `.lss` (issue #285): `badge.lss`. Looks render the element from [`BadgeLook`].

use gpui::{Div, Hsla, IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma_style::{PartView, Value};

use crate::style::StyleBinding;
use crate::theme::LumaTextStyle;

/// A badge's colors, frame and text.
#[derive(Clone, Debug, PartialEq)]
pub struct BadgeLook {
    pub background: Hsla,
    pub foreground: Hsla,
    /// The edge, when it has color.
    pub border: Option<Hsla>,
    pub border_width: f32,
    pub radius: f32,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub icon_size: f32,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

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

    /// The badge for the binding's attributes and states.
    pub fn look(&self) -> BadgeLook {
        let resolved = self.binding.resolve(&[], 1.0);
        let root = resolved.root();
        let border = color(root, "border");
        BadgeLook {
            background: color(root, "background"),
            foreground: color(root, "foreground"),
            border: (border.a > 0.0).then_some(border),
            border_width: number(root, "border_width"),
            radius: number(root, "radius"),
            min_height: number(root, "min_height"),
            padding_x: number(root, "padding_x"),
            padding_y: number(root, "padding_y"),
            gap: number(root, "gap"),
            icon_size: number(root, "icon_size"),
            typography: LumaTextStyle {
                size: root.font_size().unwrap_or(12.0),
                line_height: root.line_height().unwrap_or(16.0),
                weight: gpui::FontWeight(number(root, "font_weight").max(100.0)),
            },
            font_family: root.font_family().unwrap_or("").to_string().into(),
        }
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
        let look = self.styled.look();
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(look.gap))
            .min_h(px(look.min_height))
            .px(px(look.padding_x))
            .py(px(look.padding_y))
            .rounded(px(look.radius))
            .bg(look.background)
            .text_color(look.foreground)
            .when(!look.font_family.is_empty(), |badge| badge.font_family(look.font_family.clone()))
            .font_weight(look.typography.weight)
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .when_some(look.border, |badge, border| badge.border(px(look.border_width)).border_color(border))
            .child(self.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_badge_is_a_pill_with_label_text() {
        let look = StyledBadge::sdk().look();
        assert_eq!(look.radius, 999.0);
        assert!(look.border.is_none());
        assert_eq!(look.min_height, look.typography.line_height + 2.0 * look.padding_y);
        assert!(look.background.a > 0.0 && look.foreground.a > 0.0);
    }
}
