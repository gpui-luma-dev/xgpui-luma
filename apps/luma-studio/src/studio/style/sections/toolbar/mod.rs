use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::selector::SelectorItem;
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::controls::toolbar::{Toolbar, ToolbarVariant};
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnToolbarItemExt};
use xgpui_luma_look_shadcn as shadcn;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

const TOOLBAR_VARIANT_COLUMN_WIDTH: f32 = 120.0;
const TOOLBAR_STATE_COLUMN_WIDTH: f32 = 420.0;
const TOOLBAR_TABLE_HEADER_HEIGHT: f32 = 28.0;
const TOOLBAR_TABLE_ROW_HEIGHT: f32 = 56.0;

pub(crate) struct ToolbarPreview {
    look: Arc<ShadcnLook>,
    outline_enabled: Toolbar,
    outline_disabled: Toolbar,
    ghost_enabled: Toolbar,
    ghost_disabled: Toolbar,
    size_sm: Toolbar,
    size_md: Toolbar,
    size_lg: Toolbar,
}

impl ToolbarPreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        Self {
            outline_enabled: spawn_stripped_toolbar(
                &look,
                "outline-enabled",
                ToolbarVariant::Outline,
                shadcn::ShadcnSize::Md,
                true,
                cx,
            ),
            outline_disabled: spawn_stripped_toolbar(
                &look,
                "outline-disabled",
                ToolbarVariant::Outline,
                shadcn::ShadcnSize::Md,
                false,
                cx,
            ),
            ghost_enabled: spawn_stripped_toolbar(
                &look,
                "ghost-enabled",
                ToolbarVariant::Ghost,
                shadcn::ShadcnSize::Md,
                true,
                cx,
            ),
            ghost_disabled: spawn_stripped_toolbar(
                &look,
                "ghost-disabled",
                ToolbarVariant::Ghost,
                shadcn::ShadcnSize::Md,
                false,
                cx,
            ),
            size_sm: spawn_stripped_toolbar(
                &look,
                "size-sm",
                ToolbarVariant::Outline,
                shadcn::ShadcnSize::Sm,
                true,
                cx,
            ),
            size_md: spawn_stripped_toolbar(
                &look,
                "size-md",
                ToolbarVariant::Outline,
                shadcn::ShadcnSize::Md,
                true,
                cx,
            ),
            size_lg: spawn_stripped_toolbar(
                &look,
                "size-lg",
                ToolbarVariant::Outline,
                shadcn::ShadcnSize::Lg,
                true,
                cx,
            ),
            look,
        }
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        let template = look.toolbar_template();
        for toolbar in self.all_toolbars() {
            let template = template.clone();
            toolbar.update(cx, move |toolbar, cx| {
                toolbar.set_template(template, cx);
                toolbar.notify_items(cx);
            });
        }
        cx.notify();
    }

    fn all_toolbars(&self) -> [Toolbar; 7] {
        [
            self.outline_enabled.clone(),
            self.outline_disabled.clone(),
            self.ghost_enabled.clone(),
            self.ghost_disabled.clone(),
            self.size_sm.clone(),
            self.size_md.clone(),
            self.size_lg.clone(),
        ]
    }
}

impl Render for ToolbarPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub(crate) fn render_toolbar_template_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    preview: Entity<ToolbarPreview>,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));
    let preview = preview.read(cx);

    section_shell_with_width(
        960.0,
        "Toolbar",
        "Outline and ghost shell variants with a stripped editing-toolbar sample from Controls exposition.",
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
                _ => render_template_body(preview, &chrome),
            }))
            .into_any_element(),
    )
}

fn render_template_body(preview: &ToolbarPreview, chrome: &xgpui_luma::theme::LumaChrome) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .child(
            VariantStateTable::new(
                VariantStateTableStyle::from_chrome(chrome)
                    .variant_column_width(TOOLBAR_VARIANT_COLUMN_WIDTH)
                    .state_column_width(TOOLBAR_STATE_COLUMN_WIDTH)
                    .header_height(TOOLBAR_TABLE_HEADER_HEIGHT)
                    .header_corner_padding_bottom(0.0)
                    .row_height(TOOLBAR_TABLE_ROW_HEIGHT),
            )
            .column_headers(
                ["Enabled", "Disabled"].into_iter().map(|label| table_header_cell(label, chrome.muted_text)),
            )
            .rows([
                toolbar_variant_row("Outline", preview.outline_enabled.clone(), preview.outline_disabled.clone()),
                toolbar_variant_row("Ghost", preview.ghost_enabled.clone(), preview.ghost_disabled.clone()),
            ])
            .build(),
        )
        .into_any_element()
}

fn toolbar_variant_row(label: &'static str, enabled: Toolbar, disabled: Toolbar) -> VariantStateTableRow {
    VariantStateTableRow {
        label: SharedString::from(label),
        description: SharedString::from(""),
        cells: vec![toolbar_preview_cell(enabled), toolbar_preview_cell(disabled)],
    }
}

fn toolbar_preview_cell(toolbar: Toolbar) -> AnyElement {
    div().w_full().flex().items_center().justify_center().child(toolbar).into_any_element()
}

fn render_sizes_body(preview: &ToolbarPreview, muted: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .justify_center()
        .gap(px(16.0))
        .child(size_column("Sm", muted, preview.size_sm.clone()))
        .child(size_column("Md", muted, preview.size_md.clone()))
        .child(size_column("Lg", muted, preview.size_lg.clone()))
        .into_any_element()
}

fn size_column(label: &'static str, muted: gpui::Hsla, toolbar: Toolbar) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted)
                .child(label),
        )
        .child(toolbar)
        .into_any_element()
}

fn table_header_cell(label: &'static str, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .line_height(px(15.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted_text)
        .child(label)
        .into_any_element()
}

fn spawn_stripped_toolbar(
    look: &Arc<ShadcnLook>,
    id_suffix: &str,
    variant: ToolbarVariant,
    size: shadcn::ShadcnSize,
    enabled: bool,
    cx: &mut Context<ToolbarPreview>,
) -> Toolbar {
    let id = format!("luma-studio-toolbar-{id_suffix}");
    shadcn::Toolbar::new(id)
        .look(look.as_ref())
        .size(size)
        .enabled(enabled)
        .variant(variant)
        .item(look.toolbar_selector(
            format!("{id_suffix}-style"),
            "Paragraph",
            [
                SelectorItem::new("paragraph").label("Paragraph").icon(LucideIcon::Pilcrow),
                SelectorItem::new("heading").label("Heading").icon(LucideIcon::TypeIcon),
            ],
            "paragraph",
            cx,
        ))
        .separator(format!("{id_suffix}-format-separator"))
        .item(look.toolbar_toggle(format!("{id_suffix}-bold"), LucideIcon::Bold, cx))
        .item(look.toolbar_toggle(format!("{id_suffix}-italic"), LucideIcon::Italic, cx))
        .item(look.toolbar_button(format!("{id_suffix}-link"), LucideIcon::Link, cx))
        .spawn(cx)
}
