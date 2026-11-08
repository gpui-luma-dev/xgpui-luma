//! Look-owned selection-panel builder. Spawn synthesizes the SDK [`xgpui_luma::controls::selection_panel::SelectionPanelControl`].

use gpui::{App, Context, Entity, IntoElement, SharedString};
use xgpui_luma::controls::selection_panel::{
    SelectionPanelBuilder, SelectionPanelControl, SelectionPanelItem, SelectionPanelItemLike,
    SelectionPanelItemRenderModel, SelectionPanelLookProvider,
};
use xgpui_luma::infra::icon::SelectionStatusIcons;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a selection panel: Shadcn look plus SDK options, until `.spawn(cx)`.
pub struct SelectionPanel<T = SelectionPanelItem>
where
    T: SelectionPanelItemLike + 'static,
{
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::selection_panel::SelectionPanelEvent>,

    look: Option<ShadcnLook>,
    builder: SelectionPanelBuilder<T>,
    custom_look_provider: bool,
    size: ShadcnSize,
}

impl SelectionPanel<SelectionPanelItem> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self::new_typed(id)
    }
}

impl<T> SelectionPanel<T>
where
    T: SelectionPanelItemLike + 'static,
{
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::selection_panel::SelectionPanelEvent, &mut gpui::Context<M>)
        + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SelectionPanelEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selection_panel::SelectionPanelEvent::HoverChanged { visible_index } = event {
                callback(owner, visible_index, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::ActivateRow` through the existing event stream.
    pub fn on_activate_row<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, usize, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selection_panel::SelectionPanelEvent::ActivateRow {
                source_index,
                visible_index,
                item_id,
            } = event
            {
                callback(owner, *source_index, *visible_index, item_id, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::ActiveIndexChanged` through the existing event stream.
    pub fn on_active_index_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selection_panel::SelectionPanelEvent::ActiveIndexChanged { visible_index } =
                event
            {
                callback(owner, visible_index, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selection_panel::SelectionPanelEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selection_panel::SelectionPanelEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    pub fn new_typed(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            look: None,
            builder: SelectionPanelBuilder::new(id),
            custom_look_provider: false,
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn panel_id(mut self, panel_id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.panel_id(panel_id);
        self
    }

    pub fn item(mut self, item: T) -> Self {
        self.builder = self.builder.item(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn visible_indices(mut self, visible_indices: impl IntoIterator<Item = usize>) -> Self {
        self.builder = self.builder.visible_indices(visible_indices);
        self
    }

    pub fn selected_source_index(mut self, selected_source_index: Option<usize>) -> Self {
        self.builder = self.builder.selected_source_index(selected_source_index);
        self
    }

    pub fn active_visible_index(mut self, active_visible_index: Option<usize>) -> Self {
        self.builder = self.builder.active_visible_index(active_visible_index);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.builder = self.builder.open(open);
        self
    }

    /// Override the SDK interaction policy.
    /// Override focus behavior independently of wheel routing.
    pub fn pointer_focus_policy(mut self, policy: xgpui_luma::interaction::PointerFocusPolicy) -> Self {
        self.builder = self.builder.pointer_focus_policy(policy);
        self
    }

    pub fn wheel_scroll_policy(mut self, policy: xgpui_luma::interaction::WheelScrollPolicy) -> Self {
        self.builder = self.builder.wheel_scroll_policy(policy);
        self
    }

    /// Override the SDK interaction policy.
    pub fn scroll_boundary_policy(mut self, policy: xgpui_luma::interaction::ScrollBoundaryPolicy) -> Self {
        self.builder = self.builder.scroll_boundary_policy(policy);
        self
    }

    /// Override the SDK interaction policy.
    pub fn wheel_focus_scope(mut self, policy: xgpui_luma::interaction::WheelFocusScope) -> Self {
        self.builder = self.builder.wheel_focus_scope(policy);
        self
    }

    /// Select embedded or popup hover defaults.
    pub fn role(mut self, role: xgpui_luma::controls::selection_panel::SelectionPanelRole) -> Self {
        self.builder = self.builder.role(role);
        self
    }
    /// Explicitly override hover activation; does not focus or select.
    pub fn hover_activation_policy(
        mut self,
        policy: xgpui_luma::controls::selection_panel::HoverActivationPolicy,
    ) -> Self {
        self.builder = self.builder.hover_activation_policy(policy);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn show_selection_marker(mut self, show_selection_marker: bool) -> Self {
        self.builder = self.builder.show_selection_marker(show_selection_marker);
        self
    }

    pub fn icons(mut self, icons: SelectionStatusIcons) -> Self {
        self.builder = self.builder.icons(icons);
        self
    }

    pub fn scrolling(mut self, scrolling: bool) -> Self {
        self.builder = self.builder.scrolling(scrolling);
        self
    }

    /// Always show, hide, or auto-hide the panel scrollbar.
    pub fn scrollbar_visibility(
        mut self,
        visibility: xgpui_luma::controls::scroll_container::ScrollbarVisibility,
    ) -> Self {
        self.builder = self.builder.scrollbar_visibility(visibility);
        self
    }

    /// Choose hover, timed scroll activity, or both for auto-hiding chrome.
    pub fn scrollbar_auto_hide_activate(
        mut self,
        activate: xgpui_luma::controls::scroll_container::ScrollbarAutoHideActivate,
    ) -> Self {
        self.builder = self.builder.scrollbar_auto_hide_activate(activate);
        self
    }

    pub fn min_visible_rows(mut self, min_visible_rows: usize) -> Self {
        self.builder = self.builder.min_visible_rows(min_visible_rows);
        self
    }

    pub fn max_visible_rows(mut self, max_visible_rows: usize) -> Self {
        self.builder = self.builder.max_visible_rows(max_visible_rows);
        self
    }

    pub fn visible_row_limits(mut self, min_visible_rows: usize, max_visible_rows: usize) -> Self {
        self.builder = self.builder.visible_row_limits(min_visible_rows, max_visible_rows);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&SelectionPanelItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.builder = self.builder.with_item_template(template);
        self
    }

    pub fn look_provider(mut self, provider: SelectionPanelLookProvider) -> Self {
        self.custom_look_provider = true;
        self.builder = self.builder.look_provider(provider);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<SelectionPanelControl<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look_from(self.look.as_ref(), cx);
            self.into_sdk_builder(look).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> SelectionPanelBuilder<T> {
        let theme = look.clone();
        let mut builder = self
            .builder
            .stylesheet(&look.stylesheet().common)
            .size(self.size.control_size())
            .scrollbar_template(theme.scrollbar_template());
        if !self.custom_look_provider {
            builder = builder.look_provider(theme.selection_panel_look_provider());
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = SelectionPanel::new("ok")
            .look(&look)
            .items([SelectionPanelItem::new("a").label("A")])
            .into_sdk_builder(look);
    }

    #[test]
    fn typed_into_sdk_builder_does_not_panic() {
        struct ThemeItem {
            id: SharedString,
            label: SharedString,
        }
        impl SelectionPanelItemLike for ThemeItem {
            fn id(&self) -> &SharedString {
                &self.id
            }
            fn label(&self) -> &SharedString {
                &self.label
            }
        }

        let look = ShadcnLook::built_in();
        let _builder = SelectionPanel::new_typed("ok")
            .look(&look)
            .items([ThemeItem { id: "a".into(), label: "A".into() }])
            .into_sdk_builder(look);
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn forwards_independent_wheel_policies_after_look_synthesis() {
    use xgpui_luma::interaction::{WheelScrollPolicy, ScrollBoundaryPolicy, WheelFocusScope};
    let mut app = gpui::TestAppContext::single();
    let control = SelectionPanel::new("policy")
        .wheel_scroll_policy(WheelScrollPolicy::PassThrough)
        .scroll_boundary_policy(ScrollBoundaryPolicy::Chain)
        .wheel_focus_scope(WheelFocusScope::Owner)
        .into_sdk_builder(ShadcnLook::built_in())
        .spawn(&mut app);
    control.read_with(&app, |view, _| {
        let policy = view.scroll_interaction();
        assert_eq!(policy.wheel, WheelScrollPolicy::PassThrough);
        assert_eq!(policy.boundary, ScrollBoundaryPolicy::Chain);
        assert_eq!(policy.focus_scope, WheelFocusScope::Owner);
    });
}
