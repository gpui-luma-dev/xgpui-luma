//! Shadcn card surface: a look-layer styled container, not an SDK control.

use crate::stylesheet::config::FloatingMenuSurfaceElevationRule;
use serde::Deserialize;

use std::sync::Arc;

use gpui::AnyElement;
use gpui::App;
use gpui::AppContext;
use gpui::BoxShadow;
use gpui::Context;
use gpui::Div;
use gpui::Entity;
use gpui::Hsla;
use gpui::IntoElement;
use gpui::Render;
use gpui::SharedString;
use gpui::Stateful;
use gpui::Window;
use gpui::div;
use gpui::prelude::*;
use gpui::px;
use crate::size::ShadcnSize;
use xgpui_luma::theme::LumaTextStyle;
use xgpui_luma::theme::observe_theme_revision;

use crate::look::ShadcnLook;

#[derive(Clone, Debug)]
pub struct CardLook {
    pub background: Hsla,
    pub border: Hsla,
    pub title_color: Hsla,
    pub description_color: Hsla,
    pub body_color: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub section_gap: f32,
    pub header_gap: f32,
    pub body_gap: f32,
    pub title: LumaTextStyle,
    pub description: LumaTextStyle,
    pub body: LumaTextStyle,
    pub font_family: SharedString,
}

/// The card from `.lss` (`style/card.lss`) at `size`.
pub fn card_look(theme: &ShadcnLook, size: ShadcnSize) -> CardLook {
    let look = theme.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "card",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || look.style_theme())),
    );
    let binding = if size != ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    let card = xgpui_luma::controls::card::StyledCard::new(binding).look();
    CardLook {
        background: card.background.unwrap_or_else(gpui::transparent_black),
        border: card.border.unwrap_or_else(gpui::transparent_black),
        title_color: card.title_color,
        description_color: card.description_color,
        body_color: card.body_color,
        shadow: card.shadow,
        radius: card.radius,
        padding: card.padding,
        section_gap: card.section_gap,
        header_gap: card.header_gap,
        body_gap: card.body_gap,
        title: card.title,
        description: card.description,
        body: card.body,
        font_family: card.font_family,
    }
}

pub type CardElementRenderer = Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + Send + Sync>;

#[derive(Clone)]
struct ShadcnCardConfig {
    id: SharedString,
    size: ShadcnSize,
    title: Option<SharedString>,
    description: Option<SharedString>,
    header: Option<CardElementRenderer>,
    body: Vec<CardElementRenderer>,
    footer: Option<CardElementRenderer>,
    elevated: bool,
    full_height: bool,
    body_fill: bool,
}

/// Builder in the guise of a card: Shadcn surface plus content slots, until `.spawn(cx)` / `.render(...)`.
pub struct Card {
    look: Option<ShadcnLook>,
    config: ShadcnCardConfig,
}

impl Card {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            config: ShadcnCardConfig {
                id: id.into(),
                size: ShadcnSize::Md,
                title: None,
                description: None,
                header: None,
                body: Vec::new(),
                footer: None,
                elevated: true,
                full_height: false,
                body_fill: false,
            },
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        crate::look::resolve_look(self.look.as_ref(), cx.try_global::<ShadcnLook>())
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.config.size = size;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.config.description = Some(description.into());
        self
    }

    pub fn header(mut self, header: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        self.config.header = Some(Arc::new(header));
        self
    }

    pub fn header_element(mut self, header: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        self.config.header = Some(renderer_from_element(header));
        self
    }

    pub fn child(mut self, child: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        self.config.body.push(renderer_from_element(child));
        self
    }

    pub fn child_render(mut self, child: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        self.config.body.push(Arc::new(child));
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = CardElementRenderer>) -> Self {
        self.config.body.extend(children);
        self
    }

    pub fn footer(mut self, footer: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        self.config.footer = Some(Arc::new(footer));
        self
    }

    pub fn footer_element(mut self, footer: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        self.config.footer = Some(renderer_from_element(footer));
        self
    }

    pub fn elevated(mut self, elevated: bool) -> Self {
        self.config.elevated = elevated;
        self
    }

    pub fn full_height(mut self, full_height: bool) -> Self {
        self.config.full_height = full_height;
        self
    }

    pub fn body_fill(mut self, body_fill: bool) -> Self {
        self.config.body_fill = body_fill;
        self
    }

    pub fn render(self, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let look = self.resolve_look(cx);
        render_card(&look, &self.config, window, cx)
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<ShadcnCard> {
        let look = self.resolve_look(cx);
        cx.new(|cx| ShadcnCard::from_config(look, self.config, cx))
    }
}

pub struct ShadcnCard {
    look: ShadcnLook,
    config: ShadcnCardConfig,
}

impl ShadcnCard {
    fn from_config(look: ShadcnLook, config: ShadcnCardConfig, cx: &mut Context<Self>) -> Self {
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self { look, config }
    }
}

impl Render for ShadcnCard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        render_card(&self.look, &self.config, window, cx)
    }
}

fn render_card(look: &ShadcnLook, config: &ShadcnCardConfig, window: &mut Window, cx: &mut App) -> Stateful<Div> {
    let card_look = card_look(look, config.size);

    let mut root = div()
        .id(config.id.clone())
        .relative()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(card_look.section_gap))
        .rounded(px(card_look.radius))
        .border_1()
        .border_color(card_look.border)
        .overflow_hidden()
        .bg(card_look.background)
        .p(px(card_look.padding))
        .text_color(card_look.body_color)
        .font_family(card_look.font_family.clone())
        .text_size(px(card_look.body.size))
        .line_height(px(card_look.body.line_height))
        .font_weight(card_look.body.weight);

    if config.elevated {
        root = root.shadow(card_look.shadow.clone());
    }

    if config.full_height {
        root = root.h_full().min_h(px(0.0));
    }

    if let Some(header) = &config.header {
        root = root.child(header(window, cx));
    } else if config.title.is_some() || config.description.is_some() {
        let mut header = div().w_full().flex().flex_col().gap(px(card_look.header_gap));

        if let Some(title) = &config.title {
            header = header.child(
                div()
                    .text_size(px(card_look.title.size))
                    .line_height(px(card_look.title.line_height))
                    .font_weight(card_look.title.weight)
                    .text_color(card_look.title_color)
                    .child(title.clone()),
            );
        }

        if let Some(description) = &config.description {
            header = header.child(
                div()
                    .text_size(px(card_look.description.size))
                    .line_height(px(card_look.description.line_height))
                    .font_weight(card_look.description.weight)
                    .text_color(card_look.description_color)
                    .child(description.clone()),
            );
        }

        root = root.child(header);
    }

    if !config.body.is_empty() {
        let mut body = div().w_full().flex().flex_col().gap(px(card_look.body_gap));

        if config.body_fill {
            body = body.flex_1().min_h(px(0.0));
        }

        for child in &config.body {
            body = body.child(child(window, cx));
        }

        root = root.child(body);
    }

    if let Some(footer) = &config.footer {
        root = root.child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_end()
                .text_color(card_look.body_color)
                .child(footer(window, cx)),
        );
    }

    root
}

fn renderer_from_element(element: impl IntoElement + Clone + Send + Sync + 'static) -> CardElementRenderer {
    Arc::new(move |_, _| element.clone().into_any_element())
}

#[cfg(test)]
mod tests {
    use crate::size::ShadcnSize;

    use crate::look::ShadcnLook;
    use crate::tokens::ShadcnToken;

    use super::card_look;

    fn sample_look() -> ShadcnLook {
        ShadcnLook::from_css_str(
            ":root { \
                --background: oklch(0.9735 0.0261 90.0953); \
                --foreground: oklch(0.3092 0.0518 219.6516); \
                --card: oklch(0.9306 0.0260 92.4020); \
                --card-foreground: oklch(0.3092 0.0518 219.6516); \
                --muted: oklch(0.90 0.01 220); \
                --muted-foreground: oklch(0.45 0.02 220); \
                --primary: oklch(0.5924 0.2025 355.8943); \
                --primary-foreground: oklch(1 0 0); \
                --secondary: oklch(0.6437 0.1019 187.3840); \
                --secondary-foreground: oklch(1 0 0); \
                --accent: oklch(0.90 0.05 250); \
                --accent-foreground: oklch(0.20 0.05 250); \
                --border: oklch(0.6537 0.0197 205.2618); \
                --input: oklch(0.6537 0.0197 205.2618); \
                --ring: oklch(0.5924 0.2025 355.8943); \
                --shadow-xs: 0 1px 3px 0px hsl(0 0% 0% / 0.05); \
                --shadow: 0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 1px 2px -1px hsl(0 0% 0% / 0.10); \
            } \
            .dark { \
                --background: oklch(0.205 0 0); \
                --foreground: oklch(0.985 0 0); \
                --card: oklch(0.205 0 0); \
                --card-foreground: oklch(0.985 0 0); \
                --muted: oklch(0.30 0 0); \
                --muted-foreground: oklch(0.75 0 0); \
                --primary: oklch(0.5924 0.2025 355.8943); \
                --primary-foreground: oklch(1 0 0); \
                --secondary: oklch(0.6437 0.1019 187.3840); \
                --secondary-foreground: oklch(1 0 0); \
                --accent: oklch(0.90 0.05 250); \
                --accent-foreground: oklch(0.20 0.05 250); \
                --border: oklch(0.40 0 0); \
                --input: oklch(0.40 0 0); \
                --ring: oklch(0.5924 0.2025 355.8943); \
                --shadow-xs: 0 1px 3px 0px hsl(0 0% 0% / 0.05); \
                --shadow: 0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 1px 2px -1px hsl(0 0% 0% / 0.10); \
            }",
        )
        .expect("look")
    }

    #[test]
    fn card_resolves_stylesheet_shadow() {
        let look = card_look(&sample_look(), ShadcnSize::Md);
        assert!(!look.shadow.is_empty());
    }

    #[test]
    fn card_uses_card_tokens() {
        let shadcn = sample_look();
        let look = card_look(&shadcn, ShadcnSize::Md);

        crate::assert_close(look.background, shadcn.color(ShadcnToken::Card));
        crate::assert_close(look.title_color, shadcn.color(ShadcnToken::CardForeground));
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct CardStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<FloatingMenuSurfaceElevationRule>,
    #[serde(default)]
    pub color_rule: Option<CardColorRule>,
}

impl CardStylesheet {
    /// Native bundled card surface.
    pub fn native_default() -> Self {
        Self {
            elevation_rules: vec![FloatingMenuSurfaceElevationRule { shadow: "shadow-xs".into() }],
            color_rule: Some(CardColorRule {
                background: "card".into(),
                foreground: "card-foreground".into(),
                muted_foreground: "muted-foreground".into(),
                border: "border".into(),
            }),
        }
    }

    pub fn elevation_rule(&self) -> Option<&FloatingMenuSurfaceElevationRule> {
        self.elevation_rules.first()
    }

    pub fn color_rule(&self) -> Option<&CardColorRule> {
        self.color_rule.as_ref()
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct CardColorRule {
    pub background: String,
    pub foreground: String,
    pub muted_foreground: String,
    pub border: String,
}

#[derive(Clone, Debug)]
pub struct CardMetricTable {
    pub padding: crate::ResolvedMetric,
    pub section_gap: crate::ResolvedMetric,
    pub header_gap: crate::ResolvedMetric,
    pub body_gap: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
}
