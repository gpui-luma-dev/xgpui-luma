//! The checkbox template driven by the Luma style language (issue #285).
//!
//! It reads every value from a [`Theme`] (the SDK's `.lss` files, plus a look's). A look whose mode or
//! palette changes while running passes a [`ThemeProvider`], asked on every render.

use std::sync::Arc;

use gpui::{AnyElement, App, Div, Hsla, Stateful, Window, div, hsla, prelude::*, px};
use xgpui_luma_style::{
    AttrId, AttrValue, ControlId, Overrides, PartId, PartView, Prop, Resolved, StateId, StyleKey, Theme, Value,
    box_shadows,
};

use crate::controls::button::{ButtonRenderModel, ButtonTemplate, button_content_context};
use crate::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
use crate::controls::checkbox::CheckboxData;
use crate::infra::focus_adorner::FocusAdorner;
use crate::infra::shadow_layout::should_paint_shadow;
use crate::infra::template::{Modifier, TemplateWithModifiers};
use crate::theme::{FocusMetricTokens, LumaTextStyle};

/// Handles looked up once from the theme's stylesheet.
struct Handles {
    control: ControlId,
    indicator: PartId,
    glyph: PartId,
    label: PartId,
    hover: StateId,
    pressed: StateId,
    focused: StateId,
    disabled: StateId,
    invalid: StateId,
    checked: StateId,
    indicator_size: Prop<f32>,
    indicator_radius: Prop<f32>,
    glyph_size: Prop<f32>,
    label_baseline_shift: Prop<f32>,
    disabled_opacity: Prop<f32>,
    focus_gap: Prop<f32>,
    focus_width: Prop<f32>,
    focus_offset: Prop<f32>,
}

impl Handles {
    fn new(theme: &Theme) -> Self {
        let sheet = theme.stylesheet();
        let config = sheet.config();
        let control = sheet.control("checkbox").expect("the SDK style files declare checkbox");
        let part = |name: &str| sheet.part(control, name).expect("checkbox part");
        let prop = |name: &str| sheet.prop::<f32>(control, name).expect("checkbox property");
        let state = |name: &str| config.state(name).expect("SDK state");
        Self {
            control,
            indicator: part("indicator"),
            glyph: part("glyph"),
            label: part("label"),
            hover: state("hover"),
            pressed: state("pressed"),
            focused: state("focused"),
            disabled: state("disabled"),
            invalid: state("invalid"),
            checked: state("checked"),
            indicator_size: prop("indicator_size"),
            indicator_radius: prop("indicator_radius"),
            glyph_size: prop("glyph_size"),
            label_baseline_shift: prop("label_baseline_shift"),
            disabled_opacity: prop("disabled_opacity"),
            focus_gap: prop("focus_gap"),
            focus_width: prop("focus_width"),
            focus_offset: prop("focus_offset"),
        }
    }
}

/// The SDK's default checkbox template: the SDK's `.lss` files alone.
pub fn default_template() -> std::sync::Arc<dyn ButtonTemplate<CheckboxData>> {
    static TEMPLATE: std::sync::OnceLock<std::sync::Arc<dyn ButtonTemplate<CheckboxData>>> = std::sync::OnceLock::new();
    TEMPLATE.get_or_init(|| std::sync::Arc::new(StyledCheckboxTemplate::sdk())).clone()
}

/// The current theme of a look whose mode or palette can change. Called on every render, so it should
/// return a cached [`Theme`] (cloning one is cheap) and rebind only when the look changed.
pub type ThemeProvider = Arc<dyn Fn() -> Theme + Send + Sync>;

enum ThemeSource {
    Fixed(Theme),
    Live(ThemeProvider),
}

/// A checkbox template whose values come from `.lss` files.
pub struct StyledCheckboxTemplate {
    source: ThemeSource,
    handles: Handles,
    attrs: Vec<(AttrId, AttrValue)>,
    states: Vec<StateId>,
    overrides: Overrides,
    modifiers: Vec<Modifier<ButtonRenderModel<CheckboxData>>>,
}

impl StyledCheckboxTemplate {
    /// A template that always uses `theme`.
    pub fn new(theme: Theme) -> Self {
        let handles = Handles::new(&theme);
        Self::with_source(ThemeSource::Fixed(theme), handles)
    }

    /// A template that asks `provider` for the theme on every render. Every theme it returns must share
    /// the first one's configuration (state and attribute handles are looked up once).
    pub fn live(provider: ThemeProvider) -> Self {
        let handles = Handles::new(&provider());
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

    fn theme(&self) -> Theme {
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

    fn key(&self, model: &ButtonRenderModel<CheckboxData>, checked: bool, scale: f32) -> StyleKey {
        let h = &self.handles;
        let s = model.state;
        let key = StyleKey::new(h.control)
            .state_if(h.hover, s.hovered)
            .state_if(h.pressed, s.pressed)
            .state_if(h.focused, s.focused)
            .state_if(h.disabled, s.disabled)
            .state_if(h.invalid, s.invalid)
            .state_if(h.checked, checked)
            .scale(scale);
        let key = self.states.iter().fold(key, |key, state| key.state(*state));
        self.attrs.iter().fold(key, |key, (attr, value)| key.attr(*attr, *value))
    }
}

impl TemplateWithModifiers<ButtonRenderModel<CheckboxData>> for StyledCheckboxTemplate {
    fn modifiers(&self) -> &[Modifier<ButtonRenderModel<CheckboxData>>] {
        &self.modifiers
    }
}

fn hsl(color: Option<xgpui_luma_style::Color>) -> Option<Hsla> {
    color.map(Hsla::from)
}

fn transparent() -> Hsla {
    hsla(0.0, 0.0, 0.0, 0.0)
}

impl ButtonTemplate<CheckboxData> for StyledCheckboxTemplate {
    /// The root's `transition`, chosen by the destination state (checked or unchecked).
    fn transition(&self, data: &CheckboxData) -> Option<crate::motion::Transition> {
        let h = &self.handles;
        let key = StyleKey::new(h.control).state_if(h.checked, data.checked);
        let key = self.states.iter().fold(key, |key, state| key.state(*state));
        let key = self.attrs.iter().fold(key, |key, (attr, value)| key.attr(*attr, *value));
        let resolved = self.theme().resolve_with(&key, &self.overrides);
        resolved
            .root()
            .get(xgpui_luma_style::props::TRANSITION)
            .map(|motion| crate::style::transition(&motion))
    }

    fn render(&self, model: &ButtonRenderModel<CheckboxData>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let h = &self.handles;
        let progress = model.data.progress.clamp(0.0, 1.0);
        let checked = model.data.checked;
        let scale = window.scale_factor();
        let theme = self.theme();
        let off = theme.resolve_with(&self.key(model, false, scale), &self.overrides);
        let on = theme.resolve_with(&self.key(model, true, scale), &self.overrides);
        // Colors animate between unchecked and checked; everything else uses the settled state.
        let animated = Resolved::lerp(&off, &on, progress);
        let settled = if checked { on } else { off };

        let root = settled.root();
        let num = |part: PartView<'_>, name: &str| part.value(name).and_then(Value::as_number).unwrap_or(0.0);
        let indicator_size = root.get(h.indicator_size).unwrap_or(0.0);
        let indicator_radius = root.get(h.indicator_radius).unwrap_or(0.0);
        let glyph_size = root.get(h.glyph_size).unwrap_or(0.0);
        let border_width = num(root, "border_width");
        let control_radius = root.radius().unwrap_or(0.0);
        let gap = root.gap().unwrap_or(0.0);
        let height = root.height().unwrap_or(0.0);
        let padding_x = num(root, "padding_x");
        let padding_y = num(root, "padding_y");

        let label = settled.part(h.label);
        let label_color = hsl(label.foreground()).unwrap_or_else(transparent);
        let typography = LumaTextStyle {
            size: label.font_size().unwrap_or(14.0),
            line_height: label.line_height().unwrap_or(20.0),
            weight: gpui::FontWeight(num(label, "font_weight").max(100.0)),
        };
        let font_family: gpui::SharedString = label.font_family().unwrap_or("").to_string().into();

        let focus = FocusAdorner::new(FocusMetricTokens {
            gap: root.get(h.focus_gap).unwrap_or(0.0),
            width: root.get(h.focus_width).unwrap_or(0.0),
            offset: root.get(h.focus_offset).unwrap_or(0.0),
        });
        let focus_ring_extent = focus.extent();
        let focus_color = hsl(root.focus_border()).unwrap_or_else(transparent);
        let focused = model.state.focused && !model.state.disabled;

        let animated_indicator = animated.part(h.indicator);
        let mut indicator = div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(indicator_size))
            .bg(hsl(animated_indicator.background()).unwrap_or_else(transparent))
            .border(px(border_width))
            .border_color(hsl(animated_indicator.border()).unwrap_or_else(transparent))
            .rounded(px(indicator_radius))
            .child(render_checkmark(
                progress,
                glyph_size,
                hsl(animated.part(h.glyph).foreground()).unwrap_or_else(transparent),
                &model.data.icons.selected,
            ));
        if let Some(layers) = settled.part(h.indicator).shadow()
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
            .id(model.id.clone())
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
        if matches!(model.role, ButtonFamilyRole::Icon) {
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
                icon_size: glyph_size,
                shadow: None,
            };
            let content_model = button_content_context(model, content_look);
            let baseline = root.get(h.label_baseline_shift).unwrap_or(0.0);
            let label = div().mt(px(baseline)).child((model.content)(&content_model, cx));
            control = control
                .gap(px(gap))
                .min_h(px(height))
                .px(px(padding_x))
                .py(px(padding_y))
                .child(indicator)
                .child(label);
        }

        let animated_root = animated.root();
        if let Some(background) = hsl(animated_root.background()) {
            control = control.bg(background);
        }
        // Optional paint must not add or remove a layout border when focus changes.
        control = control
            .border(px(border_width))
            .border_color(hsl(animated_root.border()).unwrap_or_else(transparent));

        let mut root_element = self.apply_modifiers(control, model);
        if model.state.disabled {
            root_element = root_element.opacity(root.get(h.disabled_opacity).unwrap_or(1.0));
        } else {
            root_element = root_element.cursor_pointer();
        }
        root_element
    }
}

fn render_checkmark(progress: f32, size: f32, color: Hsla, icon: &crate::infra::icon::IconSource) -> AnyElement {
    if progress <= f32::EPSILON {
        return div().size(px(size)).into_any_element();
    }
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .opacity(progress)
        .child(crate::infra::icon::render_icon_source(icon, color, size))
        .into_any_element()
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::IntoElement;

    use crate::theme::InteractionState;

    use super::*;
    use crate::controls::button::{ButtonRenderModel, ButtonTemplate};

    fn model(state: InteractionState, checked: bool, role: ButtonFamilyRole) -> ButtonRenderModel<CheckboxData> {
        ButtonRenderModel { data: CheckboxData::new(checked), state, role, ..Default::default() }
    }

    #[test]
    fn styled_checkbox_layout_is_stable_across_states_and_scales() {
        for role in [ButtonFamilyRole::Text, ButtonFamilyRole::Icon] {
            let template = StyledCheckboxTemplate::sdk();
            crate::infra::focus_adorner::testing::assert_stable_layout(move |state, checked, window, cx| {
                template.render(&model(state, checked, role), window, cx).into_any_element()
            });
        }
    }
}
