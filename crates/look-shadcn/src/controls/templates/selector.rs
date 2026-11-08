use std::sync::Arc;

use xgpui_luma::controls::selector::{SelectorItemLike, SelectorTheme, SelectorVisualState, ThemedSelectorTemplate};
use xgpui_luma::controls::selector_list::default_selector_items_template;
use xgpui_luma::theme::{ControlSize, InteractionState};

use crate::look::ShadcnLook;

pub fn selector_theme(theme: ShadcnLook) -> Arc<dyn SelectorTheme> {
    Arc::new(ShadcnSelectorTheme { theme: theme.clone() })
}

struct ShadcnSelectorTheme {
    theme: ShadcnLook,
}

impl SelectorTheme for ShadcnSelectorTheme {
    fn resolve(
        &self,
        trigger_style: xgpui_luma::controls::selector::SelectorTriggerStyle,
        state: InteractionState,
        without_elevation: bool,
    ) -> xgpui_luma::controls::selector::SelectorPalette {
        let tokens = self.theme.mode_tokens();
        crate::controls::selector::selector_palette(
            tokens.as_ref(),
            self.theme.mode(),
            trigger_style,
            state,
            without_elevation,
        )
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        trigger_style: xgpui_luma::controls::selector::SelectorTriggerStyle,
        state: InteractionState,
        size: ControlSize,
        scale: &xgpui_luma::theme::StandardBoxScale,
        without_elevation: bool,
    ) -> xgpui_luma::controls::selector::SelectorLook {
        let tokens = self.theme.mode_tokens();
        crate::controls::selector::selector_look(
            tokens.as_ref(),
            self.theme.mode(),
            trigger_style,
            state,
            size,
            scale,
            without_elevation,
        )
    }

    fn resolve_visual_look(
        &self,
        trigger_style: xgpui_luma::controls::selector::SelectorTriggerStyle,
        visual_state: SelectorVisualState,
        size: ControlSize,
        scale: &xgpui_luma::theme::StandardBoxScale,
        without_elevation: bool,
    ) -> xgpui_luma::controls::selector::SelectorLook {
        let tokens = self.theme.mode_tokens();
        let look = crate::controls::selector::selector_look_with_visual_state(
            tokens.as_ref(),
            self.theme.mode(),
            trigger_style,
            visual_state,
            size,
            scale,
            without_elevation,
        );
        look
    }
}

pub fn selector_template<T>(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::selector::SelectorTemplate<T>>
where
    T: SelectorItemLike + 'static,
{
    Arc::new(ThemedSelectorTemplate::new(
        Arc::new(ShadcnSelectorTheme { theme: theme.clone() }),
        default_selector_items_template::<T>(),
    ))
}
