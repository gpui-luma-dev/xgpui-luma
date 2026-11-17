use crate::infra::locale::{LayoutDirection, LogicalStyled};
use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, ClickEvent, Div, SharedString, Stateful, Window, div, px, relative, prelude::*, svg};
use crate::infra::menu_item::{MenuItem, MenuItemIcon};
use crate::motion::overlay_presence::OverlayPresence;
use crate::infra::state::MenuPath;
use crate::controls::floating_menu::FloatingMenuLook;
use crate::infra::icon::DisclosureIcons;

pub type FloatingMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type FloatingMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug)]
pub struct FloatingMenuHighlight {
    pub from: MenuPath,
    pub to: MenuPath,
    pub progress: f32,
}

pub struct FloatingMenuRenderModel<'a> {
    pub id: &'a SharedString,
    /// Direction of menu chrome and submenu opening; item artwork is unchanged.
    pub direction: LayoutDirection,
    pub items: &'a [MenuItem],
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
    pub highlight: Option<FloatingMenuHighlight>,
    pub submenu_presence: OverlayPresence,
    pub look: FloatingMenuLook,
    pub disclosure_icons: &'a DisclosureIcons,
    pub submenu_transition: Option<(usize, f32)>,
}

pub struct FloatingMenuTemplateHandlers {
    pub item_hovers: Vec<FloatingMenuHoverHandler>,
    pub submenu_hovers: Vec<Vec<FloatingMenuHoverHandler>>,
    pub controlled_hover: bool,
    pub item_clicks: Vec<FloatingMenuClickHandler>,
}

pub type FloatingMenuTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &FloatingMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait FloatingMenuTemplate: Send + Sync {
    fn render(&self, model: &FloatingMenuRenderModel<'_>, handlers: FloatingMenuTemplateHandlers) -> Stateful<Div>;
}

/// Visual structure for a noninteractive menu divider. Its outer height must match
/// `separator_thickness + 2 * separator_spacing` for highlight and submenu alignment.
pub trait FloatingMenuSeparatorTemplate: Send + Sync {
    fn render(&self, item: &MenuItem, look: &FloatingMenuLook) -> Stateful<Div>;
}

#[derive(Default)]
pub struct ThemedFloatingMenuSeparatorTemplate;

impl FloatingMenuSeparatorTemplate for ThemedFloatingMenuSeparatorTemplate {
    fn render(&self, item: &MenuItem, look: &FloatingMenuLook) -> Stateful<Div> {
        div()
            .id((item.id().clone(), 0usize))
            .h(px(separator_height(look)))
            .py(px(look.separator_spacing))
            .px(px(look.separator_inset))
            .child(div().h(px(look.separator_thickness)).bg(look.separator_color))
    }
}

pub struct ThemedFloatingMenuTemplate {
    separator_template: Arc<dyn FloatingMenuSeparatorTemplate>,
    modifiers: Vec<FloatingMenuTemplateModifier>,
}

impl ThemedFloatingMenuTemplate {
    pub fn new() -> Self {
        Self { separator_template: Arc::new(ThemedFloatingMenuSeparatorTemplate), modifiers: Vec::new() }
    }

    /// Customize divider structure while retaining the shared menu behavior.
    pub fn with_separator_template(mut self, template: Arc<dyn FloatingMenuSeparatorTemplate>) -> Self {
        self.separator_template = template;
        self
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &FloatingMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &FloatingMenuRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

impl Default for ThemedFloatingMenuTemplate {
    fn default() -> Self {
        Self::new()
    }
}

struct ModifiedFloatingMenuTemplate {
    base: Arc<dyn FloatingMenuTemplate>,
    modifiers: Vec<FloatingMenuTemplateModifier>,
}

impl ModifiedFloatingMenuTemplate {
    fn new(base: Arc<dyn FloatingMenuTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: FloatingMenuTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &FloatingMenuRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

impl FloatingMenuTemplate for ThemedFloatingMenuTemplate {
    fn render(&self, model: &FloatingMenuRenderModel<'_>, handlers: FloatingMenuTemplateHandlers) -> Stateful<Div> {
        let FloatingMenuTemplateHandlers { item_hovers, mut submenu_hovers, controlled_hover, item_clicks } = handlers;
        let look = &model.look;
        let direction = model.direction;

        let mut menu = div()
            .id((model.id.clone(), 0usize))
            .relative()
            .min_w(px(look.min_width))
            .p(px(look.padding))
            .bg(look.background)
            .border(px(look.border_width))
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow.clone())
            .occlude();

        let highlight_rect = pane_highlight(model.highlight, None, controlled_hover, model.items, look);
        if let Some(rect) = highlight_rect {
            menu = menu.child(render_highlight(rect, look));
        }

        let mut item_clicks = item_clicks.into_iter();
        let mut submenu = None;

        for ((index, item), item_hover) in model.items.iter().enumerate().zip(item_hovers) {
            if item.is_separator() {
                menu = menu.child(self.separator_template.render(item, look));
                continue;
            }
            let enabled = item.is_enabled();
            let is_active = matches!(model.active_path, Some(MenuPath::Root(active)) if active == index);
            let paint = row_paint(enabled, is_active, controlled_hover, highlight_rect.is_some(), look);
            let color = paint.foreground;
            let mut row = div()
                .id((item.id().clone(), 0usize))
                .debug_selector({
                    let id = item.id().clone();
                    move || format!("floating-row-{id}")
                })
                .flex()
                .flex_row_logical(direction)
                .items_center()
                .gap(px(look.item_gap))
                .min_h(px(look.item_height))
                .px(px(look.item_padding_x))
                .rounded(px(look.item_radius))
                .text_color(color)
                .text_size(px(look.item_typography.size))
                .line_height(px(look.item_typography.line_height))
                .font_weight(look.item_typography.weight)
                .child(
                    div()
                        .debug_selector({
                            let id = item.id().clone();
                            move || format!("floating-icon-{id}")
                        })
                        .child(render_item_icon(item.icon_ref(), color, look.item_icon_size)),
                )
                .child(
                    div()
                        .debug_selector({
                            let id = item.id().clone();
                            move || format!("floating-label-{id}")
                        })
                        .flex_1()
                        .when(direction == LayoutDirection::Rtl, |label| label.text_right())
                        .child(item.label_text().clone()),
                );

            if enabled {
                row = row.cursor_pointer().on_hover(item_hover).child(render_submenu_affordance(
                    !item.submenu_items().is_empty(),
                    color,
                    look.item_icon_size,
                    model.disclosure_icons,
                    submenu_transition_progress(index, model.open_submenu, model.submenu_transition),
                    direction,
                ));

                if item.submenu_items().is_empty() {
                    if let Some(item_click) = item_clicks.next() {
                        row = row.on_click(item_click);
                    }
                } else if model.open_submenu == Some(index) {
                    submenu = Some(render_floating_submenu(
                        model.items,
                        self.separator_template.as_ref(),
                        item,
                        look,
                        &mut item_clicks,
                        index,
                        model.active_path,
                        (index < submenu_hovers.len()).then(|| std::mem::take(&mut submenu_hovers[index])),
                        controlled_hover,
                        model.highlight,
                        model.submenu_presence.opacity(),
                        model.disclosure_icons,
                        direction,
                    ));
                }
            } else {
                row = row.opacity(look.disabled_opacity).child(render_submenu_affordance(
                    !item.submenu_items().is_empty(),
                    color,
                    look.item_icon_size,
                    model.disclosure_icons,
                    submenu_transition_progress(index, model.open_submenu, model.submenu_transition),
                    direction,
                ));
            }

            menu = menu.child(apply_row_paint(row, paint));
        }

        if let Some(submenu) = submenu {
            menu = menu.child(submenu);
        }

        self.apply_modifiers(menu, model)
    }
}

impl FloatingMenuTemplate for ModifiedFloatingMenuTemplate {
    fn render(&self, model: &FloatingMenuRenderModel<'_>, handlers: FloatingMenuTemplateHandlers) -> Stateful<Div> {
        let root = self.base.render(model, handlers);
        self.apply_modifiers(root, model)
    }
}

pub fn default_floating_menu_template() -> Arc<dyn FloatingMenuTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn FloatingMenuTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedFloatingMenuTemplate::new())).clone()
}

pub fn floating_menu_template_with_modifier<F>(
    template: Arc<dyn FloatingMenuTemplate>,
    modifier: F,
) -> Arc<dyn FloatingMenuTemplate>
where
    F: Fn(Stateful<Div>, &FloatingMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedFloatingMenuTemplate::new(template).with_modifier(Box::new(modifier)))
}

pub fn render_floating_menu(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> Stateful<Div> {
    render_floating_menu_with_template(
        default_floating_menu_template(),
        id,
        items,
        open_submenu,
        active_path,
        look,
        item_hovers,
        item_clicks,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_floating_menu_with_submenu_presence(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    submenu_presence: OverlayPresence,
) -> Stateful<Div> {
    render_floating_menu_with_submenu_presence_and_transition(
        id,
        items,
        open_submenu,
        active_path,
        look,
        item_hovers,
        item_clicks,
        submenu_presence,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_floating_menu_with_submenu_presence_and_transition(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    submenu_presence: OverlayPresence,
    submenu_transition: Option<(usize, f32)>,
) -> Stateful<Div> {
    render_floating_menu_with_template_and_submenu_presence(
        default_floating_menu_template(),
        id,
        items,
        open_submenu,
        active_path,
        look,
        item_hovers,
        item_clicks,
        submenu_presence,
        submenu_transition,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_floating_menu_with_template(
    template: Arc<dyn FloatingMenuTemplate>,
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> Stateful<Div> {
    let disclosure_icons = DisclosureIcons::default();
    template.render(
        &FloatingMenuRenderModel {
            direction: LayoutDirection::Ltr,
            id,
            items,
            open_submenu,
            active_path,
            highlight: None,
            submenu_presence: OverlayPresence::new(true, false),
            look,
            disclosure_icons: &disclosure_icons,
            submenu_transition: None,
        },
        FloatingMenuTemplateHandlers { item_hovers, submenu_hovers: Vec::new(), controlled_hover: false, item_clicks },
    )
}

#[allow(clippy::too_many_arguments)]
fn render_floating_menu_with_template_and_submenu_presence(
    template: Arc<dyn FloatingMenuTemplate>,
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    submenu_presence: OverlayPresence,
    submenu_transition: Option<(usize, f32)>,
) -> Stateful<Div> {
    template.render(
        &FloatingMenuRenderModel {
            direction: LayoutDirection::Ltr,
            id,
            items,
            open_submenu,
            active_path,
            highlight: None,
            submenu_presence,
            look,
            disclosure_icons: &DisclosureIcons::default(),
            submenu_transition,
        },
        FloatingMenuTemplateHandlers { item_hovers, submenu_hovers: Vec::new(), controlled_hover: false, item_clicks },
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_floating_menu_with_submenu_hovers(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    submenu_hovers: Vec<Vec<FloatingMenuHoverHandler>>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    highlight: Option<FloatingMenuHighlight>,
) -> Stateful<Div> {
    render_floating_menu_with_submenu_hovers_and_icons(
        id,
        items,
        open_submenu,
        active_path,
        look,
        item_hovers,
        submenu_hovers,
        item_clicks,
        highlight,
        DisclosureIcons::default(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_floating_menu_with_submenu_hovers_and_icons(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    submenu_hovers: Vec<Vec<FloatingMenuHoverHandler>>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    highlight: Option<FloatingMenuHighlight>,
    disclosure_icons: DisclosureIcons,
) -> Stateful<Div> {
    render_floating_menu_with_submenu_hovers_and_icons_and_transition(
        id,
        items,
        open_submenu,
        active_path,
        look,
        item_hovers,
        submenu_hovers,
        item_clicks,
        highlight,
        disclosure_icons,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_floating_menu_with_submenu_hovers_and_icons_and_transition(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    submenu_hovers: Vec<Vec<FloatingMenuHoverHandler>>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    highlight: Option<FloatingMenuHighlight>,
    disclosure_icons: DisclosureIcons,
    submenu_transition: Option<(usize, f32)>,
) -> Stateful<Div> {
    default_floating_menu_template().render(
        &FloatingMenuRenderModel {
            direction: LayoutDirection::Ltr,
            id,
            items,
            open_submenu,
            active_path,
            highlight,
            submenu_presence: OverlayPresence::new(true, false),
            look,
            disclosure_icons: &disclosure_icons,
            submenu_transition,
        },
        FloatingMenuTemplateHandlers { item_hovers, submenu_hovers, controlled_hover: true, item_clicks },
    )
}

#[allow(clippy::too_many_arguments)]
fn render_floating_submenu(
    items: &[MenuItem],
    separator_template: &dyn FloatingMenuSeparatorTemplate,
    item: &MenuItem,
    look: &FloatingMenuLook,
    item_clicks: &mut std::vec::IntoIter<FloatingMenuClickHandler>,
    index: usize,
    active_path: Option<MenuPath>,
    submenu_hovers: Option<Vec<FloatingMenuHoverHandler>>,
    controlled_hover: bool,
    highlight: Option<FloatingMenuHighlight>,
    submenu_opacity: f32,
    disclosure_icons: &DisclosureIcons,
    direction: LayoutDirection,
) -> Stateful<Div> {
    let mut submenu = div()
        .id((item.id().clone(), 1usize))
        .debug_selector({
            let id = item.id().clone();
            move || format!("floating-submenu-{id}")
        })
        .absolute()
        .top(px(look.padding + row_offset(items, index, look)))
        .start(relative(1.0), direction)
        .min_w(px(look.min_width))
        .p(px(look.padding))
        .bg(look.background)
        .border(px(look.border_width))
        .border_color(look.border)
        .rounded(px(look.radius))
        .shadow(look.shadow.clone())
        .occlude();

    let highlight_rect = pane_highlight(highlight, Some(index), controlled_hover, items, look);
    if let Some(rect) = highlight_rect {
        submenu = submenu.child(render_highlight(rect, look));
    }

    let mut submenu_hovers = submenu_hovers.map(Vec::into_iter);
    for (submenu_index, submenu_item) in item.submenu_items().iter().enumerate() {
        let hover = submenu_hovers.as_mut().and_then(Iterator::next);
        if submenu_item.is_separator() {
            submenu = submenu.child(separator_template.render(submenu_item, look));
            continue;
        }
        let enabled = submenu_item.is_enabled();
        let is_active = active_path.is_some_and(|path| path.is_submenu(index, submenu_index));
        let paint = row_paint(enabled, is_active, controlled_hover, highlight_rect.is_some(), look);
        let color = paint.foreground;
        let mut row = div()
            .id((submenu_item.id().clone(), 0usize))
            .debug_selector({
                let id = submenu_item.id().clone();
                move || format!("floating-row-{id}")
            })
            .flex()
            .flex_row_logical(direction)
            .items_center()
            .gap(px(look.item_gap))
            .min_h(px(look.item_height))
            .px(px(look.item_padding_x))
            .rounded(px(look.item_radius))
            .text_color(color)
            .text_size(px(look.item_typography.size))
            .line_height(px(look.item_typography.line_height))
            .font_weight(look.item_typography.weight)
            .child(
                div()
                    .debug_selector({
                        let id = submenu_item.id().clone();
                        move || format!("floating-icon-{id}")
                    })
                    .child(render_item_icon(submenu_item.icon_ref(), color, look.item_icon_size)),
            )
            .child(
                div()
                    .debug_selector({
                        let id = submenu_item.id().clone();
                        move || format!("floating-label-{id}")
                    })
                    .flex_1()
                    .when(direction == LayoutDirection::Rtl, |label| label.text_right())
                    .child(submenu_item.label_text().clone()),
            )
            .child(render_submenu_affordance(
                !submenu_item.submenu_items().is_empty(),
                color,
                look.item_icon_size,
                disclosure_icons,
                0.0,
                direction,
            ));

        if enabled && submenu_item.submenu_items().is_empty() {
            if let Some(hover) = hover {
                row = row.on_hover(hover);
            }
            if let Some(item_click) = item_clicks.next() {
                row = row.cursor_pointer().on_click(item_click);
            }
        } else if !enabled {
            row = row.opacity(look.disabled_opacity);
        }

        submenu = submenu.child(apply_row_paint(row, paint));
    }

    submenu.opacity(submenu_opacity)
}

#[derive(Clone, Copy, Debug)]
struct MenuRowPaint {
    foreground: gpui::Hsla,
    background: Option<gpui::Hsla>,
    hover: Option<(gpui::Hsla, gpui::Hsla)>,
}

// Static and animated panes share selected colors; only ownership of the fill differs.
fn row_paint(
    enabled: bool,
    active: bool,
    controlled_hover: bool,
    highlight_painted: bool,
    look: &FloatingMenuLook,
) -> MenuRowPaint {
    MenuRowPaint {
        foreground: if !enabled {
            look.item_disabled_foreground
        } else if active {
            look.item_hover_foreground
        } else {
            look.foreground
        },
        background: (enabled && active && !highlight_painted).then_some(look.item_hover_background),
        hover: (enabled && !controlled_hover).then_some((look.item_hover_background, look.item_hover_foreground)),
    }
}

fn apply_row_paint(mut row: Stateful<Div>, paint: MenuRowPaint) -> Stateful<Div> {
    row = row.text_color(paint.foreground);
    if let Some(background) = paint.background {
        row = row.bg(background);
    }
    if let Some((background, foreground)) = paint.hover {
        row = row.hover(move |style| style.bg(background).text_color(foreground));
    }
    row
}

fn pane_highlight(
    highlight: Option<FloatingMenuHighlight>,
    parent: Option<usize>,
    controlled_hover: bool,
    items: &[MenuItem],
    look: &FloatingMenuLook,
) -> Option<(f32, f32, f32, f32)> {
    if !controlled_hover {
        return None;
    }
    let highlight = highlight?;
    if let Some(parent) = parent {
        for path in [highlight.from, highlight.to] {
            if !matches!(path, MenuPath::Submenu { parent: actual, .. } if actual == parent) {
                return None;
            }
        }
    }
    highlight_rect(highlight, parent.is_some(), items, look)
}

fn render_highlight((left, top, width, height): (f32, f32, f32, f32), look: &FloatingMenuLook) -> Div {
    div()
        .absolute()
        .left(px(left))
        .top(px(top))
        .w(px(width))
        .h(px(height))
        .rounded(px(look.item_radius))
        .bg(look.item_hover_background)
}

fn separator_height(look: &FloatingMenuLook) -> f32 {
    look.separator_thickness + 2.0 * look.separator_spacing
}

fn row_offset(items: &[MenuItem], index: usize, look: &FloatingMenuLook) -> f32 {
    look.rows_height(&items[..index.min(items.len())])
}

fn highlight_rect(
    highlight: FloatingMenuHighlight,
    submenu: bool,
    items: &[MenuItem],
    look: &FloatingMenuLook,
) -> Option<(f32, f32, f32, f32)> {
    let offset = |path| {
        let (rows, index) = match (submenu, path) {
            (false, MenuPath::Root(index)) => (items, index),
            (true, MenuPath::Submenu { parent, child }) => (items.get(parent)?.submenu_items(), child),
            _ => return None,
        };
        rows.get(index).filter(|item| item.is_enabled())?;
        Some(row_offset(rows, index, look))
    };
    let from = offset(highlight.from)?;
    let to = offset(highlight.to)?;
    Some((
        look.padding,
        look.padding + from + (to - from) * highlight.progress.clamp(0.0, 1.0),
        (look.min_width - look.padding * 2.0).max(0.0),
        look.item_height,
    ))
}

fn render_item_icon(icon: Option<&MenuItemIcon>, color: gpui::Hsla, size: f32) -> AnyElement {
    if let Some(icon) = icon.and_then(MenuItemIcon::lucide) {
        svg().path(icon.asset_path()).size(px(size)).text_color(color).into_any_element()
    } else if let Some(path) = icon.and_then(MenuItemIcon::asset_path) {
        svg().path(path.clone()).size(px(size)).text_color(color).into_any_element()
    } else if let Some(path) = icon.and_then(MenuItemIcon::svg_path) {
        svg().external_path(path.clone()).size(px(size)).text_color(color).into_any_element()
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_submenu_affordance(
    has_submenu: bool,
    color: gpui::Hsla,
    size: f32,
    icons: &DisclosureIcons,
    progress: f32,
    direction: LayoutDirection,
) -> AnyElement {
    if has_submenu
        && direction == LayoutDirection::Rtl
        && icons.collapsed.lucide() == Some(lucide_svg_static::Icon::ChevronRight)
        && icons.expanded.lucide() == Some(lucide_svg_static::Icon::ChevronDown)
    {
        return svg()
            .path(lucide_svg_static::Icon::ChevronLeft.asset_path())
            .size(px(size))
            .text_color(color)
            .with_transformation(gpui::Transformation::rotate(gpui::radians(
                -progress.clamp(0.0, 1.0) * std::f32::consts::FRAC_PI_2,
            )))
            .into_any_element();
    }
    if has_submenu {
        crate::infra::icon::render_disclosure_icon(icons, progress, color, size)
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn submenu_transition_progress(index: usize, open_submenu: Option<usize>, transition: Option<(usize, f32)>) -> f32 {
    if let Some((transition_index, progress)) = transition
        && transition_index == index
    {
        return progress;
    }

    if open_submenu == Some(index) { 1.0 } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::floating_menu::default_floating_menu_theme;

    #[test]
    fn static_and_animated_panes_keep_matching_selected_colors() {
        let mut look = default_floating_menu_theme().resolve();
        look.foreground = gpui::black();
        look.item_hover_foreground = gpui::white();
        let items = [MenuItem::new("action").submenu([MenuItem::new("child")])];
        for (parent, path) in [(None, MenuPath::Root(0)), (Some(0), MenuPath::Submenu { parent: 0, child: 0 })] {
            for controlled in [false, true] {
                for progress in [0.0, 0.5, 1.0] {
                    let highlight = FloatingMenuHighlight { from: path, to: path, progress };
                    let rect = pane_highlight(Some(highlight), parent, controlled, &items, &look);
                    let paint = row_paint(true, true, controlled, rect.is_some(), &look);
                    assert_eq!(paint.foreground, gpui::white());
                    assert_eq!(paint.background, (!controlled).then_some(look.item_hover_background));
                    assert_eq!(
                        rect.map(|_| look.item_hover_background).or(paint.background),
                        Some(look.item_hover_background)
                    );
                    let idle = row_paint(true, false, controlled, rect.is_some(), &look);
                    assert_eq!(idle.foreground, gpui::black());
                    assert_eq!(idle.background, None);
                    let disabled = row_paint(false, true, controlled, rect.is_some(), &look);
                    assert_eq!(disabled.foreground, look.item_disabled_foreground);
                    assert_eq!(disabled.background, None);
                    assert_eq!(disabled.hover, None);
                }
            }
        }
    }

    #[test]
    fn absent_or_unpaintable_highlights_keep_static_selected_fill() {
        let look = default_floating_menu_theme().resolve();
        let items = [MenuItem::new("action").submenu([MenuItem::new("child")]), MenuItem::separator("separator")];
        for (parent, highlight) in [
            (None, None),
            (None, Some(FloatingMenuHighlight { from: MenuPath::Root(0), to: MenuPath::Root(1), progress: 1.0 })),
            (
                Some(0),
                Some(FloatingMenuHighlight { from: MenuPath::Root(0), to: MenuPath::Root(0), progress: 1.0 }),
            ),
            (
                None,
                Some(FloatingMenuHighlight {
                    from: MenuPath::Submenu { parent: 0, child: 0 },
                    to: MenuPath::Root(0),
                    progress: 0.5,
                }),
            ),
            (
                Some(0),
                Some(FloatingMenuHighlight {
                    from: MenuPath::Submenu { parent: 1, child: 0 },
                    to: MenuPath::Submenu { parent: 1, child: 0 },
                    progress: 1.0,
                }),
            ),
        ] {
            let rect = pane_highlight(highlight, parent, true, &items, &look);
            assert!(rect.is_none());
            let paint = row_paint(true, true, true, rect.is_some(), &look);
            assert_eq!(paint.background, Some(look.item_hover_background));
            assert_eq!(paint.foreground, look.item_hover_foreground);
        }
    }

    #[test]
    fn mixed_rows_position_submenus_and_animated_highlights() {
        let mut look = default_floating_menu_theme().resolve();
        look.padding = 4.0;
        look.item_height = 30.0;
        look.separator_thickness = 2.0;
        look.separator_spacing = 3.0;
        let items = [
            MenuItem::new("first"),
            MenuItem::separator("divider"),
            MenuItem::new("parent").submenu([
                MenuItem::new("child-first"),
                MenuItem::separator("child-divider"),
                MenuItem::new("child-last"),
            ]),
        ];
        assert_eq!(row_offset(&items, 2, &look), 38.0);
        assert_eq!(look.rows_height(&items), 68.0);
        for (submenu, from, to) in [
            (false, MenuPath::Root(0), MenuPath::Root(2)),
            (true, MenuPath::Submenu { parent: 2, child: 0 }, MenuPath::Submenu { parent: 2, child: 2 }),
        ] {
            for (progress, top) in [(0.0, 4.0), (0.5, 23.0), (1.0, 42.0)] {
                let rect =
                    highlight_rect(FloatingMenuHighlight { from, to, progress }, submenu, &items, &look).unwrap();
                assert_eq!(rect.1, top);
                assert_eq!(rect.3, 30.0);
            }
        }
        assert!(
            highlight_rect(
                FloatingMenuHighlight { from: MenuPath::Root(0), to: MenuPath::Root(1), progress: 1.0 },
                false,
                &items,
                &look,
            )
            .is_none()
        );
    }
}
