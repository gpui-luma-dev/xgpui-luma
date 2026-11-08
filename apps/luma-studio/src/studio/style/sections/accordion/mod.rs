use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::accordion::{
    Accordion, AccordionContent, AccordionItem, AccordionItemRenderModel, AccordionRenderModel, AccordionSelectionMode,
    AccordionTemplate, AccordionTemplateHandlers, AccordionTrigger,
};
use xgpui_luma::infra::state::{CompositeItemState, ControlFocusState};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::theme::ControlSize;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn as shadcn;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::button_matrix::render_icon_button_state_header_cell;
use crate::studio::style::shared::preview_handlers::{
    input_noop_click, input_noop_hover, input_noop_mouse_down, input_noop_mouse_up,
};
use crate::studio::style::shared::samples::ButtonStateSample;
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

const ACCORDION_TABLE_STATE_COLUMN_WIDTH: f32 = 200.0;
const ACCORDION_TABLE_ROW_HEIGHT: f32 = 56.0;

#[derive(Clone, Copy)]
struct AccordionTemplateRow {
    id: &'static str,
    label: &'static str,
    expanded: bool,
}

const ACCORDION_TEMPLATE_ROWS: [AccordionTemplateRow; 2] = [
    AccordionTemplateRow { id: "collapsed", label: "Collapsed", expanded: false },
    AccordionTemplateRow { id: "expanded", label: "Expanded", expanded: true },
];

pub(crate) struct AccordionPreview {
    look: Arc<ShadcnLook>,
    sm: Accordion,
    md: Accordion,
    lg: Accordion,
}

impl AccordionPreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        Self {
            sm: sample_accordion(&look, "luma-studio-accordion-sm", shadcn::ShadcnSize::Sm, cx),
            md: sample_accordion(&look, "luma-studio-accordion-md", shadcn::ShadcnSize::Md, cx),
            lg: sample_accordion(&look, "luma-studio-accordion-lg", shadcn::ShadcnSize::Lg, cx),
            look,
        }
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        let template = look.accordion_template();
        for accordion in [&self.sm, &self.md, &self.lg] {
            let template = template.clone();
            accordion.update(cx, move |accordion, cx| accordion.set_template(template, cx));
        }
        cx.notify();
    }
}

impl Render for AccordionPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub(crate) fn render_accordion_template_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    preview: Entity<AccordionPreview>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));
    let preview = preview.read(cx);

    section_shell_with_width(
        960.0,
        "Accordion",
        "Collapsed vs expanded triggers across interaction states. Sizes tab: Sm/Md/Lg live samples.",
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
                _ => render_template_preview_body(&look, window, cx),
            }))
            .into_any_element(),
    )
}

fn render_template_preview_body(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let template = look.accordion_template();
    let samples = accordion_template_state_samples();

    VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(168.0)
            .state_column_width(ACCORDION_TABLE_STATE_COLUMN_WIDTH)
            .row_height(ACCORDION_TABLE_ROW_HEIGHT),
    )
    .row_group_label("EXPANSION")
    .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
    .rows(ACCORDION_TEMPLATE_ROWS.iter().map(|row| {
        VariantStateTableRow {
            label: SharedString::from(row.label),
            description: SharedString::from(""),
            cells: samples
                .iter()
                .map(|sample| render_accordion_state_cell(&template, row, sample, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn render_sizes_body(preview: &AccordionPreview, muted: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(px(16.0))
        .child(size_column("Sm", muted, preview.sm.clone(), 240.0))
        .child(size_column("Md", muted, preview.md.clone(), 240.0))
        .child(size_column("Lg", muted, preview.lg.clone(), 240.0))
        .into_any_element()
}

fn size_column(label: &'static str, muted: gpui::Hsla, accordion: Accordion, width: f32) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .w(px(width))
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted)
                .child(label),
        )
        .child(accordion)
        .into_any_element()
}

fn render_accordion_state_cell(
    template: &Arc<dyn AccordionTemplate>,
    row: &AccordionTemplateRow,
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let root_id = SharedString::from(format!("luma-studio-accordion-preview-{}-{}", row.id, sample.id));
    let item_id = SharedString::from(format!("{root_id}-item"));
    let trigger = AccordionTrigger::new("People").icon(LucideIcon::User);
    let content = AccordionContent::custom(move |_, _| div().text_sm().child("Description...").into_any_element());

    let enabled = !sample.state.disabled;
    let item_state = CompositeItemState {
        disabled: sample.state.disabled,
        hovered: enabled && sample.state.hovered,
        pressed: enabled && sample.state.pressed,
        selected: row.expanded,
        active: enabled && sample.state.focused,
        focus_visible: enabled && sample.state.focused,
    };
    let focus =
        ControlFocusState { focused: enabled && sample.state.focused, focus_visible: enabled && sample.state.focused };

    let model = AccordionRenderModel {
        id: &root_id,
        items: vec![AccordionItemRenderModel {
            id: &item_id,
            trigger: &trigger,
            content: &content,
            expanded: row.expanded,
            progress: if row.expanded { 1.0 } else { 0.0 },
            content_height_px: 0.0,
            enabled,
            state: item_state,
        }],
        selection_mode: AccordionSelectionMode::Single,
        collapsible: true,
        enabled: true,
        size: ControlSize::Md,
        item_dividers: true,
        content_padding_y: None,
        content_padding_top: None,
        content_padding_bottom: None,
        trigger_min_height: None,
        disclosure_icons: &xgpui_luma::infra::icon::DisclosureIcons::default(),
        trigger_padding_y: None,
        focus,
    };

    div()
        .w_full()
        .px(px(8.0))
        .py(px(6.0))
        .child(template.render(&model, noop_template_handlers(1), window, cx))
        .into_any_element()
}

fn noop_template_handlers(count: usize) -> AccordionTemplateHandlers {
    AccordionTemplateHandlers {
        drag: None,
        trigger_hovers: (0..count).map(|_| Box::new(input_noop_hover) as _).collect(),
        trigger_mouse_downs: (0..count).map(|_| Box::new(input_noop_mouse_down) as _).collect(),
        trigger_mouse_ups: (0..count).map(|_| Box::new(input_noop_mouse_up) as _).collect(),
        trigger_clicks: (0..count).map(|_| Box::new(input_noop_click) as _).collect(),
        content_height_reports: Vec::new(),
    }
}

fn accordion_template_state_samples() -> [ButtonStateSample; 5] {
    [
        ButtonStateSample { id: "default", header: "default", state: xgpui_luma::theme::InteractionState::default() },
        ButtonStateSample {
            id: "hover",
            header: "hover",
            state: xgpui_luma::theme::InteractionState {
                hovered: true,
                ..xgpui_luma::theme::InteractionState::default()
            },
        },
        ButtonStateSample {
            id: "focused",
            header: "focused",
            state: xgpui_luma::theme::InteractionState {
                focused: true,
                ..xgpui_luma::theme::InteractionState::default()
            },
        },
        ButtonStateSample {
            id: "pressed",
            header: "pressed",
            state: xgpui_luma::theme::InteractionState {
                hovered: true,
                pressed: true,
                ..xgpui_luma::theme::InteractionState::default()
            },
        },
        ButtonStateSample {
            id: "disabled",
            header: "disabled",
            state: xgpui_luma::theme::InteractionState {
                disabled: true,
                ..xgpui_luma::theme::InteractionState::default()
            },
        },
    ]
}

fn sample_accordion<M: 'static>(
    look: &Arc<ShadcnLook>,
    id: impl Into<SharedString>,
    size: shadcn::ShadcnSize,
    cx: &mut Context<M>,
) -> Accordion {
    let id = id.into();
    shadcn::Accordion::new(id.clone())
        .look(look.as_ref())
        .single()
        .size(size)
        .item(demo_item(format!("{id}-account"), "Account", LucideIcon::User, "Profile and security.").expanded(true))
        .item(demo_item(format!("{id}-billing"), "Billing", LucideIcon::CreditCard, "Plan and invoices."))
        .item(demo_item(format!("{id}-team"), "Team", LucideIcon::Users, "Members and roles."))
        .spawn(cx)
}

fn demo_item(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    icon: LucideIcon,
    body: &'static str,
) -> AccordionItem {
    AccordionItem::new(
        id,
        AccordionTrigger::new(label).icon(icon),
        AccordionContent::custom(move |_, _| div().text_sm().child(body).into_any_element()),
    )
}
