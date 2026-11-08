//! Look-owned, non-interactive content container with Radix Card treatments.
//! Size metrics follow the Radix Themes Card; application content stays external.
use std::sync::Arc;
use gpui::{Hsla, SharedString, AnyElement, BoxShadow, Div, IntoElement, div, point, prelude::*, px};
use xgpui_luma::theme::ThemeMode;
use crate::{Look, Tone};

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
    pub fn padding(self) -> f32 {
        crate::look::embedded_common_stylesheet()
            .card
            .resolve_geometry(self.as_str(), Default::default())
            .padding
    }
    pub fn radius(self) -> f32 {
        match self {
            Self::One | Self::Two => 8.0,
            Self::Three => 12.0,
        }
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
    /// Resolve current palette and size defaults, then apply the local override.
    pub fn resolve_style(&self) -> CardStyle {
        let gray = |step| Tone::Gray.step(&self.look, step);
        let dark = self.look.mode() == ThemeMode::Dark;
        let geometry = self.look.common_stylesheet().card.resolve_geometry(
            self.size.as_str(),
            xgpui_luma::theme::stylesheet::CardGeometry { padding: self.size.padding(), ..Default::default() },
        );
        let mut style = CardStyle {
            padding: geometry.padding,
            radius: self.size.radius(),
            margin: 0.0,
            background: None,
            foreground: gray(12),
            font_family: crate::typography::font_family(&self.look),
            shadows: Vec::new(),
        };
        if self.variant == CardVariant::Ghost {
            style.margin = -style.padding;
        } else {
            // Solid scale equivalents of Radix's panel and translucent neutral borders.
            let edge = gray(match self.variant {
                CardVariant::Classic if dark => 6,
                CardVariant::Classic => 3,
                _ => 5,
            });
            let mut shadows = vec![BoxShadow {
                offset: point(px(0.0), px(0.0)),
                blur_radius: px(0.0),
                spread_radius: px(1.0),
                color: edge,
                inset: true,
            }];
            if self.variant == CardVariant::Classic {
                for (y, blur, spread, opacity) in [
                    (1.0, 1.0, -1.0, if dark { 0.3 } else { 0.05 }),
                    (2.0, 1.0, -2.0, if dark { 0.3 } else { 0.05 }),
                    (1.0, 3.0, -1.0, if dark { 0.2 } else { 0.05 }),
                ] {
                    shadows.push(BoxShadow {
                        offset: point(px(0.0), px(y)),
                        blur_radius: px(blur),
                        spread_radius: px(spread),
                        color: gpui::hsla(0.0, 0.0, 0.0, opacity),
                        inset: false,
                    });
                }
            }
            style.background = Some(gray(if dark { 2 } else { 1 }));
            style.shadows = shadows;
        }
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

/// Native common geometry owned by this control.
pub(crate) fn native_card_stylesheet() -> xgpui_luma::theme::stylesheet::CardStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    CardStylesheet {
        sizes: [("1", 12.0), ("2", 16.0), ("3", 24.0)]
            .into_iter()
            .map(|(key, padding)| (key.into(), CardGeometryStylesheet { padding: Some(padding), ..Default::default() }))
            .collect(),
        ..Default::default()
    }
}
