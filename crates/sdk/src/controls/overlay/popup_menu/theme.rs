use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::style::StyleBinding;
use crate::theme::{LumaTextStyle, MetricTokens};

#[derive(Clone, Debug)]
pub struct PopupMenuPalette {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub trigger_focus_border: gpui::Hsla,
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
    pub trigger_typography: LumaTextStyle,
    /// The menu's style (the `floating_menu` control).
    pub floating_menu: StyleBinding,
}

/// Build one from [`Default`]; it may gain fields.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct PopupMenuTriggerMetrics {
    pub without_elevation: bool,
    pub icon_only: bool,
    pub trigger_radius_override: Option<f32>,
    /// The menu is showing: sets the `open` state (the trigger styles of `trigger_button.lss`).
    pub open: bool,
}

pub trait PopupMenuTheme: Send + Sync {
    /// The trigger's style (#433): the binding the template resolves in the trigger's style, state and `open`,
    /// and draws from.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("popup_menu")
    }

    /// The style of the menu the trigger opens (the `floating_menu` control).
    fn menu(&self) -> StyleBinding {
        StyleBinding::sdk("floating_menu")
    }

    /// The trigger's border width and focus geometry at rest.
    fn metrics(&self) -> MetricTokens {
        super::styled::metrics(&self.style())
    }
}

pub fn default_popup_menu_theme() -> Arc<dyn PopupMenuTheme> {
    static THEME: OnceLock<Arc<dyn PopupMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledPopupMenuTheme::sdk())).clone()
}
