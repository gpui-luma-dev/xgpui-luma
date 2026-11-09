//! Compact SV + hue + alpha picker shown under the selected stop.
//!
//! Composed from the same `xgpui_luma_color` primitives as the Compositions →
//! Color Picker example. Edits are emitted as [`InlineColorPickerEvent`];
//! [`InlineColorPicker::set_color`] syncs from the host without emitting.

use std::sync::Arc;

use gpui::{
    App, AppContext, Context, Entity, EventEmitter, IntoElement, Render, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};
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
    pub fn spawn(id: impl Into<SharedString>, look: &Look, color: ColorValue, cx: &mut App) -> Entity<Self> {
        let id = id.into();
        let radius = px(look.metrics().radius.sm);
        cx.new(|cx| Self::new(id, radius, color, cx))
    }

    fn new(id: SharedString, radius: gpui::Pixels, color: ColorValue, cx: &mut Context<Self>) -> Self {
        let hsv = hsv_from(color);
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
                this.sync_controls(true, cx);
                this.emit_change(cx);
            }),
        ];

        let this = Self {
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
        gpui_bridge::from_hsla(self.hsv.to_hsla_ext())
    }

    /// Host sync; never emits.
    pub fn set_color(&mut self, color: ColorValue, cx: &mut Context<Self>) {
        let hsv = hsv_from(color);
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

/// Wide-gamut sources are previewed through sRGB; the HSV edit model is sRGB.
fn hsv_from(color: ColorValue) -> Hsv {
    gpui_bridge::to_rgba(color, GamutMapping::CssLocalMinde).map(Hsv::from_rgba).unwrap_or(Hsv {
        h: 0.0,
        s: 0.0,
        v: 0.0,
        a: 1.0,
    })
}
