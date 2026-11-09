//! Compact SV + hue + alpha picker shown under the selected stop.
//!
//! Composed from the same `xgpui_luma_color` primitives as the Compositions →
//! Color Picker example. Edits are emitted as [`InlineColorPickerEvent`];
//! [`InlineColorPicker::set_color`] syncs from the host without emitting.
//!
//! The SV/hue plane edits HSV over the encoded RGB of an explicit edit gamut
//! (sRGB or Display P3). The picker keeps the host's source color: syncing,
//! opening or changing alpha never re-authors it; only an SV/hue edit authors
//! a new color, tagged with the edit gamut. The picker's own field and sliders
//! are ordinary GPUI (sRGB) previews of that plane.

use std::sync::Arc;

use gpui::{
    App, AppContext, Context, Entity, EventEmitter, IntoElement, Render, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use xgpui_luma::color::{ColorValue, Gamut, GamutMapping};
use xgpui_luma::controls::slider::SliderControl;
use xgpui_luma::theme::ControlSize;
use xgpui_luma_color::color_field::{ColorFieldEvent, ColorFieldState};
use xgpui_luma_color::color_slider::color_spec::Hsv;
use xgpui_luma_color::color_slider::{AlphaDelegate, ColorSliderBuilder, ColorSliderDomainRenderer, primary_slider_value};
use xgpui_luma_color::composition::ColorCompositionSync;

use crate::theme::Look;

const FIELD_HEIGHT: f32 = 120.0;
const FIELD_THUMB: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum InlineColorPickerEvent {
    /// Continuous edit while dragging or stepping.
    Change(ColorValue),
}

pub(crate) struct InlineColorPicker {
    gamut: Gamut,
    /// The color as the host gave it, or as last authored by an SV/hue edit.
    source: ColorValue,
    hsv: Hsv,
    sync: ColorCompositionSync,
    field: Entity<ColorFieldState>,
    hue_slider: Entity<SliderControl>,
    alpha_slider: Entity<SliderControl>,
    alpha_domain: Arc<ColorSliderDomainRenderer>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<InlineColorPickerEvent> for InlineColorPicker {}

impl InlineColorPicker {
    pub fn spawn(
        id: impl Into<SharedString>,
        look: &Look,
        color: ColorValue,
        gamut: Gamut,
        cx: &mut App,
    ) -> Entity<Self> {
        let id = id.into();
        let radius = px(look.metrics().radius.sm);
        cx.new(|cx| Self::new(id, radius, color, gamut, cx))
    }

    fn new(id: SharedString, radius: gpui::Pixels, color: ColorValue, gamut: Gamut, cx: &mut Context<Self>) -> Self {
        let hsv = hsv_from(color, gamut);
        let field = cx.new(|_| {
            ColorFieldState::saturation_value(format!("{id}-field"), hsv, FIELD_THUMB).rounded(radius).vector()
        });
        let hue_slider = ColorSliderBuilder::hue(format!("{id}-hue"), hsv.h)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge()
            .spawn(cx);
        let alpha_builder = ColorSliderBuilder::alpha(format!("{id}-alpha"), hsv.a, hsv)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge();
        let alpha_domain = alpha_builder.domain_renderer();
        let alpha_slider = alpha_builder.spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&field, |this, _, event: &ColorFieldEvent, cx| {
                let (ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv)) = event else {
                    return;
                };
                let Some(_guard) = this.sync.begin_guard() else {
                    return;
                };
                this.hsv = Hsv { a: this.hsv.a, ..*hsv };
                this.source = color_from(this.hsv, this.gamut);
                this.sync_controls(false, cx);
                this.emit_change(cx);
            }),
            cx.subscribe(&hue_slider, |this, _, event, cx| {
                let Some(hue) = primary_slider_value(event) else {
                    return;
                };
                let Some(_guard) = this.sync.begin_guard() else {
                    return;
                };
                this.hsv.h = hue;
                this.source = color_from(this.hsv, this.gamut);
                this.sync_controls(true, cx);
                this.emit_change(cx);
            }),
            cx.subscribe(&alpha_slider, |this, _, event, cx| {
                let Some(alpha) = primary_slider_value(event) else {
                    return;
                };
                let Some(_guard) = this.sync.begin_guard() else {
                    return;
                };
                this.hsv.a = alpha;
                // Replace alpha only; the source's space and channels are kept.
                if let Ok(source) = this.source.with_alpha(alpha) {
                    this.source = source;
                }
                this.sync_controls(true, cx);
                this.emit_change(cx);
            }),
        ];

        let this = Self {
            gamut,
            source: color,
            hsv,
            sync: ColorCompositionSync::new(),
            field,
            hue_slider,
            alpha_slider,
            alpha_domain,
            _subscriptions: subscriptions,
        };
        this.sync_controls(true, cx);
        this
    }

    pub fn color(&self) -> ColorValue {
        self.source
    }

    /// Host sync; never emits. The source is kept as given.
    pub fn set_color(&mut self, color: ColorValue, cx: &mut Context<Self>) {
        if color == self.source {
            return;
        }
        self.source = color;
        let hsv = hsv_from(color, self.gamut);
        if hsv == self.hsv {
            return;
        }
        let Some(_guard) = self.sync.begin_guard() else {
            return;
        };
        self.hsv = hsv;
        self.sync_controls(true, cx);
        cx.notify();
    }

    fn emit_change(&self, cx: &mut Context<Self>) {
        cx.emit(InlineColorPickerEvent::Change(self.color()));
        cx.notify();
    }

    fn sync_controls(&self, sync_field: bool, cx: &mut Context<Self>) {
        let hsv = self.hsv;
        if sync_field {
            self.field.update(cx, |field, cx| field.set_hsv(hsv, cx));
        }
        self.sync.sync_slider_value(&self.hue_slider, hsv.h, cx);
        self.sync.sync_color_slider(
            &self.alpha_slider,
            &self.alpha_domain,
            Arc::new(AlphaDelegate { spec: hsv }),
            self.alpha_domain.context(),
            hsv.a,
            cx,
        );
    }
}

impl Render for InlineColorPicker {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(div().w_full().h(px(FIELD_HEIGHT)).child(self.field.clone()))
            .child(div().w_full().child(self.hue_slider.clone()))
            .child(div().w_full().child(self.alpha_slider.clone()))
    }
}

/// HSV of `color`'s encoded RGB in the edit gamut. A source outside that gamut
/// is placed at its CSS local-MINDE fallback; the source itself is not changed.
pub(crate) fn hsv_from(color: ColorValue, gamut: Gamut) -> Hsv {
    let encoded = match gamut {
        Gamut::Srgb => color.to_srgba_fallback(GamutMapping::CssLocalMinde).map(|c| (c.red, c.green, c.blue, c.alpha)),
        Gamut::DisplayP3 => {
            color.to_display_p3_fallback(GamutMapping::CssLocalMinde).map(|c| (c.red, c.green, c.blue, c.alpha))
        }
    };
    encoded.map(|(r, g, b, a)| Hsv::from_rgba(gpui::Rgba { r, g, b, a })).unwrap_or(Hsv {
        h: 0.0,
        s: 0.0,
        v: 0.0,
        a: 1.0,
    })
}

/// Author a source color from an HSV position over the edit gamut's encoded RGB.
/// HSV↔RGB is independent of primaries, so the same math serves both gamuts;
/// only the resulting tag differs.
pub(crate) fn color_from(hsv: Hsv, gamut: Gamut) -> ColorValue {
    use palette::convert::FromColorUnclamped;
    let rgb = palette::Srgba::from_color_unclamped(palette::Hsva::new(hsv.h, hsv.s, hsv.v, hsv.a));
    match gamut {
        Gamut::Srgb => ColorValue::Srgb(rgb),
        Gamut::DisplayP3 => ColorValue::display_p3(rgb.red, rgb.green, rgb.blue, rgb.alpha),
    }
}
