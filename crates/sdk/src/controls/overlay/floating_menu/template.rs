use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, ClickEvent, Div, SharedString, Stateful, Window, div, px, prelude::*, svg};
use xgpui_luma_style::{PartId, PartView, Resolved};

use crate::infra::icon::DisclosureIcons;
use crate::infra::menu_item::{MenuItem, MenuItemIcon};
use crate::infra::popup_placement::{PopupSide, attached_popup};
use crate::infra::state::MenuPath;
use crate::motion::overlay_presence::OverlayPresence;
use crate::style::{PartRead, StyleBinding, StyleStates};
use crate::theme::InteractionState;

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
    pub items: &'a [MenuItem],
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
    pub highlight: Option<FloatingMenuHighlight>,
    pub submenu_presence: OverlayPresence,
    /// The menu's style (the `floating_menu` control, #433): the template resolves it and draws from its parts.
    pub style: StyleBinding,
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

impl RenderModel for dyn FloatingMenuTemplate {
    type Model<'a> = FloatingMenuRenderModel<'a>;
}

/// Visual structure for a noninteractive menu divider, drawn from the menu's resolved `separator` part. Its outer
/// height must be the part's `height + 2 * padding_y` for highlight and submenu alignment.
pub trait FloatingMenuSeparatorTemplate: Send + Sync {
    fn render(&self, item: &MenuItem, separator: PartView<'_>) -> Stateful<Div>;
}

#[derive(Default)]
pub struct ThemedFloatingMenuSeparatorTemplate;

impl FloatingMenuSeparatorTemplate for ThemedFloatingMenuSeparatorTemplate {
    fn render(&self, item: &MenuItem, separator: PartView<'_>) -> Stateful<Div> {
        let (thickness, spacing) = (separator.number("height"), separator.number("padding_y"));
        div()
            .id((item.id().clone(), 0usize))
            .h(px(thickness + 2.0 * spacing))
            .py(px(spacing))
            .px(px(separator.number("padding_x")))
            .child(div().h(px(thickness)).bg(separator.color("background")))
    }
}

/// The menu's resolved style for one render (#433): at rest, with a row hovered and with a row disabled.
/// The template reads parts from these directly.
struct MenuStyle {
    rest: Arc<Resolved>,
    hover: Arc<Resolved>,
    disabled: Arc<Resolved>,
}

impl MenuStyle {
    fn resolve(binding: &StyleBinding) -> Self {
        let resolve = |interaction| binding.resolve_states(StyleStates::new(interaction), 1.0);
        Self {
            rest: resolve(InteractionState::default()),
            hover: resolve(InteractionState { hovered: true, ..Default::default() }),
            disabled: resolve(InteractionState { disabled: true, ..Default::default() }),
        }
    }

    fn panel(&self) -> PartView<'_> {
        self.rest.root()
    }

    fn item(&self) -> PartView<'_> {
        self.rest.part(PartId::new("item"))
    }

    fn separator(&self) -> PartView<'_> {
        self.rest.part(PartId::new("separator"))
    }

    fn geometry(&self) -> MenuGeometry {
        let (panel, item, separator) = (self.panel(), self.item(), self.separator());
        MenuGeometry {
            padding: panel.number("padding"),
            min_width: panel.number("min_width"),
            item_height: item.number("height"),
            separator_height: separator.number("height") + 2.0 * separator.number("padding_y"),
        }
    }

    fn row_colors(&self) -> RowColors {
        let (hover, disabled) = (self.hover.part(PartId::new("item")), self.disabled.part(PartId::new("item")));
        RowColors {
            foreground: self.panel().color("foreground"),
            hover_background: hover.color("background"),
            hover_foreground: hover.color("foreground"),
            disabled_foreground: disabled.color("foreground"),
        }
    }

    fn disabled_opacity(&self) -> f32 {
        self.disabled.part(PartId::new("item")).opacity()
    }

    fn item_radius(&self) -> f32 {
        self.item().number("radius")
    }

    fn icon_size(&self) -> f32 {
        self.panel().number("item_icon_size")
    }

    fn window_margin(&self) -> f32 {
        self.panel().number("window_margin")
    }

    /// The panel around the rows: fill, stroke, corners, shadow, padding and minimum width.
    fn pane(&self, pane: Stateful<Div>) -> Stateful<Div> {
        let panel = self.panel();
        pane.min_w(px(panel.number("min_width")))
            .p(px(panel.number("padding")))
            .bg(panel.color("background"))
            .border(px(panel.number("border_width")))
            .border_color(panel.color("border"))
            .rounded(px(panel.number("radius")))
            .shadow(panel.box_shadow())
            .occlude()
    }

    /// A row's box and text, before its paint.
    fn row(&self, row: Stateful<Div>, color: gpui::Hsla) -> Stateful<Div> {
        let item = self.item();
        let typography = item.typography(14.0, 20.0);
        row.flex()
            .items_center()
            .gap(px(item.number("gap")))
            .min_h(px(item.number("height")))
            .px(px(item.number("padding_x")))
            .rounded(px(item.number("radius")))
            .text_color(color)
            .text_size(px(typography.size))
            .line_height(px(typography.line_height))
            .font_weight(typography.weight)
    }
}

/// Row and panel geometry for layout math: where rows, the highlight and submenus sit.
#[derive(Clone, Copy, Debug)]
struct MenuGeometry {
    padding: f32,
    min_width: f32,
    item_height: f32,
    separator_height: f32,
}

impl MenuGeometry {
    fn rows_height(&self, items: &[MenuItem]) -> f32 {
        items
            .iter()
            .map(|item| {
                if item.is_separator() {
                    self.separator_height
                } else {
                    self.item_height
                }
            })
            .sum()
    }

    fn row_offset(&self, items: &[MenuItem], index: usize) -> f32 {
        self.rows_height(&items[..index.min(items.len())])
    }
}

/// The colors a row draws in: at rest, hovered or active, and disabled.
#[derive(Clone, Copy, Debug)]
struct RowColors {
    foreground: gpui::Hsla,
    hover_background: gpui::Hsla,
    hover_foreground: gpui::Hsla,
    disabled_foreground: gpui::Hsla,
}

/// The size a menu of `items` opens at in `style`: its minimum width, and its rows plus padding (#433). For
/// placing the menu before it is drawn.
pub fn floating_menu_size(style: &StyleBinding, items: &[MenuItem]) -> gpui::Size<gpui::Pixels> {
    let geometry = MenuStyle::resolve(style).geometry();
    gpui::size(px(geometry.min_width), px(geometry.padding * 2.0 + geometry.rows_height(items)))
}

/// The edge margin a menu in `style` keeps from the window.
pub fn floating_menu_window_margin(style: &StyleBinding) -> gpui::Pixels {
    px(MenuStyle::resolve(style).window_margin())
}

pub struct ThemedFloatingMenuTemplate {
    separator_template: Arc<dyn FloatingMenuSeparatorTemplate>,
    modifiers: RootModifiers<dyn FloatingMenuTemplate>,
}

impl ThemedFloatingMenuTemplate {
    pub fn new() -> Self {
        Self {
            separator_template: Arc::new(ThemedFloatingMenuSeparatorTemplate),
            modifiers: RootModifiers::default(),
        }
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
}

impl Default for ThemedFloatingMenuTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl FloatingMenuTemplate for ThemedFloatingMenuTemplate {
    fn render(&self, model: &FloatingMenuRenderModel<'_>, handlers: FloatingMenuTemplateHandlers) -> Stateful<Div> {
        let FloatingMenuTemplateHandlers { item_hovers, mut submenu_hovers, controlled_hover, item_clicks } = handlers;
        // Drawn straight from the resolved style (#433).
        let style = MenuStyle::resolve(&model.style);
        let (geometry, colors, icon_size) = (style.geometry(), style.row_colors(), style.icon_size());

        let mut menu = style.pane(div().id((model.id.clone(), 0usize)).relative());

        let highlight_rect = pane_highlight(model.highlight, None, controlled_hover, model.items, geometry);
        if let Some(rect) = highlight_rect {
            menu = menu.child(render_highlight(rect, style.item_radius(), colors.hover_background));
        }

        let mut item_clicks = item_clicks.into_iter();
        let mut submenu = None;

        for ((index, item), item_hover) in model.items.iter().enumerate().zip(item_hovers) {
            if item.is_separator() {
                menu = menu.child(self.separator_template.render(item, style.separator()));
                continue;
            }
            let enabled = item.is_enabled();
            let is_active = matches!(model.active_path, Some(MenuPath::Root(active)) if active == index);
            let paint = row_paint(enabled, is_active, controlled_hover, highlight_rect.is_some(), colors);
            let color = paint.foreground;
            let mut row = style
                .row(div().id((item.id().clone(), 0usize)), color)
                .child(render_item_icon(item.icon_ref(), color, icon_size))
                .child(div().flex_1().child(item.label_text().clone()));

            if enabled {
                row = row.cursor_pointer().on_hover(item_hover).child(render_submenu_affordance(
                    !item.submenu_items().is_empty(),
                    color,
                    icon_size,
                    model.disclosure_icons,
                    submenu_transition_progress(index, model.open_submenu, model.submenu_transition),
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
                        &style,
                        &mut item_clicks,
                        index,
                        model.active_path,
                        (index < submenu_hovers.len()).then(|| std::mem::take(&mut submenu_hovers[index])),
                        controlled_hover,
                        model.highlight,
                        model.submenu_presence.opacity(),
                        model.disclosure_icons,
                    ));
                }
            } else {
                row = crate::style::fade(row, style.disabled_opacity()).child(render_submenu_affordance(
                    !item.submenu_items().is_empty(),
                    color,
                    icon_size,
                    model.disclosure_icons,
                    submenu_transition_progress(index, model.open_submenu, model.submenu_transition),
                ));
            }

            menu = menu.child(apply_row_paint(row, paint));
        }

        if let Some(submenu) = submenu {
            // Beside its row, flipping to the other side at the window edge (#433).
            // Deferred, so a scrolling parent menu does not clip it.
            let row_top = px(geometry.padding + geometry.row_offset(model.items, model.open_submenu.unwrap_or(0)));
            menu = menu.child(
                gpui::deferred(
                    attached_popup(PopupSide::End, submenu)
                        .band(row_top, px(geometry.item_height))
                        .window_margin(px(style.window_margin())),
                )
                .with_priority(2),
            );
        }

        self.modifiers.apply(menu, model)
    }
}

impl FloatingMenuTemplate for ModifiedTemplate<dyn FloatingMenuTemplate> {
    fn render(&self, model: &FloatingMenuRenderModel<'_>, handlers: FloatingMenuTemplateHandlers) -> Stateful<Div> {
        let root = self.base.render(model, handlers);
        self.apply(root, model)
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
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

pub fn render_floating_menu(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    style: StyleBinding,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> Stateful<Div> {
    render_floating_menu_with_template(
        default_floating_menu_template(),
        id,
        items,
        open_submenu,
        active_path,
        style,
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
    style: StyleBinding,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    submenu_presence: OverlayPresence,
) -> Stateful<Div> {
    render_floating_menu_with_submenu_presence_and_transition(
        id,
        items,
        open_submenu,
        active_path,
        style,
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
    style: StyleBinding,
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
        style,
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
    style: StyleBinding,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> Stateful<Div> {
    let disclosure_icons = DisclosureIcons::default();
    template.render(
        &FloatingMenuRenderModel {
            id,
            items,
            open_submenu,
            active_path,
            highlight: None,
            submenu_presence: OverlayPresence::new(true, false),
            style,
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
    style: StyleBinding,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    submenu_presence: OverlayPresence,
    submenu_transition: Option<(usize, f32)>,
) -> Stateful<Div> {
    template.render(
        &FloatingMenuRenderModel {
            id,
            items,
            open_submenu,
            active_path,
            highlight: None,
            submenu_presence,
            style,
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
    style: StyleBinding,
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
        style,
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
    style: StyleBinding,
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
        style,
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
    style: StyleBinding,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    submenu_hovers: Vec<Vec<FloatingMenuHoverHandler>>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    highlight: Option<FloatingMenuHighlight>,
    disclosure_icons: DisclosureIcons,
    submenu_transition: Option<(usize, f32)>,
) -> Stateful<Div> {
    default_floating_menu_template().render(
        &FloatingMenuRenderModel {
            id,
            items,
            open_submenu,
            active_path,
            highlight,
            submenu_presence: OverlayPresence::new(true, false),
            style,
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
    style: &MenuStyle,
    item_clicks: &mut std::vec::IntoIter<FloatingMenuClickHandler>,
    index: usize,
    active_path: Option<MenuPath>,
    submenu_hovers: Option<Vec<FloatingMenuHoverHandler>>,
    controlled_hover: bool,
    highlight: Option<FloatingMenuHighlight>,
    submenu_opacity: f32,
    disclosure_icons: &DisclosureIcons,
) -> Stateful<Div> {
    let (geometry, colors, icon_size) = (style.geometry(), style.row_colors(), style.icon_size());
    let mut submenu = style.pane(div().id((item.id().clone(), 1usize)));

    let highlight_rect = pane_highlight(highlight, Some(index), controlled_hover, items, geometry);
    if let Some(rect) = highlight_rect {
        submenu = submenu.child(render_highlight(rect, style.item_radius(), colors.hover_background));
    }

    let mut submenu_hovers = submenu_hovers.map(Vec::into_iter);
    for (submenu_index, submenu_item) in item.submenu_items().iter().enumerate() {
        let hover = submenu_hovers.as_mut().and_then(Iterator::next);
        if submenu_item.is_separator() {
            submenu = submenu.child(separator_template.render(submenu_item, style.separator()));
            continue;
        }
        let enabled = submenu_item.is_enabled();
        let is_active = active_path.is_some_and(|path| path.is_submenu(index, submenu_index));
        let paint = row_paint(enabled, is_active, controlled_hover, highlight_rect.is_some(), colors);
        let color = paint.foreground;
        let mut row = style
            .row(div().id((submenu_item.id().clone(), 0usize)), color)
            .child(render_item_icon(submenu_item.icon_ref(), color, icon_size))
            .child(div().flex_1().child(submenu_item.label_text().clone()))
            .child(render_submenu_affordance(
                !submenu_item.submenu_items().is_empty(),
                color,
                icon_size,
                disclosure_icons,
                0.0,
            ));

        if enabled && submenu_item.submenu_items().is_empty() {
            if let Some(hover) = hover {
                row = row.on_hover(hover);
            }
            if let Some(item_click) = item_clicks.next() {
                row = row.cursor_pointer().on_click(item_click);
            }
        } else if !enabled {
            row = crate::style::fade(row, style.disabled_opacity());
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
    colors: RowColors,
) -> MenuRowPaint {
    MenuRowPaint {
        foreground: if !enabled {
            colors.disabled_foreground
        } else if active {
            colors.hover_foreground
        } else {
            colors.foreground
        },
        background: (enabled && active && !highlight_painted).then_some(colors.hover_background),
        hover: (enabled && !controlled_hover).then_some((colors.hover_background, colors.hover_foreground)),
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
    geometry: MenuGeometry,
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
    highlight_rect(highlight, parent.is_some(), items, geometry)
}

fn render_highlight((left, top, width, height): (f32, f32, f32, f32), radius: f32, fill: gpui::Hsla) -> Div {
    div().absolute().left(px(left)).top(px(top)).w(px(width)).h(px(height)).rounded(px(radius)).bg(fill)
}

fn highlight_rect(
    highlight: FloatingMenuHighlight,
    submenu: bool,
    items: &[MenuItem],
    geometry: MenuGeometry,
) -> Option<(f32, f32, f32, f32)> {
    let offset = |path| {
        let (rows, index) = match (submenu, path) {
            (false, MenuPath::Root(index)) => (items, index),
            (true, MenuPath::Submenu { parent, child }) => (items.get(parent)?.submenu_items(), child),
            _ => return None,
        };
        rows.get(index).filter(|item| item.is_enabled())?;
        Some(geometry.row_offset(rows, index))
    };
    let from = offset(highlight.from)?;
    let to = offset(highlight.to)?;
    Some((
        geometry.padding,
        geometry.padding + from + (to - from) * highlight.progress.clamp(0.0, 1.0),
        (geometry.min_width - geometry.padding * 2.0).max(0.0),
        geometry.item_height,
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
) -> AnyElement {
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

    fn colors() -> RowColors {
        RowColors {
            foreground: gpui::black(),
            hover_background: gpui::red(),
            hover_foreground: gpui::white(),
            disabled_foreground: gpui::blue(),
        }
    }

    fn sdk_geometry() -> MenuGeometry {
        MenuStyle::resolve(&StyleBinding::sdk("floating_menu")).geometry()
    }

    #[test]
    fn static_and_animated_panes_keep_matching_selected_colors() {
        let colors = colors();
        let items = [MenuItem::new("action").submenu([MenuItem::new("child")])];
        for (parent, path) in [(None, MenuPath::Root(0)), (Some(0), MenuPath::Submenu { parent: 0, child: 0 })] {
            for controlled in [false, true] {
                for progress in [0.0, 0.5, 1.0] {
                    let highlight = FloatingMenuHighlight { from: path, to: path, progress };
                    let rect = pane_highlight(Some(highlight), parent, controlled, &items, sdk_geometry());
                    let paint = row_paint(true, true, controlled, rect.is_some(), colors);
                    assert_eq!(paint.foreground, gpui::white());
                    assert_eq!(paint.background, (!controlled).then_some(colors.hover_background));
                    assert_eq!(
                        rect.map(|_| colors.hover_background).or(paint.background),
                        Some(colors.hover_background)
                    );
                    let idle = row_paint(true, false, controlled, rect.is_some(), colors);
                    assert_eq!(idle.foreground, gpui::black());
                    assert_eq!(idle.background, None);
                    let disabled = row_paint(false, true, controlled, rect.is_some(), colors);
                    assert_eq!(disabled.foreground, colors.disabled_foreground);
                    assert_eq!(disabled.background, None);
                    assert_eq!(disabled.hover, None);
                }
            }
        }
    }

    #[test]
    fn absent_or_unpaintable_highlights_keep_static_selected_fill() {
        let colors = colors();
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
            let rect = pane_highlight(highlight, parent, true, &items, sdk_geometry());
            assert!(rect.is_none());
            let paint = row_paint(true, true, true, rect.is_some(), colors);
            assert_eq!(paint.background, Some(colors.hover_background));
            assert_eq!(paint.foreground, colors.hover_foreground);
        }
    }

    #[test]
    fn mixed_rows_position_submenus_and_animated_highlights() {
        let geometry = MenuGeometry { padding: 4.0, min_width: 200.0, item_height: 30.0, separator_height: 8.0 };
        let items = [
            MenuItem::new("first"),
            MenuItem::separator("divider"),
            MenuItem::new("parent").submenu([
                MenuItem::new("child-first"),
                MenuItem::separator("child-divider"),
                MenuItem::new("child-last"),
            ]),
        ];
        assert_eq!(geometry.row_offset(&items, 2), 38.0);
        assert_eq!(geometry.rows_height(&items), 68.0);
        for (submenu, from, to) in [
            (false, MenuPath::Root(0), MenuPath::Root(2)),
            (true, MenuPath::Submenu { parent: 2, child: 0 }, MenuPath::Submenu { parent: 2, child: 2 }),
        ] {
            for (progress, top) in [(0.0, 4.0), (0.5, 23.0), (1.0, 42.0)] {
                let rect =
                    highlight_rect(FloatingMenuHighlight { from, to, progress }, submenu, &items, geometry).unwrap();
                assert_eq!(rect.1, top);
                assert_eq!(rect.3, 30.0);
            }
        }
        assert!(
            highlight_rect(
                FloatingMenuHighlight { from: MenuPath::Root(0), to: MenuPath::Root(1), progress: 1.0 },
                false,
                &items,
                geometry,
            )
            .is_none()
        );
    }

    /// The template draws from the resolved style (#433) with the values the `FloatingMenuLook` compatibility
    /// type reads.
    #[test]
    fn the_resolved_style_matches_the_look() {
        let binding = StyleBinding::sdk("floating_menu");
        let look = crate::controls::floating_menu::styled_floating_menu_look(&binding);
        let style = MenuStyle::resolve(&binding);
        let (geometry, colors) = (style.geometry(), style.row_colors());
        assert_eq!(
            (geometry.padding, geometry.min_width, geometry.item_height),
            (look.padding, look.min_width, look.item_height)
        );
        assert_eq!(geometry.separator_height, look.separator_thickness + 2.0 * look.separator_spacing);
        assert_eq!(
            (colors.foreground, colors.hover_background, colors.hover_foreground, colors.disabled_foreground),
            (
                look.foreground,
                look.item_hover_background,
                look.item_hover_foreground,
                look.item_disabled_foreground
            )
        );
        assert_eq!(style.disabled_opacity(), look.item_disabled_opacity);
        assert_eq!((style.item_radius(), style.icon_size()), (look.item_radius, look.item_icon_size));
        let items = [MenuItem::new("a"), MenuItem::separator("s"), MenuItem::new("b")];
        assert_eq!(
            floating_menu_size(&binding, &items),
            gpui::size(px(look.min_width), px(look.padding * 2.0 + look.rows_height(&items)))
        );
    }
}
