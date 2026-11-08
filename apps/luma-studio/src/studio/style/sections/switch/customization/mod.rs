use xgpui_luma::controls::switch::SwitchEventExt;
use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonContentContext;
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::switch::{Switch, SwitchData, SwitchOrientation};
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_shadcn::paint::switch_look;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::icons::render_lucide_icon;

const DEMO_STACK_GAP: f32 = 4.0;
const LABEL_TRACK_EXTRA_WIDTH: f32 = 12.0;
const LABEL_TRACK_PADDING_X: f32 = 4.0;

pub(crate) struct SwitchCustomizationPreview {
    look: Arc<ShadcnLook>,
    primary_switch: Switch,
    secondary_switch: Switch,
    vertical_switch: Switch,
}

impl SwitchCustomizationPreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let primary_switch = labeled_switch(
            shadcn::Switch::new("luma-studio-labeled-switch-primary")
                .look(look.as_ref())
                .primary()
                .with_data(true),
            look.clone(),
            ShadcnButtonStyle::Primary,
        )
        .content(|_, _| div().into_any_element())
        .spawn(cx);
        let secondary_switch = labeled_switch(
            shadcn::Switch::new("luma-studio-labeled-switch-secondary")
                .look(look.as_ref())
                .secondary()
                .with_data(false),
            look.clone(),
            ShadcnButtonStyle::Secondary,
        )
        .content(|_, _| div().into_any_element())
        .spawn(cx);
        let vertical_switch = shadcn::Switch::new("luma-studio-vertical-switch-primary")
            .look(look.as_ref())
            .primary()
            .vertical()
            .with_data(true)
            .thumb_content(icon_thumb_content(look.clone(), ShadcnButtonStyle::Primary))
            .content(|_, _| div().into_any_element())
            .spawn(cx);
        let mut preview = Self { look, primary_switch, secondary_switch, vertical_switch };
        preview.bind_callbacks(cx);
        preview
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.primary_switch.update(cx, |switch, cx| {
            switch.set_template(look.switch_template(ShadcnButtonStyle::Primary), cx);
            switch.set_switch_track_content(labeled_track_content(look.clone(), ShadcnButtonStyle::Primary), cx);
            switch.set_switch_thumb_content(icon_thumb_content(look.clone(), ShadcnButtonStyle::Primary), cx);
        });
        self.secondary_switch.update(cx, |switch, cx| {
            switch.set_template(look.switch_template(ShadcnButtonStyle::Secondary), cx);
            switch.set_switch_track_content(labeled_track_content(look.clone(), ShadcnButtonStyle::Secondary), cx);
            switch.set_switch_thumb_content(icon_thumb_content(look.clone(), ShadcnButtonStyle::Secondary), cx);
        });
        self.vertical_switch.update(cx, |switch, cx| {
            switch.set_template(look.switch_template(ShadcnButtonStyle::Primary), cx);
            switch.set_switch_orientation(SwitchOrientation::Vertical, cx);
            switch.set_switch_thumb_content(icon_thumb_content(look.clone(), ShadcnButtonStyle::Primary), cx);
        });
        cx.notify();
    }

    fn bind_callbacks(&mut self, cx: &mut Context<Self>) {
        self.primary_switch.clone().on_value_changed(cx, |_, _, cx| cx.notify());
        self.secondary_switch.clone().on_value_changed(cx, |_, _, cx| cx.notify());
        self.vertical_switch.clone().on_value_changed(cx, |_, _, cx| cx.notify());
    }
}

fn labeled_switch(builder: shadcn::Switch, look: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> shadcn::Switch {
    builder
        .track_length_extra(LABEL_TRACK_EXTRA_WIDTH)
        .track_content(labeled_track_content(look.clone(), style))
        .thumb_content(icon_thumb_content(look, style))
}

fn labeled_track_content(
    look: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> impl Fn(&ButtonContentContext<SwitchData>, &mut App) -> AnyElement + Send + Sync + 'static {
    move |model, _| {
        let checked = model.data.checked;
        let tokens = look.mode_tokens();
        let palette = switch_look(tokens.as_ref(), look.mode(), style, checked, model.state, model.size);
        let on_palette =
            switch_look(tokens.as_ref(), look.mode(), style, true, InteractionState::default(), model.size);
        let state_label = if checked { "ON" } else { "OFF" };
        let state_label_color = if checked && !model.state.disabled {
            on_palette.thumb_background
        } else {
            palette.label_color
        };

        div()
            .size_full()
            .flex()
            .items_center()
            .px(px(LABEL_TRACK_PADDING_X))
            .when(checked, |label_row| label_row.justify_start())
            .when(!checked, |label_row| label_row.justify_end())
            .text_size(px((palette.label_typography.size * 0.85).max(10.0)))
            .line_height(px((palette.label_typography.line_height * 0.85).max(12.0)))
            .font_family(palette.label_font_family.clone())
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(state_label_color)
            .child(state_label)
            .into_any_element()
    }
}

fn icon_thumb_content(
    look: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> impl Fn(&ButtonContentContext<SwitchData>, &mut App) -> AnyElement + Send + Sync + 'static {
    move |model, _| {
        let checked = model.data.checked;
        let tokens = look.mode_tokens();
        let palette = switch_look(tokens.as_ref(), look.mode(), style, checked, model.state, model.size);
        let icon = if checked { LucideIcon::Check } else { LucideIcon::X };
        div()
            .text_color(palette.track_background)
            .child(render_lucide_icon(icon, palette.track_background, 11.0))
            .into_any_element()
    }
}

impl Render for SwitchCustomizationPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let primary_checked = *self.primary_switch.read(cx).data();
        let secondary_checked = *self.secondary_switch.read(cx).data();
        let vertical_checked = *self.vertical_switch.read(cx).data();

        div().w_full().flex().flex_col().items_center().gap_8().child(
            div()
                .w_full()
                .flex()
                .flex_row()
                .items_start()
                .justify_center()
                .gap_8()
                .child(demo_block(
                    "Primary labeled switch",
                    chrome.muted_text,
                    self.primary_switch.clone(),
                    state_label(primary_checked, chrome.body_text),
                ))
                .child(demo_block(
                    "Secondary labeled switch",
                    chrome.muted_text,
                    self.secondary_switch.clone(),
                    state_label(secondary_checked, chrome.body_text),
                ))
                .child(demo_block(
                    "Vertical icon switch",
                    chrome.muted_text,
                    self.vertical_switch.clone(),
                    state_label(vertical_checked, chrome.body_text),
                )),
        )
    }
}

fn demo_block(label: &'static str, label_color: gpui::Hsla, control: Switch, value: impl IntoElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(DEMO_STACK_GAP))
        .child(section_label(label, label_color))
        .child(control)
        .child(value)
        .into_any_element()
}

fn section_label(label: &'static str, color: gpui::Hsla) -> impl IntoElement {
    div().text_size(px(12.0)).line_height(px(16.0)).text_color(color).child(label)
}

fn state_label(checked: bool, color: gpui::Hsla) -> impl IntoElement {
    div().text_size(px(12.0)).line_height(px(16.0)).text_color(color).child(if checked {
        "State: ON"
    } else {
        "State: OFF"
    })
}

pub(crate) fn render_switch_customization_body(preview: Entity<SwitchCustomizationPreview>) -> AnyElement {
    preview.into_any_element()
}
