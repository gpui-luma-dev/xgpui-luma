//! Non-interactive Radix badge element. Applications supply labels and placement.

use gpui::{Div, FontWeight, Hsla, IntoElement, SharedString, div, prelude::*, px};
use crate::{Look, Tone};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    Solid,
    #[default]
    Soft,
    Surface,
    Outline,
}

impl BadgeVariant {
    pub const ALL: [Self; 4] = [Self::Solid, Self::Soft, Self::Surface, Self::Outline];
    pub fn label(self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::Soft => "Soft",
            Self::Surface => "Surface",
            Self::Outline => "Outline",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeSize {
    #[default]
    One,
    Two,
    Three,
}

impl BadgeSize {
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
    fn metrics(self) -> (f32, f32, f32, f32, f32) {
        let geometry =
            crate::look::embedded_common_stylesheet().badge.resolve_geometry(self.as_str(), Default::default());
        let radius = match self {
            Self::One => 3.0,
            Self::Two | Self::Three => 4.0,
        };
        (geometry.font_size, geometry.line_height, geometry.padding_x, geometry.padding_y, radius)
    }
}

/// Read-only label with look-owned colors and geometry; no entity or input handlers.
pub struct Badge {
    look: Look,
    label: SharedString,
    variant: BadgeVariant,
    tone: Tone,
    high_contrast: bool,
    size: BadgeSize,
    pill: bool,
}

impl Badge {
    pub fn new(look: &Look, label: impl Into<SharedString>) -> Self {
        Self {
            look: look.clone(),
            label: label.into(),
            variant: BadgeVariant::Soft,
            tone: Tone::Accent,
            high_contrast: false,
            size: BadgeSize::One,
            pill: false,
        }
    }
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    pub fn high_contrast(mut self, high_contrast: bool) -> Self {
        self.high_contrast = high_contrast;
        self
    }
    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }
    pub fn pill(mut self) -> Self {
        self.pill = true;
        self
    }

    fn colors(&self) -> (Hsla, Hsla, Option<Hsla>) {
        let step = |n| self.tone.step(&self.look, n);
        let text = step(if self.high_contrast { 12 } else { 11 });
        // Solid scale equivalents follow the look's existing palette treatment.
        match self.variant {
            BadgeVariant::Solid if self.high_contrast => (step(12), step(1), None),
            BadgeVariant::Solid => (step(9), self.tone.contrast(&self.look), None),
            BadgeVariant::Soft => (step(3), text, None),
            BadgeVariant::Surface => (step(2), text, Some(step(6))),
            BadgeVariant::Outline => (
                gpui::hsla(0.0, 0.0, 0.0, 0.0),
                text,
                Some(if self.high_contrast {
                    Tone::Gray.step(&self.look, 11)
                } else {
                    step(8)
                }),
            ),
        }
    }
}

impl IntoElement for Badge {
    type Element = Div;
    fn into_element(self) -> Div {
        let (background, foreground, border) = self.colors();
        let (font_size, line_height, padding_x, padding_y, radius) = self.size.metrics();
        let geometry = self.look.common_stylesheet().badge.resolve_geometry(
            self.size.as_str(),
            xgpui_luma::theme::stylesheet::BadgeGeometry {
                font_size,
                line_height,
                padding_x,
                padding_y,
                ..Default::default()
            },
        );
        let (font_size, line_height, padding_x, padding_y) =
            (geometry.font_size, geometry.line_height, geometry.padding_x, geometry.padding_y);
        let radius = if self.pill { 999.0 } else { radius };
        // An inset outline preserves identical dimensions across variants.
        div()
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .font_family(crate::typography::font_family(&self.look))
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(font_size))
            .line_height(px(line_height))
            .px(px(padding_x))
            .py(px(padding_y))
            .rounded(px(radius))
            .bg(background)
            .text_color(foreground)
            .when_some(border, |badge, border| {
                badge.child(div().absolute().inset_0().rounded(px(radius)).border_1().border_color(border))
            })
            .child(self.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::ThemeMode;
    #[test]
    fn variants_follow_both_tones_and_modes() {
        let look = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for tone in [Tone::Accent, Tone::Gray] {
                for variant in BadgeVariant::ALL {
                    let normal = Badge::new(&look, "New").tone(tone).variant(variant).colors();
                    let high = Badge::new(&look, "New").tone(tone).variant(variant).high_contrast(true).colors();
                    if variant == BadgeVariant::Solid {
                        assert_eq!(normal.0, tone.step(&look, 9));
                        assert_eq!(normal.1, tone.contrast(&look));
                        assert_eq!(high.0, tone.step(&look, 12));
                        assert_eq!(high.1, tone.step(&look, 1));
                    } else {
                        assert_eq!(normal.1, tone.step(&look, 11));
                        assert_eq!(high.1, tone.step(&look, 12));
                    }
                    assert_eq!(normal.2.is_some(), matches!(variant, BadgeVariant::Surface | BadgeVariant::Outline));
                    if variant == BadgeVariant::Outline {
                        assert_eq!(normal.0.a, 0.0);
                    }
                }
            }
        }
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_badge_stylesheet() -> xgpui_luma::theme::stylesheet::BadgeStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    BadgeStylesheet {
        sizes: [("1", 12.0, 16.0, 6.0, 2.0), ("2", 12.0, 16.0, 8.0, 4.0), ("3", 14.0, 20.0, 10.0, 4.0)]
            .into_iter()
            .map(|(key, font, line, x, y)| {
                (
                    key.into(),
                    BadgeGeometryStylesheet {
                        font_size: Some(font),
                        line_height: Some(line),
                        padding_x: Some(x),
                        padding_y: Some(y),
                        ..Default::default()
                    },
                )
            })
            .collect(),
        ..Default::default()
    }
}
