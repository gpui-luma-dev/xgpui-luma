use std::collections::BTreeMap;
use std::sync::Arc;

use gpui::{AnyElement, Context, FontWeight, IntoElement, Render, Window, div, prelude::*, px};
use xgpui_luma::declare_form;
use xgpui_luma::theme::{ThemePartUsage, ThemeUsage};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnTextRole, ShadcnTextSize, all_shadcn_theme_usages};

use crate::studio::color_format::format_compact_hsla;

type UsageRef = (&'static str, &'static ThemePartUsage);

#[derive(Clone)]
struct CatalogToken {
    token: String,
    color: gpui::Hsla,
}

declare_form! {
    pub struct ThemeUsagePanel {
        controls: {},
        args: {
            look: Arc<ShadcnLook>,
        },
        fields: {}
    }
}

impl ThemeUsagePanel {
    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }
}

impl Render for ThemeUsagePanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let heading_style = self.look.typography_role(ShadcnTextRole::H3);
            let body_style = self.look.typography_scale(ShadcnTextSize::Sm);
            let section_style = self.look.typography_scale(ShadcnTextSize::Sm);
            let detail_style = self.look.typography_scale(ShadcnTextSize::Sm);
            let caption_style = self.look.typography_scale(ShadcnTextSize::Xs);
            let catalog_tokens = catalog_tokens(self.look.as_ref());
            let usages = all_shadcn_theme_usages();
            let by_token = usage_by_token(usages);
            let sdk_token_count =
                catalog_tokens.iter().filter(|token| by_token.contains_key(token.token.as_str())).count();
            let shared_value_count = shared_value_groups(&catalog_tokens).len();

            div()
                .id("luma-studio-theme-usage")
                .size_full()
                .min_h_0()
                .flex()
                .flex_col()
                .overflow_hidden()
                .bg(chrome.content_background)
                .p(px(28.0))
                .child(
                    div()
                        .id("theme-usage-content")
                        .size_full()
                        .flex()
                        .flex_col()
                        .gap(px(18.0))
                        .overflow_y_scroll()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .typography_style(heading_style)
                                        .text_color(chrome.title_text)
                                        .child("Theme Usage"),
                                )
                                .child(div().typography_style(body_style).text_color(chrome.muted_text).child(
                                    if self.look.has_css_catalog() {
                                        "Shadcn CSS token usage metadata for migrated controls"
                                    } else {
                                        "SDK resolver metadata — CSS catalog empty on native default theme"
                                    },
                                )),
                        )
                        .when_some(render_sparse_catalog_callout(self.look.as_ref()), |panel, callout| {
                            panel.child(callout)
                        })
                        .child(div().flex().gap(px(8.0)).children([
                            render_count_badge(
                                "Components",
                                usages.len().to_string(),
                                self.look.as_ref(),
                                caption_style,
                            ),
                            render_count_badge(
                                "Catalog tokens",
                                catalog_tokens.len().to_string(),
                                self.look.as_ref(),
                                caption_style,
                            ),
                            render_count_badge(
                                "Used by SDK",
                                sdk_token_count.to_string(),
                                self.look.as_ref(),
                                caption_style,
                            ),
                            render_count_badge(
                                "Shared values",
                                shared_value_count.to_string(),
                                self.look.as_ref(),
                                caption_style,
                            ),
                        ]))
                        .child(
                            div()
                                .flex()
                                .gap(px(20.0))
                                .items_start()
                                .child(render_by_token(&catalog_tokens, &by_token, self.look.as_ref(), detail_style))
                                .child(render_by_component(usages, self.look.as_ref(), detail_style, caption_style)),
                        )
                        .child(render_shared_values(
                            &catalog_tokens,
                            &by_token,
                            self.look.as_ref(),
                            caption_style,
                            detail_style,
                            section_style,
                        ))
                        .child(render_shadow_usage(usages, self.look.as_ref(), detail_style, caption_style)),
                )
        })
    }
}

fn catalog_tokens(look: &ShadcnLook) -> Vec<CatalogToken> {
    let catalog = &look.mode_tokens().catalog;
    let mut tokens: Vec<CatalogToken> = catalog
        .tokens
        .keys()
        .filter_map(|token| catalog.color(token).ok().map(|color| CatalogToken { token: token.clone(), color }))
        .collect();
    tokens.sort_by(|left, right| left.token.cmp(&right.token));
    tokens
}

fn usage_by_token(usages: &'static [&'static ThemeUsage]) -> BTreeMap<&'static str, Vec<UsageRef>> {
    let mut by_token: BTreeMap<&'static str, Vec<UsageRef>> = BTreeMap::new();

    for usage in usages {
        for part in usage.parts {
            by_token.entry(part.token).or_default().push((usage.label, part));
        }
    }

    by_token
}

fn render_by_token(
    catalog_tokens: &[CatalogToken],
    by_token: &BTreeMap<&'static str, Vec<UsageRef>>,
    look: &ShadcnLook,
    detail_style: xgpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(section_title("By Token", look, detail_style))
        .children(catalog_tokens.iter().map(|token| {
            let consumers = by_token.get(token.token.as_str());

            div()
                .flex()
                .flex_col()
                .gap(px(7.0))
                .border_1()
                .border_color(chrome.border)
                .rounded(px(6.0))
                .p(px(10.0))
                .child(render_token_header(token, consumers.is_some(), look))
                .children(consumers.into_iter().flat_map(|parts| {
                    parts.iter().map(|(component, part)| {
                        div()
                            .pl(px(44.0))
                            .typography_style(detail_style)
                            .text_color(chrome.body_text)
                            .child(format!("{component} {}", part.part))
                    })
                }))
                .when(consumers.is_none(), |row| {
                    row.child(
                        div()
                            .pl(px(44.0))
                            .typography_style(detail_style)
                            .text_color(chrome.muted_text)
                            .child("No current SDK resolver usage"),
                    )
                })
        }))
        .into_any_element()
}

fn render_by_component(
    usages: &'static [&'static ThemeUsage],
    look: &ShadcnLook,
    detail_style: xgpui_luma::theme::LumaTextStyle,
    caption_style: xgpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(section_title("By Component", look, detail_style))
        .children(usages.iter().map(|usage| {
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .border_1()
                .border_color(chrome.border)
                .rounded(px(6.0))
                .p(px(10.0))
                .child(
                    div()
                        .typography_style(detail_style)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(usage.label),
                )
                .children(usage.parts.iter().map(|part| render_component_part(part, look, detail_style, caption_style)))
        }))
        .into_any_element()
}

fn render_shadow_usage(
    usages: &'static [&'static ThemeUsage],
    look: &ShadcnLook,
    detail_style: xgpui_luma::theme::LumaTextStyle,
    caption_style: xgpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let chrome = look.chrome();
    let by_token = usage_by_token(usages);
    let catalog = look.mode_tokens();
    let shadow_tokens = catalog
        .catalog
        .tokens
        .keys()
        .filter(|token| token.starts_with("shadow") && look.parse_shadow_token(token).is_ok())
        .cloned()
        .collect::<Vec<_>>();

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(section_title("Shadow Tokens", look, detail_style))
        .children(shadow_tokens.into_iter().map(|token| {
            let raw = catalog.catalog.get(token.as_str()).unwrap_or("unresolved").to_string();
            let consumers = by_token.get(token.as_str());
            div()
                .flex()
                .flex_col()
                .gap(px(7.0))
                .border_1()
                .border_color(chrome.border)
                .rounded(px(6.0))
                .p(px(10.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            div()
                                .min_w(px(0.0))
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(1.0))
                                .child(
                                    div()
                                        .truncate()
                                        .font_family("Monaco")
                                        .typography_style(detail_style)
                                        .text_color(chrome.title_text)
                                        .child(format!("--{token}")),
                                )
                                .child(
                                    div()
                                        .truncate()
                                        .font_family("Monaco")
                                        .typography_style(caption_style)
                                        .text_color(chrome.muted_text)
                                        .child(raw),
                                ),
                        )
                        .child(render_status_badge(token_status(consumers.is_some()), look)),
                )
                .children(consumers.into_iter().flat_map(|parts| {
                    parts.iter().map(|(component, part)| {
                        div()
                            .pl(px(44.0))
                            .typography_style(caption_style)
                            .text_color(chrome.body_text)
                            .child(format!("{component} {}", part.part))
                    })
                }))
        }))
        .into_any_element()
}

fn render_shared_values(
    catalog_tokens: &[CatalogToken],
    by_token: &BTreeMap<&'static str, Vec<UsageRef>>,
    look: &ShadcnLook,
    caption_style: xgpui_luma::theme::LumaTextStyle,
    detail_style: xgpui_luma::theme::LumaTextStyle,
    section_style: xgpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let chrome = look.chrome();
    let groups = shared_value_groups(catalog_tokens);

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(section_title("Shared Values", look, section_style))
        .child(div().flex().flex_wrap().gap(px(12.0)).children(groups.into_iter().map(|(value, tokens)| {
            div()
                .w(px(360.0))
                .min_h(px(0.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .border_1()
                .border_color(chrome.border)
                .rounded(px(6.0))
                .p(px(10.0))
                .child(
                    div()
                        .font_family("Monaco")
                        .typography_style(caption_style)
                        .text_color(chrome.muted_text)
                        .child(value),
                )
                .children(tokens.into_iter().map(|token| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().size(px(16.0)).bg(token.color).border_1().border_color(chrome.border))
                        .child(
                            div()
                                .min_w(px(0.0))
                                .truncate()
                                .font_family("Monaco")
                                .typography_style(detail_style)
                                .text_color(chrome.title_text)
                                .child(token_label(token)),
                        )
                        .child(render_status_badge(token_status(by_token.contains_key(token.token.as_str())), look))
                }))
        })))
        .into_any_element()
}

fn shared_value_groups(catalog_tokens: &[CatalogToken]) -> Vec<(String, Vec<&CatalogToken>)> {
    let mut by_value: BTreeMap<String, Vec<&CatalogToken>> = BTreeMap::new();

    for token in catalog_tokens {
        by_value.entry(format_compact_hsla(token.color)).or_default().push(token);
    }

    by_value.into_iter().filter(|(_, tokens)| tokens.len() > 1).collect()
}

fn section_title(title: &'static str, look: &ShadcnLook, style: xgpui_luma::theme::LumaTextStyle) -> AnyElement {
    let chrome = look.chrome();

    div()
        .typography_style(style)
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(chrome.title_text)
        .child(title)
        .into_any_element()
}

fn render_count_badge(
    label: &'static str,
    value: String,
    look: &ShadcnLook,
    style: xgpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(4.0))
        .px(px(8.0))
        .py(px(4.0))
        .typography_style(style)
        .child(div().font_weight(FontWeight::SEMIBOLD).text_color(chrome.title_text).child(value))
        .child(div().text_color(chrome.muted_text).child(label))
        .into_any_element()
}

fn render_token_header(token: &CatalogToken, used_by_sdk: bool, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    let detail_style = look.typography_scale(ShadcnTextSize::Sm);
    let caption_style = look.typography_scale(ShadcnTextSize::Xs);

    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().size(px(34.0)).bg(token.color).border_1().border_color(chrome.border).rounded(px(3.0)))
        .child(
            div()
                .min_w(px(0.0))
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(1.0))
                .child(
                    div()
                        .truncate()
                        .font_family("Monaco")
                        .typography_style(detail_style)
                        .text_color(chrome.title_text)
                        .child(token_label(token)),
                )
                .child(
                    div()
                        .truncate()
                        .font_family("Monaco")
                        .typography_style(caption_style)
                        .text_color(chrome.muted_text)
                        .child(format_compact_hsla(token.color)),
                ),
        )
        .child(render_status_badge(token_status(used_by_sdk), look))
        .into_any_element()
}

fn render_component_part(
    part: &ThemePartUsage,
    look: &ShadcnLook,
    detail_style: xgpui_luma::theme::LumaTextStyle,
    caption_style: xgpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .min_w(px(172.0))
                        .typography_style(detail_style)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.title_text)
                        .child(part.part),
                )
                .child(
                    div()
                        .min_w(px(0.0))
                        .truncate()
                        .font_family("Monaco")
                        .typography_style(detail_style)
                        .text_color(chrome.body_text)
                        .child(format!("--{}", part.token)),
                ),
        )
        .child(div().pl(px(180.0)).typography_style(caption_style).text_color(chrome.muted_text).child(format!(
            "{} -> {}",
            part.states.join(", "),
            part.look_fields.join(", ")
        )))
        .into_any_element()
}

fn render_status_badge(status: &'static str, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    let caption_style = look.typography_scale(ShadcnTextSize::Xs);

    div()
        .flex_none()
        .border_1()
        .border_color(chrome.border)
        .rounded(px(3.0))
        .px(px(6.0))
        .py(px(2.0))
        .typography_style(caption_style)
        .text_color(chrome.muted_text)
        .child(status)
        .into_any_element()
}

fn render_sparse_catalog_callout(look: &ShadcnLook) -> Option<AnyElement> {
    if look.has_css_catalog() {
        return None;
    }

    let chrome = look.chrome();
    let title = "Native default theme — sparse CSS catalog";
    let body = "This view lists tweakcn `--*` custom properties from the active theme. The native default has no CSS catalog, so token swatches and cross-reference counts are empty. SDK controls still resolve colors from the embedded palette. Pick a tweakcn theme in the sidebar for full catalog data.";

    Some(
        div()
            .w_full()
            .max_w(px(860.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .border_1()
            .border_color(chrome.border)
            .rounded(px(10.0))
            .bg(chrome.panel_background)
            .p(px(14.0))
            .child(div().typography_sm().font_weight(FontWeight::SEMIBOLD).text_color(chrome.title_text).child(title))
            .child(div().typography_xs().text_color(chrome.body_text).child(body))
            .into_any_element(),
    )
}

fn token_status(used_by_sdk: bool) -> &'static str {
    if used_by_sdk { "Used by SDK" } else { "No current usage" }
}

fn token_label(token: &CatalogToken) -> String {
    format!("--{}", token.token)
}
