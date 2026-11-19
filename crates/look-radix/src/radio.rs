//! Radix radio: variants, sizes and templates.
//!
//! The radio is drawn from `.lss` (`style/radio.lss`): geometry matches `@radix-ui/themes` BaseRadio at
//! `--scaling: 1` (sizes 1–3 → 14 / 16 / 20, dot scaled by 0.4); the indicator is always round.

use std::sync::Arc;

use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::radio_button::StyledRadioButtonTemplate;

use crate::button::Paint;
use crate::look::Look;
use crate::tone::Tone;

/// The Radix Themes radio variant scheme (Classic / Surface / Soft).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RadioVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

/// Radix `size` prop on Radio. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RadioSize {
    One,
    #[default]
    Two,
    Three,
}

impl RadioSize {
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

pub fn radio_template(
    look: &Look,
    variant: RadioVariant,
    paint: Paint,
) -> Arc<dyn ButtonTemplate<xgpui_luma::controls::radio_button::RadioButtonData>> {
    let size = match look.size() {
        crate::RadixSize::One => RadioSize::One,
        crate::RadixSize::Two => RadioSize::Two,
        crate::RadixSize::Three => RadioSize::Three,
    };
    radio_template_for(look, variant, paint, size)
}

pub fn radio_template_for(
    look: &Look,
    variant: RadioVariant,
    paint: Paint,
    size: RadioSize,
) -> Arc<dyn ButtonTemplate<xgpui_luma::controls::radio_button::RadioButtonData>> {
    Arc::new(styled_radio_template(look, variant, paint, size))
}

/// The Radix radio drawn from `.lss` (`style/radio.lss`), following the look's mode and palettes.
pub(crate) fn styled_radio_template(
    look: &Look,
    variant: RadioVariant,
    paint: Paint,
    size: RadioSize,
) -> StyledRadioButtonTemplate {
    let source = look.clone();
    let mut template = StyledRadioButtonTemplate::live(Arc::new(move || source.style_theme()))
        .state("high_contrast", paint.high_contrast);
    match variant {
        RadioVariant::Surface => {}
        RadioVariant::Classic => template = template.variant("classic"),
        RadioVariant::Soft => template = template.variant("soft"),
    }
    if size != RadioSize::Two {
        template = template.attr("size", size.as_str());
    }
    if paint.tone == Tone::Gray {
        template = template.attr("tone", "gray");
    }
    template
}
