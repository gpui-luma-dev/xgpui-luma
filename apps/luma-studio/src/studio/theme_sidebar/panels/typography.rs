use xgpui_luma::controls::search_selector::SearchSelectorEventExt;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Stateful, Window, div, prelude::*, px};
use xgpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionItem, AccordionTrigger};
use xgpui_luma::controls::search_selector::{
    SearchSelector, SearchSelectorEvent, SearchSelectorItemRenderModel, SearchSelectorItemTemplate,
    SearchSelectorRenderModel, SearchSelectorTemplate, SearchSelectorTemplateHandlers, SelectionItem,
    default_search_selector_template, make_search_selector_item_template,
};
use xgpui_luma::vstack;
use crate::studio::font_catalog::{FontCatalog, FontSlot};
use crate::studio::font_family_match::{clean_css_family_name, match_css_named_family};
use xgpui_luma_look_shadcn::{ShadcnFont, ShadcnLook};
use xgpui_luma_look_shadcn as shadcn;
use lucide_svg_static::Icon as LucideIcon;

use super::super::model::TYPOGRAPHY_CATEGORIES;
use super::{category_item_id, expanded_category_ids};
use crate::studio::app::LumaStudioApp;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FontFamilyRole {
    token: &'static str,
    row_label: &'static str,
    placeholder: &'static str,
    stack_role: FontStackRole,
}

const FONT_FAMILY_ROLES: &[FontFamilyRole] = &[
    FontFamilyRole {
        token: "font-sans",
        row_label: "Sans-Serif",
        placeholder: "Sans-serif font...",
        stack_role: FontStackRole::Sans,
    },
    FontFamilyRole {
        token: "font-serif",
        row_label: "Serif",
        placeholder: "Serif font...",
        stack_role: FontStackRole::Serif,
    },
    FontFamilyRole {
        token: "font-mono",
        row_label: "Mono",
        placeholder: "Mono font...",
        stack_role: FontStackRole::Mono,
    },
];

const ALL_FONTS_DIVIDER_ID: &str = "__all_fonts_divider__";

struct FontRoleControl {
    search_selector: SearchSelector,
    preview_font: SharedString,
    selected_font: Arc<RwLock<SharedString>>,
}

pub struct TypographyPanel {
    look: Arc<ShadcnLook>,
    font_catalog: FontCatalog,
    font_names: Vec<String>,
    sans: FontRoleControl,
    serif: FontRoleControl,
    mono: FontRoleControl,
    typography_accordion: Entity<AccordionControl>,
}

impl TypographyPanel {
    pub fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let font_names = cx.text_system().all_font_names();
        let font_catalog = load_font_catalog();
        let sans = spawn_font_role_control(
            &look,
            &font_catalog,
            "luma-studio-font-sans",
            FONT_FAMILY_ROLES[0],
            &font_names,
            cx,
        );
        let serif = spawn_font_role_control(
            &look,
            &font_catalog,
            "luma-studio-font-serif",
            FONT_FAMILY_ROLES[1],
            &font_names,
            cx,
        );
        let mono = spawn_font_role_control(
            &look,
            &font_catalog,
            "luma-studio-font-mono",
            FONT_FAMILY_ROLES[2],
            &font_names,
            cx,
        );

        let typography_accordion = Self::build_typography_accordion(cx.entity(), look.clone(), &HashSet::new(), cx);

        Self { look, font_catalog, font_names, sans, serif, mono, typography_accordion }
    }

    pub fn wire_subscriptions(panel: &Entity<Self>, cx: &mut Context<LumaStudioApp>) {
        let sans = panel.read(cx).sans.search_selector.clone();
        sans.clone().on_event(cx, |app, event: &SearchSelectorEvent, cx| {
            let item_id = match event {
                SearchSelectorEvent::Select { item_id, .. } | SearchSelectorEvent::Complete { item_id, .. } => item_id,
                _ => return,
            };
            if item_id.as_ref() == ALL_FONTS_DIVIDER_ID {
                return;
            }
            app.set_font_sans(item_id.as_ref(), cx);
        });

        let serif = panel.read(cx).serif.search_selector.clone();
        serif.clone().on_event(cx, |app, event: &SearchSelectorEvent, cx| {
            let item_id = match event {
                SearchSelectorEvent::Select { item_id, .. } | SearchSelectorEvent::Complete { item_id, .. } => item_id,
                _ => return,
            };
            if item_id.as_ref() == ALL_FONTS_DIVIDER_ID {
                return;
            }
            app.set_font_serif(item_id.as_ref(), cx);
        });

        let mono = panel.read(cx).mono.search_selector.clone();
        mono.clone().on_event(cx, |app, event: &SearchSelectorEvent, cx| {
            let item_id = match event {
                SearchSelectorEvent::Select { item_id, .. } | SearchSelectorEvent::Complete { item_id, .. } => item_id,
                _ => return,
            };
            if item_id.as_ref() == ALL_FONTS_DIVIDER_ID {
                return;
            }
            app.set_font_mono(item_id.as_ref(), cx);
        });
    }

    pub fn sync_font_selectors(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        let theme = self.look.clone();
        let catalog = self.font_catalog.clone();

        sync_font_role_control(&theme, &catalog, &self.font_names, FONT_FAMILY_ROLES[0], &mut self.sans, cx);
        sync_font_role_control(&theme, &catalog, &self.font_names, FONT_FAMILY_ROLES[1], &mut self.serif, cx);
        sync_font_role_control(&theme, &catalog, &self.font_names, FONT_FAMILY_ROLES[2], &mut self.mono, cx);
    }

    pub fn apply_theme_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        let expanded = self.expanded_typography_category_ids(cx);
        self.font_names = cx.text_system().all_font_names();
        self.sync_font_selectors(look, cx);

        self.typography_accordion = Self::build_typography_accordion(cx.entity(), self.look.clone(), &expanded, cx);
        cx.notify();
    }

    fn build_typography_accordion(
        panel: Entity<Self>,
        look: Arc<ShadcnLook>,
        expanded_categories: &HashSet<String>,
        cx: &mut Context<Self>,
    ) -> Entity<AccordionControl> {
        let mut accordion_builder = shadcn::Accordion::new("luma-studio-typography-accordion")
            .look(look.as_ref())
            .multiple()
            .item_dividers(false)
            .trigger_min_height(28.0)
            .trigger_padding_y(4.0)
            .content_padding_top(0.0)
            .content_padding_bottom(4.0)
            .template(look.accordion_template());

        for category in TYPOGRAPHY_CATEGORIES {
            let panel = panel.clone();
            let id = category_item_id("typography", category);
            let expanded = if expanded_categories.is_empty() {
                *category == "FONT FAMILY"
            } else {
                expanded_categories.contains(&id)
            };
            accordion_builder = accordion_builder.item(
                AccordionItem::new(
                    id,
                    AccordionTrigger::new(*category),
                    AccordionContent::custom(move |_window, cx| panel.read(cx).font_family_category_content()),
                )
                .expanded(expanded),
            );
        }

        accordion_builder.spawn(cx)
    }

    fn expanded_typography_category_ids(&self, cx: &App) -> HashSet<String> {
        expanded_category_ids(&self.typography_accordion, TYPOGRAPHY_CATEGORIES.iter().copied(), "typography", cx)
    }

    fn font_family_category_content(&self) -> AnyElement {
        let chrome = self.look.chrome();
        let row_label_typography = self.look.mode_tokens().typography.text.label;

        let mut body = vstack! { gap=10; };

        if !self.look.has_css_catalog() {
            body = vstack! {
                gap=10;
                div()
                    .text_xs()
                    .line_height(px(row_label_typography.line_height))
                    .text_color(chrome.muted_text)
                    .child("Native default theme: pick a tweakcn theme above for catalog-backed typography."),
            };
        } else if self.any_role_needs_embedding() {
            body = vstack! {
                gap=10;
                div()
                    .text_xs()
                    .line_height(px(row_label_typography.line_height))
                    .text_color(chrome.muted_text)
                    .child("Custom fonts require embedding. Learn more"),
            };
        }

        for (role, control) in self.font_role_rows() {
            body = vstack! {
                gap=10;
                body,
                self.font_family_row(role, &control.search_selector, row_label_typography, chrome.body_text),
            };
        }

        body.into_any_element()
    }

    fn font_role_rows(&self) -> [(&FontFamilyRole, &FontRoleControl); 3] {
        [
            (&FONT_FAMILY_ROLES[0], &self.sans),
            (&FONT_FAMILY_ROLES[1], &self.serif),
            (&FONT_FAMILY_ROLES[2], &self.mono),
        ]
    }

    fn any_role_needs_embedding(&self) -> bool {
        self.font_role_rows().into_iter().any(|(role, control)| {
            let Some(primary) = self.primary_css_family_name(role.token) else {
                return false;
            };
            !font_name_matches_primary(&control.preview_font, &primary)
        })
    }

    fn primary_css_family_name(&self, token: &str) -> Option<SharedString> {
        self.look.mode_tokens().catalog.get(token).and_then(primary_css_family_name)
    }

    fn font_family_row(
        &self,
        role: &FontFamilyRole,
        search_selector: &SearchSelector,
        label_typography: xgpui_luma::theme::LumaTextStyle,
        label_color: gpui::Hsla,
    ) -> AnyElement {
        vstack! {
            gap=6;
            div()
                .text_size(px(label_typography.size))
                .line_height(px(label_typography.line_height))
                .font_weight(label_typography.weight)
                .text_color(label_color)
                .child(role.row_label),
            div().w_full().child(search_selector.clone()),
        }
        .into_any_element()
    }
}

impl Render for TypographyPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(10.0))
            .child(div().w_full().child(self.typography_accordion.clone()))
    }
}

fn load_font_catalog() -> FontCatalog {
    FontCatalog::load_system().unwrap_or_else(|error| {
        tracing::warn!(?error, "failed to classify system fonts; using flat font lists");
        FontCatalog::default()
    })
}

fn spawn_font_role_control(
    look: &Arc<ShadcnLook>,
    font_catalog: &FontCatalog,
    id: impl Into<SharedString>,
    role: FontFamilyRole,
    font_names: &[String],
    cx: &mut Context<TypographyPanel>,
) -> FontRoleControl {
    let (search_selector, preview_font, selected_font) =
        build_font_search_selector(look, font_catalog, id, role, font_names, cx);
    FontRoleControl { search_selector, preview_font, selected_font }
}

fn sync_font_role_control(
    look: &Arc<ShadcnLook>,
    font_catalog: &FontCatalog,
    font_names: &[String],
    role: FontFamilyRole,
    control: &mut FontRoleControl,
    cx: &mut Context<TypographyPanel>,
) {
    let (selected, items, fallback_label) = font_search_selector_state(look, font_catalog, role, font_names);
    if let Ok(mut selected_font) = control.selected_font.write() {
        *selected_font = selected.clone();
    }
    control.search_selector.update(cx, |search_selector, cx| {
        search_selector.set_placeholder(fallback_label, cx);
        search_selector.set_items(items, cx);
        search_selector.set_selected_id(selected.clone(), cx);
        search_selector.set_item_template(Some(font_item_template(font_names)), cx);
        search_selector.set_enabled(true, cx);
    });
    control.preview_font = selected;
}

fn build_font_search_selector(
    look: &Arc<ShadcnLook>,
    font_catalog: &FontCatalog,
    id: impl Into<SharedString>,
    role: FontFamilyRole,
    font_names: &[String],
    cx: &mut Context<TypographyPanel>,
) -> (SearchSelector, SharedString, Arc<RwLock<SharedString>>) {
    let id = id.into();
    let (selected, items, fallback_label) = font_search_selector_state(look, font_catalog, role, font_names);
    let preview_font = selected.clone();
    let selected_font = Arc::new(RwLock::new(selected.clone()));
    let search_selector = shadcn::SearchSelector::new(id, items)
        .look(look.as_ref())
        .placeholder(fallback_label)
        .search_placeholder("Search fonts...")
        .selected_id(selected)
        .enabled(true)
        .full_width(true)
        .template(font_search_selector_template(selected_font.clone()))
        .with_item_template(font_item_template_builder(font_names))
        .spawn(cx);
    (search_selector, preview_font, selected_font)
}

fn font_search_selector_state(
    look: &ShadcnLook,
    font_catalog: &FontCatalog,
    role: FontFamilyRole,
    font_names: &[String],
) -> (SharedString, Vec<SelectionItem>, SharedString) {
    let primary = look.mode_tokens().catalog.get(role.token).and_then(primary_css_family_name);
    let selected = theme_font_selection_for_role(look, role, font_names);
    let fallback_label = primary.clone().unwrap_or_else(|| SharedString::from(role.placeholder));
    let items = font_search_selector_items(font_catalog, role.stack_role, font_names, primary.as_ref(), &selected);
    (selected, items, fallback_label)
}

fn font_search_selector_items(
    font_catalog: &FontCatalog,
    stack_role: FontStackRole,
    font_names: &[String],
    primary_label: Option<&SharedString>,
    selected_id: &SharedString,
) -> Vec<SelectionItem> {
    let slot = font_slot_for_stack_role(stack_role);
    let picker_names = picker_font_names(font_names);
    let mut items = Vec::new();
    let mut listed = HashSet::new();

    let mut suggested: Vec<&String> =
        picker_names.iter().copied().filter(|name| font_catalog.is_suggested_for_slot(name, slot)).collect();
    suggested.sort_by_key(|left| left.to_ascii_lowercase());

    for name in suggested {
        items.push(font_item(name, primary_label, selected_id));
        listed.insert(name.as_str());
    }

    if !font_catalog.families().is_empty() {
        items.push(SelectionItem::new(ALL_FONTS_DIVIDER_ID, "— All fonts —").enabled(false));
    }

    let mut others: Vec<&String> =
        picker_names.iter().copied().filter(|name| !listed.contains(name.as_str())).collect();
    others.sort_by_key(|left| left.to_ascii_lowercase());

    for name in &others {
        items.push(font_item(name, primary_label, selected_id));
    }

    let selected_is_listed =
        listed.contains(selected_id.as_ref()) || others.iter().any(|name| name.as_str() == selected_id.as_ref());
    if !selected_is_listed && picker_names.iter().any(|name| name.as_str() == selected_id.as_ref()) {
        items.insert(0, font_item(selected_id.as_ref(), primary_label, selected_id));
    }

    items
}

fn font_item(name: &str, primary_label: Option<&SharedString>, selected_id: &SharedString) -> SelectionItem {
    let id = SharedString::from(name.to_string());
    let label = if name == selected_id.as_ref()
        && let Some(primary) = primary_label
        && primary.as_ref() != name
    {
        SharedString::from(format!("{} → {} ✓", primary.as_ref(), name))
    } else if name == selected_id.as_ref() {
        SharedString::from(format!("{} ✓", name))
    } else {
        id.clone()
    };
    SelectionItem::new(id, label).trailing_icon(LucideIcon::Check)
}

fn font_item_template(font_names: &[String]) -> SearchSelectorItemTemplate<SelectionItem> {
    let loaded = loaded_font_names(font_names);
    make_search_selector_item_template(move |item: &SearchSelectorItemRenderModel<'_, SelectionItem>, _cx| {
        font_item_template_element_with_loaded(item, &loaded)
    })
}

fn font_item_template_builder(
    font_names: &[String],
) -> impl for<'a> Fn(&SearchSelectorItemRenderModel<'a, SelectionItem>, &mut App) -> gpui::Div + Send + Sync + 'static {
    let loaded = loaded_font_names(font_names);
    move |item, _cx| font_item_template_element_with_loaded(item, &loaded)
}

fn font_item_template_element_with_loaded(
    item: &SearchSelectorItemRenderModel<'_, SelectionItem>,
    loaded: &HashSet<String>,
) -> gpui::Div {
    let is_loaded = loaded.contains(&item.item.id.to_string().to_ascii_lowercase());
    let label = div()
        .when(item.item.id.as_ref() != ALL_FONTS_DIVIDER_ID, |label| label.font_family(item.item.id.clone()))
        .child(item.item.label.clone());
    let mut content = div().flex().items_center().justify_between().flex_1().child(label);
    if is_loaded {
        content = content.child(div().ml_2().child(xgpui_luma::infra::icon::lucide_icon(
            LucideIcon::Check,
            gpui::hsla(0.38, 0.65, 0.48, 1.0),
            14.0,
        )));
    }
    content
}

/// TypographyPanel-only trigger customization. The SDK's template modifier can
/// style the trigger root, but it cannot observe the selector's current item;
/// keep the resolved, already-registered family in shared app state instead.
fn font_search_selector_template(selected_font: Arc<RwLock<SharedString>>) -> Arc<dyn SearchSelectorTemplate> {
    Arc::new(FontSearchSelectorTemplate { base: default_search_selector_template(), selected_font })
}

struct FontSearchSelectorTemplate {
    base: Arc<dyn SearchSelectorTemplate>,
    selected_font: Arc<RwLock<SharedString>>,
}

impl SearchSelectorTemplate for FontSearchSelectorTemplate {
    fn render(
        &self,
        model: SearchSelectorRenderModel,
        handlers: SearchSelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let mut element = self.base.render(model, handlers, window, cx);
        if let Ok(selected_font) = self.selected_font.read() {
            element = element.font_family(selected_font.clone());
        }
        element
    }
}

fn loaded_font_names(font_names: &[String]) -> HashSet<String> {
    font_names.iter().map(|name| name.to_ascii_lowercase()).collect()
}

fn picker_font_names(font_names: &[String]) -> Vec<&String> {
    font_names.iter().filter(|name| !is_picker_excluded(name)).collect()
}

fn is_picker_excluded(name: &str) -> bool {
    name.starts_with('.') || is_decorative_font_family(name)
}

fn font_slot_for_stack_role(role: FontStackRole) -> FontSlot {
    match role {
        FontStackRole::Sans => FontSlot::Sans,
        FontStackRole::Serif => FontSlot::Serif,
        FontStackRole::Mono => FontSlot::Mono,
    }
}

#[cfg(test)]
fn font_items(
    font_names: &[String],
    primary_label: Option<&SharedString>,
    selected_id: &SharedString,
) -> Vec<SelectionItem> {
    font_names
        .iter()
        .filter(|name| !is_picker_excluded(name))
        .map(|name| font_item(name, primary_label, selected_id))
        .collect()
}

fn font_name_matches_primary(selected: &SharedString, primary: &SharedString) -> bool {
    selected.eq_ignore_ascii_case(primary)
        || selected.as_ref().starts_with(&format!("{} ", primary.as_ref()))
        || primary.eq_ignore_ascii_case(selected)
}

/// First non-generic family in a CSS `font-family` list (tweakcn trigger label).
fn primary_css_family_name(raw: &str) -> Option<SharedString> {
    for family in parse_css_font_stack(raw) {
        if is_decorative_font_family(&family) {
            continue;
        }
        if is_css_system_ui_family(&family) || is_css_generic_family(&family) {
            continue;
        }
        return Some(SharedString::from(family));
    }
    None
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FontStackRole {
    Sans,
    Serif,
    Mono,
}

impl FontStackRole {
    fn shadcn_font(self) -> ShadcnFont {
        match self {
            Self::Sans => ShadcnFont::Sans,
            Self::Serif => ShadcnFont::Serif,
            Self::Mono => ShadcnFont::Mono,
        }
    }

    fn typography_family(self, look: &ShadcnLook) -> String {
        match self {
            Self::Sans => look.mode_tokens().typography.font.sans.family.clone(),
            Self::Serif => look.mode_tokens().typography.font.serif.family.clone(),
            Self::Mono => look.mode_tokens().typography.font.mono.family.clone(),
        }
    }
}

fn theme_font_selection_for_role(look: &ShadcnLook, role: FontFamilyRole, font_names: &[String]) -> SharedString {
    if let Some(raw) = look.mode_tokens().catalog.get(role.token)
        && let Some(matched) = match_css_font_stack(raw, role.stack_role, font_names)
    {
        return matched;
    }

    let resolved = role.stack_role.typography_family(look);
    if !resolved.is_empty()
        && !is_css_generic_family(&resolved)
        && let Some(matched) = match_css_named_family(&resolved, font_names)
    {
        return matched;
    }

    let preferred = look.font(role.stack_role.shadcn_font());
    if !preferred.is_empty()
        && !is_css_generic_family(preferred.as_ref())
        && let Some(matched) = match_css_named_family(preferred.as_ref(), font_names)
    {
        return matched;
    }

    role_default_fallback(role.stack_role, font_names)
        .or_else(|| resolve_system_ui_font(font_names))
        .unwrap_or_else(|| font_names.first().map(|name| SharedString::from(name.clone())).unwrap_or(preferred))
}

fn role_default_fallback(role: FontStackRole, font_names: &[String]) -> Option<SharedString> {
    match role {
        FontStackRole::Sans => resolve_generic_sans_serif(font_names),
        FontStackRole::Serif => resolve_generic_serif(font_names),
        FontStackRole::Mono => resolve_generic_monospace(font_names),
    }
}

fn match_css_font_stack(raw: &str, role: FontStackRole, font_names: &[String]) -> Option<SharedString> {
    for family in parse_css_font_stack(raw) {
        if let Some(resolved) = resolve_css_font_family(&family, role, font_names) {
            return Some(resolved);
        }
    }
    None
}

fn resolve_css_font_family(family: &str, role: FontStackRole, font_names: &[String]) -> Option<SharedString> {
    if is_decorative_font_family(family) {
        return None;
    }

    if is_css_system_ui_family(family) {
        return resolve_system_ui_font(font_names);
    }

    match (role, family) {
        (_, family) if family.eq_ignore_ascii_case("sans-serif") => resolve_generic_sans_serif(font_names),
        (_, family) if family.eq_ignore_ascii_case("serif") => resolve_generic_serif(font_names),
        (_, family) if family.eq_ignore_ascii_case("monospace") => resolve_generic_monospace(font_names),
        (_, family) if is_css_generic_family(family) => None,
        (_, family) => match_css_named_family(family, font_names),
    }
}

fn resolve_system_ui_font(font_names: &[String]) -> Option<SharedString> {
    platform_system_ui_font(font_names).or_else(|| match_css_named_family("System UI", font_names))
}

#[cfg(target_os = "macos")]
fn platform_system_ui_font(font_names: &[String]) -> Option<SharedString> {
    match_css_named_family(".SystemUIFont", font_names)
        .or_else(|| match_css_named_family(".AppleSystemUIFont", font_names))
        .or_else(|| match_css_named_family("SF Pro Text", font_names))
        .or_else(|| match_css_named_family("SF Pro Display", font_names))
}

#[cfg(target_os = "windows")]
fn platform_system_ui_font(font_names: &[String]) -> Option<SharedString> {
    match_css_named_family("Segoe UI", font_names).or_else(|| match_css_named_family("Calibri", font_names))
}

#[cfg(target_os = "linux")]
fn platform_system_ui_font(font_names: &[String]) -> Option<SharedString> {
    match_css_named_family("Ubuntu", font_names)
        .or_else(|| match_css_named_family("Cantarell", font_names))
        .or_else(|| match_css_named_family("Noto Sans", font_names))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_system_ui_font(_font_names: &[String]) -> Option<SharedString> {
    None
}

fn resolve_generic_sans_serif(font_names: &[String]) -> Option<SharedString> {
    resolve_named_fallbacks(CSS_SANS_SERIF_FALLBACKS, font_names)
}

fn resolve_generic_serif(font_names: &[String]) -> Option<SharedString> {
    resolve_named_fallbacks(CSS_SERIF_FALLBACKS, font_names)
}

fn resolve_generic_monospace(font_names: &[String]) -> Option<SharedString> {
    resolve_named_fallbacks(CSS_MONO_FALLBACKS, font_names)
}

fn resolve_named_fallbacks(fallbacks: &[&str], font_names: &[String]) -> Option<SharedString> {
    for fallback in fallbacks {
        if let Some(matched) = match_css_named_family(fallback, font_names) {
            return Some(matched);
        }
    }
    None
}

fn parse_css_font_stack(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|part| part.trim().trim_matches(['\'', '"']))
        .filter(|part| !part.is_empty())
        .map(clean_css_family_name)
        .collect()
}

fn is_css_generic_family(name: &str) -> bool {
    CSS_GENERIC_FAMILIES.iter().any(|generic| name.eq_ignore_ascii_case(generic))
}

fn is_css_system_ui_family(name: &str) -> bool {
    CSS_SYSTEM_UI_FAMILIES.iter().any(|alias| name.eq_ignore_ascii_case(alias))
        || name.eq_ignore_ascii_case("System UI")
}

fn is_decorative_font_family(name: &str) -> bool {
    CSS_DECORATIVE_FAMILIES.iter().any(|decorative| name.eq_ignore_ascii_case(decorative))
}

const CSS_GENERIC_FAMILIES: &[&str] = &[
    "ui-sans-serif",
    "ui-serif",
    "ui-monospace",
    "ui-rounded",
    "system-ui",
    "-apple-system",
    "BlinkMacSystemFont",
    "sans-serif",
    "serif",
    "monospace",
    "cursive",
    "fantasy",
    "math",
    "emoji",
];

const CSS_SYSTEM_UI_FAMILIES: &[&str] = &["ui-sans-serif", "system-ui", "-apple-system", "BlinkMacSystemFont"];

const CSS_DECORATIVE_FAMILIES: &[&str] =
    &["Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"];

const CSS_SANS_SERIF_FALLBACKS: &[&str] = &[
    "Helvetica Neue",
    "Helvetica",
    "Arial",
    "Liberation Sans",
    "Noto Sans",
    "Segoe UI",
    "Roboto",
    "Ubuntu",
    "Cantarell",
    "DejaVu Sans",
];

const CSS_SERIF_FALLBACKS: &[&str] =
    &["New York", "Georgia", "Times New Roman", "Times", "Libre Baskerville", "Palatino", "Baskerville"];

const CSS_MONO_FALLBACKS: &[&str] = &[
    "SF Mono",
    "Menlo",
    "Monaco",
    "Consolas",
    "JetBrains Mono",
    "IBM Plex Mono",
    "Geist Mono",
    "Courier New",
    "DejaVu Sans Mono",
    "Liberation Mono",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_stack_uses_first_available_named_family() {
        let font_names = vec![".SystemUIFont".to_string(), "Inter".to_string(), "Helvetica Neue".to_string()];
        let matched = match_css_font_stack("Inter, sans-serif", FontStackRole::Sans, &font_names).expect("match Inter");
        assert_eq!(matched.as_ref(), "Inter");
    }

    #[test]
    fn css_primary_family_skips_generics_and_system_aliases() {
        let primary = primary_css_family_name("ui-sans-serif, system-ui, 'SF Pro Display', 'Inter', sans-serif")
            .expect("primary family");
        assert_eq!(primary.as_ref(), "SF Pro Display");
    }

    #[test]
    fn css_system_ui_generics_resolve_to_platform_default() {
        #[cfg(target_os = "macos")]
        {
            let font_names = vec![".SystemUIFont".to_string(), "Segoe UI".to_string()];
            let matched = resolve_system_ui_font(&font_names).expect("match system ui");
            assert_eq!(matched.as_ref(), ".SystemUIFont");
        }

        #[cfg(target_os = "windows")]
        {
            let font_names = vec!["Segoe UI".to_string(), "Calibri".to_string()];
            let matched = resolve_system_ui_font(&font_names).expect("match system ui");
            assert_eq!(matched.as_ref(), "Segoe UI");
        }

        #[cfg(target_os = "linux")]
        {
            let font_names = vec!["Ubuntu".to_string(), "Cantarell".to_string(), "Noto Sans".to_string()];
            let matched = resolve_system_ui_font(&font_names).expect("match system ui");
            assert_eq!(matched.as_ref(), "Ubuntu");
        }
    }

    #[test]
    fn css_sans_serif_generic_uses_web_fallback_chain() {
        let font_names = vec!["Helvetica Neue".to_string(), "Arial".to_string()];
        let matched = match_css_font_stack("'Missing Font', sans-serif", FontStackRole::Sans, &font_names)
            .expect("match sans-serif");
        assert_eq!(matched.as_ref(), "Helvetica Neue");
    }

    #[test]
    fn css_mono_stack_resolves_jetbrains_mono() {
        let font_names = vec!["JetBrains Mono".to_string(), "Menlo".to_string()];
        let matched = match_css_font_stack("'JetBrains Mono', monospace", FontStackRole::Mono, &font_names)
            .expect("match mono stack");
        assert_eq!(matched.as_ref(), "JetBrains Mono");
    }

    #[test]
    fn search_selector_item_uses_css_primary_label_when_installed_name_differs() {
        let items = font_items(
            &["Rajdhani Variable".to_string()],
            Some(&SharedString::from("Rajdhani")),
            &SharedString::from("Rajdhani Variable"),
        );
        assert_eq!(items[0].label.as_ref(), "Rajdhani → Rajdhani Variable ✓");
    }

    #[test]
    fn search_selector_lists_suggested_fonts_before_all_fonts_divider() {
        use crate::studio::font_catalog::{FontCatalog, FontFamily, SuggestedRole};

        let catalog = FontCatalog::from_families(vec![
            FontFamily { name: "Arial".to_string(), suggested: SuggestedRole::Sans },
            FontFamily { name: "Georgia".to_string(), suggested: SuggestedRole::Serif },
            FontFamily { name: "Impact".to_string(), suggested: SuggestedRole::Fantasy },
            FontFamily { name: "Menlo".to_string(), suggested: SuggestedRole::Mono },
        ]);
        let font_names = vec!["Arial".to_string(), "Georgia".to_string(), "Impact".to_string(), "Menlo".to_string()];

        let items =
            font_search_selector_items(&catalog, FontStackRole::Sans, &font_names, None, &SharedString::from("Arial"));

        assert_eq!(items[0].id.as_ref(), "Arial");
        assert_eq!(items[1].id.as_ref(), ALL_FONTS_DIVIDER_ID);
        assert!(items.iter().any(|item| item.id.as_ref() == "Georgia"));
        assert!(items.iter().any(|item| item.id.as_ref() == "Menlo"));
        assert!(items.iter().any(|item| item.id.as_ref() == "Impact"));
    }
}
