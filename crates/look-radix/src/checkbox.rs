//! Radix checkbox: variants, sizes and templates.
//!
//! The checkbox is drawn from `.lss` (`style/checkbox.lss`): geometry matches `@radix-ui/themes`
//! BaseCheckbox at `--scaling: 1` (sizes 1–3 → box 14 / 16 / 20; radius from `--radius-1` × size ×
//! theme radius factor, never a pill).

use std::sync::Arc;

use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::checkbox::StyledCheckboxTemplate;

use crate::button::Paint;
use crate::button_layout::Radius;
use crate::look::Look;
use crate::tone::Tone;

/// The Radix Themes checkbox variant scheme.
///
/// `Surface` is Radix's default: a panel face behind a gray edge that fills with the
/// accent when checked. `Classic` adds the raised inset treatment. `Soft` drops the
/// edge entirely and keeps an accent tint in both check states.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CheckboxVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

/// Radix `size` prop on Checkbox. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CheckboxSize {
    One,
    #[default]
    Two,
    Three,
}

impl CheckboxSize {
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

/// The Radix checkbox drawn from `.lss` (`style/checkbox.lss`) at the look's size and medium radius,
/// following the look's mode and palettes.
pub fn checkbox_template(
    look: &Look,
    variant: CheckboxVariant,
    paint: Paint,
) -> Arc<dyn ButtonTemplate<xgpui_luma::controls::checkbox::CheckboxData>> {
    let size = match look.size() {
        crate::RadixSize::One => CheckboxSize::One,
        crate::RadixSize::Two => CheckboxSize::Two,
        crate::RadixSize::Three => CheckboxSize::Three,
    };
    checkbox_template_for(look, variant, paint, size, Radius::Medium)
}

/// The `.lss` Radix checkbox for a size × radius style-guide cell.
pub fn checkbox_template_for(
    look: &Look,
    variant: CheckboxVariant,
    paint: Paint,
    size: CheckboxSize,
    radius: Radius,
) -> Arc<dyn ButtonTemplate<xgpui_luma::controls::checkbox::CheckboxData>> {
    let source = look.clone();
    let mut template = StyledCheckboxTemplate::live(Arc::new(move || source.style_theme()))
        .state("high_contrast", paint.high_contrast);
    match variant {
        CheckboxVariant::Surface => {}
        CheckboxVariant::Classic => template = template.variant("classic"),
        CheckboxVariant::Soft => template = template.variant("soft"),
    }
    if size != CheckboxSize::Two {
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
