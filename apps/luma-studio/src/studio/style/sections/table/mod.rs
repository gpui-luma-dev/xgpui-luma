use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::table::Table;
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma_look_shadcn::{ShadcnLook};
use xgpui_luma_look_shadcn as shadcn;

use crate::studio::style::shared::shell::section_shell_with_width;

#[derive(Clone)]
struct PreviewRow {
    title: SharedString,
}

pub(crate) struct TablePreview {
    look: Arc<ShadcnLook>,
    sm: Table<PreviewRow>,
    md: Table<PreviewRow>,
    lg: Table<PreviewRow>,
    template: Table<PreviewRow>,
}

impl TablePreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        Self {
            sm: mailbox_table(&look, "luma-studio-table-sm", shadcn::ShadcnSize::Sm, cx),
            md: mailbox_table(&look, "luma-studio-table-md", shadcn::ShadcnSize::Md, cx),
            lg: mailbox_table(&look, "luma-studio-table-lg", shadcn::ShadcnSize::Lg, cx),
            template: mailbox_table(&look, "luma-studio-table-template", shadcn::ShadcnSize::Md, cx),
            look,
        }
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        let template = look.table_template();
        for table in [&self.sm, &self.md, &self.lg, &self.template] {
            let template = template.clone();
            table.update(cx, move |table, cx| table.set_template(template, cx));
        }
        cx.notify();
    }
}

impl Render for TablePreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub(crate) fn render_table_template_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    preview: Entity<TablePreview>,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));
    let preview = preview.read(cx);

    section_shell_with_width(
        960.0,
        "Table",
        "Scrollable list rows. Sizes tab: Sm/Md/Lg label typography.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .w_full()
            .flex()
            .flex_col()
            .child(div().w_full().flex().justify_start().child(preview_tabs))
            .child(div().w_full().h(px(1.0)).bg(chrome.border))
            .child(div().w_full().flex().justify_center().mt(px(16.0)).child(match active_tab.as_ref() {
                "sizes" => render_sizes_body(preview, chrome.muted_text),
                _ => render_template_body(preview, chrome.muted_text),
            }))
            .into_any_element(),
    )
}

fn render_template_body(preview: &TablePreview, muted: gpui::Hsla) -> AnyElement {
    size_column("Template", muted, preview.template.clone(), 220.0)
}

fn render_sizes_body(preview: &TablePreview, muted: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(px(16.0))
        .child(size_column("Sm", muted, preview.sm.clone(), 200.0))
        .child(size_column("Md", muted, preview.md.clone(), 200.0))
        .child(size_column("Lg", muted, preview.lg.clone(), 200.0))
        .into_any_element()
}

fn size_column(label: &'static str, muted: gpui::Hsla, table: Table<PreviewRow>, width: f32) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .w(px(width))
        .h(px(140.0))
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted)
                .child(label),
        )
        .child(div().flex_1().w_full().child(table))
        .into_any_element()
}

fn mailbox_table<M: 'static>(
    look: &Arc<ShadcnLook>,
    id: impl Into<SharedString>,
    size: shadcn::ShadcnSize,
    cx: &mut Context<M>,
) -> Table<PreviewRow> {
    shadcn::Table::new(id)
        .look(look.as_ref())
        .items([
            PreviewRow { title: SharedString::from("Inbox") },
            PreviewRow { title: SharedString::from("Drafts") },
            PreviewRow { title: SharedString::from("Archive") },
        ])
        .size(size)
        .visible_rows(3)
        .selected_index(0)
        .row_label(|row| row.title.clone())
        .spawn(cx)
}
