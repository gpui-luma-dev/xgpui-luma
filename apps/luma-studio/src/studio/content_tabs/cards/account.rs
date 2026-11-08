use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::textfield::TextField;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma::{declare_form, form_field, hstack, vstack};

use super::common::{or_divider, titled_card};

declare_form! {
    pub struct AccountPanel {
        controls: {
            github_button: Entity<Button> = shadcn::Button::new("account-github").look(look.as_ref()).outline().label("GitHub").size(size),
            google_button: Entity<Button> = shadcn::Button::new("account-google").look(look.as_ref()).outline().label("Google").size(size),
            email_field: TextField = shadcn::TextField::new("account-email").look(look.as_ref()).placeholder("Email").full_width(true),
            password_field: TextField = shadcn::TextField::new("account-password").look(look.as_ref())

                .placeholder("Password")
                .full_width(true),
            create_button: Entity<Button> = shadcn::Button::new("account-create").look(look.as_ref()).primary().label("Create account").size(size),
        },
        args: {
            look: Arc<ShadcnLook>,
            size: shadcn::ShadcnSize,
        },
        fields: {}
    }
}

impl Render for AccountPanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let github_button = self.github_button.clone();
        let google_button = self.google_button.clone();
        let email_field = self.email_field.clone();
        let password_field = self.password_field.clone();
        let create_button = self.create_button.clone();

        titled_card(
            "luma-studio-account-card",
            &self.look,
            340.0,
            "Create an account",
            "Enter your email below to create your account",
            move |_, _| {
                vstack! {
                    gap=12;
                    hstack! {
                        gap=8;
                        div().flex_1().child(github_button.clone()),
                        div().flex_1().child(google_button.clone()),
                    },
                    or_divider("OR CONTINUE WITH", chrome.border, chrome.muted_text),
                    form_field!("Email", chrome; email_field.clone()),
                    form_field!("Password", chrome; password_field.clone()),
                    div().w_full().child(create_button.clone()),
                }
                .into_any_element()
            },
            window,
            _cx,
        )
    }
}
