use xgpui_luma::controls::textfield::TextFieldEventExt;
use xgpui_luma::controls::slider::SliderEventExt;
use crate::color_hex::format_hex;
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};
use std::sync::Arc;

use gpui::{Context, Entity, EventEmitter, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{Button, ButtonEvent};
use xgpui_luma::controls::popover_button::{PopoverButton, PopoverDismissPolicy, PopoverPlacement};
use xgpui_luma::controls::slider::{SliderControl, SliderEvent};
use xgpui_luma::controls::textfield::{TextField, TextFieldEvent};
use xgpui_luma::prelude::{TooltipEntityExt, HasPresenter};
use xgpui_luma::theme::{ControlSize, InteractionState};
use xgpui_luma::infra::attachments::{AttachmentHost, AttachmentTarget};
use xgpui_luma_color::color_field::{ColorFieldEvent, ColorFieldState};
use xgpui_luma_color::color_slider::color_spec::{Hsv, ColorSpecification};
use xgpui_luma_color::color_slider::{ColorSliderBuilder, primary_slider_value, sizing};
use xgpui_luma_color::composition::ColorCompositionSync;
use xgpui_luma_color::{ColorSwatchButtonTemplate, ColorSwatchData};
use xgpui_luma_look_radix::{Look, SemanticRole};
use xgpui_luma_look_radix as radix;

const SWATCH_SIZE: f32 = 16.0;
const SWATCH_RADIUS: f32 = 2.0;

#[derive(Clone, Debug)]
pub enum ColorTextFieldEvent {
    Change { color: ColorValue },
    LinkChange { linked: bool },
}

pub struct ColorTextField {
    id: SharedString,
    look: Arc<Look>,
    color: ColorValue,
    hsv: Hsv,
    focused: bool,
    hovered: bool,
    attachments: AttachmentHost,
    swatch: Entity<Button<ColorSwatchData>>,
    field: TextField,
    accent_link: Option<Entity<Button<bool>>>,
    picker_field: Entity<ColorFieldState>,
    hue_slider: Entity<SliderControl>,
    picker_hex: TextField,
    popover: Entity<PopoverButton>,
    sync: ColorCompositionSync,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ButtonEvent> for ColorTextField {}
impl EventEmitter<ColorTextFieldEvent> for ColorTextField {}

impl ColorTextField {
    pub const HEIGHT: f32 = 32.0;

    pub fn new(look: Arc<Look>, id: impl Into<SharedString>, color: ColorValue, cx: &mut Context<Self>) -> Self {
        let id = id.into();
        let hsv = preview_hsv(color);

        let swatch = Button::new(format!("{id}-swatch"))
            .typed(ColorSwatchData {
                color,
                size: SWATCH_SIZE,
                radius: SWATCH_RADIUS,
                open: false,
                checkerboard: false,
            })
            .template(Arc::new(ColorSwatchButtonTemplate))
            .tab_stop(false)
            .spawn(cx);
        let field = embedded_textfield(&look, format!("{id}-field"), rgb_text(color), cx);
        let picker_field = cx.new(|_| {
            ColorFieldState::saturation_value(format!("{id}-picker-sv"), hsv, sizing::THUMB_SIZE_MEDIUM)
                .no_border()
                .rounded(px(0.0))
        });
        let hue_builder = ColorSliderBuilder::hue(format!("{id}-picker-hue"), hsv.h)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge();
        let hue_slider = hue_builder.spawn(cx);
        let picker_hex = embedded_textfield(&look, format!("{id}-picker-hex"), rgb_text(color), cx);

        let swatch_for_popover = swatch.clone();
        let picker_field_for_popover = picker_field.clone();
        let hue_slider_for_popover = hue_slider.clone();
        let picker_hex_for_popover = picker_hex.clone();
        let popup_look = Arc::clone(&look);
        let popover = PopoverButton::new(format!("{id}-popover"))
            .typed(())
            .trigger(move |_, _| swatch_for_popover.clone().into_any_element())
            .content(move |_, _, _| {
                div()
                    .w(px(360.0))
                    .p(px(12.0))
                    .gap(px(10.0))
                    .flex()
                    .flex_col()
                    .bg(popup_look.resolve_role(SemanticRole::Surface).hsla())
                    .child(div().w_full().h(px(180.0)).child(picker_field_for_popover.clone()))
                    .child(div().w_full().child(hue_slider_for_popover.clone()))
                    .child(div().w_full().child(picker_hex_for_popover.clone()))
                    .into_any_element()
            })
            .placement(PopoverPlacement::Smart)
            .dismiss_policy(PopoverDismissPolicy::CloseOnClickAway)
            .offset_y(px(4.0))
            .initial_content_size(gpui::size(px(384.0), px(264.0)))
            .trigger_on_pointer_down(false)
            .spawn(cx);

        let mut subscriptions = Vec::new();
        field.clone().on_event(cx, |this, event: &TextFieldEvent, cx| match event {
            TextFieldEvent::FocusChanged { focused } => {
                this.focused = *focused;
                cx.notify();
            }
            TextFieldEvent::Change { value } => this.apply_hex(value, cx),
            _ => {}
        });
        picker_hex.clone().on_event(cx, |this, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                this.apply_hex(value, cx);
            }
        });
        subscriptions.push(cx.subscribe(&picker_field, |this, _, event: &ColorFieldEvent, cx| {
            let (ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv)) = event else {
                return;
            };
            this.hsv.s = hsv.s;
            this.hsv.v = hsv.v;
            this.apply_hsv(cx);
        }));
        hue_slider.clone().on_event(cx, |this, event: &SliderEvent, cx| {
            let Some(value) = primary_slider_value(event) else {
                return;
            };
            this.hsv.h = value;
            this.apply_hsv(cx);
        });
        let popover_for_swatch = popover.downgrade();
        swatch.clone().on_click(cx, move |this, cx| {
            let Some(popover_for_swatch) = popover_for_swatch.upgrade() else {
                return;
            };

            popover_for_swatch.update(cx, |popover, cx| popover.toggle_guarded(cx));
            cx.emit(ButtonEvent::Click);
            this.focused = false;
            cx.notify();
        });

        let mut attachments = AttachmentHost::default();
        attachments.set_tooltip_theme(radix::tooltip_theme(&look));
        Self {
            id,
            look,
            color,
            hsv,
            focused: false,
            hovered: false,
            attachments,
            swatch,
            field,
            accent_link: None,
            picker_field,
            hue_slider,
            picker_hex,
            popover,
            sync: ColorCompositionSync::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn enable_accent_link(&mut self, cx: &mut Context<Self>) {
        let button = super::icon_button::ghost_no_hover(format!("{}-link", self.id), &self.look, true)
            .content(|model, _| {
                crate::assets::icon_named(if model.data { "link-2" } else { "link-break-2" })
                    .map(|icon| crate::assets::react_icon(icon, model.look.foreground, 15.0))
                    .unwrap_or_else(|| div().into_any_element())
            })
            .spawn(cx)
            .help("Link accent edits across light and dark palettes", cx);
        {
            let callback_button = (&button).downgrade();
            button.clone().on_click(cx, move |_, cx| {
                let Some(button) = callback_button.upgrade() else {
                    return;
                };

                let linked = !button.read(cx).data();
                button.update(cx, |button, cx| button.set_data(linked, cx));
                cx.emit(ColorTextFieldEvent::LinkChange { linked });
            })
        };
        self.accent_link = Some(button);
        cx.notify();
    }

    pub fn set_color(&mut self, color: ColorValue, cx: &mut Context<Self>) {
        self.color = color;
        self.hsv = preview_hsv(color);
        self.sync_fields(cx);
        cx.notify();
    }

    fn apply_hex(&mut self, value: &str, cx: &mut Context<Self>) {
        let input = value.trim();
        let hex = input.strip_prefix('#').unwrap_or(input);
        let normalized;
        let input = if hex.len() == 6 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            normalized = format!("#{hex}");
            normalized.as_str()
        } else {
            input
        };
        let Ok(color) = ColorValue::parse_css(input) else {
            return;
        };
        self.set_color(color, cx);
        cx.emit(ColorTextFieldEvent::Change { color });
    }

    fn apply_hsv(&mut self, cx: &mut Context<Self>) {
        let color = self.hsv.to_color_value();
        // Keep HSV as the gesture source: converting black/gray back from HSL
        // discards hue (and at black, saturation), moving the hue slider spuriously.
        self.color = color;
        self.sync_fields(cx);
        cx.notify();
        cx.emit(ColorTextFieldEvent::Change { color });
    }

    fn sync_fields(&self, cx: &mut Context<Self>) {
        let color = self.color;
        self.swatch.update(cx, |swatch, cx| {
            swatch.set_data(
                ColorSwatchData { color, size: SWATCH_SIZE, radius: SWATCH_RADIUS, open: false, checkerboard: false },
                cx,
            );
        });

        self.picker_field.update(cx, |field, cx| field.set_hsv(self.hsv, cx));
        self.sync.sync_slider_value(&self.hue_slider, self.hsv.h, cx);
        let hex = rgb_text(self.color);
        // Both text fields represent the same selected color as the swatch.
        // Avoid resetting the caret when the text is already synchronized.
        for field in [&self.field, &self.picker_hex] {
            field.update(cx, |field, cx| {
                if field.value().as_ref() != hex {
                    field.set_value(hex.clone(), cx);
                }
            });
        }
    }
}

impl Render for ColorTextField {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let border = if self.focused {
            self.look.resolve_role(SemanticRole::Primary).hsla()
        } else {
            self.look.resolve_role(SemanticRole::Border).hsla()
        };
        let background = self.look.resolve_role(SemanticRole::Background).hsla();

        let root = div()
            .id(self.id.clone())
            .h(px(Self::HEIGHT))
            .w_full()
            .flex()
            .items_center()
            .gap(px(8.0))
            .px(px(7.0))
            .bg(background)
            .border_1()
            .border_color(border)
            .rounded(px(4.0))
            .child(self.popover.clone())
            .child(div().min_w(px(0.0)).flex_1().child(self.field.clone()))
            .children(self.accent_link.clone())
            .on_hover(cx.listener(|field, hovered: &bool, _, cx| {
                field.hovered = *hovered;
                cx.notify();
            }));
        self.attachments.render(
            root,
            InteractionState { hovered: self.hovered, focused: self.focused, ..Default::default() },
            cx,
        )
    }
}

fn embedded_textfield(
    look: &Arc<Look>,
    id: impl Into<SharedString>,
    value: impl Into<String>,
    cx: &mut Context<ColorTextField>,
) -> TextField {
    radix::TextField::new(id)
        .look(look)
        .value(value.into())
        .look_override(|mut field| {
            field.background = gpui::transparent_black();
            field.border = gpui::transparent_black();
            field.focus_border = None;
            field.shadow = None;
            field.padding_x = 0.0;
            field.padding_y = 0.0;
            // The enclosing color field supplies the chrome and vertical spacing.
            // Center the text line itself alongside the 16px swatch.
            field.min_height = field.typography.line_height;
            field.radius = 0.0;
            field.border_width = 0.0;
            field
        })
        .spawn(cx)
}

impl AttachmentTarget for ColorTextField {
    fn attachments(&self) -> &AttachmentHost {
        &self.attachments
    }
    fn attachments_mut(&mut self) -> &mut AttachmentHost {
        &mut self.attachments
    }
}

fn preview_hsv(source: ColorValue) -> Hsv {
    Hsv::from_hsla_ext(
        gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde).unwrap_or_else(|_| gpui::transparent_black()),
    )
}
// Display RGB hex independently of the retained source representation.
fn rgb_text(source: ColorValue) -> String {
    format_hex(gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde).unwrap_or_else(|_| gpui::black()))
}

#[cfg(all(test, feature = "test-support"))]
mod source_tests {
    use super::*;
    #[test]
    fn loading_and_css_editing_preserve_wide_gamut_source() {
        let app = gpui::TestAppContext::single();
        let source = ColorValue::display_p3(1.123456, -0.123456, 0.234567, 1.0);
        app.update(|cx| {
            let field = cx.new(|cx| ColorTextField::new(Arc::new(Look::built_in()), "source-test", source, cx));
            field.update(cx, |field, cx| {
                assert_eq!(field.color, source);
                field.set_color(source, cx);
                field.sync_fields(cx);
                assert_eq!(field.color, source);
                let edited = ColorValue::oklch(0.75, 0.345678, 412.12345, 1.0);
                field.apply_hex(&edited.to_css().unwrap(), cx);
                assert_eq!(field.color, edited);
                let displayed = rgb_text(edited);
                assert_eq!(field.field.read(cx).value().as_str(), displayed);
                assert_eq!(field.picker_hex.read(cx).value().as_str(), displayed);
                field.apply_hex("3E63DD", cx);
                assert_eq!(field.color, ColorValue::srgb(62.0 / 255.0, 99.0 / 255.0, 221.0 / 255.0, 1.0));
                assert_eq!(field.field.read(cx).value().as_str(), "3E63DD");
                assert_eq!(field.picker_hex.read(cx).value().as_str(), "3E63DD");
            });
        });
    }
}
