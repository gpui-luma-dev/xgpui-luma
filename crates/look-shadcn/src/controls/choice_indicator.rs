//! shadcn checkbox / radio: indicator and label colors do not follow hover layers.

use xgpui_luma::theme::{InteractionLayer, InteractionState};

pub(crate) fn choice_indicator_color_layer(state: InteractionState) -> InteractionLayer {
    if state.disabled {
        InteractionLayer::Disabled
    } else {
        InteractionLayer::Default
    }
}

/// Shared border precedence for checkbox and radio indicators.
pub(crate) fn resolve_indicator_border(
    resolver: &crate::LookResolver<'_>,
    _style: super::ShadcnButtonStyle,
    selected: bool,
    state: InteractionState,
    selection: &crate::ResolvedColor,
) -> crate::ResolvedColor {
    if selected && !state.disabled {
        selection.clone()
    } else {
        resolver.resolve_decl("border").unwrap_or_else(|_| crate::ResolvedColor::fallback_foreground())
    }
}
