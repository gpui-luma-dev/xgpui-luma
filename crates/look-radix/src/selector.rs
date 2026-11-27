//! Radix selector theme and builder backed by the shared SDK selector.
use std::sync::Arc;

use gpui::{Context, Entity, SharedString};
use xgpui_luma::controls::selector::{
    SelectorBuilder, SelectorItem, SelectorItemLike, SelectorOpeningMode, SelectorPlacement, SelectorTemplate,
    SelectorTheme, StyledSelectorTheme, ThemedSelectorTemplate,
};
use xgpui_luma::controls::selector_list::default_selector_items_template;

use crate::{ButtonSize, Look, PopupMenuVariant, Tone};
use crate::look::resolve_look;

/// The Radix selector trigger drawn from `.lss` (`style/selector.lss`) at the look's size; the popup panel
/// keeps the look's solid accent menu values. Follows the look's later edits.
pub fn selector_theme(look: &Look) -> Arc<dyn SelectorTheme> {
    let size = look.size();
    let panel = look.clone();
    Arc::new(
        StyledSelectorTheme::new(
            crate::textfield::live_source(look),
            Arc::new(move || crate::selector_items_panel_look(&panel, PopupMenuVariant::Solid, Tone::Accent)),
        )
        .with(|binding| {
            if size != crate::RadixSize::Two {
                binding.attr("size", size.key())
            } else {
                binding
            }
        }),
    )
}

/// Build a selector template using the Radix trigger and menu palette.
pub fn selector_template<T: SelectorItemLike + 'static>(look: &Look) -> Arc<dyn SelectorTemplate<T>> {
    Arc::new(ThemedSelectorTemplate::new(selector_theme(look), default_selector_items_template()))
}

/// Radix selector builder. Opening and placement behavior are shared with the SDK.
pub struct Selector<T: SelectorItemLike + 'static = SelectorItem> {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::selector::SelectorEvent>,

    look: Option<Look>,
    size: ButtonSize,
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
        Self {
            callbacks: Default::default(),
            look: None,
            size: ButtonSize::default(),
            builder: SelectorBuilder::new(id),
        }
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
        self.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::selector::Selector<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>()).with_size(self.size.control_size());
            self.builder.stylesheet(&look.stylesheet_snapshot()).template(selector_template(&look)).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod lss_tests {
    use super::*;
    use crate::SemanticRole;
    use crate::control_group::assert_close;
    use xgpui_luma::controls::selector::{SelectorTriggerStyle, SelectorVisualState};
    use xgpui_luma::theme::{InteractionState, StandardBoxScale};

    #[test]
    fn trigger_matches_the_popup_menu_trigger() {
        let look = Look::built_in();
        let theme = selector_theme(&look);
        let scale = StandardBoxScale::compute(&theme.metrics(), 1.0);
        let at = |style, interaction| {
            theme.resolve_visual_look(style, SelectorVisualState { interaction, ..Default::default() }, &scale, false)
        };
        let rest = at(SelectorTriggerStyle::Outline, InteractionState::default());
        assert_close(rest.trigger_background, look.resolve_role(SemanticRole::Background).hsla());
        assert_close(rest.trigger_border.unwrap(), look.resolve_role(SemanticRole::Border).hsla());
        assert_close(rest.trigger_icon, rest.trigger_foreground);
        assert_eq!(
            (
                rest.trigger_height,
                rest.trigger_padding_x,
                rest.trigger_gap,
                rest.trigger_radius,
                rest.trigger_icon_size
            ),
            (32.0, 12.0, 8.0, 4.0, 14.0)
        );
        assert_eq!((rest.trigger_typography.size, rest.trigger_typography.line_height), (14.0, 20.0));
        assert_eq!(rest.trigger_typography.weight, gpui::FontWeight::MEDIUM);
        assert_eq!(rest.menu_offset_y, 4.0);
        let hover = at(SelectorTriggerStyle::Ghost, InteractionState { hovered: true, ..Default::default() });
        assert_close(hover.trigger_background, Tone::Accent.step(&look, 3));
        let disabled = at(SelectorTriggerStyle::Ghost, InteractionState { disabled: true, ..Default::default() });
        assert_close(disabled.trigger_background, look.resolve_role(SemanticRole::Surface).hsla());
        assert_close(disabled.trigger_foreground, look.resolve_role(SemanticRole::MutedForeground).hsla());
        assert_close(disabled.trigger_icon, disabled.trigger_foreground);
        assert_eq!(disabled.trigger_opacity, 1.0, "Radix draws disabled colors and does not fade");
        let small = selector_theme(&look.with_size(crate::RadixSize::One));
        let small = small.resolve_look(SelectorTriggerStyle::Outline, Default::default(), &scale, false);
        assert_eq!((small.trigger_height, small.trigger_typography.size, small.menu_offset_y), (24.0, 14.0, 2.0));
    }
}
