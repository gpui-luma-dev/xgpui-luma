//! Look-owned table builder. Spawn synthesizes the SDK [`xgpui_luma::controls::table::Table`].

use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, ListAlignment, SharedString, Window};
use xgpui_luma::controls::table::{
    TableSelectionMode, TableBuilder, TableColumn, TableControl, TableRowRenderModel, TableTemplate, TableTheme,
};
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a table: Shadcn theme/template plus SDK options, until `.spawn(cx)`.
pub struct Table<T>
where
    T: 'static,
{
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::table::TableEvent>,

    look: Option<ShadcnLook>,
    builder: TableBuilder<T>,
    custom_template: bool,
    custom_theme: bool,
    size: ShadcnSize,
}

impl<T> Table<T>
where
    T: 'static,
{
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(enabled);
        self
    }

    xgpui_luma::forward_motion!(xgpui_luma::controls::table::TableMotion, builder builder);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::table::TableEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Observe every drag/drop lifecycle phase. Alias of `on_row_drag`.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::table::TableRowDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_row_drag(cx, callback)
    }

    /// Observe `DragStarted` through the existing drag/drop lifecycle.
    pub fn on_drag_started<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::DragStarted { source, keys } = event {
                callback(owner, *source, keys, cx);
            }
        })
    }

    /// Observe `ItemsRemoved` through the existing drag/drop lifecycle.
    pub fn on_items_removed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::ItemsRemoved { source, target, keys } = event {
                callback(owner, *source, *target, keys, cx);
            }
        })
    }

    /// Observe `ItemsAdded` through the existing drag/drop lifecycle.
    pub fn on_items_added<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::ItemsAdded { source, target, keys, before } = event {
                callback(owner, *source, *target, keys, before, cx);
            }
        })
    }

    /// Observe `ItemsReordered` through the existing drag/drop lifecycle.
    pub fn on_items_reordered<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::ItemsReordered { target, keys, before } = event {
                callback(owner, *target, keys, before, cx);
            }
        })
    }

    /// Observe `Dropped` through the existing drag/drop lifecycle.
    pub fn on_dropped<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::Dropped { source, target, keys, before, changed } =
                event
            {
                callback(owner, *source, *target, keys, before, *changed, cx);
            }
        })
    }

    /// Observe `DropRejected` through the existing drag/drop lifecycle.
    pub fn on_drop_rejected<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &gpui::SharedString,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::DropRejected { source, target, keys, before, error } =
                event
            {
                callback(owner, *source, *target, keys, before, error, cx);
            }
        })
    }

    /// Observe `DragEnded` through the existing drag/drop lifecycle.
    pub fn on_drag_ended<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            xgpui_luma::infra::drag_drop::DragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::DragEnded { source, keys, reason } = event {
                callback(owner, *source, keys, *reason, cx);
            }
        })
    }

    /// Handle `TableEvent::RowDrag` through the existing event stream.
    pub fn on_row_drag<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::table::TableRowDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::table::TableEvent::RowDrag(event) = event {
                callback(owner, event, cx);
            }
        })
    }

    /// Handle `TableEvent::SelectionChanged` through the existing event stream.
    pub fn on_selection_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Vec<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::table::TableEvent::SelectionChanged { selected_indices } = event {
                callback(owner, selected_indices, cx);
            }
        })
    }

    /// Handle `TableEvent::SelectedKeysChanged` through the existing event stream.
    pub fn on_selected_keys_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::table::TableEvent::SelectedKeysChanged { selected_keys } = event {
                callback(owner, selected_keys, cx);
            }
        })
    }

    /// Handle `TableEvent::ActiveIndexChanged` through the existing event stream.
    pub fn on_active_index_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::table::TableEvent::ActiveIndexChanged { active_index } = event {
                callback(owner, active_index, cx);
            }
        })
    }

    /// Handle `TableEvent::PageChanged` through the existing event stream.
    pub fn on_page_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::table::TableEvent::PageChanged { page } = event {
                callback(owner, *page, cx);
            }
        })
    }

    /// Handle `TableEvent::PageSizeChanged` through the existing event stream.
    pub fn on_page_size_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::table::TableEvent::PageSizeChanged { page_size } = event {
                callback(owner, *page_size, cx);
            }
        })
    }

    /// Handle `TableEvent::ScrollChanged` through the existing event stream.
    pub fn on_scroll_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::table::TableEvent::ScrollChanged { top_index } = event {
                callback(owner, *top_index, cx);
            }
        })
    }

    /// Handle `TableEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::table::TableEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `TableEvent::RowHoverChanged` through the existing event stream.
    pub fn on_row_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::table::TableEvent::RowHoverChanged { index, hovered } = event {
                callback(owner, *index, *hovered, cx);
            }
        })
    }

    /// Handle `TableEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::table::TableEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            look: None,
            builder: xgpui_luma::controls::table::new_typed(id),
            custom_template: false,
            custom_theme: false,
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
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

    pub fn selection_mode(mut self, selection_mode: TableSelectionMode) -> Self {
        self.builder = self.builder.selection_mode(selection_mode);
        self
    }

    pub fn single(mut self) -> Self {
        self.builder = self.builder.single();
        self
    }

    pub fn multiple(mut self) -> Self {
        self.builder = self.builder.multiple();
        self
    }

    pub fn extended(mut self) -> Self {
        self.builder = self.builder.extended();
        self
    }

    pub fn no_selection(mut self) -> Self {
        self.builder = self.builder.no_selection();
        self
    }

    pub fn select_on_row_click(mut self, select_on_row_click: bool) -> Self {
        self.builder = self.builder.select_on_row_click(select_on_row_click);
        self
    }

    pub fn selected_index(mut self, selected_index: usize) -> Self {
        self.builder = self.builder.selected_index(selected_index);
        self
    }

    pub fn selected_indices(mut self, selected_indices: impl IntoIterator<Item = usize>) -> Self {
        self.builder = self.builder.selected_indices(selected_indices);
        self
    }

    pub fn active_index(mut self, active_index: usize) -> Self {
        self.builder = self.builder.active_index(active_index);
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

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn alignment(mut self, alignment: ListAlignment) -> Self {
        self.builder = self.builder.alignment(alignment);
        self
    }

    pub fn overdraw(mut self, overdraw: impl Into<f64>) -> Self {
        self.builder = self.builder.overdraw(overdraw);
        self
    }

    pub fn paged(mut self, page_size: usize) -> Self {
        self.builder = self.builder.paged(page_size);
        self
    }

    pub fn scroll_snap(mut self, enabled: bool) -> Self {
        self.builder = self.builder.scroll_snap(enabled);
        self
    }

    pub fn visible_rows(mut self, count: usize) -> Self {
        self.builder = self.builder.visible_rows(count);
        self
    }

    pub fn fill_height(mut self) -> Self {
        self.builder = self.builder.fill_height();
        self
    }

    pub fn visible_row_height(mut self, height: impl Into<f32>) -> Self {
        self.builder = self.builder.visible_row_height(height);
        self
    }

    pub fn row_label<F, S>(mut self, label: F) -> Self
    where
        F: Fn(&T) -> S + Send + Sync + 'static,
        S: Into<SharedString>,
    {
        self.builder = self.builder.row_label(label);
        self
    }

    pub fn row_enabled<F>(mut self, enabled: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.builder = self.builder.row_enabled(enabled);
        self
    }

    pub fn template(mut self, template: Arc<dyn TableTemplate>) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn theme(mut self, theme: Arc<dyn TableTheme>) -> Self {
        self.custom_theme = true;
        self.builder = self.builder.theme(theme);
        self
    }

    pub fn with_row_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&TableRowRenderModel<'a, T>, AnyElement, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.builder = self.builder.with_row_template(template);
        self
    }

    pub fn grid_view(mut self, columns: impl IntoIterator<Item = TableColumn<T>>) -> Self {
        self.builder = self.builder.grid_view(columns);
        self
    }

    /// Enable manual resizing between adjacent resizable grid columns.
    pub fn column_resizing(mut self, enabled: bool) -> Self {
        self.builder = self.builder.column_resizing(enabled);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<TableControl<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look_from(self.look.as_ref(), cx);
            self.into_sdk_builder(look).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> TableBuilder<T> {
        let self_builder = self.builder.motion_defaults(look.motion_defaults().table);
        let mut builder = self_builder.size(self.size.control_size());
        if !self.custom_theme {
            builder = builder.theme(look.table_theme());
        }
        if !self.custom_template {
            builder = builder.template(look.table_template());
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
        let _builder = Table::new("ok")
            .look(&look)
            .items([SharedString::from("one")])
            .row_label(|row| row.clone())
            .into_sdk_builder(look);
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn forwards_independent_wheel_policies_after_look_synthesis() {
    use xgpui_luma::interaction::{WheelScrollPolicy, ScrollBoundaryPolicy, WheelFocusScope};
    let mut app = gpui::TestAppContext::single();
    let control = Table::<usize>::new("policy")
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
