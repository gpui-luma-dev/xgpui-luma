//! The selector trigger read from `.lss` (issue #285): `selector.lss`, which extends the button. The
//! popup panel is supplied separately (it is drawn with the floating menu's values).

use crate::style::StyleStates;
use crate::style::PartRead;
use std::sync::Arc;

use super::model::SelectorTriggerStyle;
use super::theme::{SelectorTheme, SelectorVisualState};
use crate::controls::selector_list::SelectorItemsPanelLook;
use crate::style::{StyleBinding, ThemeSource};
use crate::theme::{FocusMetricTokens, MetricTokens};

/// Supplies the selector's popup panel look.
pub type ItemsPanelSource = Arc<dyn Fn() -> SelectorItemsPanelLook + Send + Sync>;

/// A selector theme whose trigger is read from `.lss`.
#[derive(Clone)]
pub struct StyledSelectorTheme {
    binding: StyleBinding,
    items_panel: ItemsPanelSource,
}

impl StyledSelectorTheme {
    /// The `selector` control from `source`, with its popup panel from `items_panel`.
    pub fn new(source: ThemeSource, items_panel: ItemsPanelSource) -> Self {
        Self { binding: StyleBinding::new("selector", source), items_panel }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self {
            binding: StyleBinding::sdk("selector"),
            items_panel: Arc::new(|| {
                crate::controls::selector_list::default_selector_items_panel_look(&Default::default())
            }),
        }
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

    fn items_panel(&self) -> SelectorItemsPanelLook {
        (self.items_panel)()
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

/// The trigger's look in `trigger_style` and `visual` state, built from `theme`'s resolved style (#433): for code
/// that still reads a [`SelectorLook`](super::SelectorLook). Templates read the resolved style directly.
pub fn selector_look<T: SelectorTheme + ?Sized>(
    theme: &T,
    trigger_style: SelectorTriggerStyle,
    visual: SelectorVisualState,
    without_elevation: bool,
) -> super::SelectorLook {
    super::SelectorLook::from_resolved(
        &resolve_trigger(&theme.style(), trigger_style, visual, 1.0),
        theme.items_panel(),
        without_elevation,
    )
}
