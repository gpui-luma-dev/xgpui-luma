use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, prelude::*};
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::controls::popup_menu::{PopupMenu, PopupMenuEvent};
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma::{declare_form, form_field, vstack};

use super::common::titled_card;

declare_form! {
    pub struct DateRangePanel {
        controls: {
            date_menu: Entity<PopupMenu> = shadcn::PopupMenu::new("date-range").look(look.as_ref())
                .label("Jan 20, 2023 - Feb 09, 2023")
                .size(shadcn::ShadcnSize::Sm)
                .items(date_stub_items())
                => PopupMenuEvent |this, event, cx| {
                    let PopupMenuEvent::Select { label, .. } = event else {
                        return;
                    };
                    this.date_menu.update(cx, |menu, cx| menu.set_label(label.clone(), cx));
                },
        },
        args: {
            look: Arc<ShadcnLook>,
        },
        fields: {}
    }
}

impl Render for DateRangePanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let date_menu = self.date_menu.clone();

        titled_card(
            "luma-studio-date-range-card",
            &self.look,
            380.0,
            "Date picker with range",
            "Select a date range for the report.",
            move |_, _| {
                vstack! {
                    gap=12;
                    form_field!("Date", chrome; date_menu.clone()),
                }
                .into_any_element()
            },
            window,
            _cx,
        )
    }
}

fn date_stub_items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("jan").label("Jan 20, 2023 - Feb 09, 2023"),
        MenuItem::new("feb").label("Feb 10, 2023 - Mar 01, 2023"),
        MenuItem::new("mar").label("Mar 02, 2023 - Mar 31, 2023"),
    ]
}
