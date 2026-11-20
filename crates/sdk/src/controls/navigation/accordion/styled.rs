//! The accordion's trigger, content and layout read from `.lss` (issue #285): `accordion.lss`. Looks keep
//! their own drag preview, handle icon and tooltip, and delegate the rest to [`StyledAccordion`].

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value};

use super::theme::{AccordionContentPalette, AccordionPalette, AccordionScale, AccordionTheme};
use crate::style::StyleBinding;
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens};

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

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
        let background = color(root, "background");
        let foreground = color(root, "foreground");
        AccordionPalette {
            background: (background.a > 0.0).then_some(background),
            foreground,
            border_color: color(root, "border"),
            icon_color: foreground,
            chevron_color: color(root, "chevron"),
            typography: LumaTextStyle {
                size: root.font_size().unwrap_or(14.0),
                line_height: root.line_height().unwrap_or(20.0),
                weight: gpui::FontWeight(number(root, "font_weight").max(100.0)),
            },
            font_family: root.font_family().unwrap_or("").to_string().into(),
        }
    }

    /// The content's colors.
    pub fn content(&self) -> AccordionContentPalette {
        let resolved = self.binding.resolve(&[], 1.0);
        AccordionContentPalette { background: None, foreground: color(resolved.root(), "content_foreground") }
    }

    /// The row layout at `scale_factor` device pixels per pixel.
    pub fn scale(&self, scale_factor: f32) -> AccordionScale {
        let resolved = self.binding.resolve(&[], scale_factor);
        let root = resolved.root();
        AccordionScale {
            drag_handle_padding: number(root, "drag_handle_padding"),
            disabled_opacity: number(root, "disabled_opacity"),
            trigger_height: number(root, "height"),
            padding_x: number(root, "padding_x"),
            padding_y: number(root, "padding_y"),
            content_padding_y: number(root, "content_padding_y"),
            radius: number(root, "radius"),
            item_gap: number(root, "item_gap"),
            inner_gap: number(root, "inner_gap"),
            icon_size: number(root, "icon_size"),
            chevron_size: number(root, "chevron_size"),
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

    #[test]
    fn sdk_accordion_matches_the_token_accordion() {
        let close = |a: Hsla, b: Hsla| {
            let (a, b) = (a.to_rgb(), b.to_rgb());
            [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
        };
        let native = super::super::theme::DefaultAccordionTheme::default();
        let lss = StyledAccordionTheme::sdk();
        for bits in 0..8u8 {
            let state = InteractionState {
                hovered: bits & 1 != 0,
                pressed: bits & 2 != 0,
                disabled: bits & 4 != 0,
                ..Default::default()
            };
            let (got, want) = (lss.resolve_trigger(state), native.resolve_trigger(state));
            assert_eq!(got.background.is_some(), want.background.is_some(), "{state:?}");
            if let (Some(g), Some(w)) = (got.background, want.background) {
                assert!(close(g, w), "{state:?} background");
            }
            assert!(close(got.foreground, want.foreground) && close(got.border_color, want.border_color));
            assert!(close(got.chevron_color, want.chevron_color), "{state:?} chevron");
            assert_eq!(
                (got.typography.size, got.typography.line_height),
                (want.typography.size, want.typography.line_height)
            );
        }
        assert_eq!(lss.resolve_scale(1.0), native.resolve_scale(1.0));
    }
}
