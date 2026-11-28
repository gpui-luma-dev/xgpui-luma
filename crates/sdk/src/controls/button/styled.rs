//! The button-family template driven by the Luma style language (issue #285).
//!
//! It draws text buttons, icon buttons and toggles like [`DefaultButtonTemplate`](super::DefaultButtonTemplate),
//! reading every value from a [`Theme`]: the `button` control, or `toggle` for the toggle role (a toggle
//! is selected through the `selected` state). Data that animates between off and on (a toggle's) blends
//! the face colors with its progress. A model's own `look` override still wins.

use crate::style::PartRead;
use std::sync::Arc;

use gpui::{App, Div, Hsla, Stateful, Window, div, prelude::*, px};
use xgpui_luma_style::{AttrId, AttrValue, ControlId, Overrides, PartView, Resolved, StateId, StyleKey, Theme, Value};

use super::{ButtonRenderModel, ButtonTemplate, button_content_context};
use super::face::{Face, FaceLook, FaceShape};
use crate::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
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

    /// For a trigger's data ([`TriggerData`](crate::controls::trigger_button::TriggerData)): whether it is open,
    /// stretches to its parent's width, and its chevron.
    fn trigger(&self) -> Option<&crate::controls::trigger_button::TriggerData> {
        (self as &dyn std::any::Any).downcast_ref::<crate::controls::trigger_button::TriggerData>()
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
    /// The control read instead of `button` / `toggle`, e.g. `trigger_button`.
    control: Option<ControlId>,
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
        Self {
            source,
            control: None,
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

    /// Read `control` (one that extends `button`, e.g. `trigger_button` or `selector`) instead of `button`, or
    /// `toggle` for the toggle role.
    pub fn control(mut self, control: &str) -> Self {
        self.control = Some(ControlId::named(control));
        self
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

    /// The settled face for `role` in `state` (#433): for chrome a look draws itself (control-group rows) from the
    /// same `.lss` values.
    pub fn resolve_face(&self, role: ButtonFamilyRole, state: InteractionState, scale_factor: f32) -> Arc<Resolved> {
        let theme = self.theme();
        let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
        theme.resolve_with(&self.key(&theme, role, state, selected, scale_factor), &self.overrides)
    }

    /// The settled face for `role` in `state`, as a button-family look.
    pub fn look(&self, role: ButtonFamilyRole, state: InteractionState, scale_factor: f32) -> ButtonFamilyLook {
        let resolved = self.resolve_face(role, state, scale_factor);
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
        let control = self.control.unwrap_or_else(|| {
            ControlId::new(if matches!(role, ButtonFamilyRole::Toggle { .. }) {
                "toggle"
            } else {
                "button"
            })
        });
        let key = StyleKey::new(control).scale(scale);
        let key = self.states.iter().fold(key, |key, state| key.state(*state));
        let mut key = self.attrs.iter().fold(key, |key, (attr, value)| key.attr(*attr, *value));
        for (name, on) in crate::style::StyleStates::new(model_state).selected(selected).pairs() {
            if let Some(state) = config.state(name) {
                key = key.state_if(state, on);
            }
        }
        key
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
        font_family: root.font_family_name(),
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
        shadow: root.box_shadow_opt(),
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
                // An open trigger looks hovered (the selector's rule) and sets `open` for looks that style it.
                let open = model.data.trigger().is_some_and(|trigger| trigger.open);
                let state = InteractionState { hovered: model.state.hovered || open, ..model.state };
                let mut key = self.key(&theme, model.role, state, role_selected, scale);
                if let Some(open_state) = theme.stylesheet().config().state("open") {
                    key = key.state_if(open_state, open);
                }
                let settled = theme.resolve_with(&key, &self.overrides);
                let colors = Resolved::lerp(&settled, &settled, 0.0);
                (settled, colors)
            }
        }
    }
}

/// The states a styled button draws, for tools ([`ButtonTemplate::style_states`]); its key takes them from
/// [`crate::style::StyleStates`].
const BUTTON_STYLE_STATES: [&str; 6] = ["hover", "pressed", "focused", "disabled", "invalid", "selected"];

impl<D: ButtonData> ButtonTemplate<D> for StyledButtonTemplate<D> {
    fn style_states(&self) -> Vec<&'static str> {
        BUTTON_STYLE_STATES.to_vec()
    }

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
        let look = match &model.look {
            Some(resolve) => resolve(model),
            None => look_from(root, colors.root(), model.role),
        };
        let presenter_model = button_content_context(model, look.clone());
        let trigger = model.data.trigger();

        let toggle = matches!(model.role, ButtonFamilyRole::Toggle { .. });
        let square_icon_toggle = toggle && !model.round && look.padding_x <= 0.0 && look.padding_y <= 0.0;
        let corner_radius = model.radius_override.get().unwrap_or(look.radius);
        let shape = if matches!(model.role, ButtonFamilyRole::Icon) || (toggle && model.round) || square_icon_toggle {
            if model.round {
                FaceShape::Round
            } else {
                FaceShape::Square
            }
        } else if model.round {
            FaceShape::Round
        } else {
            FaceShape::Padded
        };
        let radius = if let Some(radius) = model.radius_override.get() {
            radius
        } else if model.round {
            look.height / 2.0
        } else {
            look.radius
        };
        let face = Face {
            id: model.id.clone(),
            shape,
            radius: if shape == FaceShape::Padded || shape == FaceShape::Square {
                corner_radius
            } else {
                radius
            },
            full_width: trigger.is_some_and(|trigger| trigger.full_width),
            min_height: false,
            focused: model.state.focused && !model.state.disabled,
            disabled: model.state.disabled,
            elevation: model.elevation,
            compact: model.compact,
        };
        let face_look = FaceLook {
            background: look.background,
            foreground: look.foreground,
            border: look.border,
            border_width: length(root, "border_width"),
            shadow: look.shadow.clone(),
            typography: look.typography,
            font_family: look.font_family.clone(),
            padding_x: look.padding_x,
            padding_y: look.padding_y,
            gap: look.gap,
            height: look.height,
            focus: focus_adorner(root),
            focus_border: look.focus_border,
            opacity: root.opacity(),
        };

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
        // A trigger's chevron takes the `icon` color (`trigger_button.lss`), dimmed with its state.
        let trailing = trigger.and_then(|trigger| trigger.chevron.as_ref()).map(|chevron| {
            let color = hsl(colors.root().value("icon").and_then(Value::as_color)).unwrap_or(look.muted_foreground);
            crate::infra::icon::render_icon_source(chevron, color, look.icon_size)
        });
        let mut control = face.control(&face_look, content, trailing);
        // A two-stop face (Radix classic) paints over the flat fill, even under a model look override.
        let colors_root = colors.root();
        let stop = |name: &str| colors_root.value(name).and_then(Value::as_color).map(Hsla::from);
        if let (Some(top), Some(bottom)) = (stop("background_start"), stop("background_end")) {
            control = control.bg(gpui::linear_gradient(
                180.0,
                gpui::linear_color_stop(top, 0.0),
                gpui::linear_color_stop(bottom, 1.0),
            ));
        }
        let control = self.apply_modifiers(control, model);
        let root = face.root(control, &face_look);
        // A trigger tells assistive technology whether its popup is showing.
        match trigger {
            Some(trigger) => root.aria_expanded(trigger.open),
            None => root,
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
mod style_states_tests {
    use super::*;
    use crate::theme::InteractionState;

    /// Each name `style_states` reports is a state the key sets, and the key sets no other.
    #[test]
    fn style_states_are_exactly_the_states_the_key_sets() {
        let template = StyledButtonTemplate::<()>::sdk();
        let theme = template.theme();
        let config = theme.stylesheet().config().clone();
        let role = ButtonFamilyRole::Toggle { selected: false };
        let active = |key: &StyleKey| -> Vec<String> {
            config
                .state_names()
                .filter(|n| key.active_states().contains(config.state(n).unwrap()))
                .map(str::to_string)
                .collect()
        };
        assert!(active(&template.key(&theme, role, InteractionState::default(), false, 1.0)).is_empty());
        for name in template.style_states() {
            let state = InteractionState {
                hovered: name == "hover",
                pressed: name == "pressed",
                focused: name == "focused",
                disabled: name == "disabled",
                invalid: name == "invalid",
            };
            let key = template.key(&theme, role, state, name == "selected", 1.0);
            assert_eq!(active(&key), [name], "{name}");
        }
    }
}

#[cfg(test)]
mod disabled_opacity_tests {
    use crate::style::{PartRead as _, default_theme};

    /// SDK controls whose `:disabled` draws its own fill and text don't also fade: a faded fill lets whatever is
    /// behind the control show through (#397). Migrated controls carry the fade as `:disabled { opacity }` (#413).
    #[test]
    fn sdk_disabled_controls_do_not_fade() {
        let theme = default_theme();
        for control in [
            "button",
            "toggle",
            "popup_menu",
            "trigger_button",
            "selector",
            "checkbox",
            "radio",
            "switch",
            "textfield",
            "textarea",
            "scrollbar",
            "slider",
            "resizable_panels",
            "control_group",
            "context_menu",
            "radio_group",
            "floating_menu",
            "tree_view",
            "accordion",
            "table",
            "sidebar",
            "pager",
        ] {
            let sheet = theme.stylesheet();
            let id = sheet.control(control).expect("declared");
            assert!(sheet.prop::<f32>(id, "disabled_opacity").is_err(), "{control}: no separate fade property");
            let key = theme.key_named(control, &[], &["disabled"]).expect("key");
            let resolved = theme.resolve(&key);
            for part in resolved.parts() {
                assert_eq!(resolved.part(part).opacity(), 1.0, "{control} {part:?}");
            }
        }
    }

    /// The check flags a disabled part that fades and changes its fill, and accepts a fade over the resting fill.
    #[test]
    fn the_fade_check_flags_a_faded_changed_fill() {
        let with = |lss: &str| {
            let theme = default_theme();
            let source = crate::style::lss::Source::new("app.lss", lss.to_string());
            let sheet = theme.stylesheet().with_source("app", source).expect("compiles");
            crate::style::disabled_fade_problems(&theme.with_stylesheet(&sheet).expect("binds"))
        };
        let stacked = with("button { part root { :disabled { background = #ff0000  opacity = 0.5 } } }");
        assert!(stacked.iter().any(|p| p.starts_with("button")), "{stacked:?}");
        let resting = with("button { part root { background := #00ff00  :disabled { opacity = 0.5 } } }");
        assert!(resting.is_empty(), "{resting:?}");
    }

    /// No SDK control, in any state or mode, both fades and changes its fill when disabled.
    #[test]
    fn no_disabled_part_fades_a_changed_fill() {
        for theme in [default_theme(), crate::style::dark_theme()] {
            let problems = crate::style::disabled_fade_problems(&theme);
            assert!(
                problems.is_empty(),
                "{}",
                problems.join(
                    "
"
                )
            );
        }
    }
}
