//! The button-family template driven by the Luma style language (issue #285).
//!
//! It draws text buttons, icon buttons and toggles like [`DefaultButtonTemplate`](super::DefaultButtonTemplate),
//! reading every value from a [`Theme`]: the `button` control, or `toggle` for the toggle role (a toggle
//! is selected through the `selected` state). Data that animates between off and on (a toggle's) blends
//! the face colors with its progress. A model's own `look` override still wins.

use std::sync::Arc;

use gpui::{App, Div, Hsla, Stateful, Window, div, prelude::*, px};
use xgpui_luma_style::{
    AttrId, AttrValue, ControlId, Overrides, PartView, Resolved, StateId, StyleKey, Theme, Value, box_shadows,
};

use super::{ButtonRenderModel, ButtonTemplate, button_content_context};
use crate::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole, button_family_effective_border};
use crate::controls::choice::styled::{ThemeProvider, focus_adorner, hsl, length, transparent};
use crate::infra::template::{Modifier, TemplateWithModifiers};
use crate::theme::{InteractionState, LumaTextStyle};

/// Button data, as the button template needs it: any cloneable data. Toggle data
/// ([`ToggleData`](crate::controls::toggle::ToggleData)) is on or off, and blends between the two.
pub trait ButtonData: Clone + 'static {
    /// For data that is on or off (a toggle's): whether it is on, and the animation progress toward on.
    fn toggle_state(&self) -> Option<(bool, f32)> {
        (self as &dyn std::any::Any)
            .downcast_ref::<crate::controls::toggle::ToggleData>()
            .map(|toggle| (toggle.selected, toggle.progress))
    }
}

impl<D: Clone + 'static> ButtonData for D {}

enum ThemeSource {
    Fixed(Theme),
    Live(ThemeProvider),
}

/// A button-family template whose values come from `.lss` files.
pub struct StyledButtonTemplate<D> {
    source: ThemeSource,
    attrs: Vec<(AttrId, AttrValue)>,
    states: Vec<StateId>,
    overrides: Overrides,
    modifiers: Vec<Modifier<ButtonRenderModel<D>>>,
}

impl<D: ButtonData> StyledButtonTemplate<D> {
    /// A template that always uses `theme`.
    pub fn new(theme: Theme) -> Self {
        Self::with_source(ThemeSource::Fixed(theme))
    }

    /// A template that asks `provider` for the theme on every render.
    pub fn live(provider: ThemeProvider) -> Self {
        Self::with_source(ThemeSource::Live(provider))
    }

    fn with_source(source: ThemeSource) -> Self {
        Self { source, attrs: Vec::new(), states: Vec::new(), overrides: Overrides::new(), modifiers: Vec::new() }
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

    /// A variant of a look, e.g. `"secondary"`: shorthand for `attr("variant", variant)`.
    pub fn variant(self, variant: &str) -> Self {
        self.attr("variant", variant)
    }

    /// An attribute value for every instance. An attribute the configuration does not declare is ignored.
    pub fn attr(mut self, name: &str, value: &str) -> Self {
        if let Some(attr) = self.theme().stylesheet().config().attribute(name) {
            self.attrs.retain(|(a, _)| *a != attr);
            self.attrs.push((attr, AttrValue::named(value)));
        }
        self
    }

    /// A state that is always on or off for this template. A state the configuration does not declare is ignored.
    pub fn state(mut self, name: &str, on: bool) -> Self {
        if let Some(state) = self.theme().stylesheet().config().state(name) {
            self.states.retain(|s| *s != state);
            if on {
                self.states.push(state);
            }
        }
        self
    }

    /// The settled face for `role` in `state`, as a button-family look: for chrome a look draws itself
    /// (control-group rows) from the same `.lss` values.
    pub fn look(&self, role: ButtonFamilyRole, state: InteractionState, scale_factor: f32) -> ButtonFamilyLook {
        let theme = self.theme();
        let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
        let resolved = theme.resolve_with(&self.key(&theme, role, state, selected, scale_factor), &self.overrides);
        look_from(resolved.root(), resolved.root(), role)
    }

    /// Values set on this instance.
    pub fn overrides(mut self, overrides: Overrides) -> Self {
        self.overrides = overrides;
        self
    }

    /// Change the rendered root after the template builds it.
    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ButtonRenderModel<D>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn key(
        &self,
        theme: &Theme,
        role: ButtonFamilyRole,
        model_state: crate::theme::InteractionState,
        selected: bool,
        scale: f32,
    ) -> StyleKey {
        let config = theme.stylesheet().config();
        let control = if matches!(role, ButtonFamilyRole::Toggle { .. }) {
            "toggle"
        } else {
            "button"
        };
        let key = StyleKey::new(ControlId::new(control)).scale(scale);
        let key = self.states.iter().fold(key, |key, state| key.state(*state));
        let mut key = self.attrs.iter().fold(key, |key, (attr, value)| key.attr(*attr, *value));
        for (name, on) in [
            ("hover", model_state.hovered),
            ("pressed", model_state.pressed),
            ("focused", model_state.focused),
            ("disabled", model_state.disabled),
            ("invalid", model_state.invalid),
            ("selected", selected),
        ] {
            if let Some(state) = config.state(name) {
                key = key.state_if(state, on);
            }
        }
        key
    }
}

/// A button-family theme read from `.lss`: the faces of a [`StyledButtonTemplate`], for chrome that takes a
/// [`ButtonFamilyTheme`](crate::controls::button_family::ButtonFamilyTheme).
pub struct StyledButtonFamilyTheme {
    template: StyledButtonTemplate<()>,
    metrics: crate::theme::MetricTokens,
}

impl StyledButtonFamilyTheme {
    pub fn new(template: StyledButtonTemplate<()>, metrics: crate::theme::MetricTokens) -> Self {
        Self { template, metrics }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyledButtonTemplate::sdk(), crate::theme::MetricTokens::default())
    }
}

impl crate::controls::button_family::ButtonFamilyTheme for StyledButtonFamilyTheme {
    fn resolve(
        &self,
        role: ButtonFamilyRole,
        state: InteractionState,
    ) -> crate::controls::button_family::ButtonFamilyPalette {
        let look = self.template.look(role, state, 1.0);
        crate::controls::button_family::ButtonFamilyPalette {
            focus_border: look.focus_border,
            background: look.background,
            foreground: look.foreground,
            muted_foreground: look.muted_foreground,
            border: look.border,
            typography: look.typography,
            font_family: look.font_family,
        }
    }

    fn metrics(&self) -> crate::theme::MetricTokens {
        self.metrics
    }

    fn resolve_look(
        &self,
        role: ButtonFamilyRole,
        state: InteractionState,
        _scale: &crate::theme::StandardBoxScale,
        _pill_radius: f32,
    ) -> Option<ButtonFamilyLook> {
        Some(self.template.look(role, state, 1.0))
    }
}

/// The SDK's default button template: the SDK's `.lss` files alone.
pub fn default_template() -> Arc<dyn ButtonTemplate<()>> {
    static TEMPLATE: std::sync::OnceLock<Arc<dyn ButtonTemplate<()>>> = std::sync::OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(StyledButtonTemplate::<()>::sdk())).clone()
}

impl<D: ButtonData> TemplateWithModifiers<ButtonRenderModel<D>> for StyledButtonTemplate<D> {
    fn modifiers(&self) -> &[Modifier<ButtonRenderModel<D>>] {
        &self.modifiers
    }
}

/// A button-family look from a resolved face: geometry by role, colors, typography and shadow.
fn look_from(root: PartView<'_>, colors: PartView<'_>, role: ButtonFamilyRole) -> ButtonFamilyLook {
    let icon = matches!(role, ButtonFamilyRole::Icon);
    let typography = LumaTextStyle {
        size: root.font_size().unwrap_or(14.0),
        line_height: root.line_height().unwrap_or(20.0),
        weight: gpui::FontWeight(root.value("font_weight").and_then(Value::as_number).unwrap_or(400.0)),
    };
    ButtonFamilyLook {
        focus_border: hsl(root.focus_border()).unwrap_or_else(transparent),
        background: hsl(colors.background()).unwrap_or_else(transparent),
        foreground: hsl(colors.foreground()).unwrap_or_else(transparent),
        muted_foreground: hsl(root.value("muted_foreground").and_then(Value::as_color)).unwrap_or_else(transparent),
        border: hsl(colors.border()),
        typography,
        font_family: root.font_family().unwrap_or("").to_string().into(),
        radius: if icon {
            length(root, "icon_radius")
        } else {
            root.radius().unwrap_or(0.0)
        },
        padding_x: if icon { 0.0 } else { length(root, "padding_x") },
        padding_y: if icon { 0.0 } else { length(root, "padding_y") },
        gap: root.gap().unwrap_or(0.0),
        height: root.height().unwrap_or(0.0),
        icon_size: if icon {
            length(root, "icon_size")
        } else {
            length(root, "text_icon_size")
        },
        shadow: root.shadow().filter(|layers| !layers.is_empty()).map(|layers| box_shadows(&layers)),
    }
}

impl<D: ButtonData> StyledButtonTemplate<D> {
    /// The face for a render: the settled style, and the colors (blended for on/off data).
    fn resolve(&self, model: &ButtonRenderModel<D>, scale: f32) -> (Arc<Resolved>, Resolved) {
        let theme = self.theme();
        let role_selected = matches!(model.role, ButtonFamilyRole::Toggle { selected: true });
        match model.data.toggle_state() {
            Some((on, progress)) => {
                let resolve = |selected: bool| {
                    theme.resolve_with(&self.key(&theme, model.role, model.state, selected, scale), &self.overrides)
                };
                let (off_style, on_style) = (resolve(false), resolve(true));
                let colors = Resolved::lerp(&off_style, &on_style, progress.clamp(0.0, 1.0));
                (if on { on_style } else { off_style }, colors)
            }
            None => {
                let settled = theme
                    .resolve_with(&self.key(&theme, model.role, model.state, role_selected, scale), &self.overrides);
                let colors = Resolved::lerp(&settled, &settled, 0.0);
                (settled, colors)
            }
        }
    }
}

impl<D: ButtonData> ButtonTemplate<D> for StyledButtonTemplate<D> {
    /// The face's `transition` toward on or off, for data that is on or off.
    fn transition(&self, data: &D) -> Option<crate::motion::Transition> {
        let (on, _) = data.toggle_state()?;
        let theme = self.theme();
        let key = self.key(&theme, ButtonFamilyRole::Toggle { selected: on }, Default::default(), on, 1.0);
        let resolved = theme.resolve_with(&key, &self.overrides);
        resolved
            .root()
            .get(xgpui_luma_style::props::TRANSITION)
            .map(|motion| crate::style::transition(&motion))
    }

    fn render(&self, model: &ButtonRenderModel<D>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let (settled, colors) = self.resolve(model, window.scale_factor());
        let root = settled.root();
        let mut look = match &model.look {
            Some(resolve) => resolve(model),
            None => look_from(root, colors.root(), model.role),
        };
        let presenter_model = button_content_context(model, look.clone());
        let focused = model.state.focused && !model.state.disabled;
        let border: Hsla = button_family_effective_border(look.border);
        let focus = focus_adorner(root);
        let focus_extent = if model.compact { 0.0 } else { focus.extent() };
        let border_width = length(root, "border_width");

        let mut control = div()
            .id((model.id.clone(), 0usize))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(look.gap))
            .bg(look.background)
            .text_color(look.foreground)
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .font_family(look.font_family.clone())
            .font_weight(look.typography.weight)
            .h(px(look.height))
            .border(px(border_width))
            .border_color(border);

        let toggle = matches!(model.role, ButtonFamilyRole::Toggle { .. });
        let square_icon_toggle = toggle && !model.round && look.padding_x <= 0.0 && look.padding_y <= 0.0;
        if matches!(model.role, ButtonFamilyRole::Icon) || (toggle && model.round) || square_icon_toggle {
            let corner_radius = model.radius_override.get().unwrap_or(look.radius);
            control = control.w(px(look.height)).flex_shrink_0().p_0();
            control = if model.round || corner_radius >= look.height / 2.0 {
                control.rounded_full()
            } else {
                control.rounded(px(corner_radius))
            };
        } else if model.round {
            control = control.w(px(look.height)).p_0().rounded_full();
        } else {
            let corner_radius = model.radius_override.get().unwrap_or(look.radius);
            control = control.px(px(look.padding_x)).py(px(look.padding_y)).rounded(px(corner_radius));
        }

        let content = if let Some(icon) = &model.icon {
            match icon {
                super::model::ControlIcon::Lucide(icon) => {
                    crate::infra::icon::lucide_icon(*icon, look.foreground, look.icon_size)
                }
                super::model::ControlIcon::SvgPath(path) => gpui::svg()
                    .size(px(look.icon_size))
                    .text_color(look.foreground)
                    .path(path.clone())
                    .into_any_element(),
            }
        } else {
            (model.content)(&presenter_model, cx)
        };
        let content = if matches!(model.role, ButtonFamilyRole::Icon) || square_icon_toggle {
            div()
                .size(px(look.icon_size))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(look.icon_size))
                .line_height(px(look.icon_size))
                .child(content)
                .into_any_element()
        } else {
            content
        };
        control = control.child(div().text_color(look.foreground).child(content));
        // A two-stop face (Radix classic) paints over the flat fill, even under a model look override.
        let face = colors.root();
        let stop = |name: &str| face.value(name).and_then(Value::as_color).map(Hsla::from);
        if let (Some(top), Some(bottom)) = (stop("background_start"), stop("background_end")) {
            control = control.bg(gpui::linear_gradient(
                180.0,
                gpui::linear_color_stop(top, 0.0),
                gpui::linear_color_stop(bottom, 1.0),
            ));
        }
        control = self.apply_modifiers(control, model);

        if crate::infra::shadow_layout::should_paint_shadow(
            model.elevation,
            model.state.disabled,
            look.shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()),
        ) && let Some(shadows) = look.shadow.take()
        {
            control = control.shadow(shadows);
        }

        let radius = if let Some(radius) = model.radius_override.get() {
            radius
        } else if model.round {
            look.height / 2.0
        } else {
            look.radius
        };
        let adorned = div().id((model.id.clone(), 1usize)).relative().child(control);
        let mut root_element = div().id(model.id.clone()).relative();
        root_element = if focus_extent > 0.0 {
            root_element.p(px(focus_extent)).child(adorned)
        } else {
            root_element.child(adorned)
        };
        root_element = root_element.child(focus.overlay(look.focus_border, focused, radius, model.compact));
        if model.state.disabled {
            root_element =
                root_element.opacity(root.value("disabled_opacity").and_then(Value::as_number).unwrap_or(1.0));
        } else {
            root_element = root_element.cursor_pointer();
        }
        root_element
    }
}
