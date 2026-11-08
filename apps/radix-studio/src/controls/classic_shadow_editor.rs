//! Live editor for the Classic button's bubble shadow.
//!
//! Every slider writes straight into [`Look::set_classic_params`], so the
//! whole app retunes as you drag. The readout under the preview is the literal to paste
//! back into `ClassicButtonParams::default` once the numbers look right.

use xgpui_luma::controls::slider::SliderEventExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui::{
    AnyElement, ClipboardItem, Context, Entity, EventEmitter, FontWeight, Hsla, IntoElement, Render, Window, div,
    prelude::*, px,
};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::slider::{Slider, SliderEvent};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::{hstack, vstack};
use xgpui_luma_look_radix as radix;
use xgpui_luma_look_radix::{ClassicButtonParams, ButtonSize, Look, SemanticRole};

use crate::assets::{icon_named, react_icon};

const LABEL_WIDTH: f32 = 118.0;
/// Radix icons are drawn on a native 15x15 grid.
const COPY_ICON_SIZE: f32 = 15.0;
const VALUE_WIDTH: f32 = 52.0;
/// Big enough to read the top fade as a gradient rather than a line.
const PREVIEW_HEIGHT: f32 = 50.0;
const PREVIEW_WIDTH: f32 = 110.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Field {
    Radius,
    FillTop,
    FillBottom,
    Border,
    RimLighten,
    RimOffset,
    RimAlpha,
    FadeOffset,
    FadeBlur,
    FadeSpread,
    FadeAlpha,
    BevelDarken,
    BevelOffset,
    BevelAlpha,
    DropOffset,
    DropBlur,
    DropAlpha,
}

impl Field {
    /// Single place that binds a slider to its parameter.
    fn slot(self, params: &mut ClassicButtonParams) -> &mut f32 {
        match self {
            Self::Radius => &mut params.radius,
            Self::FillTop => &mut params.fill_top_delta,
            Self::FillBottom => &mut params.fill_bottom_delta,
            Self::Border => &mut params.border_delta,
            Self::RimLighten => &mut params.rim_lighten,
            Self::RimOffset => &mut params.rim_offset,
            Self::RimAlpha => &mut params.rim_alpha,
            Self::FadeOffset => &mut params.fade_offset,
            Self::FadeBlur => &mut params.fade_blur,
            Self::FadeSpread => &mut params.fade_spread,
            Self::FadeAlpha => &mut params.fade_alpha,
            Self::BevelDarken => &mut params.bevel_darken,
            Self::BevelOffset => &mut params.bevel_offset,
            Self::BevelAlpha => &mut params.bevel_alpha,
            Self::DropOffset => &mut params.drop_offset,
            Self::DropBlur => &mut params.drop_blur,
            Self::DropAlpha => &mut params.drop_alpha,
        }
    }
}

/// How a value reads back under its slider.
#[derive(Clone, Copy, PartialEq)]
enum Unit {
    /// Length in px.
    Px,
    /// `0..1`, shown as a percentage.
    Alpha,
    /// Lightness offset applied to the accent fill.
    Delta,
}

impl Unit {
    fn format(self, value: f32) -> String {
        match self {
            Self::Px => format!("{value:.1}px"),
            Self::Alpha => format!("{:.0}%", value * 100.0),
            Self::Delta => format!("{value:+.3}"),
        }
    }
}

struct FieldSpec {
    field: Field,
    id: &'static str,
    label: &'static str,
    min: f32,
    max: f32,
    step: f32,
    unit: Unit,
}

const SHAPE: [FieldSpec; 1] = [FieldSpec {
    field: Field::Radius,
    id: "radius",
    label: "Corner radius",
    min: 0.0,
    max: 25.0,
    step: 0.5,
    unit: Unit::Px,
}];

const FACE: [FieldSpec; 3] = [
    FieldSpec {
        field: Field::FillTop,
        id: "fill-top",
        label: "Wash top",
        min: -0.1,
        max: 0.1,
        step: 0.005,
        unit: Unit::Delta,
    },
    FieldSpec {
        field: Field::FillBottom,
        id: "fill-bottom",
        label: "Wash bottom",
        min: -0.1,
        max: 0.2,
        step: 0.005,
        unit: Unit::Delta,
    },
    FieldSpec {
        field: Field::Border,
        id: "border",
        label: "Border",
        min: -0.2,
        max: 0.05,
        step: 0.005,
        unit: Unit::Delta,
    },
];

const BEVEL: [FieldSpec; 7] = [
    FieldSpec {
        field: Field::RimLighten,
        id: "rim-lighten",
        label: "Rim lighten",
        min: 0.0,
        max: 0.5,
        step: 0.01,
        unit: Unit::Delta,
    },
    FieldSpec {
        field: Field::RimOffset,
        id: "rim-y",
        label: "Rim offset",
        min: 0.0,
        max: 6.0,
        step: 0.5,
        unit: Unit::Px,
    },
    FieldSpec {
        field: Field::RimAlpha,
        id: "rim-a",
        label: "Rim alpha",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        unit: Unit::Alpha,
    },
    FieldSpec {
        field: Field::FadeOffset,
        id: "fade-y",
        label: "Fade offset",
        min: 0.0,
        max: 12.0,
        step: 0.5,
        unit: Unit::Px,
    },
    FieldSpec {
        field: Field::FadeBlur,
        id: "fade-blur",
        label: "Fade blur",
        min: 0.0,
        max: 24.0,
        step: 0.5,
        unit: Unit::Px,
    },
    FieldSpec {
        field: Field::FadeSpread,
        id: "fade-spread",
        label: "Fade spread",
        min: -8.0,
        max: 4.0,
        step: 0.5,
        unit: Unit::Px,
    },
    FieldSpec {
        field: Field::FadeAlpha,
        id: "fade-a",
        label: "Fade alpha",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        unit: Unit::Alpha,
    },
];

const BOTTOM: [FieldSpec; 3] = [
    FieldSpec {
        field: Field::BevelDarken,
        id: "bevel-darken",
        label: "Lip darken",
        min: 0.0,
        max: 0.3,
        step: 0.005,
        unit: Unit::Delta,
    },
    FieldSpec {
        field: Field::BevelOffset,
        id: "bevel-y",
        label: "Lip offset",
        min: 0.0,
        max: 6.0,
        step: 0.5,
        unit: Unit::Px,
    },
    FieldSpec {
        field: Field::BevelAlpha,
        id: "bevel-a",
        label: "Lip alpha",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        unit: Unit::Alpha,
    },
];

const DROP: [FieldSpec; 3] = [
    FieldSpec {
        field: Field::DropOffset,
        id: "drop-y",
        label: "Offset",
        min: 0.0,
        max: 12.0,
        step: 0.5,
        unit: Unit::Px,
    },
    FieldSpec {
        field: Field::DropBlur,
        id: "drop-blur",
        label: "Blur",
        min: 0.0,
        max: 24.0,
        step: 0.5,
        unit: Unit::Px,
    },
    FieldSpec {
        field: Field::DropAlpha,
        id: "drop-a",
        label: "Alpha",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        unit: Unit::Alpha,
    },
];

const GROUPS: [(&str, &[FieldSpec]); 5] = [
    ("Shape", &SHAPE),
    ("Face & border", &FACE),
    ("Top bevel", &BEVEL),
    ("Bottom bevel", &BOTTOM),
    ("Drop shadow", &DROP),
];

pub enum ClassicShadowEditorEvent {
    /// The look was retuned; ancestors should repaint.
    Changed,
}

pub struct ClassicShadowEditor {
    look: Arc<Look>,
    params: ClassicButtonParams,
    sliders: Vec<(Field, Slider)>,
    preview: Entity<Button>,
    reset: Entity<Button>,
    copy: Entity<Button>,
    /// Shared with the copy button's content closure so the face can confirm the copy.
    copied: Arc<AtomicBool>,
}

impl EventEmitter<ClassicShadowEditorEvent> for ClassicShadowEditor {}

impl ClassicShadowEditor {
    pub fn new(look: &Arc<Look>, cx: &mut Context<Self>) -> Self {
        let mut params = look.classic_params();
        let mut sliders = Vec::new();

        for spec in GROUPS.iter().flat_map(|(_, specs)| specs.iter()) {
            let field = spec.field;
            let control = radix::Slider::new(format!("classic-shadow-{}", spec.id))
                .look(look)
                .range(spec.min..spec.max)
                .step(spec.step)
                .value(*field.slot(&mut params))
                .spawn(cx);
            control.clone().on_event(cx, move |this, event: &SliderEvent, cx| {
                let value = match event {
                    SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                    _ => return,
                };
                this.set_field(field, value, cx);
            });
            sliders.push((field, control));
        }

        let preview = radix::Button::new("classic-shadow-preview")
            .look(look)
            .classic()
            .size(ButtonSize::Three)
            .label("Next")
            .with_template_modifier(|button, _| button.h(px(PREVIEW_HEIGHT)).w(px(PREVIEW_WIDTH)))
            .spawn(cx);
        let reset = radix::Button::new("classic-shadow-reset").look(look).soft().label("Reset").spawn(cx);
        reset.clone().on_click(cx, |this, cx| {
            this.reset(cx);
        });

        let copied = Arc::new(AtomicBool::new(false));
        let copy_state = Arc::clone(&copied);
        let copy = radix::Button::new("classic-shadow-copy")
            .look(look)
            .ghost_quiet()
            .content(move |model, _| {
                let color = model.look.foreground;
                let name = if copy_state.load(Ordering::Relaxed) {
                    "check"
                } else {
                    "clipboard-copy"
                };
                icon_named(name)
                    .map(|icon| react_icon(icon, color, COPY_ICON_SIZE))
                    .unwrap_or_else(|| div().into_any_element())
            })
            .spawn(cx)
            .on_click(cx, |this, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(this.snippet()));
                this.copied.store(true, Ordering::Relaxed);
                cx.notify();
            });

        Self { look: Arc::clone(look), params, sliders, preview, reset, copy, copied }
    }

    pub(crate) fn reset(&mut self, cx: &mut Context<Self>) {
        self.set_params(ClassicButtonParams::default(), cx);
    }

    fn set_field(&mut self, field: Field, value: f32, cx: &mut Context<Self>) {
        *field.slot(&mut self.params) = value;
        self.publish(cx);
    }

    fn set_params(&mut self, params: ClassicButtonParams, cx: &mut Context<Self>) {
        self.params = params;
        for (field, control) in self.sliders.clone() {
            let value = *field.slot(&mut self.params);
            control.update(cx, |slider, cx| slider.set_value(value, cx));
        }
        self.publish(cx);
    }

    fn publish(&mut self, cx: &mut Context<Self>) {
        // Any edit invalidates what is on the clipboard, so the tick goes away.
        self.copied.store(false, Ordering::Relaxed);
        self.look.set_classic_params(self.params);
        cx.emit(ClassicShadowEditorEvent::Changed);
        cx.notify();
    }

    fn slider_for(&self, field: Field) -> Option<Slider> {
        self.sliders.iter().find(|(candidate, _)| *candidate == field).map(|(_, control)| control.clone())
    }

    fn row(&self, spec: &FieldSpec, fg: Hsla, muted: Hsla) -> AnyElement {
        let mut params = self.params;
        let value = *spec.field.slot(&mut params);

        hstack! {
            gap=10;
            div().w(px(LABEL_WIDTH)).flex_none().text_xs().text_color(muted).child(spec.label),
            div().flex_1().children(self.slider_for(spec.field)),
            div()
                .w(px(VALUE_WIDTH))
                .flex_none()
                .font_family("Monaco")
                .text_xs()
                .text_color(fg)
                .child(spec.unit.format(value)),
        }
        .w_full()
        .items_center()
        .into_any_element()
    }

    fn group(&self, title: &'static str, specs: &[FieldSpec], fg: Hsla, muted: Hsla) -> AnyElement {
        vstack! {
            gap=8;
            div().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(fg).child(title),
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(specs.iter().map(|spec| self.row(spec, fg, muted))),
        }
        .w_full()
        .into_any_element()
    }

    /// Current values as `field: value` pairs, in declaration order.
    fn fields(&self) -> [String; 17] {
        let p = self.params;
        [
            format!("radius: {:.1}", p.radius),
            format!("fill_top_delta: {:.3}", p.fill_top_delta),
            format!("fill_bottom_delta: {:.3}", p.fill_bottom_delta),
            format!("border_delta: {:.3}", p.border_delta),
            format!("bevel_darken: {:.3}", p.bevel_darken),
            format!("bevel_offset: {:.1}", p.bevel_offset),
            format!("bevel_alpha: {:.2}", p.bevel_alpha),
            format!("rim_lighten: {:.2}", p.rim_lighten),
            format!("rim_offset: {:.1}", p.rim_offset),
            format!("rim_alpha: {:.2}", p.rim_alpha),
            format!("fade_offset: {:.1}", p.fade_offset),
            format!("fade_blur: {:.1}", p.fade_blur),
            format!("fade_spread: {:.1}", p.fade_spread),
            format!("fade_alpha: {:.2}", p.fade_alpha),
            format!("drop_offset: {:.1}", p.drop_offset),
            format!("drop_blur: {:.1}", p.drop_blur),
            format!("drop_alpha: {:.2}", p.drop_alpha),
        ]
    }

    fn readout(&self) -> String {
        self.fields().join(", ")
    }

    /// A `ClassicButtonParams` literal that drops straight into `Default::default`.
    fn snippet(&self) -> String {
        let body = self.fields().map(|field| format!("    {field},")).join("\n");
        format!("ClassicButtonParams {{\n{body}\n}}")
    }
}

impl Render for ClassicShadowEditor {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let fg = self.look.resolve_role(SemanticRole::Foreground).hsla();
        let muted = self.look.resolve_role(SemanticRole::MutedForeground).hsla();
        let border = self.look.resolve_role(SemanticRole::Border).hsla();
        let surface = self.look.resolve_role(SemanticRole::Surface).hsla();

        hstack! {
            gap=28;
            vstack! {
                gap=14;
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .p(px(32.0))
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(border)
                    .bg(surface)
                    .child(self.preview.clone()),
                hstack! { gap=8 align=center justify=center; self.reset.clone(), self.copy.clone() }.w_full(),
                div().w_full().font_family("Monaco").text_xs().text_color(muted).child(self.readout()),
            }
            .w(px(300.0))
            .flex_none(),
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(18.0))
                .children(GROUPS.iter().map(|(title, specs)| self.group(title, specs, fg, muted))),
        }
        .w_full()
        .items_start()
    }
}
