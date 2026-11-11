//! Radix selector theme and builder backed by the shared SDK selector.
use std::sync::Arc;

use gpui::{Context, Entity, SharedString};
use xgpui_luma::controls::selector::{
    SelectorBuilder, SelectorItem, SelectorItemLike, SelectorOpeningMode, SelectorPalette, SelectorPlacement,
    SelectorTemplate, SelectorTheme, SelectorTriggerStyle, ThemedSelectorTemplate,
};
use xgpui_luma::controls::selector_list::default_selector_items_template;
use xgpui_luma::theme::{InteractionState, MetricTokens};

use crate::{ButtonSize, Look, PopupMenuVariant, Tone};
use crate::look::resolve_look;

struct SelectorThemeAdapter {
    look: Look,
}

impl SelectorTheme for SelectorThemeAdapter {
    fn resolve(
        &self,
        style: SelectorTriggerStyle,
        state: InteractionState,
        without_elevation: bool,
    ) -> SelectorPalette {
        use xgpui_luma::controls::popup_menu::{PopupMenuTriggerMetrics, PopupMenuTriggerStyle};
        let trigger_style = match style {
            SelectorTriggerStyle::Outline => PopupMenuTriggerStyle::Outline,
            SelectorTriggerStyle::Ghost => PopupMenuTriggerStyle::Ghost,
        };
        let palette = crate::popup_menu_theme(&self.look, PopupMenuVariant::Solid, Tone::Accent).resolve(
            trigger_style,
            PopupMenuTriggerMetrics::default(),
            state,
        );
        SelectorPalette {
            trigger_focus_border: palette.trigger_focus_border,
            trigger_background: palette.trigger_background,
            trigger_foreground: palette.trigger_foreground,
            trigger_icon: palette.trigger_foreground,
            trigger_border: palette.trigger_border,
            trigger_shadow: if without_elevation {
                None
            } else {
                palette.trigger_shadow
            },
            trigger_typography: palette.trigger_typography,
            items_panel: crate::selector_items_panel_look(&self.look, PopupMenuVariant::Solid, Tone::Accent),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
}

/// Shared Radix selector trigger and panel theme for SDK composition.
/// Retains the supplied Look and resolves palette changes on the next render.
pub fn selector_theme(look: &Look) -> Arc<dyn SelectorTheme> {
    Arc::new(SelectorThemeAdapter { look: look.clone() })
}

/// Build a selector template using the Radix trigger and menu palette.
pub fn selector_template<T: SelectorItemLike + 'static>(look: &Look) -> Arc<dyn SelectorTemplate<T>> {
    Arc::new(ThemedSelectorTemplate::new(selector_theme(look), default_selector_items_template()))
}

/// Radix selector builder. Opening and placement behavior are shared with the SDK.
pub struct Selector<T: SelectorItemLike + 'static = SelectorItem> {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::selector::SelectorEvent>,

    look: Option<Look>,
    builder: SelectorBuilder<T>,
}

impl Selector<SelectorItem> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self::new_typed(id)
    }
}

impl<T: SelectorItemLike + 'static> Selector<T> {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::selector::SelectorEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SelectorEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selector::SelectorEvent::Change { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `SelectorEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selector::SelectorEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `SelectorEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selector::SelectorEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `SelectorEvent::Dismiss` through the existing event stream.
    pub fn on_dismiss<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selector::SelectorEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    pub fn new_typed(id: impl Into<SharedString>) -> Self {
        Self { callbacks: Default::default(), look: None, builder: SelectorBuilder::new(id) }
    }

    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.builder = self.builder.label(label);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn selected_id(mut self, id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.selected_id(id);
        self
    }

    pub fn opening_mode(mut self, mode: SelectorOpeningMode) -> Self {
        self.builder = self.builder.opening_mode(mode);
        self
    }

    pub fn placement(mut self, placement: SelectorPlacement) -> Self {
        self.builder = self.builder.placement(placement);
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.builder = self.builder.size(size.control_size());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::selector::Selector<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
            self.builder.stylesheet(&look.common_stylesheet()).template(selector_template(&look)).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod stylesheet_tests {
    use super::*;

    #[test]
    fn default_panel_applies_common_selector_list_overrides() {
        let look = Look::built_in().fork();
        let sheet = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[
            "[common.selector_list.geometry]\nmin_width=0\nradius=7\nborder_width=0\ndisabled_opacity=0\npopup_offset_y=0\nwindow_margin=0\n[common.selector_list.sizes.\"2\"]\nitem_height=41",
        ]).unwrap();
        look.set_common_stylesheet(sheet);
        let theme = SelectorThemeAdapter { look };
        let panel = theme.resolve(SelectorTriggerStyle::Outline, Default::default(), false).items_panel;
        assert_eq!((panel.min_width, panel.radius, panel.item_height), (0.0, 7.0, 41.0));
        assert_eq!(
            (panel.border_width, panel.disabled_opacity, panel.popup_offset_y, panel.window_margin),
            (0.0, 0.0, 0.0, 0.0)
        );
    }
}
