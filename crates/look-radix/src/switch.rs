//! Radix switch: variants, sizes and templates.
//!
//! The switch is drawn from `.lss` (`style/switch.lss`): geometry matches `@radix-ui/themes` Switch at
//! `--scaling: 1` (height 16 / 20 / 24, width = height × 1.75, thumb inset 1; radius =
//! `max(--radius-{1|2} × factor, --radius-thumb)`, never more than a pill).

use std::sync::Arc;

use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::switch::StyledSwitchTemplate;

use crate::button::Paint;
use crate::button_layout::Radius;
use crate::look::Look;
use crate::tone::Tone;

/// The Radix Themes switch variant scheme (Classic / Surface / Soft).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SwitchVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

/// Radix `size` prop on Switch. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SwitchSize {
    One,
    #[default]
    Two,
    Three,
}

impl SwitchSize {
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

    pub fn control_size(self) -> crate::RadixSize {
        match self {
            Self::One => crate::RadixSize::One,
            Self::Two => crate::RadixSize::Two,
            Self::Three => crate::RadixSize::Three,
        }
    }
}

pub fn switch_template(
    look: &Look,
    variant: SwitchVariant,
    paint: Paint,
) -> Arc<dyn ButtonTemplate<xgpui_luma::controls::switch::SwitchData>> {
    let size = match look.size() {
        crate::RadixSize::One => SwitchSize::One,
        crate::RadixSize::Two => SwitchSize::Two,
        crate::RadixSize::Three => SwitchSize::Three,
    };
    switch_template_for(look, variant, paint, size, Radius::Medium)
}

pub fn switch_template_for(
    look: &Look,
    variant: SwitchVariant,
    paint: Paint,
    size: SwitchSize,
    radius: Radius,
) -> Arc<dyn ButtonTemplate<xgpui_luma::controls::switch::SwitchData>> {
    let source = look.clone();
    let mut template =
        StyledSwitchTemplate::live(Arc::new(move || source.style_theme())).state("high_contrast", paint.high_contrast);
    match variant {
        SwitchVariant::Surface => {}
        SwitchVariant::Classic => template = template.variant("classic"),
        SwitchVariant::Soft => template = template.variant("soft"),
    }
    if size != SwitchSize::Two {
        template = template.attr("size", size.as_str());
    }
    if paint.tone == Tone::Gray {
        template = template.attr("tone", "gray");
    }
    let radius = match radius {
        Radius::None => Some("none"),
        Radius::Small => Some("small"),
        Radius::Medium => None,
        Radius::Large => Some("large"),
        Radius::Full => Some("full"),
    };
    if let Some(radius) = radius {
        template = template.attr("radius", radius);
    }
    Arc::new(template)
}
