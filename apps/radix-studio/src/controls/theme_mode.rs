//! Theme choices and wiring; segmented styling is supplied by the Radix look.

use gpui::{Context, FontWeight, div, prelude::*, px};
use xgpui_luma::controls::radio_group::{self, RadioGroup, RadioGroupItem, RadioGroupItemLike};
use xgpui_luma::theme::ThemeMode;
use xgpui_luma_look_radix::{Look, ScaleFamily};
use crate::assets::{icon_named, react_icon};

pub fn mode_id(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    }
}

pub fn spawn<M: 'static>(id: &'static str, look: &Look, cx: &mut Context<M>) -> RadioGroup<RadioGroupItem> {
    let content_look = look.clone();
    radio_group::horizontal(id)
        .single_required()
        .selection_follows_active(true)
        .items([RadioGroupItem::new("light").label("Light"), RadioGroupItem::new("dark").label("Dark")])
        .selected(mode_id(look.mode()))
        .template(xgpui_luma_look_radix::segmented_radio_template(look))
        .with_item_template(move |item, _, _| {
            let name = if item.item.id().as_ref() == "light" {
                "sun"
            } else {
                "moon"
            };
            let color = content_look.resolve_step(ScaleFamily::Gray, 12).hsla();
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .children(icon_named(name).map(|icon| react_icon(icon, color, 16.0)))
                .child(
                    div()
                        // User-tested: Helvetica Neue makes the selected 900 weight
                        // visibly distinct at 14px; inherited "System UI" did not.
                        // Keep this font override local to the Light/Dark labels.
                        .font_family("Helvetica Neue")
                        .text_size(px(14.0))
                        .font_weight(if item.selected {
                            FontWeight(900.0)
                        } else {
                            FontWeight::NORMAL
                        })
                        .child(item.item.label().clone()),
                )
        })
        .spawn(cx)
}
