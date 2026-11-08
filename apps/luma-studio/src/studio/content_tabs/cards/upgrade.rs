use std::cell::Cell;
use std::sync::Arc;

use gpui::{Context, Entity, FontWeight, Render, SharedString, Window, div, prelude::*, px, transparent_black};
use xgpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use xgpui_luma::controls::checkbox::{Checkbox, CheckboxEvent};
use xgpui_luma::controls::button::{Button, ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::control_group::{ControlGroupItemElementTemplate, ControlGroupItemRenderModel};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::radio_button::{RadioButtonData, ThemedRadioButtonTemplate};
use xgpui_luma::controls::radio_group::{RadioGroup, RadioGroupItemLike, horizontal as horizontal_radio_group};
use xgpui_luma::controls::textarea::TextArea;
use xgpui_luma::controls::textfield::TextField;
use xgpui_luma::theme::{InteractionLayer, LumaTextStyle};
use xgpui_luma_look_shadcn::ShadcnToken;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma::{declare_form, form_field, hstack, vstack};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize};

use super::common::titled_card;

const PLAN_CARD_RADIUS: f32 = 8.0;
const PLAN_CARD_PADDING: f32 = 12.0;
const PLAN_CARD_GAP: f32 = 8.0;
const PLAN_DISABLED_OPACITY: f32 = 0.56;
const PLAN_SELECTED_BORDER_WIDTH: f32 = 1.0;
const PLAN_FOCUS_BORDER_WIDTH: f32 = 2.0;

#[derive(Clone, Debug)]
struct PlanOptionItem {
    id: SharedString,
    title: SharedString,
    description: SharedString,
}

impl PlanOptionItem {
    fn new(id: impl Into<SharedString>, title: impl Into<SharedString>, description: impl Into<SharedString>) -> Self {
        Self { id: id.into(), title: title.into(), description: description.into() }
    }
}

impl RadioGroupItemLike for PlanOptionItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.title
    }
}

struct PlanOptionTemplateSpec {
    title_style: LumaTextStyle,
    caption_style: LumaTextStyle,
}

declare_form! {
    pub struct UpgradePanel {
        controls: {
            name_field: TextField = shadcn::TextField::new("upgrade-name").look(look.as_ref()).placeholder("Name").full_width(true),
            email_field: TextField = shadcn::TextField::new("upgrade-email").look(look.as_ref()).placeholder("Email").full_width(true),
            card_field: TextField = shadcn::TextField::new("upgrade-card").look(look.as_ref()).placeholder("Card Number").full_width(true),
            // full_width fills the fixed host (72/64px below); it does not grow with typed content.
            expiry_field: TextField = shadcn::TextField::new("upgrade-expiry").look(look.as_ref()).placeholder("MM/YY").full_width(true),
            cvc_field: TextField = shadcn::TextField::new("upgrade-cvc").look(look.as_ref()).placeholder("CVC").full_width(true),
            plan_group: RadioGroup<PlanOptionItem> = horizontal_radio_group("upgrade-plan")
                .item_element_template(plan_option_item_element_template(look.clone()))
                .with_item_layout(|items, _, _, _| {
                    div()
                        .flex()
                        .w_full()
                        .gap(px(PLAN_CARD_GAP))
                        .children(items.into_elements())
                })
                .items(plan_items())
                .selected("starter"),
            notes_area: Entity<TextArea> = shadcn::TextArea::new("upgrade-notes").look(look.as_ref())

                .placeholder("Notes")
                .full_width(true)
                .rows(3),
            terms_checkbox: Checkbox = shadcn::Checkbox::new("upgrade-terms").look(look.as_ref()).primary()
                .with_data(true)
                .content(|_, _| div().child("I agree to the terms and conditions").into_any_element())
                => CheckboxEvent |this, event, cx| {
                    if let CheckboxEvent::Change { checked } = event {
                        this.terms_accepted = *checked;
                        cx.notify();
                    }
                },
            email_checkbox: Checkbox = shadcn::Checkbox::new("upgrade-email-opt").look(look.as_ref()).primary()
                .with_data(false)
                .content(|_, _| div().child("Allow us to send you emails").into_any_element())
                => CheckboxEvent |this, event, cx| {
                    if let CheckboxEvent::Change { checked } = event {
                        this.email_opt_in = *checked;
                        cx.notify();
                    }
                },
            cancel_button: Entity<Button> = shadcn::Button::new("upgrade-cancel").look(look.as_ref()).outline().label("Cancel").size(size),
            upgrade_button: Entity<Button> = shadcn::Button::new("upgrade-submit").look(look.as_ref()).primary().label("Upgrade Plan").size(size),
        },
        args: {
            look: Arc<ShadcnLook>,
            size: ShadcnSize,
        },
        fields: {
            terms_accepted: bool = true,
            email_opt_in: bool = false,
        }
    }
}

impl Render for UpgradePanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let label_style = self.look.typography_scale(ShadcnTextSize::Sm);
        let caption_style = self.look.typography_scale(ShadcnTextSize::Xs);
        let name_field = self.name_field.clone();
        let email_field = self.email_field.clone();
        let card_field = self.card_field.clone();
        let expiry_field = self.expiry_field.clone();
        let cvc_field = self.cvc_field.clone();
        let plan_group = self.plan_group.clone();
        let notes_area = self.notes_area.clone();
        let terms_checkbox = self.terms_checkbox.clone();
        let email_checkbox = self.email_checkbox.clone();
        let cancel_button = self.cancel_button.clone();
        let upgrade_button = self.upgrade_button.clone();

        titled_card(
            "luma-studio-upgrade-card",
            &self.look,
            380.0,
            "Upgrade your subscription",
            "You are currently on the free plan. Upgrade to unlock all features.",
            move |_, _| {
                vstack! {
                    gap=12;
                    hstack! {
                        gap=10;
                        form_field!("Name", chrome; name_field.clone()).flex_1(),
                        form_field!("Email", chrome; email_field.clone()).flex_1(),
                    },
                    form_field!("Card Number", chrome;
                        hstack! {
                            gap=8;
                            div().flex_1().child(card_field.clone()),
                            div().w(px(80.0)).child(expiry_field.clone()),
                            div().w(px(64.0)).child(cvc_field.clone()),
                        }
                    ),
                    vstack! {
                        gap=4;
                        div()
                            .typography_style(label_style)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child("Plan"),
                        div()
                            .typography_style(caption_style)
                            .text_color(chrome.muted_text)
                            .child("Select the plan that best fits your needs."),
                        plan_group.clone(),
                    },
                    form_field!("Notes", chrome; notes_area.clone()),
                    vstack! {
                        gap=4;
                        terms_checkbox.clone(),
                        email_checkbox.clone(),
                    },
                    hstack! {
                        gap=8 align=center justify=end;
                        cancel_button.clone(),
                        upgrade_button.clone(),
                    },
                }
                .into_any_element()
            },
            window,
            _cx,
        )
    }
}

fn plan_items() -> [PlanOptionItem; 2] {
    [
        PlanOptionItem::new("starter", "Starter Plan", "Perfect for small businesses."),
        PlanOptionItem::new("pro", "Pro Plan", "More features and storage."),
    ]
}

fn plan_radio_indicator_template(look: Arc<ShadcnLook>) -> Arc<dyn ButtonTemplate<RadioButtonData>> {
    Arc::new(
        ThemedRadioButtonTemplate::new(look.radio_button_theme())
            .with_modifier(|element, _| element.min_h(px(0.0)).px(px(0.0)).py(px(0.0))),
    )
}

fn plan_option_item_element_template(look: Arc<ShadcnLook>) -> ControlGroupItemElementTemplate<PlanOptionItem> {
    let radio_template = plan_radio_indicator_template(look.clone());
    let spec = Arc::new(PlanOptionTemplateSpec {
        title_style: look.typography_scale(ShadcnTextSize::Sm),
        caption_style: look.typography_scale(ShadcnTextSize::Xs),
    });

    Arc::new(move |item: &ControlGroupItemRenderModel<'_, PlanOptionItem>, _, window, cx| {
        let chrome = look.chrome();
        let border = chrome.border;
        let focus_ring = look.token_color("ring").unwrap_or(chrome.border);
        let body_text = chrome.body_text;
        let muted_text = chrome.muted_text;
        let card_surface = look.color(ShadcnToken::Card);
        let selected_surface = look.adjust_surface_color(card_surface, InteractionLayer::Pressed);
        let interaction = item.state.interaction_state();
        let render_model = ButtonRenderModel {
            id: format!("{}-{}", item.group_id, item.item.id()).into(),
            data: RadioButtonData::new(item.selected),
            content: Arc::new(|_, _| div().into_any_element()),
            role: ButtonFamilyRole::Icon,
            size: ButtonSize::Sm,
            state: interaction,
            round: false,
            radius_override: Cell::new(None),
            elevation: false,
            compact: true,
            look: None,
            ..Default::default()
        };

        let indicator = radio_template.render(&render_model, window, cx);
        let title = item.item.title.clone();
        let description = item.item.description.clone();
        let title_style = spec.title_style;
        let caption_style = spec.caption_style;

        let body = hstack! {
            gap=10 align=start;
            div().flex_none().child(indicator),
            vstack! {
                gap=2;
                div()
                    .w_full()
                    .typography_style(title_style)
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(body_text)
                    .child(title),
                div()
                    .w_full()
                    .typography_style(caption_style)
                    .text_color(muted_text)
                    .line_clamp(3)
                    .child(description),
            }
            .flex_1()
            .min_w_0()
            .overflow_hidden(),
        }
        .w_full()
        .overflow_hidden();

        // Always reserve the focus-ring width so focusing a plan does not reflow the row.
        let focus_border_color = if item.state.active {
            focus_ring
        } else {
            transparent_black()
        };
        let card = div()
            .id(format!("{}-card", render_model.id))
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .rounded(px(PLAN_CARD_RADIUS + PLAN_FOCUS_BORDER_WIDTH))
            .border(px(PLAN_FOCUS_BORDER_WIDTH))
            .border_color(focus_border_color)
            .child(
                div()
                    .w_full()
                    .overflow_hidden()
                    .rounded(px(PLAN_CARD_RADIUS))
                    .border(px(PLAN_SELECTED_BORDER_WIDTH))
                    .border_color(border)
                    .when(item.selected, |inner| inner.bg(selected_surface))
                    .p(px(PLAN_CARD_PADDING))
                    .child(body),
            );

        if item.enabled {
            card
        } else {
            card.opacity(PLAN_DISABLED_OPACITY)
        }
    })
}
