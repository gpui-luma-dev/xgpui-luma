//! Numeric fields attached to retained SDK sliders; owners keep their existing models.
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};
use gpui::{App, Context};
use xgpui_luma::controls::{slider::Slider, textfield::TextField};
use xgpui_luma_look_radix::{self as radix, Look};
use crate::screen::{Param, value_field_look};

pub(crate) struct Number {
    pub field: TextField,
    slider: Slider,
    label: &'static str,
    range: String,
    value: Arc<AtomicU32>,
    decimals: usize,
    bounds: (f32, f32),
}
impl Number {
    #[allow(clippy::too_many_arguments)]
    pub fn attach<M: 'static>(
        id: &str,
        label: &'static str,
        range: (f32, f32, f32),
        value: f32,
        slider: Slider,
        look: &Look,
        cx: &mut Context<M>,
        edit: impl Fn(&mut M, f32, &mut Context<M>) -> f32 + 'static,
    ) -> Self {
        let (min, max, step) = range;
        let decimals = if step >= 1. { 1 } else { 3 };
        let current = Arc::new(AtomicU32::new(value.to_bits()));
        let state = current.clone();
        let validation = current.clone();
        let peer = slider.clone();
        type Normalize = Rc<RefCell<Option<Box<dyn Fn(&mut App)>>>>;
        let normalize: Normalize = Rc::new(RefCell::new(None));
        let blur = normalize.clone();
        let submit = normalize.clone();
        let field = radix::TextField::new(format!("{id}-field"))
            .look(look)
            .surface()
            .look_override(value_field_look(look))
            .full_width(true)
            .value(format_value(value, decimals, min, max))
            .select_all_on_tab_focus(true)
            .validator(Arc::new(move |text| {
                text.trim().parse::<f32>().is_ok_and(|value| {
                    value.is_finite()
                        && ((min..=max).contains(&value) || value == f32::from_bits(validation.load(Ordering::Relaxed)))
                })
            }))
            .on_value_changed(cx, move |owner, text, cx| {
                if let Ok(value) = text.trim().parse::<f32>()
                    && value.is_finite()
                    && (min..=max).contains(&value)
                {
                    let value = edit(owner, value, cx);
                    state.store(value.to_bits(), Ordering::Relaxed);
                    peer.update(cx, |slider, cx| slider.set_value(value, cx));
                    cx.notify();
                }
            })
            .on_focus_changed(cx, move |_, focused, cx| {
                if !focused && let Some(normalize) = blur.borrow().as_ref() {
                    normalize(cx);
                }
            })
            .on_submit(cx, move |_, _, cx| {
                if let Some(normalize) = submit.borrow().as_ref() {
                    normalize(cx);
                }
            })
            .spawn(cx);
        let weak = field.downgrade();
        let formatted = current.clone();
        *normalize.borrow_mut() = Some(Box::new(move |cx| {
            if let Some(field) = weak.upgrade() {
                field.update(cx, |field, cx| {
                    field.set_value(
                        format_value(f32::from_bits(formatted.load(Ordering::Relaxed)), decimals, min, max),
                        cx,
                    )
                });
            }
        }));
        Self { field, slider, label, range: format!("{min}–{max}"), value: current, decimals, bounds: (min, max) }
    }
    pub fn sync(&self, value: f32, cx: &mut App) {
        self.value.store(value.to_bits(), Ordering::Relaxed);
        self.field.update(cx, |field, cx| {
            field.set_value(format_value(value, self.decimals, self.bounds.0, self.bounds.1), cx)
        });
        self.slider.update(cx, |slider, cx| slider.set_value(value, cx));
    }
    pub fn observe(&self, value: f32, cx: &mut App) {
        if f32::from_bits(self.value.load(Ordering::Relaxed)) != value {
            self.sync(value, cx);
        }
    }
    pub fn param(&self) -> Param {
        Param::new(self.label, self.field.clone(), self.slider.clone()).range(self.range.clone())
    }
}

// Rounding an irrational endpoint (for example PI) must not put its own field
// outside the accepted range. Keep round-trip precision at those endpoints.
fn format_value(value: f32, decimals: usize, min: f32, max: f32) -> String {
    let text = format!("{value:.decimals$}");
    if text.parse::<f32>().is_ok_and(|rounded| (min..=max).contains(&rounded)) {
        text
    } else {
        value.to_string()
    }
}
