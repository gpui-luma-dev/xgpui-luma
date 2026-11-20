//! Text-field and textarea themes read from `.lss` (issue #285): `textfield.lss`, and `textarea`, which
//! extends it. They plug into the existing templates and every control that takes a text-input theme.

use gpui::{Hsla, SharedString};
use xgpui_luma_style::{PartView, Resolved, Value, box_shadows};

use crate::controls::textarea::{TextAreaLook, TextAreaPalette, TextAreaState, TextAreaTheme};
use crate::controls::textfield::{TextFieldLook, TextFieldPalette, TextFieldState, TextFieldTheme};
use crate::style::{StyleBinding, ThemeSource};
use crate::theme::{FocusMetricTokens, LumaTextStyle, MetricTokens, StandardBoxScale};

fn color(root: PartView<'_>, name: &str) -> Hsla {
    root.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(root: PartView<'_>, name: &str) -> f32 {
    root.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// The values both inputs share, resolved for one state.
struct Input {
    resolved: std::sync::Arc<Resolved>,
}

impl Input {
    fn resolve(binding: &StyleBinding, hovered: bool, focused: bool, invalid: bool, enabled: bool) -> Self {
        let states =
            [("hover", hovered && enabled), ("focused", focused), ("invalid", invalid), ("disabled", !enabled)];
        Self { resolved: binding.resolve(&states, 1.0) }
    }

    fn root(&self) -> PartView<'_> {
        self.resolved.root()
    }

    fn typography(&self) -> LumaTextStyle {
        let root = self.root();
        LumaTextStyle {
            size: root.font_size().unwrap_or(14.0),
            line_height: root.line_height().unwrap_or(20.0),
            weight: gpui::FontWeight(number(root, "font_weight").max(100.0)),
        }
    }

    fn font_family(&self) -> SharedString {
        self.root().font_family().unwrap_or("").to_string().into()
    }

    fn shadow(&self) -> Option<Vec<gpui::BoxShadow>> {
        self.root().shadow().filter(|layers| !layers.is_empty()).map(|layers| box_shadows(&layers))
    }

    /// The focus geometry and border width, for templates that read them from the theme's metrics.
    fn metrics(&self) -> MetricTokens {
        let root = self.root();
        MetricTokens {
            border_width: number(root, "border_width"),
            focus: FocusMetricTokens {
                gap: number(root, "focus_gap"),
                width: number(root, "focus_width"),
                offset: number(root, "focus_offset"),
            },
            disabled_opacity: number(root, "disabled_opacity"),
            ..MetricTokens::default()
        }
    }
}

/// A text-field theme read from `.lss`.
#[derive(Clone)]
pub struct StyledTextFieldTheme {
    binding: StyleBinding,
}

impl StyledTextFieldTheme {
    /// The `textfield` control from `source`.
    pub fn new(source: ThemeSource) -> Self {
        Self { binding: StyleBinding::new("textfield", source) }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("textfield") }
    }

    /// Change the binding (a look's variant, size, tone).
    pub fn with(self, f: impl FnOnce(StyleBinding) -> StyleBinding) -> Self {
        Self { binding: f(self.binding) }
    }

    fn input(&self, state: TextFieldState, enabled: bool) -> Input {
        Input::resolve(&self.binding, state.hovered, state.focused, state.invalid, enabled)
    }
}

impl TextFieldTheme for StyledTextFieldTheme {
    fn resolve(&self, state: TextFieldState, enabled: bool) -> TextFieldPalette {
        let input = self.input(state, enabled);
        let root = input.root();
        TextFieldPalette {
            focus_border: color(root, "focus_border"),
            background: color(root, "background"),
            foreground: color(root, "foreground"),
            border: color(root, "border"),
            placeholder: color(root, "placeholder"),
            icon: color(root, "icon"),
            selection_background: color(root, "selection_background"),
            selection_foreground: color(root, "selection_foreground"),
            caret: color(root, "caret"),
            shadow: input.shadow(),
            typography: input.typography(),
            font_family: input.font_family(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.input(TextFieldState::default(), true).metrics()
    }

    fn resolve_look(&self, state: TextFieldState, enabled: bool, _scale: &StandardBoxScale) -> TextFieldLook {
        let input = self.input(state, enabled);
        let root = input.root();
        TextFieldLook {
            helper_geometry: crate::theme::stylesheet::CommonStylesheet::sdk_default().shared_geometry(),
            caret_width: number(root, "caret_width"),
            caret_height_extra: number(root, "caret_height_extra"),
            disabled_opacity: number(root, "disabled_opacity"),
            background: color(root, "background"),
            foreground: color(root, "foreground"),
            border: color(root, "border"),
            focus_border: color(root, "focus_border"),
            placeholder: color(root, "placeholder"),
            icon: color(root, "icon"),
            selection_background: color(root, "selection_background"),
            selection_foreground: color(root, "selection_foreground"),
            caret: color(root, "caret"),
            shadow: input.shadow(),
            typography: input.typography(),
            font_family: input.font_family(),
            min_height: root.height().unwrap_or(0.0),
            padding_x: number(root, "padding_x"),
            padding_y: number(root, "padding_y"),
            gap: root.gap().unwrap_or(0.0),
            radius: root.radius().unwrap_or(0.0),
            border_width: number(root, "border_width"),
            icon_size: number(root, "icon_size"),
        }
    }
}

/// A textarea theme read from `.lss`.
#[derive(Clone)]
pub struct StyledTextAreaTheme {
    binding: StyleBinding,
}

impl StyledTextAreaTheme {
    /// The `textarea` control from `source`.
    pub fn new(source: ThemeSource) -> Self {
        Self { binding: StyleBinding::new("textarea", source) }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("textarea") }
    }

    /// Change the binding (a look's variant, size, tone).
    pub fn with(self, f: impl FnOnce(StyleBinding) -> StyleBinding) -> Self {
        Self { binding: f(self.binding) }
    }

    fn input(&self, state: TextAreaState, enabled: bool) -> Input {
        Input::resolve(&self.binding, state.hovered, state.focused, state.invalid, enabled)
    }
}

impl TextAreaTheme for StyledTextAreaTheme {
    fn resolve(&self, state: TextAreaState, enabled: bool) -> TextAreaPalette {
        let input = self.input(state, enabled);
        let root = input.root();
        TextAreaPalette {
            focus_border: color(root, "focus_border"),
            background: color(root, "background"),
            foreground: color(root, "foreground"),
            border: color(root, "border"),
            placeholder: color(root, "placeholder"),
            selection_background: color(root, "selection_background"),
            selection_foreground: color(root, "selection_foreground"),
            caret: color(root, "caret"),
            shadow: input.shadow(),
            typography: input.typography(),
            font_family: input.font_family().to_string(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.input(TextAreaState::default(), true).metrics()
    }

    fn resolve_look(&self, state: TextAreaState, enabled: bool, _scale: &StandardBoxScale) -> TextAreaLook {
        let input = self.input(state, enabled);
        let root = input.root();
        TextAreaLook {
            helper_geometry: crate::theme::stylesheet::CommonStylesheet::sdk_default().shared_geometry(),
            caret_width: number(root, "caret_width"),
            caret_height_extra: number(root, "caret_height_extra"),
            disabled_opacity: number(root, "disabled_opacity"),
            background: color(root, "background"),
            foreground: color(root, "foreground"),
            border: color(root, "border"),
            focus_border: color(root, "focus_border"),
            placeholder: color(root, "placeholder"),
            selection_background: color(root, "selection_background"),
            selection_foreground: color(root, "selection_foreground"),
            caret: color(root, "caret"),
            shadow: input.shadow(),
            typography: input.typography(),
            font_family: input.font_family().to_string(),
            min_height: root.height().unwrap_or(0.0),
            padding_x: number(root, "padding_x"),
            padding_y: number(root, "padding_y"),
            radius: root.radius().unwrap_or(0.0),
            border_width: number(root, "border_width"),
        }
    }
}
