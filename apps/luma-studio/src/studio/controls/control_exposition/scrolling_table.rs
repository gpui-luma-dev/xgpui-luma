//! Scrolling table control exposition — task grid that fills the available pane.

use xgpui_luma::controls::table::TableEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::table::{Table, TableEvent};
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionItem, AccordionTrigger};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::{column, column_emphasis};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::collection_theme_inspectors::TableThemeInspector;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::table_demo::{Task, build_task_rows_with_count, email_column, status_cell, tag_pill};
use super::inspector::{TableInspectorAdapter, TABLE_INSPECTOR_SPEC, table_motion_report};
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

const TASK_COUNT: usize = 10_000;

pub struct ScrollingTableControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ScrollingTableExpositionLeftPane>,
    theme_inspector: Entity<TableThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct ScrollingTableExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    table: Table<Task>,
    sort_button: Entity<Button>,
    select_button: Entity<Button>,
    jump_button: Entity<Button>,
    event_stream: Entity<ControlEventStream>,
    details: Entity<AccordionControl>,
}

impl ScrollingTableExpositionLeftPane {
    fn handle_list_event(&mut self, event: &TableEvent, cx: &mut Context<Self>) {
        if matches!(event, TableEvent::SelectionChanged { .. } | TableEvent::SelectedKeysChanged { .. }) {
            cx.notify();
        }
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.table.update(cx, |_, cx| cx.notify());
        self.sort_button.update(cx, |_, cx| cx.notify());
        self.select_button.update(cx, |_, cx| cx.notify());
        self.jump_button.update(cx, |_, cx| cx.notify());
        self.details.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ScrollingTableExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();

            let preview = div()
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .items_start()
                .gap(px(16.0))
                .child(
                    div()
                        .w_full()
                        .min_w(px(0.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(div().text_lg().text_color(chrome.title_text).child(self.entry.title))
                        .child(
                            div()
                                .text_sm()
                                .text_color(chrome.muted_text)
                                .child("10,000 tasks · 36 px rows. Scroll, resize columns, reverse the order, or jump to the last row."),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(chrome.muted_text)
                                .child("Drag a row or selected group to reorder. Hold near an edge to scroll; Escape cancels."),
                        )
                        .child(
                            div()
                                .w_full()
                                .truncate()
                                .text_sm()
                                .text_color(chrome.body_text)
                                .child(format!("Selected task IDs: {}", self.table.read(cx).selected_keys().iter().map(|key| key.as_ref()).collect::<Vec<_>>().join(", "))),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(chrome.muted_text)
                                .child("Click selects; Ctrl/Cmd-click toggles; Shift-click or Shift+arrows extends. Ctrl/Cmd+A selects all; add Shift to clear."),
                        ),
                )
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.0))
                        .child(self.sort_button.clone())
                        .child(self.select_button.clone())
                        .child(self.jump_button.clone()),
                )
                .child(div().w_full().flex_none().child(self.details.clone()))
                .child(div().w_full().flex_1().min_h(px(0.0)).overflow_hidden().child(self.table.clone()));

            div()
                .id("controls-doc-scrolling-table-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .p(px(16.0))
                .overflow_hidden()
                .child(preview)
        })
    }
}

impl ScrollingTableControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("scrolling-table").expect("scrolling-table catalog entry");

        let tasks = build_task_rows_with_count(TASK_COUNT);
        let table = xgpui_luma_look_shadcn::Table::new("controls-doc-table-scroll")
            .look(&look)
            .items(tasks)
            .extended()
            .selected_index(1)
            .active_index(1)
            .row_label(|row: &Task| row.title.clone())
            .row_enabled(|row| row.enabled)
            .fill_height()
            .visible_row_height(36.0_f32)
            .grid_view([
                column_emphasis!("Task", width = 108 => |row: &Task| row.id.clone()),
                column!("Title" => |row: &Task| {
                    div()
                        .w_full()
                        .min_w(px(0.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(tag_pill(row.tag))
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .truncate()
                                .child(row.title.clone()),
                        )
                }),
                column!("Status", width = 132 => |row: &Task| status_cell(row.status)),
                email_column(),
                column!("", width = 44 => |_row: &Task| {
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(xgpui_luma::infra::icon::lucide_icon(
                            LucideIcon::EllipsisVertical,
                            gpui::hsla(0.0, 0.0, 0.5, 1.0),
                            16.0,
                        ))
                }),
            ])
            .with_row_template(|model, cells, _window, _cx| {
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .min_h(px(model.look.min_height))
                    .py(px(model.look.padding_y))
                    .bg(model.look.background)
                    .text_color(model.look.label_color)
                    .text_size(px(model.look.label_typography.size))
                    .line_height(px(model.look.label_typography.line_height))
                    .font_weight(model.look.label_typography.weight)
                    .child(cells)
            })
            .spawn(cx);
        table.update(cx, |table, cx| table.set_row_key(|row| row.id.clone(), cx)).expect("unique task IDs");
        table.update(cx, |table, cx| table.set_row_reordering(true, cx)).expect("keyed table");
        let sort_button = xgpui_luma_look_shadcn::Button::new("table-sort")
            .look(&look)
            .outline()
            .label("Reverse row order")
            .spawn(cx);
        let select_button = xgpui_luma_look_shadcn::Button::new("table-owner-selection")
            .look(&look)
            .outline()
            .label("Select tasks 1–3")
            .spawn(cx);
        let jump_button = xgpui_luma_look_shadcn::Button::new("table-jump-last")
            .look(&look)
            .outline()
            .label("Jump to last row")
            .spawn(cx);
        jump_button.clone().on_click(cx, {
            let table = table.downgrade();
            move |_, cx| {
                if let Some(table) = table.upgrade() {
                    table.update(cx, |table, cx| table.scroll_to_row(TASK_COUNT - 1, cx));
                }
            }
        });
        sort_button.clone().on_click(cx, {
            let table = table.downgrade();
            move |_, cx| {
                let Some(table) = table.upgrade() else {
                    return;
                };

                table.update(cx, |table, cx| {
                    let rows: Vec<_> = table.items().iter().rev().cloned().collect();
                    table.set_items(rows, cx).expect("unique task IDs");
                });
            }
        });
        select_button.clone().on_click(cx, {
            let table = table.downgrade();
            move |_, cx| {
                let Some(table) = table.upgrade() else {
                    return;
                };

                table
                    .update(cx, |table, cx| table.set_selected_keys(["T-0001", "T-0002", "T-0003"], cx))
                    .expect("enabled task IDs");
            }
        });

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-scrolling-table-event-log",
                "Select rows and scroll; TableEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|cx: &mut Context<ScrollingTableExpositionLeftPane>| {
            let pane = cx.entity().downgrade();
            let details = xgpui_luma_look_shadcn::Accordion::new("scrolling-table-details")
                .look(&look)
                .single()
                .collapsible(true)
                .motion_enabled(false)
                .item(AccordionItem::new(
                    "details",
                    AccordionTrigger::new("Events and code sample"),
                    AccordionContent::custom(move |_, cx| {
                        let Some(pane) = pane.upgrade() else {
                            return div().into_any_element();
                        };
                        let pane = pane.read(cx);
                        div()
                            .id("scrolling-table-details-content")
                            .max_h(px(240.0))
                            .overflow_y_scroll()
                            .child(render_control_exposition_card(
                                &pane.look,
                                pane.entry,
                                pane.event_stream.clone().into_any_element(),
                                None,
                                ControlExpositionLayout::BORDERLESS_NO_HEADING,
                            ))
                            .into_any_element()
                    }),
                ))
                .spawn(cx);
            ScrollingTableExpositionLeftPane {
                look: look.clone(),
                entry,
                table: table.clone(),
                sort_button,
                select_button,
                jump_button,
                event_stream: event_stream.clone(),
                details,
            }
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-scrolling-table-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &TABLE_INSPECTOR_SPEC,
            TableInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(table_motion_report([("Tasks", table.clone())]), cx);
        });

        table.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            let left_pane = left_pane.downgrade();
            move |_, event: &TableEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };
                left_pane.update(cx, |pane, cx| pane.handle_list_event(event, cx));
                if let Some(line) = format_table_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: { Vec::new() } }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for ScrollingTableControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-scrolling-table-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn format_table_event(event: &TableEvent) -> Option<String> {
    match event {
        TableEvent::RowDrag(event) => Some(format!("{event:?}")),
        TableEvent::SelectionChanged { selected_indices } => {
            Some(format!("TableEvent::SelectionChanged {{ selected_indices: {selected_indices:?} }}"))
        }
        TableEvent::SelectedKeysChanged { selected_keys } => Some(format!("Selected task IDs: {selected_keys:?}")),
        TableEvent::ActiveIndexChanged { active_index } => {
            Some(format!("TableEvent::ActiveIndexChanged {{ active_index: {active_index:?} }}"))
        }
        TableEvent::ScrollChanged { top_index } => {
            Some(format!("TableEvent::ScrollChanged {{ top_index: {top_index} }}"))
        }
        TableEvent::PageChanged { page } => Some(format!("TableEvent::PageChanged {{ page: {page} }}")),
        TableEvent::PageSizeChanged { page_size } => {
            Some(format!("TableEvent::PageSizeChanged {{ page_size: {page_size} }}"))
        }
        TableEvent::FocusChanged { focused } => Some(format!("TableEvent::FocusChanged {{ focused: {focused} }}")),
        TableEvent::RowHoverChanged { index, hovered } => {
            Some(format!("TableEvent::RowHoverChanged {{ index: {index}, hovered: {hovered} }}"))
        }
        TableEvent::EnabledChanged { enabled } => Some(format!("TableEvent::EnabledChanged {{ enabled: {enabled} }}")),
        _ => None,
    }
}
