//! Text-field and textarea themes read from `.lss` (issue #285): `textfield.lss`, and `textarea`, which
//! extends it. They plug into the existing templates and every control that takes a text-input theme.
//!
//! Templates and controls draw from the resolved style (#433); the Looks here are built from it for code that still
//! reads them.
#![allow(deprecated)]

use std::sync::Arc;

use crate::style::PartRead;
use crate::style::StyleStates;
use crate::theme::InteractionState;
use gpui::Hsla;
use xgpui_luma_style::{PartId, PartView, Resolved};

use crate::controls::textarea::{TextAreaLook, TextAreaPalette, TextAreaState, TextAreaTheme};
use crate::controls::textfield::{TextFieldLook, TextFieldPalette, TextFieldState, TextFieldTheme};
use crate::style::lss::{Overrides, Prop};
use crate::style::{StyleBinding, ThemeSource, lss_color};
use crate::theme::{FocusMetricTokens, LumaTextStyle, MetricTokens, StandardBoxScale};

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

/// Overrides that turn `before` into `after`: one per changed value, the bridge from a look closure to the
/// style language (#433). A shadow can only be removed, not replaced.
struct Delta(Overrides);

impl Delta {
    fn color(self, name: &'static str, before: Hsla, after: Hsla) -> Self {
        if before == after {
            return self;
        }
        Self(self.0.set(PartId::ROOT, Prop::<xgpui_luma_style::Color>::new(name), lss_color(after)))
    }

    fn number(self, name: &'static str, before: f32, after: f32) -> Self {
        if before == after {
            return self;
        }
        Self(self.0.set(PartId::ROOT, Prop::<f32>::new(name), after))
    }

    fn family(self, before: &str, after: &str) -> Self {
        if before == after {
            return self;
        }
        Self(self.0.set(PartId::ROOT, Prop::<Arc<str>>::new("font_family"), Arc::<str>::from(after)))
    }

    fn typography(self, before: LumaTextStyle, after: LumaTextStyle) -> Self {
        self.number("font_size", before.size, after.size)
            .number("line_height", before.line_height, after.line_height)
            .number("font_weight", before.weight.0, after.weight.0)
    }

    fn shadow_removed<S>(self, before: &Option<S>, after: &Option<S>) -> Self {
        if before.is_some() && after.is_none() {
            let none: Arc<[xgpui_luma_style::ShadowLayer]> = Arc::from(Vec::new());
            return Self(self.0.set(PartId::ROOT, Prop::<Arc<[xgpui_luma_style::ShadowLayer]>>::new("shadow"), none));
        }
        self
    }
}

impl TextFieldLook {
    /// The look `resolved` (see [`resolve_text_field`]) describes: the compatibility path for code that still reads
    /// a `TextFieldLook` (#433).
    pub fn from_resolved(resolved: &Resolved) -> Self {
        let root = resolved.root();
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
            shadow: root.box_shadow_opt(),
            typography: input_typography(root),
            font_family: root.font_family_name(),
            min_height: root.height().unwrap_or(0.0),
            padding_x: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            gap: root.gap().unwrap_or(0.0),
            radius: root.radius().unwrap_or(0.0),
            border_width: root.number("border_width"),
            icon_size: root.number("icon_size"),
        }
    }

    /// The style overrides a look closure made: what `after` changes from `before` (#433).
    pub fn overrides_between(before: &TextFieldLook, after: &TextFieldLook) -> Overrides {
        Delta(Overrides::new())
            .number("caret_width", before.caret_width, after.caret_width)
            .number("caret_height_extra", before.caret_height_extra, after.caret_height_extra)
            .color("background", before.background, after.background)
            .color("foreground", before.foreground, after.foreground)
            .color("border", before.border, after.border)
            .color("focus_border", before.focus_border, after.focus_border)
            .color("placeholder", before.placeholder, after.placeholder)
            .color("icon", before.icon, after.icon)
            .color("selection_background", before.selection_background, after.selection_background)
            .color("selection_foreground", before.selection_foreground, after.selection_foreground)
            .color("caret", before.caret, after.caret)
            .shadow_removed(&before.shadow, &after.shadow)
            .typography(before.typography, after.typography)
            .family(&before.font_family, &after.font_family)
            .number("height", before.min_height, after.min_height)
            .number("padding_x", before.padding_x, after.padding_x)
            .number("padding_y", before.padding_y, after.padding_y)
            .number("gap", before.gap, after.gap)
            .number("radius", before.radius, after.radius)
            .number("border_width", before.border_width, after.border_width)
            .number("icon_size", before.icon_size, after.icon_size)
            .0
    }
}

impl TextAreaLook {
    /// The look `resolved` (see [`resolve_text_area`]) describes: the compatibility path for code that still reads a
    /// `TextAreaLook` (#433).
    pub fn from_resolved(resolved: &Resolved) -> Self {
        let root = resolved.root();
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
            shadow: root.box_shadow_opt(),
            typography: input_typography(root),
            font_family: root.font_family_name().to_string(),
            min_height: root.height().unwrap_or(0.0),
            padding_x: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            radius: root.radius().unwrap_or(0.0),
            border_width: root.number("border_width"),
        }
    }

    /// The style overrides a look closure made: what `after` changes from `before` (#433).
    pub fn overrides_between(before: &TextAreaLook, after: &TextAreaLook) -> Overrides {
        Delta(Overrides::new())
            .number("caret_width", before.caret_width, after.caret_width)
            .number("caret_height_extra", before.caret_height_extra, after.caret_height_extra)
            .color("background", before.background, after.background)
            .color("foreground", before.foreground, after.foreground)
            .color("border", before.border, after.border)
            .color("focus_border", before.focus_border, after.focus_border)
            .color("placeholder", before.placeholder, after.placeholder)
            .color("selection_background", before.selection_background, after.selection_background)
            .color("selection_foreground", before.selection_foreground, after.selection_foreground)
            .color("caret", before.caret, after.caret)
            .shadow_removed(&before.shadow, &after.shadow)
            .typography(before.typography, after.typography)
            .family(&before.font_family, &after.font_family)
            .number("height", before.min_height, after.min_height)
            .number("padding_x", before.padding_x, after.padding_x)
            .number("padding_y", before.padding_y, after.padding_y)
            .number("radius", before.radius, after.radius)
            .number("border_width", before.border_width, after.border_width)
            .0
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

    fn resolve_look(&self, state: TextFieldState, enabled: bool, _scale: &StandardBoxScale) -> TextFieldLook {
        TextFieldLook::from_resolved(&self.resolved(state, enabled))
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

    fn resolve_look(&self, state: TextAreaState, enabled: bool, _scale: &StandardBoxScale) -> TextAreaLook {
        TextAreaLook::from_resolved(&self.resolved(state, enabled))
    }
}
