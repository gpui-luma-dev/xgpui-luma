use xgpui_luma::controls::table::TableEventExt;
use xgpui_luma::controls::checkbox::CheckboxEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, SharedString, Window, div, prelude::*, px};
use lucide_svg_static::Icon as LucideIcon;
use xgpui_luma::column_numeric;
use xgpui_luma::column_text;
use xgpui_luma::controls::checkbox::{Checkbox, CheckboxEvent};
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::table::{TableColumn, TableColumnCellTemplate, TableControl, TableEvent, TableRowRenderModel};
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::infra::drag_drop::DragDropElementExt;
use xgpui_luma::controls::popup_menu::PopupMenu;
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma::{hstack, vstack};

use super::common::titled_card;

const PAYMENTS_CARD_WIDTH: f32 = 720.0;
const PAYMENTS_PAGE_SIZE: usize = 6;
const PAYMENTS_VISIBLE_ROWS: usize = 6;

/// Data-table row height: fits text rows without reserving full command height.
fn payments_row_height(size: shadcn::ShadcnSize) -> f32 {
    match size {
        shadcn::ShadcnSize::Sm => 28.0,
        shadcn::ShadcnSize::Md => 32.0,
        shadcn::ShadcnSize::Lg => 36.0,
    }
}

#[derive(Clone)]
struct PaymentRow {
    /// Immutable sample record identity, independent of its current row position.
    id: usize,
    status: &'static str,
    email: SharedString,
    amount: SharedString,
}

pub struct PaymentsPanel {
    look: Arc<ShadcnLook>,
    table: Entity<TableControl<PaymentRow>>,
    row_checkboxes: Arc<Vec<Checkbox>>,
    prev_button: Entity<Button>,
    next_button: Entity<Button>,
    selected_count: usize,
}

impl PaymentsPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>, size: shadcn::ShadcnSize) -> Self {
        let payments = sample_payments();
        let row_checkboxes = Arc::new(spawn_row_checkboxes(look.clone(), payments.len(), cx));
        let row_menus = Arc::new(spawn_row_menus(look.clone(), payments.len(), cx));
        let list_size = shadcn::ShadcnSize::Sm;
        let table = shadcn::Table::new("studio-payments")
            .look(look.as_ref())
            .items(payments)
            .multiple()
            .select_on_row_click(false)
            .size(list_size)
            .visible_rows(PAYMENTS_VISIBLE_ROWS)
            .visible_row_height(payments_row_height(list_size))
            .paged(PAYMENTS_PAGE_SIZE)
            .row_label(|row| row.email.clone())
            .grid_view(payment_columns(Arc::clone(&row_checkboxes), Arc::clone(&row_menus)))
            .spawn(cx);

        table.update(cx, |table, cx| {
            table.set_row_key(|row| row.id.to_string(), cx).expect("unique payment IDs");
            table.set_row_reordering(true, cx).expect("keyed payments table");
        });

        table.clone().on_event(cx, |panel, event, cx| {
            match event {
                TableEvent::SelectionChanged { .. } => {
                    panel.selected_count = panel.table.read(cx).selected_indices().len();
                    panel.sync_checkboxes_from_list(cx);
                }
                TableEvent::PageChanged { .. } => {}
                _ => return,
            }
            cx.notify();
        });

        for (id, checkbox) in row_checkboxes.iter().enumerate() {
            checkbox.clone().on_event(cx, move |panel, event, cx| {
                if matches!(event, CheckboxEvent::Change { .. }) {
                    panel.toggle_row_selection(id, cx);
                }
            });
        }

        let prev_button = shadcn::Button::new("payments-prev")
            .look(look.as_ref())
            .outline()
            .label("Previous")
            .on_click(cx, |panel, cx| {
                panel.table.update(cx, |list, cx| list.prev_page(cx));
            })
            .size(size)
            .spawn(cx);
        let next_button = shadcn::Button::new("payments-next")
            .look(look.as_ref())
            .outline()
            .label("Next")
            .on_click(cx, |panel, cx| {
                panel.table.update(cx, |list, cx| list.next_page(cx));
            })
            .size(size)
            .spawn(cx);

        Self { prev_button, next_button, table, row_checkboxes, look, selected_count: 0 }
    }

    fn toggle_row_selection(&mut self, id: usize, cx: &mut Context<Self>) {
        self.table.update(cx, |list, cx| {
            if let Some(index) = list.items().iter().position(|row| row.id == id) {
                list.toggle_selected_index(index, cx);
            }
        });
    }

    fn sync_checkboxes_from_list(&self, cx: &mut Context<Self>) {
        let selected: Vec<_> = self.table.read(cx).selected_items().iter().map(|row| row.id).collect();
        for (id, checkbox) in self.row_checkboxes.iter().enumerate() {
            let checked = selected.contains(&id);
            checkbox.update(cx, |button, cx| {
                if *button.data() != checked {
                    button.set_data(checked, cx);
                }
            });
        }
    }
}

impl Render for PaymentsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let total_rows = self.table.read(cx).items().len();
        let at_first = self.table.read(cx).current_page() == 0;
        let at_last = self.table.read(cx).current_page() + 1 >= self.table.read(cx).page_count().max(1);
        let selected_count = self.selected_count;
        let table = self.table.clone();
        let prev_button = self.prev_button.clone();
        let next_button = self.next_button.clone();

        self.prev_button.update(cx, |button, cx| button.set_enabled(!at_first, cx));
        self.next_button.update(cx, |button, cx| button.set_enabled(!at_last, cx));

        titled_card(
            "luma-studio-payments-card",
            &self.look,
            PAYMENTS_CARD_WIDTH,
            "Payments",
            "Select payments with the checkboxes, then drag a row to reorder.",
            move |_, _| {
                vstack! {
                    gap=12;
                    div()
                        .w_full()
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(chrome.border)
                        .overflow_hidden()
                        .child(table.clone()),
                    hstack! {
                        justify=between align=center gap=12;
                        format!("{} of {total_rows} row(s) selected.", selected_count),
                        hstack! {
                            gap=8 align=center;
                            prev_button.clone(),
                            next_button.clone(),
                        },
                    }
                    .w_full()
                    .text_xs()
                    .line_height(px(14.0))
                    .text_color(chrome.muted_text),
                }
                .w_full()
                .overflow_hidden()
                .into_any_element()
            },
            _window,
            cx,
        )
    }
}

fn spawn_row_checkboxes(look: Arc<ShadcnLook>, row_count: usize, cx: &mut Context<PaymentsPanel>) -> Vec<Checkbox> {
    (0..row_count)
        .map(|index| {
            shadcn::Checkbox::new(format!("studio-payments-row-{index}"))
                .look(look.as_ref())
                .primary()
                .with_data(false)
                .size(shadcn::ShadcnSize::Sm)
                .indicator_only()
                .tab_stop(false)
                .spawn(cx)
        })
        .collect()
}

fn spawn_row_menus(look: Arc<ShadcnLook>, row_count: usize, cx: &mut Context<PaymentsPanel>) -> Vec<Entity<PopupMenu>> {
    (0..row_count)
        .map(|index| {
            shadcn::PopupMenu::new(format!("studio-payments-row-menu-{index}"))
                .look(look.as_ref())
                .ghost()
                .size(shadcn::ShadcnSize::Sm)
                .icon(LucideIcon::EllipsisVertical)
                .items(payment_action_items())
                .spawn(cx)
        })
        .collect()
}

fn payment_action_items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("view").label("View"),
        MenuItem::new("copy").label("Copy"),
        MenuItem::new("delete").label("Delete"),
    ]
}

fn payment_columns(
    row_checkboxes: Arc<Vec<Checkbox>>,
    row_menus: Arc<Vec<Entity<PopupMenu>>>,
) -> Vec<TableColumn<PaymentRow>> {
    vec![
        TableColumn::fixed_control("", 48.0, selection_checkbox_column(row_checkboxes)),
        TableColumn::fixed_control("Status", 72.0, |row: &PaymentRow| status_icon_cell(row.status)),
        column_text!("Email" => |row: &PaymentRow| row.email.clone()),
        column_numeric!("Amount", width = 96 => |row: &PaymentRow| row.amount.clone()),
        TableColumn::fixed_control("", 40.0, row_menu_column(row_menus)),
    ]
}

fn status_icon_cell(status: &'static str) -> impl IntoElement {
    let (icon, color) = match status {
        "Success" => (LucideIcon::CircleCheck, gpui::hsla(0.35, 0.7, 0.45, 1.0)),
        "Failed" => (LucideIcon::CircleX, gpui::hsla(0.0, 0.7, 0.55, 1.0)),
        "Processing" => (LucideIcon::LoaderCircle, gpui::hsla(0.58, 0.75, 0.5, 1.0)),
        _ => (LucideIcon::Circle, gpui::hsla(0.0, 0.0, 0.55, 1.0)),
    };

    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .child(xgpui_luma::infra::icon::lucide_icon(icon, color, 16.0))
}

fn selection_checkbox_column(row_checkboxes: Arc<Vec<Checkbox>>) -> TableColumnCellTemplate<PaymentRow> {
    Arc::new(move |model: &TableRowRenderModel<'_, PaymentRow>, _, _| {
        let id = model.row.id;
        div()
            .id("payment-checkbox-boundary")
            .debug_selector(move || format!("payments-checkbox-{id}"))
            .drag_boundary()
            .child(row_checkboxes[id].clone())
            .into_any_element()
    })
}

fn row_menu_column(row_menus: Arc<Vec<Entity<PopupMenu>>>) -> TableColumnCellTemplate<PaymentRow> {
    Arc::new(move |model: &TableRowRenderModel<'_, PaymentRow>, _, _| {
        let id = model.row.id;
        div()
            .id("payment-menu-boundary")
            .debug_selector(move || format!("payments-menu-{id}"))
            .drag_boundary()
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .child(row_menus[id].clone())
            .into_any_element()
    })
}

fn sample_payments() -> Vec<PaymentRow> {
    vec![
        PaymentRow { id: 0, status: "Success", email: "ken99@example.com".into(), amount: "$316.00".into() },
        PaymentRow { id: 1, status: "Success", email: "abe45@example.com".into(), amount: "$242.00".into() },
        PaymentRow { id: 2, status: "Processing", email: "Mason@example.com".into(), amount: "$837.00".into() },
        PaymentRow { id: 3, status: "Failed", email: "so456@example.com".into(), amount: "$721.00".into() },
        PaymentRow { id: 4, status: "Pending", email: "lee39@example.com".into(), amount: "$150.00".into() },
        PaymentRow { id: 5, status: "Success", email: "ashain@example.com".into(), amount: "$410.00".into() },
        PaymentRow { id: 6, status: "Success", email: "noah12@example.com".into(), amount: "$529.00".into() },
        PaymentRow { id: 7, status: "Processing", email: "ava88@example.com".into(), amount: "$198.00".into() },
        PaymentRow { id: 8, status: "Pending", email: "mia27@example.com".into(), amount: "$364.00".into() },
        PaymentRow { id: 9, status: "Failed", email: "liam63@example.com".into(), amount: "$912.00".into() },
        PaymentRow { id: 10, status: "Success", email: "emma14@example.com".into(), amount: "$275.00".into() },
        PaymentRow { id: 11, status: "Processing", email: "oliver52@example.com".into(), amount: "$648.00".into() },
        PaymentRow { id: 12, status: "Success", email: "sophia31@example.com".into(), amount: "$483.00".into() },
        PaymentRow { id: 13, status: "Pending", email: "james76@example.com".into(), amount: "$207.00".into() },
        PaymentRow { id: 14, status: "Failed", email: "amelia08@example.com".into(), amount: "$756.00".into() },
        PaymentRow { id: 15, status: "Success", email: "henry45@example.com".into(), amount: "$331.00".into() },
        PaymentRow { id: 16, status: "Processing", email: "isla19@example.com".into(), amount: "$584.00".into() },
        PaymentRow { id: 17, status: "Success", email: "charlie67@example.com".into(), amount: "$429.00".into() },
    ]
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{MouseButton, TestAppContext, point};

    #[test]
    fn payment_checkboxes_follow_records_after_group_reorder() {
        let mut app = TestAppContext::single();
        let (panel, cx) = app
            .add_window_view(|_, cx| PaymentsPanel::new(cx, Arc::new(ShadcnLook::built_in()), shadcn::ShadcnSize::Sm));
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        for selector in ["payments-checkbox-1", "payments-checkbox-2"] {
            let checkbox = cx.debug_bounds(selector).unwrap();
            cx.simulate_click(checkbox.center(), Default::default());
        }
        let source = cx.debug_bounds("payments-checkbox-1").unwrap();
        let target = cx.debug_bounds("payments-checkbox-0").unwrap();
        let source = point(source.right() + px(120.0), source.center().y);
        let target = point(source.x, target.top() + px(1.0));
        cx.simulate_mouse_down(source, MouseButton::Left, Default::default());
        cx.simulate_mouse_move(source + point(px(20.0), px(0.0)), MouseButton::Left, Default::default());
        cx.update(|_, app| assert!(app.has_active_drag()));
        cx.simulate_mouse_move(target, MouseButton::Left, Default::default());
        cx.simulate_mouse_up(target, MouseButton::Left, Default::default());
        cx.update(|_, app| {
            let panel = panel.read(app);
            let table = panel.table.read(app);
            assert_eq!(table.items().iter().take(3).map(|row| row.id).collect::<Vec<_>>(), [1, 2, 0]);
            assert_eq!(table.selected_keys(), vec![SharedString::from("1"), "2".into()]);
            assert_eq!(panel.selected_count, 2);
            for (id, checkbox) in panel.row_checkboxes.iter().enumerate() {
                assert_eq!(*checkbox.read(app).data(), id == 1 || id == 2);
            }
        });
        // The moved checkbox must toggle its record, not its former row position.
        let moved = cx.debug_bounds("payments-checkbox-1").unwrap();
        cx.simulate_click(moved.center(), Default::default());
        cx.update(|_, app| {
            assert_eq!(panel.read(app).table.read(app).selected_keys(), vec![SharedString::from("2")]);
            assert_eq!(panel.read(app).selected_count, 1);
        });
        // Embedded checkbox and menu gestures must not start a row drag.
        for selector in ["payments-checkbox-1", "payments-menu-1"] {
            let control = cx.debug_bounds(selector).unwrap().center();
            cx.simulate_mouse_down(control, MouseButton::Left, Default::default());
            cx.simulate_mouse_move(control + point(px(20.0), px(0.0)), MouseButton::Left, Default::default());
            cx.update(|_, app| assert!(!app.has_active_drag()));
            cx.simulate_mouse_up(control + point(px(20.0), px(0.0)), MouseButton::Left, Default::default());
        }
    }
}
