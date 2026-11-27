//! The card read from `.lss` (issue #285): `card.lss`. Looks render the element from [`CardLook`].

use crate::style::PartRead;
use gpui::{AnyElement, BoxShadow, Div, Hsla, IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma_style::{PartId, PartView, Value};

use crate::style::StyleBinding;
use crate::theme::LumaTextStyle;

/// A card's surface, text and spacing.
#[derive(Clone, Debug)]
pub struct CardLook {
    /// The surface fill, when it has color.
    pub background: Option<Hsla>,
    /// The edge, when it has color.
    pub border: Option<Hsla>,
    pub border_width: f32,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub section_gap: f32,
    pub header_gap: f32,
    pub body_gap: f32,
    pub title_color: Hsla,
    pub description_color: Hsla,
    pub body_color: Hsla,
    pub title: LumaTextStyle,
    pub description: LumaTextStyle,
    pub body: LumaTextStyle,
    pub font_family: SharedString,
}

fn text(part: PartView<'_>) -> LumaTextStyle {
    part.typography(14.0, 20.0)
}

fn opaque(color: Hsla) -> Option<Hsla> {
    (color.a > 0.0).then_some(color)
}

/// The card's `.lss` values for one binding (the `card` control, with the look's variant and size).
#[derive(Clone)]
pub struct StyledCard {
    binding: StyleBinding,
}

impl StyledCard {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyleBinding::sdk("card"))
    }

    /// The card for the binding's attributes and states.
    pub fn look(&self) -> CardLook {
        let resolved = self.binding.resolve(&[], 1.0);
        let root = resolved.root();
        let (title, description) = (resolved.part(PartId::new("title")), resolved.part(PartId::new("description")));
        CardLook {
            background: opaque(root.color("background")),
            border: opaque(root.color("border")),
            border_width: root.number("border_width"),
            shadow: root
                .value("shadow")
                .and_then(|v| match v {
                    Value::Shadow(layers) => Some(crate::style::lss::box_shadows(layers)),
                    _ => None,
                })
                .unwrap_or_default(),
            radius: root.number("radius"),
            padding: root.number("padding_x"),
            section_gap: root.number("section_gap"),
            header_gap: root.number("header_gap"),
            body_gap: root.number("body_gap"),
            title_color: title.color("foreground"),
            description_color: description.color("foreground"),
            body_color: root.color("foreground"),
            title: text(title),
            description: text(description),
            body: text(root),
            font_family: root.font_family_name(),
        }
    }
}

/// A card drawn from `.lss`: an optional header (title, description) above body content. Looks keep their
/// own card elements; this is the SDK's.
pub struct Card {
    title: Option<SharedString>,
    description: Option<SharedString>,
    children: Vec<AnyElement>,
    styled: StyledCard,
}

impl Card {
    /// A card from the SDK's `.lss` alone.
    pub fn new() -> Self {
        Self { title: None, description: None, children: Vec::new(), styled: StyledCard::sdk() }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Draw from `styled` (a look's binding, variant or size) instead of the SDK alone.
    pub fn styled(mut self, styled: StyledCard) -> Self {
        self.styled = styled;
        self
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

fn text_block(text: SharedString, color: Hsla, style: &LumaTextStyle) -> Div {
    div()
        .text_color(color)
        .text_size(px(style.size))
        .line_height(px(style.line_height))
        .font_weight(style.weight)
        .child(text)
}

impl IntoElement for Card {
    type Element = Div;

    fn into_element(self) -> Div {
        let look = self.styled.look();
        let header = (self.title.is_some() || self.description.is_some()).then(|| {
            div()
                .flex()
                .flex_col()
                .gap(px(look.header_gap))
                .children(self.title.map(|t| text_block(t, look.title_color, &look.title)))
                .children(self.description.map(|d| text_block(d, look.description_color, &look.description)))
        });
        let body =
            (!self.children.is_empty()).then(|| div().flex().flex_col().gap(px(look.body_gap)).children(self.children));
        div()
            .flex()
            .flex_col()
            .gap(px(look.section_gap))
            .p(px(look.padding))
            .rounded(px(look.radius))
            .when_some(look.background, |card, background| card.bg(background))
            .when_some(look.border, |card, border| card.border(px(look.border_width)).border_color(border))
            .shadow(look.shadow.clone())
            .when(!look.font_family.is_empty(), |card| card.font_family(look.font_family.clone()))
            .text_color(look.body_color)
            .text_size(px(look.body.size))
            .line_height(px(look.body.line_height))
            .font_weight(look.body.weight)
            .children(header)
            .children(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_card_is_an_edged_panel_with_header_text() {
        let look = StyledCard::sdk().look();
        assert!(look.background.is_some() && look.border.is_some());
        assert!(look.title.size > look.description.size);
        assert!(look.padding > 0.0 && look.section_gap > 0.0 && look.header_gap >= 2.0);
    }
}
