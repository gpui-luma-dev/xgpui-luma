//! The template shared by choice controls driven by the Luma style language (issue #285).
//!
//! [`StyledChoiceTemplate`] is the shared core: the theme (fixed, or a [`ThemeProvider`] asked on every
//! render), fixed attributes and states, instance overrides, modifiers, style keys and the `transition`
//! hook. A control supplies its data type through [`ChoiceData`] and draws itself in
//! [`ChoiceData::render`]. Controls drawn as an indicator box with a mark inside (checkbox, radio)
//! implement [`IndicatorChoice`] and render with [`render_indicator`]; the switch draws a track and thumb.

use std::sync::Arc;

use gpui::{AnyElement, App, Div, ElementId, Hsla, SharedString, Stateful, Window, div, hsla, prelude::*, px};
use xgpui_luma_style::{
    AttrId, AttrValue, ControlId, Overrides, PartId, PartView, Prop, Resolved, StateId, StyleKey, Theme, Value,
    box_shadows,
};

use crate::controls::button::{ButtonRenderModel, ButtonTemplate, button_content_context};
use crate::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
use crate::infra::focus_adorner::FocusAdorner;
use crate::infra::shadow_layout::should_paint_shadow;
use crate::infra::template::{Modifier, TemplateWithModifiers};
use crate::theme::{FocusMetricTokens, LumaTextStyle};

/// The current theme of a look whose mode or palette can change. Called on every render, so it should
/// return a cached [`Theme`] (cloning one is cheap) and rebind only when the look changed.
pub type ThemeProvider = Arc<dyn Fn() -> Theme + Send + Sync>;

/// A choice control's data, as the shared template needs it.
pub trait ChoiceData: Clone + Send + Sync + 'static {
    /// The control in the `.lss` files, e.g. `"checkbox"`.
    const CONTROL: &'static str;
    /// The state meaning "on", e.g. `"checked"`.
    const ON_STATE: &'static str;

    fn is_on(&self) -> bool;
    fn progress(&self) -> f32;
    /// The data that is on or off, for asking the template's transition.
    fn toward(on: bool) -> Self;
    /// The root element id.
    fn root_id(id: &SharedString) -> ElementId {
        id.clone().into()
    }
    /// Draw the control from the template's theme.
    fn render(
        template: &StyledChoiceTemplate<Self>,
        model: &ButtonRenderModel<Self>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

/// A choice control drawn as an indicator box with a mark inside, a label and a focus ring.
pub trait IndicatorChoice: ChoiceData {
    /// The part drawn inside the indicator, e.g. `"glyph"`, and its size property, e.g. `"glyph_size"`.
    const MARK_PART: &'static str;
    const MARK_SIZE: &'static str;
    /// Whether the indicator-only (icon) role keeps the row's gap, height and padding.
    const ICON_ROLE_KEEPS_ROW: bool = false;

    /// The mark, drawn at `progress` (0 off, 1 on).
    fn render_mark(&self, progress: f32, size: f32, color: Hsla) -> AnyElement;
}

/// State handles looked up once from the theme's configuration.
struct Handles {
    control: ControlId,
    hover: StateId,
    pressed: StateId,
    focused: StateId,
    disabled: StateId,
    invalid: StateId,
    on: StateId,
}

impl Handles {
    fn new<D: ChoiceData>(theme: &Theme) -> Self {
        let sheet = theme.stylesheet();
        let config = sheet.config();
        let control = sheet.control(D::CONTROL).expect("the SDK style files declare the choice control");
        let state = |name: &str| config.state(name).expect("SDK state");
        Self {
            control,
            hover: state("hover"),
            pressed: state("pressed"),
            focused: state("focused"),
            disabled: state("disabled"),
            invalid: state("invalid"),
            on: state(D::ON_STATE),
        }
    }
}

enum ThemeSource {
    Fixed(Theme),
    Live(ThemeProvider),
}

/// A choice control template whose values come from `.lss` files.
pub struct StyledChoiceTemplate<D: ChoiceData> {
    source: ThemeSource,
    handles: Handles,
    attrs: Vec<(AttrId, AttrValue)>,
    states: Vec<StateId>,
    overrides: Overrides,
    modifiers: Vec<Modifier<ButtonRenderModel<D>>>,
}

impl<D: ChoiceData> StyledChoiceTemplate<D> {
    /// A template that always uses `theme`.
    pub fn new(theme: Theme) -> Self {
        let handles = Handles::new::<D>(&theme);
        Self::with_source(ThemeSource::Fixed(theme), handles)
    }

    /// A template that asks `provider` for the theme on every render. Every theme it returns must share
    /// the first one's configuration (state and attribute handles are looked up once).
    pub fn live(provider: ThemeProvider) -> Self {
        let handles = Handles::new::<D>(&provider());
        Self::with_source(ThemeSource::Live(provider), handles)
    }

    fn with_source(source: ThemeSource, handles: Handles) -> Self {
        Self {
            source,
            handles,
            attrs: Vec::new(),
            states: Vec::new(),
            overrides: Overrides::new(),
            modifiers: Vec::new(),
        }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(crate::style::default_theme())
    }

    pub(crate) fn theme(&self) -> Theme {
        match &self.source {
            ThemeSource::Fixed(theme) => theme.clone(),
            ThemeSource::Live(provider) => provider(),
        }
    }

    /// A variant of a look, e.g. `"soft"`: shorthand for `attr("variant", variant)`.
    pub fn variant(self, variant: &str) -> Self {
        self.attr("variant", variant)
    }

    /// An attribute value for every instance, e.g. `attr("size", "sm")` or a look's own `attr("tone",
    /// "gray")`. An attribute the theme's configuration does not declare is ignored.
    pub fn attr(mut self, name: &str, value: &str) -> Self {
        if let Some(attr) = self.theme().stylesheet().config().attribute(name) {
            self.attrs.retain(|(a, _)| *a != attr);
            self.attrs.push((attr, AttrValue::named(value)));
        }
        self
    }

    /// A state that is always on or off for this template, e.g. a look's `state("high_contrast", true)`.
    /// A state the theme's configuration does not declare is ignored.
    pub fn state(mut self, name: &str, on: bool) -> Self {
        if let Some(state) = self.theme().stylesheet().config().state(name) {
            self.states.retain(|s| *s != state);
            if on {
                self.states.push(state);
            }
        }
        self
    }

    /// Values set on this instance.
    pub fn overrides(mut self, overrides: Overrides) -> Self {
        self.overrides = overrides;
        self
    }

    /// Change the rendered root after the template builds it (e.g. a radio group's item chrome).
    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ButtonRenderModel<D>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    /// The fixed attributes and states on a key for the control.
    fn base_key(&self) -> StyleKey {
        let key = StyleKey::new(self.handles.control);
        let key = self.states.iter().fold(key, |key, state| key.state(*state));
        self.attrs.iter().fold(key, |key, (attr, value)| key.attr(*attr, *value))
    }

    pub(crate) fn key(&self, model: &ButtonRenderModel<D>, on: bool, scale: f32) -> StyleKey {
        let h = &self.handles;
        let s = model.state;
        self.base_key()
            .state_if(h.hover, s.hovered)
            .state_if(h.pressed, s.pressed)
            .state_if(h.focused, s.focused)
            .state_if(h.disabled, s.disabled)
            .state_if(h.invalid, s.invalid)
            .state_if(h.on, on)
            .scale(scale)
    }
}

impl<D: ChoiceData> StyledChoiceTemplate<D> {
    /// The off and on styles for a render (colors animate between them), and the settled one.
    pub(crate) fn resolve_pair(&self, model: &ButtonRenderModel<D>, scale: f32) -> (Resolved, Arc<Resolved>) {
        let theme = self.theme();
        let off = theme.resolve_with(&self.key(model, false, scale), &self.overrides);
        let on = theme.resolve_with(&self.key(model, true, scale), &self.overrides);
        let animated = Resolved::lerp(&off, &on, model.data.progress().clamp(0.0, 1.0));
        (animated, if model.data.is_on() { on } else { off })
    }
}

/// A length property on a part (`0` when unset).
pub(crate) fn length(part: PartView<'_>, name: &'static str) -> f32 {
    part.get(Prop::<f32>::new(name))
        .or_else(|| part.value(name).and_then(Value::as_number))
        .unwrap_or(0.0)
}

/// The label's color, text style and font family.
pub(crate) fn label_style(label: PartView<'_>) -> (Hsla, LumaTextStyle, SharedString) {
    let color = hsl(label.foreground()).unwrap_or_else(transparent);
    let typography = LumaTextStyle {
        size: label.font_size().unwrap_or(14.0),
        line_height: label.line_height().unwrap_or(20.0),
        weight: gpui::FontWeight(label.value("font_weight").and_then(Value::as_number).unwrap_or(0.0).max(100.0)),
    };
    (color, typography, label.font_family().unwrap_or("").to_string().into())
}

/// The focus ring from the root's `focus_gap`, `focus_width` and `focus_offset`.
pub(crate) fn focus_adorner(root: PartView<'_>) -> FocusAdorner {
    FocusAdorner::new(FocusMetricTokens {
        gap: length(root, "focus_gap"),
        width: length(root, "focus_width"),
        offset: length(root, "focus_offset"),
    })
}

impl<D: ChoiceData> TemplateWithModifiers<ButtonRenderModel<D>> for StyledChoiceTemplate<D> {
    fn modifiers(&self) -> &[Modifier<ButtonRenderModel<D>>] {
        &self.modifiers
    }
}

pub(crate) fn hsl(color: Option<xgpui_luma_style::Color>) -> Option<Hsla> {
    color.map(Hsla::from)
}

pub(crate) fn transparent() -> Hsla {
    hsla(0.0, 0.0, 0.0, 0.0)
}

impl<D: ChoiceData> ButtonTemplate<D> for StyledChoiceTemplate<D> {
    /// The states [`StyledChoiceTemplate::key`] sets: interaction states and the control's on-state.
    fn style_states(&self) -> Vec<&'static str> {
        vec!["hover", "pressed", "focused", "disabled", "invalid", D::ON_STATE]
    }

    /// The root's `transition`, chosen by the destination state (on or off).
    fn transition(&self, data: &D) -> Option<crate::motion::Transition> {
        let key = self.base_key().state_if(self.handles.on, data.is_on());
        let resolved = self.theme().resolve_with(&key, &self.overrides);
        resolved
            .root()
            .get(xgpui_luma_style::props::TRANSITION)
            .map(|motion| crate::style::transition(&motion))
    }

    fn render(&self, model: &ButtonRenderModel<D>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        D::render(self, model, window, cx)
    }
}

/// The transition a choice control animates with toward `on`: the instance's override, else the
/// template's `transition` (`.lss`), else the SDK default template's.
pub(crate) fn choice_transition<D: ChoiceData>(
    template: &dyn ButtonTemplate<D>,
    sdk_default: &dyn ButtonTemplate<D>,
    transition_override: Option<&crate::motion::Transition>,
    on: bool,
) -> crate::motion::Transition {
    let data = D::toward(on);
    transition_override
        .cloned()
        .or_else(|| template.transition(&data))
        .or_else(|| sdk_default.transition(&data))
        .unwrap_or_else(crate::motion::Transition::instant)
}

/// Draw an indicator choice: the indicator box with its mark, the label, and the focus ring.
pub(crate) fn render_indicator<D: IndicatorChoice>(
    template: &StyledChoiceTemplate<D>,
    model: &ButtonRenderModel<D>,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    let progress = model.data.progress().clamp(0.0, 1.0);
    // Colors animate between off and on; everything else uses the settled state.
    let (animated, settled) = template.resolve_pair(model, window.scale_factor());

    let root = settled.root();
    let indicator_size = length(root, "indicator_size");
    let indicator_radius = length(root, "indicator_radius");
    let mark_size = length(root, D::MARK_SIZE);
    let border_width = length(root, "border_width");
    let control_radius = root.radius().unwrap_or(0.0);
    let gap = root.gap().unwrap_or(0.0);
    let height = root.height().unwrap_or(0.0);
    let padding_x = length(root, "padding_x");
    let padding_y = length(root, "padding_y");

    let (label_color, typography, font_family) = label_style(settled.part(PartId::new("label")));

    let focus = focus_adorner(root);
    let focus_ring_extent = focus.extent();
    let focus_color = hsl(root.focus_border()).unwrap_or_else(transparent);
    let focused = model.state.focused && !model.state.disabled;

    let animated_indicator = animated.part(PartId::new("indicator"));
    let mark_color = hsl(animated.part(PartId::new(D::MARK_PART)).foreground()).unwrap_or_else(transparent);
    let mut indicator = div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(indicator_size))
        .bg(hsl(animated_indicator.background()).unwrap_or_else(transparent))
        .border(px(border_width))
        .border_color(hsl(animated_indicator.border()).unwrap_or_else(transparent))
        .rounded(px(indicator_radius))
        .child(model.data.render_mark(progress, mark_size, mark_color));
    if let Some(layers) = settled.part(PartId::new("indicator")).shadow()
        && should_paint_shadow(model.elevation, model.state.disabled, !layers.is_empty())
    {
        indicator = indicator.shadow(box_shadows(layers));
    }

    let indicator = div().relative().child(indicator);
    let indicator = if focus_ring_extent > 0.0 {
        div()
            .id("indicator-slot")
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .size(px(indicator_size + focus_ring_extent * 2.0))
            .child(indicator)
            .child(focus.overlay(focus_color, focused, indicator_radius, false))
            .into_any_element()
    } else {
        indicator.into_any_element()
    };

    let mut control = div()
        .id(D::root_id(&model.id))
        .relative()
        .flex()
        .items_center()
        .text_color(label_color)
        .text_size(px(typography.size))
        .line_height(px(typography.line_height))
        .font_family(font_family.clone())
        .font_weight(typography.weight)
        .rounded(px(control_radius))
        .cursor_pointer();

    let control_background = hsl(root.background());
    let control_border = hsl(root.border());
    let row = |control: Stateful<Div>| control.gap(px(gap)).min_h(px(height)).px(px(padding_x)).py(px(padding_y));
    if matches!(model.role, ButtonFamilyRole::Icon) {
        if D::ICON_ROLE_KEEPS_ROW {
            control = row(control);
        }
        control = control.child(indicator);
    } else {
        let content_look = ButtonFamilyLook {
            focus_border: transparent(),
            background: control_background.unwrap_or_else(transparent),
            foreground: label_color,
            muted_foreground: label_color,
            border: control_border,
            typography,
            font_family,
            radius: control_radius,
            padding_x,
            padding_y,
            gap,
            height,
            icon_size: mark_size,
            shadow: None,
        };
        let content_model = button_content_context(model, content_look);
        let baseline = length(root, "label_baseline_shift");
        let label = div().mt(px(baseline)).child((model.content)(&content_model, cx));
        control = row(control).child(indicator).child(label);
    }

    let animated_root = animated.root();
    if let Some(background) = hsl(animated_root.background()) {
        control = control.bg(background);
    }
    // Optional paint must not add or remove a layout border when focus changes.
    control = control
        .border(px(border_width))
        .border_color(hsl(animated_root.border()).unwrap_or_else(transparent));

    let mut root_element = template.apply_modifiers(control, model);
    if model.state.disabled {
        root_element = root_element.opacity(root.value("disabled_opacity").and_then(Value::as_number).unwrap_or(1.0));
    } else {
        root_element = root_element.cursor_pointer();
    }
    root_element
}
