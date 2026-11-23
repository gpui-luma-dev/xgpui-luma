//! Non-interactive Radix badge element. Applications supply labels and placement.

#[cfg(test)]
use gpui::Hsla;
use gpui::{Div, IntoElement, SharedString, div, prelude::*, px};
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

    /// The badge from `.lss` (`style/badge.lss`) for this variant, tone, contrast and size.
    pub fn look(&self) -> xgpui_luma::controls::badge::BadgeLook {
        let binding = xgpui_luma::style::StyleBinding::new("badge", crate::textfield::live_source(&self.look));
        let binding = match self.variant {
            BadgeVariant::Soft => binding,
            BadgeVariant::Solid => binding.variant("solid"),
            BadgeVariant::Surface => binding.variant("surface"),
            BadgeVariant::Outline => binding.variant("outline"),
        };
        let binding = if self.size != BadgeSize::Two {
            binding.attr("size", self.size.as_str())
        } else {
            binding
        };
        let binding = if self.tone == Tone::Gray {
            binding.attr("tone", "gray")
        } else {
            binding
        };
        let binding = binding.state("high_contrast", self.high_contrast);
        xgpui_luma::controls::badge::StyledBadge::new(binding).look()
    }

    #[cfg(test)]
    fn colors(&self) -> (Hsla, Hsla, Option<Hsla>) {
        let look = self.look();
        (look.background, look.foreground, look.border)
    }
}

impl IntoElement for Badge {
    type Element = Div;
    fn into_element(self) -> Div {
        let look = self.look();
        let radius = if self.pill { 999.0 } else { look.radius };
        // An inset outline preserves identical dimensions across variants.
        div()
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .font_family(look.font_family)
            .font_weight(look.typography.weight)
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .px(px(look.padding_x))
            .py(px(look.padding_y))
            .rounded(px(radius))
            .bg(look.background)
            .text_color(look.foreground)
            .when_some(look.border, |badge, border| {
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
    fn sizes_follow_radix_metrics() {
        let look = Look::built_in();
        for (size, want) in [
            (BadgeSize::One, (12.0, 16.0, 6.0, 2.0, 3.0)),
            (BadgeSize::Two, (12.0, 16.0, 8.0, 4.0, 4.0)),
            (BadgeSize::Three, (14.0, 20.0, 10.0, 4.0, 4.0)),
        ] {
            let l = Badge::new(&look, "New").size(size).look();
            assert_eq!((l.typography.size, l.typography.line_height, l.padding_x, l.padding_y, l.radius), want);
            assert_eq!(l.typography.weight, gpui::FontWeight::MEDIUM);
        }
    }

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
                        crate::control_group::assert_close(normal.0, tone.step(&look, 9));
                        crate::control_group::assert_close(normal.1, tone.contrast(&look));
                        crate::control_group::assert_close(high.0, tone.step(&look, 12));
                        crate::control_group::assert_close(high.1, tone.step(&look, 1));
                    } else {
                        crate::control_group::assert_close(normal.1, tone.step(&look, 11));
                        crate::control_group::assert_close(high.1, tone.step(&look, 12));
                    }
                    assert_eq!(normal.2.is_some(), matches!(variant, BadgeVariant::Surface | BadgeVariant::Outline));
                    let close = crate::control_group::assert_close;
                    match variant {
                        BadgeVariant::Soft => close(normal.0, tone.step(&look, 3)),
                        BadgeVariant::Surface => {
                            close(normal.0, tone.step(&look, 2));
                            close(normal.2.unwrap(), tone.step(&look, 6));
                        }
                        BadgeVariant::Outline => {
                            assert_eq!(normal.0.a, 0.0);
                            close(normal.2.unwrap(), tone.step(&look, 8));
                            close(high.2.unwrap(), Tone::Gray.step(&look, 11));
                        }
                        BadgeVariant::Solid => {}
                    }
                }
            }
        }
    }
}
