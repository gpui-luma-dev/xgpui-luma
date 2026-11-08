//! Look-owned progress builder backed by the SDK's progress value and animation model.
use gpui::{Context, Div, SharedString, Stateful};
use xgpui_luma::controls::progress::{ProgressBuilder, ProgressDirection, ProgressRenderModel};
use xgpui_luma::infra::value::ControlRange;
use crate::look::resolve_look;
use crate::{Look, Paint, ProgressSize, ProgressVariant, Radius, Tone, progress_template};

type TemplateModifier = Box<dyn Fn(Stateful<Div>, &ProgressRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Radix progress bar; call `.spawn(cx)` to create the SDK control.
///
/// ```no_run
/// use gpui::Context;
/// use xgpui_luma_look_radix::{Look, Progress, ProgressSize, Radius};
///
/// fn loading<M: 'static>(look: &Look, cx: &mut Context<M>) -> xgpui_luma::controls::progress::Progress {
///     Progress::new("loading").look(look).soft().size(ProgressSize::Three)
///         .radius(Radius::Full).high_contrast(true).range(0..100).value(40).spawn(cx)
/// }
/// ```
pub struct Progress {
    look: Option<Look>,
    builder: ProgressBuilder,
    variant: ProgressVariant,
    size: ProgressSize,
    radius: Radius,
    paint: Paint,
    modifiers: Vec<TemplateModifier>,
}
impl Progress {
    xgpui_luma::forward_motion!(xgpui_luma::controls::progress::ProgressMotion, builder builder);

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            builder: ProgressBuilder::new(id).linear(),
            variant: ProgressVariant::default(),
            size: ProgressSize::default(),
            radius: Radius::default(),
            paint: Paint::accent(),
            modifiers: Vec::new(),
        }
    }
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }
    pub fn variant(mut self, variant: ProgressVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn surface(self) -> Self {
        self.variant(ProgressVariant::Surface)
    }
    pub fn soft(self) -> Self {
        self.variant(ProgressVariant::Soft)
    }
    pub fn size(mut self, size: ProgressSize) -> Self {
        self.size = size;
        self
    }
    pub fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.paint.tone = tone;
        self
    }
    pub fn high_contrast(mut self, high_contrast: bool) -> Self {
        self.paint.high_contrast = high_contrast;
        self
    }
    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.builder = self.builder.range(range);
        self
    }
    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.builder = self.builder.value(value);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }
    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(motion_enabled);
        self
    }
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.builder = self.builder.indeterminate(indeterminate);
        self
    }
    pub fn direction(mut self, direction: ProgressDirection) -> Self {
        self.builder = self.builder.direction(direction);
        self
    }
    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ProgressRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> xgpui_luma::controls::progress::Progress {
        let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
        let mut builder = self
            .builder
            .motion_defaults(look.motion_defaults().progress)
            .size(self.size.control_size())
            .template(progress_template(&look, self.variant, self.paint, self.radius));
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(modifier);
        }
        builder.spawn(cx)
    }
}
