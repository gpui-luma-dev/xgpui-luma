//! Text-field and textarea themes read from `.lss` (issue #285): `textfield.lss`, and `textarea`, which
//! extends it. They plug into the existing templates and every control that takes a text-input theme.

use crate::theme::InteractionState;
use crate::style::StyleStates;
use crate::style::PartRead;
use gpui::SharedString;
use xgpui_luma_style::{PartView, Resolved};

use crate::controls::textarea::{TextAreaLook, TextAreaPalette, TextAreaState, TextAreaTheme};
use crate::controls::textfield::{TextFieldLook, TextFieldPalette, TextFieldState, TextFieldTheme};
use crate::style::{StyleBinding, ThemeSource};
use crate::theme::{FocusMetricTokens, LumaTextStyle, MetricTokens, StandardBoxScale};

/// The values both inputs share, resolved for one state.
struct Input {
    resolved: std::sync::Arc<Resolved>,
}

impl Input {
    fn resolve(binding: &StyleBinding, hovered: bool, focused: bool, invalid: bool, enabled: bool) -> Self {
        let interaction = InteractionState {
            hovered: hovered && enabled,
            focused,
            invalid,
            disabled: !enabled,
            ..Default::default()
        };
        Self { resolved: binding.resolve_states(StyleStates::new(interaction), 1.0) }
    }

    fn root(&self) -> PartView<'_> {
        self.resolved.root()
    }

    fn typography(&self) -> LumaTextStyle {
        let root = self.root();
        root.typography(14.0, 20.0)
    }

    fn font_family(&self) -> SharedString {
        self.root().font_family_name()
    }

    fn shadow(&self) -> Option<Vec<gpui::BoxShadow>> {
        self.root().box_shadow_opt()
    }

    /// The focus geometry and border width, for templates that read them from the theme's metrics.
    fn metrics(&self) -> MetricTokens {
        let root = self.root();
        MetricTokens {
            border_width: root.number("border_width"),
            focus: FocusMetricTokens {
                gap: root.number("focus_gap"),
                width: root.number("focus_width"),
                offset: root.number("focus_offset"),
            },
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
            focus_border: root.color("focus_border"),
            background: root.color("background"),
            foreground: root.color("foreground"),
            border: root.color("border"),
            placeholder: root.color("placeholder"),
            icon: root.color("icon"),
            selection_background: root.color("selection_background"),
            selection_foreground: root.color("selection_foreground"),
            caret: root.color("caret"),
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
            helper_geometry: crate::theme::stylesheet::SharedGeometry::sdk(),
            caret_width: root.number("caret_width"),
            caret_height_extra: root.number("caret_height_extra"),
            opacity: root.opacity(),
            background: root.color("background"),
            foreground: root.color("foreground"),
            border: root.color("border"),
            focus_border: root.color("focus_border"),
            placeholder: root.color("placeholder"),
            icon: root.color("icon"),
            selection_background: root.color("selection_background"),
            selection_foreground: root.color("selection_foreground"),
            caret: root.color("caret"),
            shadow: input.shadow(),
            typography: input.typography(),
            font_family: input.font_family(),
            min_height: root.height().unwrap_or(0.0),
            padding_x: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            gap: root.gap().unwrap_or(0.0),
            radius: root.radius().unwrap_or(0.0),
            border_width: root.number("border_width"),
            icon_size: root.number("icon_size"),
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
            focus_border: root.color("focus_border"),
            background: root.color("background"),
            foreground: root.color("foreground"),
            border: root.color("border"),
            placeholder: root.color("placeholder"),
            selection_background: root.color("selection_background"),
            selection_foreground: root.color("selection_foreground"),
            caret: root.color("caret"),
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
            helper_geometry: crate::theme::stylesheet::SharedGeometry::sdk(),
            caret_width: root.number("caret_width"),
            caret_height_extra: root.number("caret_height_extra"),
            opacity: root.opacity(),
            background: root.color("background"),
            foreground: root.color("foreground"),
            border: root.color("border"),
            focus_border: root.color("focus_border"),
            placeholder: root.color("placeholder"),
            selection_background: root.color("selection_background"),
            selection_foreground: root.color("selection_foreground"),
            caret: root.color("caret"),
            shadow: input.shadow(),
            typography: input.typography(),
            font_family: input.font_family().to_string(),
            min_height: root.height().unwrap_or(0.0),
            padding_x: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            radius: root.radius().unwrap_or(0.0),
            border_width: root.number("border_width"),
        }
    }
}
