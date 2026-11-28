//! The selector trigger read from `.lss` (issue #285): `selector.lss`, which extends the button. The
//! popup panel is supplied separately (it is drawn with the floating menu's values).

use crate::style::StyleStates;
use crate::style::PartRead;
use std::sync::Arc;

use super::model::SelectorTriggerStyle;
use super::theme::{SelectorTheme, SelectorVisualState};
use crate::style::{StyleBinding, ThemeSource};
use crate::theme::{FocusMetricTokens, MetricTokens};

/// Supplies the selector's popup panel style: a `floating_menu` binding (#433).
pub type ItemsPanelSource = crate::controls::popup_menu::MenuSource;

/// A selector theme whose trigger is read from `.lss`.
#[derive(Clone)]
pub struct StyledSelectorTheme {
    binding: StyleBinding,
    menu: ItemsPanelSource,
}

impl StyledSelectorTheme {
    /// The `selector` control from `source`, with its popup panel's style from `menu`.
    pub fn new(source: ThemeSource, menu: ItemsPanelSource) -> Self {
        Self { binding: StyleBinding::new("selector", source), menu }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("selector"), menu: Arc::new(|| StyleBinding::sdk("floating_menu")) }
    }

    /// Change the binding (a look's size).
    pub fn with(self, f: impl FnOnce(StyleBinding) -> StyleBinding) -> Self {
        Self { binding: f(self.binding), ..self }
    }
}

impl SelectorTheme for StyledSelectorTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }

    fn menu(&self) -> StyleBinding {
        (self.menu)()
    }
}

/// The trigger's resolved style in `trigger_style` and `visual` state (#433): what the template draws from. An open
/// trigger draws hovered and sets the `open` state (the trigger styles of `trigger_button.lss`). Read its parts for
/// the trigger's values (`background`, `foreground`, `icon`, `icon_size`, …).
pub fn resolve_trigger(
    binding: &StyleBinding,
    trigger_style: SelectorTriggerStyle,
    visual: SelectorVisualState,
    scale: f32,
) -> Arc<xgpui_luma_style::Resolved> {
    let variant = match trigger_style {
        SelectorTriggerStyle::Outline => "outline",
        SelectorTriggerStyle::Ghost => "ghost",
    };
    let states = StyleStates::new(visual.resolved_interaction()).open(visual.open);
    binding.resolve_states_for(&[("variant", variant)], states, scale)
}

/// The trigger's border width and focus geometry at rest.
pub(crate) fn metrics(binding: &StyleBinding) -> MetricTokens {
    let resolved = resolve_trigger(binding, SelectorTriggerStyle::Outline, SelectorVisualState::default(), 1.0);
    let root = resolved.root();
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
