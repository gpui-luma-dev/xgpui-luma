//! Local palette preview: five Radix icon-button variants and off/on switches.

use gpui::{Context, Entity, IntoElement, div, prelude::*, px};
use xgpui_luma::controls::button::{Button, ControlIcon};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::switch::Switch;
use xgpui_luma_look_radix::{self as radix, ButtonVariant, Look};

#[derive(Clone)]
pub struct IconSamples {
    buttons: Vec<Entity<Button>>,
    switches: [Switch; 2],
}

impl IconSamples {
    pub fn spawn<T: 'static>(look: &Look, cx: &mut Context<T>) -> Self {
        let buttons = [
            ("star", ButtonVariant::Classic),
            ("bookmark", ButtonVariant::Solid),
            ("accessibility", ButtonVariant::Soft),
            ("heart", ButtonVariant::Surface),
            ("share-1", ButtonVariant::Outline),
        ]
        .into_iter()
        .map(|(icon, variant)| {
            radix::Button::new(format!("palette-icon-{icon}"))
                .look(look)
                .variant(variant)
                .role(ButtonFamilyRole::Icon)
                .icon(ControlIcon::SvgPath(format!("assets/react-icons/{icon}.svg").into()))
                .spawn(cx)
        })
        .collect();
        let switches = [false, true].map(|checked| {
            radix::Switch::new(format!("palette-switch-{checked}"))
                .look(look)
                .size(radix::SwitchSize::One)
                .checked(checked)
                .spawn(cx)
        });
        Self { buttons, switches }
    }

    pub fn notify<T: 'static>(&self, cx: &mut Context<T>) {
        for button in &self.buttons {
            button.update(cx, |_, cx| cx.notify());
        }
        for switch in &self.switches {
            switch.update(cx, |_, cx| cx.notify());
        }
    }

    pub fn render(self) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .children(self.buttons)
            .children(self.switches)
    }
}
