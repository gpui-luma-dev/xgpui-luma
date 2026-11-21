//! The `gpui` feature: apply a part's style to a gpui element (API topic 1, `.apply()`).

use gpui::{BoxShadow, FontWeight, Hsla, Styled, hsla, point, px};

use crate::color::Color;
use crate::resolve::PartView;
use crate::value::{ShadowLayer, Value};

impl From<Color> for Hsla {
    /// Gamut-mapped to sRGB first.
    fn from(color: Color) -> Self {
        let [h, s, l, a] = color.to_hsla();
        hsla(h, s, l, a)
    }
}

impl From<&ShadowLayer> for BoxShadow {
    fn from(layer: &ShadowLayer) -> Self {
        BoxShadow {
            color: layer.color.into(),
            offset: point(px(layer.x), px(layer.y)),
            blur_radius: px(layer.blur),
            spread_radius: px(layer.spread),
            inset: layer.inset,
        }
    }
}

/// gpui box shadows for shadow layers.
pub fn box_shadows(layers: &[ShadowLayer]) -> Vec<BoxShadow> {
    layers.iter().map(BoxShadow::from).collect()
}

/// Apply a resolved part to any gpui element.
pub trait ApplyStyle: Styled + Sized {
    /// Sets every core property the part has a value for. Declared properties, `focus_border` (drawn
    /// by the SDK's focus adorner) and `transition` (read by the animation runtime) are never applied.
    fn apply(mut self, part: PartView<'_>) -> Self {
        let num = |name: &str| part.value(name).and_then(Value::as_number);
        if let Some(c) = part.background() {
            self = self.bg(Hsla::from(c));
        }
        if let Some(c) = part.border() {
            self = self.border_color(Hsla::from(c));
        }
        if let Some(w) = num("border_width") {
            self = self.border(px(w));
        }
        if let Some(r) = num("radius") {
            self = self.rounded(px(r));
        }
        if let Some(p) = num("padding") {
            self = self.p(px(p));
        }
        if let Some(p) = num("padding_x") {
            self = self.px(px(p));
        }
        if let Some(p) = num("padding_y") {
            self = self.py(px(p));
        }
        if let Some(m) = num("margin") {
            self = self.m(px(m));
        }
        if let Some(g) = num("gap") {
            self = self.gap(px(g));
        }
        if let Some(w) = num("width") {
            self = self.w(px(w));
        }
        if let Some(h) = num("height") {
            self = self.h(px(h));
        }
        if let Some(w) = num("min_width") {
            self = self.min_w(px(w));
        }
        if let Some(w) = num("max_width") {
            self = self.max_w(px(w));
        }
        if let Some(h) = num("min_height") {
            self = self.min_h(px(h));
        }
        if let Some(h) = num("max_height") {
            self = self.max_h(px(h));
        }
        if let Some(o) = num("opacity") {
            self = self.opacity(o);
        }
        if let Some(layers) = part.shadow() {
            self = self.shadow(box_shadows(layers));
        }
        if let Some(c) = part.foreground() {
            self = self.text_color(Hsla::from(c));
        }
        if let Some(f) = part.font_family() {
            self = self.font_family(f.to_string());
        }
        if let Some(s) = num("font_size") {
            self = self.text_size(px(s));
        }
        if let Some(l) = num("line_height") {
            self = self.line_height(px(l));
        }
        if let Some(w) = num("font_weight") {
            self = self.font_weight(FontWeight(w));
        }
        self
    }
}

impl<E: Styled> ApplyStyle for E {}
