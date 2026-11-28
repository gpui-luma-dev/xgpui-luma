//! Text-field and textarea themes read from `.lss` (issue #285): `textfield.lss`, and `textarea`, which
//! extends it. They plug into the existing templates and every control that takes a text-input theme.
//!
//! Templates and controls draw from the resolved style (#433); the Looks here are built from it for code that still
//! reads them.

use std::sync::Arc;

use crate::style::PartRead;
use crate::style::StyleStates;
use crate::theme::InteractionState;
use xgpui_luma_style::{PartView, Resolved};

use crate::controls::textarea::{TextAreaPalette, TextAreaState, TextAreaTheme};
use crate::controls::textfield::{TextFieldPalette, TextFieldState, TextFieldTheme};
use crate::style::{StyleBinding, ThemeSource};
use crate::theme::{FocusMetricTokens, LumaTextStyle, MetricTokens};

/// A text input's style for `binding` (`textfield` or `textarea`) in the given states (#433): the input draws
/// from its root.
pub fn resolve_text_input(
    binding: &StyleBinding,
    hovered: bool,
    focused: bool,
    invalid: bool,
    enabled: bool,
) -> Arc<Resolved> {
    let interaction =
        InteractionState { hovered: hovered && enabled, focused, invalid, disabled: !enabled, ..Default::default() };
    binding.resolve_states(StyleStates::new(interaction), 1.0)
}

/// A text field's style for `binding` (the `textfield` control) in `state` (#433).
pub fn resolve_text_field(binding: &StyleBinding, state: TextFieldState, enabled: bool) -> Arc<Resolved> {
    resolve_text_input(binding, state.hovered, state.focused, state.invalid, enabled)
}

/// A textarea's style for `binding` (the `textarea` control) in `state` (#433).
pub fn resolve_text_area(binding: &StyleBinding, state: TextAreaState, enabled: bool) -> Arc<Resolved> {
    resolve_text_input(binding, state.hovered, state.focused, state.invalid, enabled)
}

/// The text style a resolved input root sets.
pub fn input_typography(root: PartView<'_>) -> LumaTextStyle {
    root.typography(14.0, 20.0)
}

/// The focus geometry and border width a resolved input root sets, for templates that read them from metrics.
pub fn input_metrics(root: PartView<'_>) -> MetricTokens {
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

    fn resolved(&self, state: TextFieldState, enabled: bool) -> Arc<Resolved> {
        resolve_text_field(&self.binding, state, enabled)
    }
}

impl TextFieldTheme for StyledTextFieldTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }

    fn resolve(&self, state: TextFieldState, enabled: bool) -> TextFieldPalette {
        let resolved = self.resolved(state, enabled);
        let root = resolved.root();
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
            shadow: root.box_shadow_opt(),
            typography: input_typography(root),
            font_family: root.font_family_name(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        input_metrics(self.resolved(TextFieldState::default(), true).root())
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

    fn resolved(&self, state: TextAreaState, enabled: bool) -> Arc<Resolved> {
        resolve_text_area(&self.binding, state, enabled)
    }
}

impl TextAreaTheme for StyledTextAreaTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }

    fn resolve(&self, state: TextAreaState, enabled: bool) -> TextAreaPalette {
        let resolved = self.resolved(state, enabled);
        let root = resolved.root();
        TextAreaPalette {
            focus_border: root.color("focus_border"),
            background: root.color("background"),
            foreground: root.color("foreground"),
            border: root.color("border"),
            placeholder: root.color("placeholder"),
            selection_background: root.color("selection_background"),
            selection_foreground: root.color("selection_foreground"),
            caret: root.color("caret"),
            shadow: root.box_shadow_opt(),
            typography: input_typography(root),
            font_family: root.font_family_name().to_string(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        input_metrics(self.resolved(TextAreaState::default(), true).root())
    }
}
