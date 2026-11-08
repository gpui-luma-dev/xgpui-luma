use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use gpui::{
    AnyElement, App, Hsla, MouseButton, Pixels, SharedString, Stateful, Window, anchored, deferred, div, point,
    prelude::*, px, transparent_black,
};
use xgpui_luma::infra::icon::{DisclosureIcons, render_disclosure_icon};
use xgpui_luma::controls::search_selector::{
    SearchSelectorItemRenderModel, SearchSelectorPanelRenderModel, SearchSelectorPanelTemplate,
    SearchSelectorRenderModel, SearchSelectorTemplate, SearchSelectorTemplateHandlers, SelectionItem,
    default_search_selector_panel_template,
};
use xgpui_luma::controls::selector::SelectorTheme;
use xgpui_luma::theme::{InteractionState, StandardBoxScale};
use xgpui_luma_look_shadcn::ShadcnLook;
use luma_app_common::{BuiltInTheme, built_in_look, fallback_look};
use lucide_svg_static::Icon as LucideIcon;

use crate::theme::available_themes;

const SWATCH_SIZE: f32 = 20.0;
const SWATCH_RADIUS: f32 = 4.0;
const SWATCH_GAP: f32 = 6.0;
const LABEL_GAP: f32 = 14.0;
const POPUP_MIN_WIDTH: f32 = 300.0;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ThemeSwatches {
    pub primary_background: Hsla,
    pub accent_background: Hsla,
    pub secondary_background: Hsla,
    pub border: Hsla,
}

pub(crate) struct ThemeSelectorSwatchCache {
    swatches: HashMap<SharedString, ThemeSwatches>,
}

impl ThemeSelectorSwatchCache {
    pub fn empty() -> Self {
        Self { swatches: HashMap::new() }
    }

    pub fn get(&self, id: &SharedString) -> Option<ThemeSwatches> {
        self.swatches.get(id).copied()
    }
}

pub(crate) fn theme_selector_state(current_look: &ShadcnLook) -> (Vec<SelectionItem>, ThemeSelectorSwatchCache) {
    let mut swatches = HashMap::new();

    let default_id = SharedString::from("default");
    swatches.insert(default_id.clone(), swatches_for_look(&fallback_look_for_mode(current_look)));
    let mut items = vec![SelectionItem::new(default_id, "Default")];

    for theme in available_themes() {
        if let Some((id, label, theme_swatches)) = built_in_theme_selector_entry(theme, current_look) {
            swatches.insert(id.clone(), theme_swatches);
            items.push(SelectionItem::new(id, label));
        }
    }

    (items, ThemeSelectorSwatchCache { swatches })
}

fn built_in_theme_selector_entry(
    theme: &BuiltInTheme,
    current_look: &ShadcnLook,
) -> Option<(SharedString, SharedString, ThemeSwatches)> {
    let look = built_in_look(theme.id).ok()?;
    look.set_mode(current_look.mode());
    Some((SharedString::from(theme.id), SharedString::from(theme.display_name()), swatches_for_look(&look)))
}

fn fallback_look_for_mode(current_look: &ShadcnLook) -> ShadcnLook {
    let look = fallback_look();
    look.set_mode(current_look.mode());
    look
}

fn swatches_for_look(look: &ShadcnLook) -> ThemeSwatches {
    let palette = look.mode_tokens().palette;
    ThemeSwatches {
        primary_background: palette.primary.background,
        accent_background: palette.accent_background,
        secondary_background: palette.secondary.background,
        border: palette.border_default,
    }
}

pub(crate) fn theme_search_selector_template(
    look: &Arc<ShadcnLook>,
    swatches: Arc<RwLock<ThemeSelectorSwatchCache>>,
    selected_id: Arc<RwLock<SharedString>>,
) -> Arc<dyn SearchSelectorTemplate> {
    Arc::new(ThemeSearchSelectorTemplate { selector_theme: look.selector_theme(), swatches, selected_id })
}

struct ThemeSearchSelectorTemplate {
    selector_theme: Arc<dyn SelectorTheme>,
    swatches: Arc<RwLock<ThemeSelectorSwatchCache>>,
    selected_id: Arc<RwLock<SharedString>>,
}

impl SearchSelectorTemplate for ThemeSearchSelectorTemplate {
    fn render(
        &self,
        model: SearchSelectorRenderModel,
        handlers: SearchSelectorTemplateHandlers,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let SearchSelectorTemplateHandlers {
            key_down,
            scroll_wheel,
            trigger_click,
            trigger_hover,
            trigger_mouse_down,
            trigger_mouse_up,
            trigger_mouse_up_out,
            trigger_bounds,
        } = handlers;

        let interaction = selector_interaction(&model);
        let look = self.selector_theme.resolve_look(
            xgpui_luma::controls::selector::SelectorTriggerStyle::Outline,
            interaction,
            model.size,
            &StandardBoxScale::compute(model.size, &self.selector_theme.metrics(), window.scale_factor()),
            false,
        );
        let selected_id = self.selected_id.read().expect("theme selector selected id lock").clone();
        let selected_swatches = self.swatches.read().expect("theme selector swatches lock").get(&selected_id);

        let trigger_content = if model.trigger_label_is_placeholder {
            render_theme_trigger_content(None, &model.trigger_label)
        } else {
            render_theme_trigger_content(selected_swatches, &model.trigger_label)
        };

        let mut trigger = div()
            .id(format!("{}-trigger", model.id))
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h_full()
            .bg(transparent_black())
            .cursor_pointer()
            .relative()
            .text_color(look.trigger_foreground)
            .text_size(px(look.trigger_typography.size))
            .line_height(px(look.trigger_typography.line_height))
            .font_weight(look.trigger_typography.weight)
            .on_hover(trigger_hover)
            .on_mouse_down(MouseButton::Left, trigger_mouse_down)
            .on_mouse_up(MouseButton::Left, trigger_mouse_up)
            .on_mouse_up_out(MouseButton::Left, trigger_mouse_up_out)
            .on_click(trigger_click)
            .child(trigger_content)
            .child(div().flex().items_center().justify_end().flex_shrink_0().text_color(look.trigger_icon).child(
                render_disclosure_icon(
                    &DisclosureIcons::new(LucideIcon::ChevronUp, LucideIcon::ChevronDown),
                    model.disclosure_progress,
                    look.trigger_icon,
                    look.trigger_icon_size,
                ),
            ));

        if !model.enabled {
            trigger = trigger.opacity(0.56);
        }

        div()
            .id(format!("{}-root", model.id))
            .when(model.full_width, |root| root.w_full().h_full())
            .flex()
            .flex_col()
            .on_key_down(key_down)
            .on_scroll(scroll_wheel)
            .child(
                div()
                    .on_children_prepainted(move |bounds, window, cx| {
                        if let Some(bounds) = bounds.first() {
                            trigger_bounds(bounds, window, cx);
                        }
                    })
                    .flex()
                    .items_center()
                    .when(model.full_width, |row| row.w_full().h_full())
                    .when(!model.full_width, |row| row.min_w(model.minimum_trigger_width))
                    .relative()
                    .child(trigger),
            )
            .when_some(model.popup_content, |root, popup_content| root.child(popup_content))
    }
}

fn selector_interaction(model: &SearchSelectorRenderModel) -> InteractionState {
    InteractionState {
        // The theme picker keeps its trigger visually neutral while retaining
        // hover events for the selector interaction lifecycle.
        hovered: false,
        pressed: false,
        focused: model.trigger_state.focused || model.trigger_state.focus_visible,
        disabled: !model.enabled,
        invalid: false,
    }
}

pub(crate) fn render_theme_search_selector_item(
    model: &SearchSelectorItemRenderModel<'_, SelectionItem>,
    swatches: &Arc<RwLock<ThemeSelectorSwatchCache>>,
    _cx: &mut App,
) -> AnyElement {
    render_theme_selector_row(
        swatches.read().expect("theme selector swatches lock").get(&model.item.id),
        &model.item.label,
    )
}

fn render_theme_trigger_content(swatches: Option<ThemeSwatches>, label: &SharedString) -> AnyElement {
    let swatch_size = px(SWATCH_SIZE);
    let swatch_radius = px(SWATCH_RADIUS);
    let swatch_gap = px(SWATCH_GAP);
    let label_gap = px(LABEL_GAP);

    let swatch_row = swatches.map(|swatches| {
        div().flex().flex_shrink_0().items_center().gap(swatch_gap).children(
            theme_swatch_colors(swatches)
                .into_iter()
                .map(|color| render_theme_swatch(color, swatch_size, swatch_radius)),
        )
    });

    div()
        .flex()
        .items_center()
        .justify_start()
        .gap(label_gap)
        .min_w(px(0.0))
        .flex_1()
        .overflow_hidden()
        .when_some(swatch_row, |row, swatches| row.child(swatches))
        .child(div().truncate().child(label.clone()))
        .into_any_element()
}

fn render_theme_selector_row(swatches: Option<ThemeSwatches>, label: &SharedString) -> AnyElement {
    let swatch_size = px(SWATCH_SIZE);
    let swatch_radius = px(SWATCH_RADIUS);
    let swatch_gap = px(SWATCH_GAP);
    let label_gap = px(LABEL_GAP);

    let swatch_row = swatches.map(|swatches| {
        div().flex().flex_shrink_0().items_center().gap(swatch_gap).children(
            theme_swatch_colors(swatches)
                .into_iter()
                .map(|color| render_theme_swatch(color, swatch_size, swatch_radius)),
        )
    });

    div()
        .w_full()
        .flex()
        .items_center()
        .gap(label_gap)
        .when_some(swatch_row, |row, swatches| row.child(swatches))
        .child(div().flex_1().min_w(px(0.0)).overflow_hidden().child(label.clone()))
        .into_any_element()
}

/// Theme picker order: primary background, accent background, secondary background, border.
fn theme_swatch_colors(swatches: ThemeSwatches) -> [Hsla; 4] {
    [
        swatches.primary_background,
        swatches.accent_background,
        swatches.secondary_background,
        swatches.border,
    ]
}

fn render_theme_swatch(color: Hsla, size: Pixels, radius: Pixels) -> impl IntoElement {
    div().size(size).flex_shrink_0().rounded(radius).bg(color)
}

pub(crate) fn theme_search_selector_panel_template() -> Arc<dyn SearchSelectorPanelTemplate> {
    Arc::new(ThemeSearchSelectorPanelTemplate { inner: default_search_selector_panel_template() })
}

struct ThemeSearchSelectorPanelTemplate {
    inner: Arc<dyn SearchSelectorPanelTemplate>,
}

impl SearchSelectorPanelTemplate for ThemeSearchSelectorPanelTemplate {
    fn render(&self, model: SearchSelectorPanelRenderModel<'_>, cx: &mut App) -> AnyElement {
        let Some(bounds) = model.popup_bounds else {
            return self.inner.render(model, cx);
        };

        let look = model.popup_look;
        let popup_width = bounds.size.width.max(px(POPUP_MIN_WIDTH));

        let panel_content = div()
            .id(format!("{}-panel-content", model.id))
            .w_full()
            .when_some(model.search_content, |panel, search| {
                panel.child(div().p(px(8.0)).child(search)).child(div().h(px(1.0)).bg(look.border))
            })
            .child(model.list_content);

        deferred(
            anchored()
                .snap_to_window_with_margin(px(8.0))
                .anchor(gpui::Anchor::TopLeft)
                .position(point(bounds.left(), bounds.bottom()))
                .offset(point(px(0.0), px(4.0)))
                .child(
                    div()
                        .id(format!("{}-popup-shell", model.id))
                        .w(popup_width)
                        .bg(look.background)
                        .border_1()
                        .border_color(look.border)
                        .rounded(px(look.radius))
                        .shadow(look.shadow)
                        .overflow_hidden()
                        .occlude()
                        .child(panel_content),
                ),
        )
        .with_priority(1)
        .into_any_element()
    }
}
