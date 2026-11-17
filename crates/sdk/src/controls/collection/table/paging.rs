use std::sync::Arc;

use gpui::{App, AppContext, Context, Entity, EventEmitter, Render, Window, div, prelude::*, px};

use crate::controls::pager::{
    PagerBuilder, PagerControl, PagerEvent, PagerRenderModel, PagerStyle, PagerTemplate, PagerTemplateHandlers,
    PagerTheme, default_pager_theme, render_info_slot, render_nav_group, render_page_indicator,
    render_page_size_select,
};

use super::control::{TableControl, TableEvent};
use super::model::TableBuilder;

pub type PagingTable<T> = Entity<PagingTableControl<T>>;

pub struct PagingTableControl<T: 'static> {
    list: Entity<TableControl<T>>,
    pager: Entity<PagerControl>,
}

impl<T: 'static> PagingTableControl<T> {
    pub fn new(list: Entity<TableControl<T>>, pager: Entity<PagerControl>, cx: &mut Context<Self>) -> Self {
        let pager_clone = pager.clone();
        cx.subscribe(&list, move |_, list_control, event, cx| {
            let list = list_control.read(cx);
            match event {
                TableEvent::SelectionChanged { selected_indices } => {
                    let count = selected_indices.len();
                    let total = list.items().len();
                    pager_clone.update(cx, |pager, cx| pager.set_info_text(Some(selection_summary(count, total)), cx));
                }
                TableEvent::PageChanged { page } => {
                    let page_size = list.page_size().unwrap_or(10);
                    let page_count = list.page_count();
                    pager_clone.update(cx, |pager, cx| {
                        pager.set_page_count(page_count, cx);
                        pager.set_page_size(page_size, cx);
                        pager.set_page(*page, cx);
                    });
                }
                TableEvent::PageSizeChanged { page_size } => {
                    let current_page = list.current_page();
                    let page_count = list.page_count();
                    pager_clone.update(cx, |pager, cx| {
                        pager.set_page_size(*page_size, cx);
                        pager.set_page_count(page_count, cx);
                        pager.set_page(current_page, cx);
                    });
                }
                TableEvent::ActiveIndexChanged { .. } => {}
                _ => {}
            }
        })
        .detach();

        let list_clone = list.clone();
        cx.subscribe(&pager, move |_, _, event, cx| {
            list_clone.update(cx, |list_control, cx| {
                if let PagerEvent::PageChanged { page } = event {
                    list_control.set_page(*page, cx);
                } else if let PagerEvent::PageSizeChanged { page_size } = event {
                    list_control.set_page_size(*page_size, cx);
                }
            });
        })
        .detach();

        cx.subscribe(&list, |_, _, event, cx| {
            cx.emit(event.clone());
        })
        .detach();

        Self { list, pager }
    }

    pub fn list(&self) -> &Entity<TableControl<T>> {
        &self.list
    }

    pub fn pager(&self) -> &Entity<PagerControl> {
        &self.pager
    }

    fn fills_height(&self, cx: &App) -> bool {
        self.list.read(cx).fills_height()
    }
}

impl<T: 'static> EventEmitter<TableEvent> for PagingTableControl<T> {}

impl<T: 'static> Render for PagingTableControl<T> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let list = self.list.clone();
        let pager = self.pager.clone();
        if self.fills_height(cx) {
            div()
                .size_full()
                .min_h_0()
                .flex()
                .flex_col()
                .child(div().flex_1().min_h_0().w_full().child(list))
                .child(div().flex_none().w_full().child(pager))
        } else {
            div().w_full().flex().flex_col().child(list).child(pager)
        }
    }
}

struct TablePagerTemplate {
    theme: Arc<dyn PagerTheme>,
}

impl TablePagerTemplate {
    fn new(theme: Arc<dyn PagerTheme>) -> Self {
        Self { theme }
    }
}

impl PagerTemplate for TablePagerTemplate {
    fn render(
        &self,
        model: &PagerRenderModel<'_>,
        handlers: PagerTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyElement {
        let look = self.theme.resolve(model.enabled, PagerStyle::MinimalEdge);

        div()
            .w_full()
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(look.group_gap))
            .py(px(look.padding_y))
            .text_color(look.muted_text)
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .child(
                render_info_slot(model, window, cx).unwrap_or_else(|| div().flex_1().min_w(px(0.0)).into_any_element()),
            )
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(px(look.group_gap))
                    .child(div().flex_none().flex().items_center().gap(px(look.page_size_label_gap)).when(
                        !model.page_size_options.is_empty(),
                        |slot| {
                            slot.when_some(model.page_size_label(), |slot, label| {
                                slot.child(
                                    div()
                                        .flex_none()
                                        .text_color(look.muted_text)
                                        .text_size(px(look.typography.size))
                                        .line_height(px(look.typography.line_height))
                                        .child(label.clone()),
                                )
                            })
                            .child(render_page_size_select(model, &look, &handlers, cx))
                        },
                    ))
                    .child(render_page_indicator(model, &look, cx))
                    .child(render_nav_group(model, &look, &self.theme, &handlers, true, window, cx)),
            )
            .into_any_element()
    }
}

pub struct PagingTableBuilder<T: 'static> {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::table::TableEvent>,

    list_builder: TableBuilder<T>,
    pager_builder: PagerBuilder,
}

impl<T: 'static> PagingTableBuilder<T> {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::table::TableEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Observe every drag/drop lifecycle phase. Alias of `on_row_drag`.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::table::TableRowDragEvent, &mut gpui::Context<M>) + 'static,
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
            if let crate::infra::drag_drop::DragDropEvent::DragStarted { source, keys } = event {
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
            if let crate::infra::drag_drop::DragDropEvent::ItemsRemoved { source, target, keys } = event {
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
            if let crate::infra::drag_drop::DragDropEvent::ItemsAdded { source, target, keys, before } = event {
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
            if let crate::infra::drag_drop::DragDropEvent::ItemsReordered { target, keys, before } = event {
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
            if let crate::infra::drag_drop::DragDropEvent::Dropped { source, target, keys, before, changed } = event {
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
            if let crate::infra::drag_drop::DragDropEvent::DropRejected { source, target, keys, before, error } = event
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
            crate::infra::drag_drop::DragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::DragEnded { source, keys, reason } = event {
                callback(owner, *source, keys, *reason, cx);
            }
        })
    }

    /// Handle `TableEvent::RowDrag` through the existing event stream.
    pub fn on_row_drag<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::table::TableRowDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::RowDrag(event) = event {
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
            if let crate::controls::table::TableEvent::SelectionChanged { selected_indices } = event {
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
            if let crate::controls::table::TableEvent::SelectedKeysChanged { selected_keys } = event {
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
            if let crate::controls::table::TableEvent::ActiveIndexChanged { active_index } = event {
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
            if let crate::controls::table::TableEvent::PageChanged { page } = event {
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
            if let crate::controls::table::TableEvent::PageSizeChanged { page_size } = event {
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
            if let crate::controls::table::TableEvent::ScrollChanged { top_index } = event {
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
            if let crate::controls::table::TableEvent::FocusChanged { focused } = event {
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
            if let crate::controls::table::TableEvent::RowHoverChanged { index, hovered } = event {
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
            if let crate::controls::table::TableEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(list_builder: TableBuilder<T>, pager_builder: PagerBuilder) -> Self {
        Self { callbacks: Default::default(), list_builder, pager_builder }
    }

    /// Fill the parent height; see [`TableBuilder::fill_height`].
    pub fn fill_height(mut self) -> Self {
        self.list_builder = self.list_builder.fill_height();
        self
    }

    /// Enable manual resizing between adjacent resizable grid columns.
    pub fn column_resizing(mut self, enabled: bool) -> Self {
        self.list_builder = self.list_builder.column_resizing(enabled);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> PagingTable<T> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let fill_height = self.list_builder.model.fill_height;
            let list = self.list_builder.spawn(cx);

            let (current_page, page_count, page_size, selected_count, total_rows) = list.read_with(cx, |list, _| {
                (
                    list.current_page(),
                    list.page_count(),
                    list.page_size().unwrap_or(10),
                    list.selected_indices().len(),
                    list.items().len(),
                )
            });

            let mut pager_builder = self.pager_builder;
            // Viewport-driven page size: hide the manual "rows per page" control.
            if fill_height {
                pager_builder = pager_builder.page_size_options([]);
            } else if pager_builder.model.template_parameters.page_size_label.is_none() {
                pager_builder = pager_builder.page_size_label("Rows per page");
            }
            let pager_theme = pager_builder.model.theme.clone().unwrap_or_else(default_pager_theme);
            pager_builder = pager_builder.style(PagerStyle::MinimalEdge);
            pager_builder = pager_builder.template(Arc::new(TablePagerTemplate::new(pager_theme)));

            let pager = pager_builder
                .current_page(current_page)
                .page_count(page_count)
                .page_size(page_size)
                .info_text(selection_summary(selected_count, total_rows))
                .spawn(cx);

            cx.new(|cx| PagingTableControl::new(list, pager, cx))
        };
        callbacks.bind(&control, cx);
        control
    }
}

fn selection_summary(selected_count: usize, total_rows: usize) -> String {
    format!("{} of {} row(s) selected.", selected_count, total_rows)
}
