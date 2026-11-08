use std::sync::Arc;

use gpui::{Context, FontWeight, IntoElement, Render, Window, div, prelude::*};
use xgpui_luma::controls::checkbox::{Checkbox, CheckboxEvent};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::progress::Progress;
use xgpui_luma::controls::slider::{Slider, SliderEvent};
use xgpui_luma::controls::switch::{Switch, SwitchEvent};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize};
use xgpui_luma::{declare_form, vstack};

use super::common::titled_card;

const SYSTEM_PREFERENCES_CARD_WIDTH: f32 = 380.0;

declare_form! {
    pub struct SystemPreferencesPanel {
        controls: {
            two_factor_switch: Switch = shadcn::Switch::new("luma-studio-system-two-factor").look(look.as_ref()).primary()
                .with_data(false)
                .content(|_, _| div().child("Two-factor authentication").into_any_element())
                => SwitchEvent |this, event, cx| {
                    this.handle_two_factor_event(event, cx);
                },
            terms_checkbox: Checkbox = shadcn::Checkbox::new("luma-studio-system-terms").look(look.as_ref()).primary()
                .with_data(false)
                .content(|_, _| div().child("I agree to the terms and conditions").into_any_element())
                => CheckboxEvent |this, event, cx| {
                    this.handle_terms_event(event, cx);
                },
            social_checkbox: Checkbox = shadcn::Checkbox::new("luma-studio-system-social").look(look.as_ref()).primary()
                .with_data(true)
                .content(|_, _| div().child("Social").into_any_element())
                => CheckboxEvent |this, event, cx| {
                    this.handle_social_event(event, cx);
                },
            referral_checkbox: Checkbox = shadcn::Checkbox::new("luma-studio-system-referral").look(look.as_ref()).primary()
                .with_data(false)
                .content(|_, _| div().child("Referral").into_any_element())
                => CheckboxEvent |this, event, cx| {
                    this.handle_referral_event(event, cx);
                },
            budget_slider: Slider = shadcn::Slider::new("luma-studio-system-budget").look(look.as_ref())
                .range(0..100)
                .step(5)
                .value(40)
                => SliderEvent |this, event, cx| {
                    this.handle_budget_event(event, cx);
                },
            completion_progress: Progress = shadcn::Progress::new("luma-studio-system-completion").look(look.as_ref())
                .range(0..100)
                .value(initial_completion as i32),
        },
        args: {
            look: Arc<ShadcnLook>,
            initial_completion: f32,
        },
        fields: {
            accepted_terms: bool = false,
            social_source: bool = true,
            referral_source: bool = false,
            two_factor_enabled: bool = false,
            budget: f32 = 40.0,
            completion: f32 = initial_completion,
        }
    }
}

impl SystemPreferencesPanel {
    fn handle_terms_event(&mut self, event: &CheckboxEvent, cx: &mut Context<Self>) {
        if let CheckboxEvent::Change { checked } = event {
            self.accepted_terms = *checked;
            cx.notify();
        }
    }

    fn handle_social_event(&mut self, event: &CheckboxEvent, cx: &mut Context<Self>) {
        if let CheckboxEvent::Change { checked } = event {
            self.social_source = *checked;
            cx.notify();
        }
    }

    fn handle_referral_event(&mut self, event: &CheckboxEvent, cx: &mut Context<Self>) {
        if let CheckboxEvent::Change { checked } = event {
            self.referral_source = *checked;
            cx.notify();
        }
    }

    fn handle_two_factor_event(&mut self, event: &SwitchEvent, cx: &mut Context<Self>) {
        if let SwitchEvent::Change { on } = event {
            self.two_factor_enabled = *on;
            cx.notify();
        }
    }

    fn handle_budget_event(&mut self, event: &SliderEvent, cx: &mut Context<Self>) {
        let value = match event {
            SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
            _ => return,
        };
        self.budget = value;
        self.completion = value;
        self.completion_progress.update(cx, |progress, cx| progress.set_value(value as f64, cx));
        cx.notify();
    }
}

impl Render for SystemPreferencesPanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let label_style = self.look.typography_scale(ShadcnTextSize::Sm);
        let two_factor_switch = self.two_factor_switch.clone();
        let terms_checkbox = self.terms_checkbox.clone();
        let social_checkbox = self.social_checkbox.clone();
        let referral_checkbox = self.referral_checkbox.clone();
        let budget_slider = self.budget_slider.clone();
        let completion_progress = self.completion_progress.clone();
        let budget = self.budget;
        let completion = self.completion;

        titled_card(
            "luma-studio-system-preferences-card",
            &self.look,
            SYSTEM_PREFERENCES_CARD_WIDTH,
            "System & Preferences",
            "Choice controls plus progress feedback.",
            move |_, _| {
                vstack! {
                    gap=10.0;
                    two_factor_switch.clone(),
                    terms_checkbox.clone(),
                    social_checkbox.clone(),
                    referral_checkbox.clone(),
                    vstack! {
                        gap=6.0;
                        div()
                            .typography_style(label_style)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(format!("Budget: {:.0}%", budget)),
                        budget_slider.clone(),
                    },
                    vstack! {
                        gap=6.0;
                        div()
                            .typography_style(label_style)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(format!("Profile completion: {:.0}%", completion)),
                        completion_progress.clone(),
                    },
                }
                .into_any_element()
            },
            window,
            _cx,
        )
    }
}
