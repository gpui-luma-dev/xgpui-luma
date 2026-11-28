//! Look-owned, non-interactive content container with Radix Card treatments.
//! Size metrics follow the Radix Themes Card; application content stays external.
use std::sync::Arc;
use gpui::{Hsla, SharedString, AnyElement, BoxShadow, Div, IntoElement, div, prelude::*, px};
#[cfg(test)]
use xgpui_luma::theme::ThemeMode;
use crate::Look;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    #[default]
    Surface,
    Classic,
    Ghost,
}
impl CardVariant {
    pub const ALL: [Self; 3] = [Self::Surface, Self::Classic, Self::Ghost];
    pub fn label(self) -> &'static str {
        match self {
            Self::Surface => "Surface",
            Self::Classic => "Classic",
            Self::Ghost => "Ghost",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardSize {
    #[default]
    One,
    Two,
    Three,
}
impl CardSize {
    pub const ALL: [Self; 3] = [Self::One, Self::Two, Self::Three];
    pub fn as_str(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::One => "Size 1",
            Self::Two => "Size 2",
            Self::Three => "Size 3",
        }
    }
    /// The card's padding, from `card.lss`.
    pub fn padding(self) -> f32 {
        crate::style::lss_number("card", &[("size", self.as_str())], "padding_x")
    }
    /// The card's corner radius, from `card.lss`.
    pub fn radius(self) -> f32 {
        crate::style::lss_number("card", &[("size", self.as_str())], "radius")
    }
}

/// Resolved Card appearance. An override is applied after palette/size resolution
/// each time the element is built, so untouched colors continue to follow the look.
#[derive(Clone, Debug)]
pub struct CardStyle {
    pub padding: f32,
    pub radius: f32,
    /// Ghost defaults to negative padding; set to zero to retain layout space.
    pub margin: f32,
    pub background: Option<Hsla>,
    pub foreground: Hsla,
    pub font_family: SharedString,
    pub shadows: Vec<BoxShadow>,
}
type StyleOverride = Arc<dyn Fn(&mut CardStyle) + Send + Sync>;

/// A passive content surface. Use SDK controls inside it for interactive content.
/// Ghost cancels its padding with negative margins, as in Radix Themes.
pub struct Card {
    look: Look,
    variant: CardVariant,
    size: CardSize,
    children: Vec<AnyElement>,
    style_override: Option<StyleOverride>,
}
impl Card {
    pub fn new(look: &Look) -> Self {
        Self {
            look: look.clone(),
            variant: CardVariant::default(),
            size: CardSize::default(),
            children: Vec::new(),
            style_override: None,
        }
    }
    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = size;
        self
    }
    /// Adjust resolved appearance locally; later calls replace the previous override.
    /// ```
    /// use xgpui_luma_look_radix::{Card, Look};
    /// let card = Card::new(&Look::built_in()).style_override(|style| {
    ///     style.padding = 20.0;
    ///     style.radius = 10.0;
    ///     style.shadows.clear();
    /// });
    /// ```
    pub fn style_override(mut self, customize: impl Fn(&mut CardStyle) + Send + Sync + 'static) -> Self {
        self.style_override = Some(Arc::new(customize));
        self
    }
}
impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}
impl Card {
    /// Resolve the `.lss` card (`style/card.lss`) for this variant and size, then apply the local override.
    pub fn resolve_style(&self) -> CardStyle {
        let binding = xgpui_luma::style::StyleBinding::new("card", crate::textfield::live_source(&self.look));
        let binding = match self.variant {
            CardVariant::Surface => binding,
            CardVariant::Classic => binding.variant("classic"),
            CardVariant::Ghost => binding.variant("ghost"),
        };
        let binding = if self.size != CardSize::Two {
            binding.attr("size", self.size.as_str())
        } else {
            binding
        };
        use xgpui_luma::style::PartRead;
        let resolved = xgpui_luma::controls::card::StyledCard::new(binding).resolve();
        let card = resolved.root();
        let mut style = CardStyle {
            padding: card.number("padding_x"),
            radius: card.number("radius"),
            // Ghost cancels its padding, as in Radix Themes.
            margin: if self.variant == CardVariant::Ghost {
                -card.number("padding_x")
            } else {
                0.0
            },
            background: card.painted("background"),
            foreground: card.color("foreground"),
            font_family: card.font_family_name(),
            shadows: card.box_shadow(),
        };
        if let Some(customize) = &self.style_override {
            customize(&mut style);
        }
        style
    }
}

impl IntoElement for Card {
    type Element = Div;
    fn into_element(self) -> Div {
        let style = self.resolve_style();
        div()
            .relative()
            .p(px(style.padding))
            .m(px(style.margin))
            .rounded(px(style.radius))
            .font_family(style.font_family)
            .text_color(style.foreground)
            .when_some(style.background, |card, background| card.bg(background))
            .shadow(style.shadows)
            .children(self.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_override_preserves_live_palette_and_default_elevation() {
        let look = Look::built_in();
        let custom = Card::new(&look).variant(CardVariant::Classic).style_override(|style| {
            style.padding = 20.0;
            style.shadows.clear();
        });
        let normal = Card::new(&look).variant(CardVariant::Classic);
        let before = custom.resolve_style().background;
        look.set_mode(ThemeMode::Dark);
        let style = custom.resolve_style();
        assert_ne!(style.background, before);
        assert_eq!(style.background, normal.resolve_style().background);
        assert!(style.shadows.is_empty());
        assert!(!normal.resolve_style().shadows.is_empty());
        assert_eq!(style.padding, 20.0);
        assert_eq!(normal.resolve_style().padding, 12.0);
    }
}
