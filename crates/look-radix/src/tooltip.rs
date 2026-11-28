//! Theme-aware tooltip defaults, bound to the same shared/forked look as the control.
#![allow(deprecated)] // Compatibility paths and parity tests still read the deprecated Looks (#433).
use std::sync::Arc;
use xgpui_luma::controls::tooltip::{StyledTooltipTheme, TooltipTheme};
use crate::Look;

/// Tooltip presentation drawn from `.lss` (`style/tooltip.lss`), resolved from this look on every render,
/// including mode and palette changes.
pub fn tooltip_theme(look: &Look) -> Arc<dyn TooltipTheme> {
    Arc::new(StyledTooltipTheme::new(xgpui_luma::style::StyleBinding::new(
        "tooltip",
        crate::textfield::live_source(look),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SemanticRole;
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn tooltip_tracks_bound_fork_and_mode_changes() {
        let ambient = Look::built_in();
        let fork = ambient.fork();
        let theme = tooltip_theme(&fork);
        let light = theme.resolve();
        fork.set_mode(ThemeMode::Dark);
        let dark = theme.resolve();
        assert_ne!(light.background, dark.background);
        assert_eq!(ambient.mode(), ThemeMode::Light);
        crate::control_group::assert_close(dark.background, fork.resolve_role(SemanticRole::Foreground).hsla());
        crate::control_group::assert_close(dark.foreground, fork.resolve_role(SemanticRole::Background).hsla());
    }
}
