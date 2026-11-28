//! Theme-aware tooltip defaults, bound to the same shared/forked look as the control.
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
    use xgpui_luma::style::PartRead;
    use crate::SemanticRole;
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn tooltip_tracks_bound_fork_and_mode_changes() {
        let ambient = Look::built_in();
        let fork = ambient.fork();
        let theme = tooltip_theme(&fork);
        let root = |theme: &Arc<dyn xgpui_luma::controls::tooltip::TooltipTheme>| {
            xgpui_luma::controls::tooltip::resolve_tooltip(&theme.style())
        };
        let light = root(&theme);
        fork.set_mode(ThemeMode::Dark);
        let dark = root(&theme);
        let (light, dark) = (light.root(), dark.root());
        assert_ne!(light.color("background"), dark.color("background"));
        assert_eq!(ambient.mode(), ThemeMode::Light);
        crate::control_group::assert_close(
            dark.color("background"),
            fork.resolve_role(SemanticRole::Foreground).hsla(),
        );
        crate::control_group::assert_close(
            dark.color("foreground"),
            fork.resolve_role(SemanticRole::Background).hsla(),
        );
    }
}
