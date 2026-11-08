use xgpui_luma::controls::textfield::TextFieldEventExt;
use crate::studio::color_format::{format_color_readout, parse_color_input};
use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionItem, AccordionTrigger};
use xgpui_luma::controls::textfield::{TextField, TextFieldEvent, TextFieldLook, TextFieldLookOverride};
use xgpui_luma::vstack;
use xgpui_luma_look_shadcn::{ShadcnFont, ShadcnLook};
use xgpui_luma_look_shadcn as shadcn;

use super::super::model::TOKEN_CATEGORIES;
use super::super::parsing::effective_token_color;
use super::{category_item_id, spawn_compact_textfield};
use crate::studio::app::LumaStudioApp;
use xgpui_luma::color::ColorValue;
use crate::studio::controls::color_picker::{ColorPickerEvent, ColorPickerPopover};
use crate::studio::overrides::StudioOverrides;
use crate::studio::token_color_row::token_color_row;

pub struct ColorsPanel {
    look: Arc<ShadcnLook>,
    pub(crate) scroll_handle: gpui::ScrollHandle,
    global_overrides: HashMap<String, ColorValue>,
    token_fields: HashMap<String, TextField>,
    token_pickers: HashMap<String, Entity<ColorPickerPopover>>,
    token_accordion: Entity<AccordionControl>,
}

impl ColorsPanel {
    pub fn new(look: Arc<ShadcnLook>, overrides: &StudioOverrides, cx: &mut Context<Self>) -> Self {
        let global_overrides = overrides.global_color_overrides.clone();
        let token_fields = build_token_fields(&look, &global_overrides, cx);
        let token_pickers = build_token_pickers(&look, &global_overrides, cx);
        let token_accordion = Self::build_token_accordion(cx.entity(), look.clone(), cx);

        let scroll_handle = gpui::ScrollHandle::default();
        token_accordion.update(cx, |accordion, cx| accordion.set_drag_scroll_handle(Some(scroll_handle.clone()), cx));
        Self { look, scroll_handle, global_overrides, token_fields, token_pickers, token_accordion }
    }

    pub fn apply_theme_snapshot(&mut self, look: Arc<ShadcnLook>, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        self.look = look;
        self.global_overrides = overrides.global_color_overrides.clone();
        let theme = self.look.clone();
        self.sync_templates(&theme, cx);
        self.sync_values(theme.as_ref(), overrides, cx);
        self.sync_pickers(theme.as_ref(), overrides, cx);
        // Retain the ordered Accordion and all keyed disclosure/editor state.
        self.token_accordion
            .update(cx, |accordion, cx| accordion.set_template(theme.accordion_template(), cx));
        cx.notify();
    }

    pub fn sync_global_overrides(&mut self, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        self.global_overrides = overrides.global_color_overrides.clone();
        let theme = self.look.clone();
        self.sync_values(theme.as_ref(), overrides, cx);
        self.sync_pickers(theme.as_ref(), overrides, cx);
        cx.notify();
    }

    pub fn wire_subscriptions(
        panel: &Entity<Self>,
        cx: &mut Context<LumaStudioApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let field = panel.read(cx).token_fields.get(*token).expect("token field").clone();
                let token_key = token.to_string();
                field.clone().on_event(cx, move |app, event: &TextFieldEvent, cx| {
                    if let TextFieldEvent::Change { value } = event {
                        let Some(color) = parse_color_input(value) else {
                            return;
                        };
                        app.set_global_color(&token_key, color, cx);
                    }
                });
                let picker = panel.read(cx).token_pickers.get(*token).expect("token picker").clone();
                let token_key = token.to_string();
                subscriptions.push(cx.subscribe(&picker, move |app, _, event: &ColorPickerEvent, cx| {
                    let ColorPickerEvent::Change(color) = event;
                    app.set_global_color(&token_key, *color, cx);
                }));
            }
        }
    }

    fn sync_templates(&self, theme: &Arc<ShadcnLook>, cx: &mut Context<Self>) {
        for field in self.token_fields.values() {
            sync_token_field_template(theme, field, cx);
        }
    }

    fn sync_values(&mut self, look: &ShadcnLook, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let Some(field) = self.token_fields.get(*token) else {
                    continue;
                };
                let value = format_color_readout(effective_token_color(look, &overrides.global_color_overrides, token));
                field.update(cx, |field, cx| field.set_value(value, cx));
            }
        }
    }

    fn sync_pickers(&self, look: &ShadcnLook, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        for (token, picker) in &self.token_pickers {
            let color = effective_token_color(look, &overrides.global_color_overrides, token);
            picker.update(cx, |picker, cx| picker.set_color(color, cx));
        }
    }

    fn build_token_accordion(
        panel: Entity<Self>,
        look: Arc<ShadcnLook>,
        cx: &mut Context<Self>,
    ) -> Entity<AccordionControl> {
        let mut accordion_builder = shadcn::Accordion::new("luma-studio-token-accordion")
            .look(look.as_ref())
            .multiple()
            .reordering(true)
            .item_dividers(false)
            .trigger_min_height(28.0)
            .trigger_padding_y(4.0)
            .content_padding_top(0.0)
            .content_padding_bottom(4.0)
            .template(look.accordion_template());

        for (category, tokens) in TOKEN_CATEGORIES {
            let items = *tokens;
            let panel = panel.downgrade();
            let id = category_item_id("token", category);
            let expanded = *category == "BASE";
            accordion_builder = accordion_builder.item(
                AccordionItem::new(
                    id,
                    AccordionTrigger::new(*category),
                    AccordionContent::custom(move |_window, cx| {
                        panel.upgrade().map_or_else(
                            || div().into_any_element(),
                            |panel| panel.read(cx).category_token_content(items),
                        )
                    }),
                )
                .expanded(expanded),
            );
        }

        accordion_builder.spawn(cx)
    }

    pub(crate) fn set_all_categories_expanded(&mut self, expanded: bool, cx: &mut Context<Self>) {
        let ids: Vec<_> = self.token_accordion.read(cx).items().iter().map(|item| item.id().clone()).collect();
        for (index, id) in ids.iter().enumerate() {
            if self.token_accordion.read(cx).is_expanded(id) != expanded {
                self.token_accordion.update(cx, |accordion, cx| accordion.toggle_item(index, cx));
            }
        }
    }

    fn category_token_content(&self, tokens: &[(&str, &str)]) -> AnyElement {
        let theme = &self.look;
        let chrome = theme.chrome();
        let row_label_typography = theme.mode_tokens().typography.text.label;

        let mut rows = vstack! {
            gap=6;
        };

        for (token, label) in tokens {
            let Some(field) = self.token_fields.get(*token) else {
                continue;
            };
            let label = label.to_string();
            let field = field.clone();
            let Some(picker) = self.token_pickers.get(*token) else {
                continue;
            };

            rows = vstack! {
                gap=6;
                rows,
                token_color_row(
                    label,
                    picker.clone(),
                    field,
                    &chrome,
                    &row_label_typography,
                ),
            };
        }

        rows.into_any_element()
    }
}

fn build_token_pickers(
    look: &Arc<ShadcnLook>,
    overrides: &HashMap<String, ColorValue>,
    cx: &mut Context<ColorsPanel>,
) -> HashMap<String, Entity<ColorPickerPopover>> {
    let mut pickers = HashMap::new();
    for (_, tokens) in TOKEN_CATEGORIES {
        for (token, _) in *tokens {
            let color = effective_token_color(look, overrides, token);
            let picker = cx.new(|cx| ColorPickerPopover::new(look.clone(), color, format!("colors-{token}"), cx));
            pickers.insert(token.to_string(), picker);
        }
    }
    pickers
}

impl Render for ColorsPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let has_catalog = self.look.has_css_catalog();

        let mut body = div().flex().flex_col().w_full().gap(px(10.0));

        if !has_catalog {
            body = body.child(
                div()
                    .text_xs()
                    .text_color(chrome.muted_text)
                    .child("Native default theme: pick a tweakcn theme above for catalog-backed swatches."),
            );
        }

        body.child(div().w_full().child(self.token_accordion.clone()))
    }
}

const TOKEN_FIELD_FONT_SIZE: f32 = 12.0;
const TOKEN_FIELD_LINE_HEIGHT: f32 = 16.0;

pub(super) fn token_field_look_override_arc(theme: &ShadcnLook) -> TextFieldLookOverride {
    let font_family = theme.font(ShadcnFont::Mono);
    Arc::new(move |mut look: TextFieldLook| {
        look.font_family = font_family.clone();
        look.typography.size = TOKEN_FIELD_FONT_SIZE;
        look.typography.line_height = TOKEN_FIELD_LINE_HEIGHT;
        look
    })
}

pub(super) fn apply_token_field_style(theme: &ShadcnLook, builder: shadcn::TextField) -> shadcn::TextField {
    let override_fn = token_field_look_override_arc(theme);
    builder.compact().look_override(move |look| override_fn(look))
}

/// Keep compact token fields on the same Primary template used by `shadcn::TextField::new().look(look.as_ref())`.
pub(super) fn sync_token_field_template<T>(theme: &Arc<ShadcnLook>, field: &TextField, cx: &mut Context<T>) {
    field.update(cx, |field, cx| {
        field.set_template(theme.primary_textfield_template(), cx);
        field.set_look_override(Some(token_field_look_override_arc(theme)), cx);
    });
}

fn build_token_fields(
    look: &Arc<ShadcnLook>,
    global_overrides: &HashMap<String, ColorValue>,
    cx: &mut Context<ColorsPanel>,
) -> HashMap<String, TextField> {
    let mut token_fields = HashMap::new();
    for (_, tokens) in TOKEN_CATEGORIES {
        for (token, _) in *tokens {
            let initial = format_color_readout(effective_token_color(look, global_overrides, token));
            let field = spawn_compact_textfield(look, &format!("token-{token}"), initial, cx);
            token_fields.insert(token.to_string(), field);
        }
    }
    token_fields
}

#[cfg(all(test, feature = "test-support"))]
mod presentation_tests {
    use super::*;

    #[test]
    fn token_fields_keep_compact_hsl_after_initialization_and_sync() {
        let app = gpui::TestAppContext::single();
        let source = ColorValue::display_p3(1.123456, -0.123456, 0.234567, 0.345678);
        app.update(|cx| {
            let look = Arc::new(ShadcnLook::built_in());
            let mut overrides = StudioOverrides::default();
            overrides.set_global_color("--primary".to_string(), source);
            let panel = cx.new(|cx| ColorsPanel::new(look.clone(), &overrides, cx));
            panel.update(cx, |panel, cx| {
                for field in panel.token_fields.values() {
                    assert!(field.read(cx).value().starts_with("hsl("));
                }
                panel.sync_global_overrides(&overrides, cx);
                assert_eq!(panel.global_overrides["--primary"], source);
                let primary = panel.token_fields["primary"].read(cx).value();
                assert!(primary.starts_with("hsl("));
                assert!(!primary.contains("color("));
            });
        });
    }
}

#[cfg(all(test, feature = "test-support"))]
mod drag_tests {
    use super::*;
    use gpui::{MouseButton, TestAppContext, point};
    #[test]
    fn native_category_order_and_editors_survive_theme_updates_and_expand_all() {
        struct Blank;
        impl Render for Blank {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
            }
        }
        struct Page {
            panel: Entity<ColorsPanel>,
            tabs: Entity<xgpui_luma::controls::tabs::Tabs>,
        }
        impl Render for Page {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().w(px(340.)).h(px(700.)).flex().flex_col().child(self.tabs.clone())
            }
        }
        let mut app = TestAppContext::single();
        let (page, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            let look = Arc::new(ShadcnLook::built_in());
            let panel = cx.new(|cx| ColorsPanel::new(look.clone(), &StudioOverrides::default(), cx));
            let content = panel.clone();
            let host = cx.new(|cx| {
                super::super::PanelContextMenuHost::new(
                    "colors-test-context",
                    move |_| content.clone(),
                    super::super::colors_context_menu_items(),
                    look.clone(),
                    cx,
                )
            });
            let blank = cx.new(|_| Blank);
            let scroll = panel.read(cx).scroll_handle.clone();
            let tabs =
                crate::studio::theme_sidebar::sidebar_tabs_builder(look, host, blank.clone(), blank, scroll).spawn(cx);
            panel.read(cx).token_accordion.clone().update(cx, |a, cx| a.set_motion_enabled(false, cx));
            Page { panel, tabs }
        });
        cx.run_until_parked();
        let panel = cx.update(|_, cx| page.read(cx).panel.clone());
        let (accordion, field, picker) = cx.update(|_, cx| {
            let p = panel.read(cx);
            (
                p.token_accordion.clone(),
                p.token_fields["background"].clone(),
                p.token_pickers["background"].clone(),
            )
        });
        let start = cx.debug_bounds("accordion-drag-token-base").unwrap().center();
        let target = cx.debug_bounds("accordion-item-token-secondary").unwrap();
        let end = point(target.center().x, target.bottom() - px(2.));
        cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
        cx.simulate_mouse_move(start + point(px(12.), px(0.)), MouseButton::Left, Default::default());
        cx.run_until_parked();
        cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
        cx.run_until_parked();
        cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
        cx.run_until_parked();
        let order = cx.update(|_, cx| accordion.read(cx).items().iter().map(|i| i.id().clone()).collect::<Vec<_>>());
        assert_eq!(
            order[..3].iter().map(|id| id.as_ref()).collect::<Vec<_>>(),
            ["token-primary", "token-secondary", "token-base"]
        );
        panel.update(cx, |p, cx| {
            p.set_all_categories_expanded(false, cx);
            let look = Arc::new(ShadcnLook::built_in());
            look.set_mode(xgpui_luma::theme::ThemeMode::Dark);
            p.apply_theme_snapshot(look, &StudioOverrides::default(), cx);
            assert_eq!(p.token_accordion.entity_id(), accordion.entity_id());
            assert_eq!(p.token_fields["background"].entity_id(), field.entity_id());
            assert_eq!(p.token_pickers["background"].entity_id(), picker.entity_id());
            assert_eq!(p.token_accordion.read(cx).items().iter().map(|i| i.id().clone()).collect::<Vec<_>>(), order);
            assert!(p.token_accordion.read(cx).items().iter().all(|i| !p.token_accordion.read(cx).is_expanded(i.id())));
            p.set_all_categories_expanded(true, cx);
            assert!(p.token_accordion.read(cx).items().iter().all(|i| p.token_accordion.read(cx).is_expanded(i.id())));
            assert_eq!(p.token_fields.len(), TOKEN_CATEGORIES.iter().map(|(_, t)| t.len()).sum::<usize>());
        });
    }
}
