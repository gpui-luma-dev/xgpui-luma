//! The accordion's trigger, content and layout read from `.lss` (issue #285): `accordion.lss`. Looks keep
//! their own drag preview, handle icon and tooltip, and delegate the rest to [`StyledAccordion`].

use crate::style::PartRead;

use super::theme::{AccordionContentPalette, AccordionPalette, AccordionScale, AccordionTheme};
use crate::style::StyleBinding;
use crate::theme::{InteractionState, MetricTokens};

/// The accordion's `.lss` values for one binding (the `accordion` control).
#[derive(Clone)]
pub struct StyledAccordion {
    binding: StyleBinding,
}

impl StyledAccordion {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("accordion") }
    }

    /// The trigger row in `state`.
    pub fn trigger(&self, state: InteractionState) -> AccordionPalette {
        let states = [
            ("hover", state.hovered),
            ("pressed", state.pressed),
            ("focused", state.focused),
            ("disabled", state.disabled),
        ];
        let resolved = self.binding.resolve(&states, 1.0);
        let root = resolved.root();
        let background = root.color("background");
        let foreground = root.color("foreground");
        AccordionPalette {
            background: (background.a > 0.0).then_some(background),
            foreground,
            border_color: root.color("border"),
            icon_color: foreground,
            chevron_color: root.color("chevron"),
            typography: root.typography(14.0, 20.0),
            font_family: root.font_family_name(),
            opacity: root.opacity(),
        }
    }

    /// The content's colors.
    pub fn content(&self) -> AccordionContentPalette {
        let resolved = self.binding.resolve(&[], 1.0);
        AccordionContentPalette { background: None, foreground: resolved.root().color("content_foreground") }
    }

    /// The row layout at `scale_factor` device pixels per pixel.
    pub fn scale(&self, scale_factor: f32) -> AccordionScale {
        let resolved = self.binding.resolve(&[], scale_factor);
        let root = resolved.root();
        AccordionScale {
            drag_handle_padding: root.number("drag_handle_padding"),
            trigger_height: root.number("height"),
            padding_x: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            content_padding_y: root.number("content_padding_y"),
            radius: root.number("radius"),
            item_gap: root.number("item_gap"),
            inner_gap: root.number("inner_gap"),
            icon_size: root.number("icon_size"),
            chevron_size: root.number("chevron_size"),
        }
    }
}

/// The SDK's accordion theme: `accordion.lss`, with the SDK's drag preview and icon.
pub struct StyledAccordionTheme(StyledAccordion);

impl StyledAccordionTheme {
    pub fn sdk() -> Self {
        Self(StyledAccordion::sdk())
    }
}

impl AccordionTheme for StyledAccordionTheme {
    fn resolve_trigger(&self, state: InteractionState) -> AccordionPalette {
        self.0.trigger(state)
    }

    fn resolve_content(&self, _expanded: bool) -> AccordionContentPalette {
        self.0.content()
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }

    fn resolve_scale(&self, scale_factor: f32) -> AccordionScale {
        self.0.scale(scale_factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A theme that supplies only metrics, given the SDK's own metrics, lays out exactly as the
    /// SDK's `.lss` accordion: the default path takes its fixed sizes and proportions from the `.lss`.
    #[test]
    fn the_default_scale_matches_the_sdk_lss() {
        for scale_factor in [1.0, 1.5, 2.0] {
            let default = crate::controls::accordion::AccordionScale::compute(
                &crate::theme::MetricTokens::default(),
                scale_factor,
            );
            assert_eq!(default, StyledAccordion::sdk().scale(scale_factor), "@{scale_factor}");
        }
    }
}
