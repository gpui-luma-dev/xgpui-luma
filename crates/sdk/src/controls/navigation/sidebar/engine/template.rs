use std::{
    rc::Rc,
    sync::{Arc, OnceLock},
};

use gpui::{
    Anchor, AnyElement, App, Bounds, ClickEvent, Div, FocusHandle, MouseButton, MouseDownEvent, MouseUpEvent, Pixels,
    SharedString, Stateful, Window, anchored, deferred, div, point, prelude::*, px,
};
use lucide_svg_static::Icon as LucideIcon;

use super::{NavNodeKind, SidebarPanelEngineRenderModel, RenderedNavNode, RenderedRailSubmenu};
use crate::controls::floating_menu::{FloatingMenuClickHandler, FloatingMenuHoverHandler, render_floating_menu};
use crate::controls::scroll_container::ScrollContainer;
use crate::theme::{ControlSize, InteractionState, LumaTextStyle, LumaTypography};
use crate::controls::floating_menu::{FloatingMenuLook, FloatingMenuTheme, default_floating_menu_theme};
use crate::infra::icon::{DisclosureIcons, render_disclosure_icon, render_icon_source};
use crate::infra::ElementExt;
use super::{SidebarTheme, default_sidebar_theme};

const CHILD_DEPTH_INDENT_MULTIPLIER: f32 = 1.0;
const SCROLL_REGION_MIN_HEIGHT: f32 = 0.0;

/// Shared callback retained across redraws; custom handlers use `Rc::new`.
pub type SidebarPanelBoundsHandler = Rc<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
/// Shared callback retained across redraws; custom handlers use `Rc::new`.
pub type SidebarPanelClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
/// Shared callback retained across redraws; custom handlers use `Rc::new`.
pub type SidebarPanelHoverHandler = Rc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
/// Shared callback retained across redraws; custom handlers use `Rc::new`.
pub type SidebarPanelMouseDownHandler = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
/// Shared callback retained across redraws; custom handlers use `Rc::new`.
pub type SidebarPanelMouseUpHandler = Rc<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
/// Shared callback retained across redraws; custom handlers use `Rc::new`.
pub type SidebarPanelChildrenHeightHandler = Rc<dyn Fn(&f32, &mut Window, &mut App) + 'static>;

/// Row callbacks are shared across redraws. Custom templates bind a callback through a
/// forwarding closure such as `.on_hover(move |event, window, cx| hover(event, window, cx))`.
/// Row containers use `Rc` too: custom templates iterate with `.iter().cloned()`;
/// custom construction converts vectors/maps with `.into()`. This keeps cache hits allocation-free.
#[derive(Default)]
pub struct SidebarPanelTemplateHandlers {
    pub row_bounds: Rc<Vec<SidebarPanelBoundsHandler>>,
    pub row_hovers: Rc<Vec<SidebarPanelHoverHandler>>,
    pub row_mouse_downs: Rc<Vec<SidebarPanelMouseDownHandler>>,
    pub row_mouse_ups: Rc<Vec<SidebarPanelMouseUpHandler>>,
    pub row_mouse_up_outs: Rc<Vec<SidebarPanelMouseUpHandler>>,
    pub row_clicks: Rc<Vec<SidebarPanelClickHandler>>,
    pub children_height_reports: Rc<std::collections::HashMap<SharedString, SidebarPanelChildrenHeightHandler>>,
    pub rail_submenu_mouse_down_out: Option<SidebarPanelMouseDownHandler>,
    pub rail_submenu_item_hovers: Vec<FloatingMenuHoverHandler>,
    pub rail_submenu_item_clicks: Vec<FloatingMenuClickHandler>,
    pub rail_submenu_bounds: Option<SidebarPanelBoundsHandler>,
}

pub type SidebarPanelTemplateModifier = Box<dyn Fn(Stateful<Div>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait SidebarPanelTemplate: Send + Sync {
    fn render(
        &self,
        model: SidebarPanelEngineRenderModel,
        main_scroll: &ScrollContainer,
        handlers: SidebarPanelTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedSidebarPanelTemplate {
    theme: Arc<dyn SidebarTheme>,
    floating_menu_theme: Arc<dyn FloatingMenuTheme>,
    modifiers: Vec<SidebarPanelTemplateModifier>,
}

impl ThemedSidebarPanelTemplate {
    pub fn new(theme: Arc<dyn SidebarTheme>) -> Self {
        Self { theme, floating_menu_theme: default_floating_menu_theme(), modifiers: Vec::new() }
    }

    pub fn new_with_floating_menu_theme(
        theme: Arc<dyn SidebarTheme>,
        floating_menu_theme: Arc<dyn FloatingMenuTheme>,
    ) -> Self {
        Self { theme, floating_menu_theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root);
        }
        root
    }
}

struct ModifiedSidebarPanelTemplate {
    base: Arc<dyn SidebarPanelTemplate>,
    modifiers: Vec<SidebarPanelTemplateModifier>,
}

impl ModifiedSidebarPanelTemplate {
    fn new(base: Arc<dyn SidebarPanelTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: SidebarPanelTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root);
        }
        root
    }
}

impl Default for ThemedSidebarPanelTemplate {
    fn default() -> Self {
        Self::new(default_sidebar_theme())
    }
}

pub fn default_sidebar_panel_template() -> Arc<dyn SidebarPanelTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SidebarPanelTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedSidebarPanelTemplate::default())).clone()
}

pub(crate) fn modified_sidebar_panel_template<F>(
    template: Arc<dyn SidebarPanelTemplate>,
    modifier: F,
) -> Arc<dyn SidebarPanelTemplate>
where
    F: Fn(Stateful<Div>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedSidebarPanelTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl SidebarPanelTemplate for ModifiedSidebarPanelTemplate {
    fn render(
        &self,
        model: SidebarPanelEngineRenderModel,
        main_scroll: &ScrollContainer,
        handlers: SidebarPanelTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, main_scroll, handlers, window, cx);
        self.apply_modifiers(root)
    }
}

impl SidebarPanelTemplate for ThemedSidebarPanelTemplate {
    fn render(
        &self,
        model: SidebarPanelEngineRenderModel,
        main_scroll: &ScrollContainer,
        handlers: SidebarPanelTemplateHandlers,
        _window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        let SidebarPanelTemplateHandlers {
            row_bounds,
            row_hovers,
            row_mouse_downs,
            row_mouse_ups,
            row_mouse_up_outs,
            row_clicks,
            children_height_reports,
            rail_submenu_mouse_down_out,
            rail_submenu_item_hovers,
            rail_submenu_item_clicks,
            rail_submenu_bounds,
        } = handlers;
        let mut row_bounds = row_bounds.iter().cloned();
        let mut row_hovers = row_hovers.iter().cloned();
        let mut row_mouse_downs = row_mouse_downs.iter().cloned();
        let mut row_mouse_ups = row_mouse_ups.iter().cloned();
        let mut row_mouse_up_outs = row_mouse_up_outs.iter().cloned();
        let mut row_clicks = row_clicks.iter().cloned();
        let mut root = div()
            .id(model.id)
            .size_full()
            .flex()
            .flex_col()
            .gap(px(theme.geometry().container_gap))
            .text_color(self.theme.foreground());

        if model.collapsed {
            root = root.child(
                render_collapsed_rail_region(
                    model.rail_nodes,
                    &self.theme,
                    &mut row_bounds,
                    &mut row_hovers,
                    &mut row_mouse_downs,
                    &mut row_mouse_ups,
                    &mut row_mouse_up_outs,
                    &mut row_clicks,
                    &model.disclosure_icons,
                    cx,
                )
                .flex_1(),
            );

            if !model.rail_footer_nodes.is_empty() {
                root = root.child(
                    render_collapsed_rail_region(
                        model.rail_footer_nodes,
                        &self.theme,
                        &mut row_bounds,
                        &mut row_hovers,
                        &mut row_mouse_downs,
                        &mut row_mouse_ups,
                        &mut row_mouse_up_outs,
                        &mut row_clicks,
                        &model.disclosure_icons,
                        cx,
                    )
                    .pt(px(theme.geometry().footer_padding_top)),
                );
            }

            if let Some(rail_submenu) = model.rail_submenu {
                root = root.child(
                    deferred(render_rail_submenu_overlay(
                        rail_submenu,
                        self.floating_menu_theme.resolve(),
                        self.theme.geometry(),
                        rail_submenu_item_hovers,
                        rail_submenu_item_clicks,
                        rail_submenu_bounds,
                        rail_submenu_mouse_down_out.expect("rail submenu should have outside click handler"),
                    ))
                    .with_priority(1),
                );
            }

            return root;
        }

        if model.title.is_some() || model.subtitle.is_some() {
            root = root.child(render_title(model.title, model.subtitle, &self.theme));
        }

        if !model.header_nodes.is_empty() {
            root = root.child(
                render_region(
                    model.header_nodes,
                    &self.theme,
                    &mut row_hovers,
                    &mut row_mouse_downs,
                    &mut row_mouse_ups,
                    &mut row_mouse_up_outs,
                    &mut row_clicks,
                    &children_height_reports,
                    &model.disclosure_icons,
                    cx,
                )
                .pb(px(theme.geometry().header_padding_bottom)),
            );
        }

        root = root.child(
            main_scroll
                .render(
                    render_region(
                        model.nodes,
                        &self.theme,
                        &mut row_hovers,
                        &mut row_mouse_downs,
                        &mut row_mouse_ups,
                        &mut row_mouse_up_outs,
                        &mut row_clicks,
                        &children_height_reports,
                        &model.disclosure_icons,
                        cx,
                    )
                    .into_any_element(),
                )
                .flex_1()
                .min_h(px(SCROLL_REGION_MIN_HEIGHT)),
        );

        if !model.footer_nodes.is_empty() {
            root = root.child(
                render_region(
                    model.footer_nodes,
                    &self.theme,
                    &mut row_hovers,
                    &mut row_mouse_downs,
                    &mut row_mouse_ups,
                    &mut row_mouse_up_outs,
                    &mut row_clicks,
                    &children_height_reports,
                    &model.disclosure_icons,
                    cx,
                )
                .pt(px(theme.geometry().footer_padding_top)),
            );
        }

        self.apply_modifiers(root)
    }
}

fn sidebar_title_style(typography: &LumaTypography) -> LumaTextStyle {
    typography.text.label
}

fn sidebar_subtitle_style(typography: &LumaTypography) -> LumaTextStyle {
    typography.text.scale.sm
}

fn render_title(title: Option<SharedString>, subtitle: Option<SharedString>, theme: &Arc<dyn SidebarTheme>) -> Div {
    let typography = theme.typography();
    let title_style = sidebar_title_style(&typography);
    let subtitle_style = sidebar_subtitle_style(&typography);
    let mut header = div()
        .flex()
        .flex_col()
        .gap(px(theme.geometry().title_gap))
        .pb(px(theme.geometry().title_padding_bottom));

    if let Some(title) = title {
        header = header.child(
            div()
                .flex()
                .items_center()
                .text_size(px(title_style.size))
                .line_height(px(title_style.line_height))
                .font_weight(title_style.weight)
                .child(title),
        );
    }

    if let Some(subtitle) = subtitle {
        header = header.child(
            div()
                .text_size(px(subtitle_style.size))
                .line_height(px(subtitle_style.line_height))
                .font_weight(subtitle_style.weight)
                .opacity(theme.geometry().subtitle_opacity)
                .child(subtitle),
        );
    }

    header
}

#[allow(clippy::too_many_arguments)]
fn render_region(
    nodes: Vec<RenderedNavNode>,
    theme: &Arc<dyn SidebarTheme>,
    row_hovers: &mut impl Iterator<Item = SidebarPanelHoverHandler>,
    row_mouse_downs: &mut impl Iterator<Item = SidebarPanelMouseDownHandler>,
    row_mouse_ups: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_mouse_up_outs: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_clicks: &mut impl Iterator<Item = SidebarPanelClickHandler>,
    children_height_reports: &std::collections::HashMap<SharedString, SidebarPanelChildrenHeightHandler>,
    disclosure_icons: &DisclosureIcons,
    cx: &mut App,
) -> Div {
    let mut region = div().w_full().min_w(px(0.0)).flex().flex_col().gap(px(theme.geometry().region_gap));

    for node in nodes {
        region = region.child(render_node(
            node,
            theme,
            row_hovers,
            row_mouse_downs,
            row_mouse_ups,
            row_mouse_up_outs,
            row_clicks,
            children_height_reports,
            disclosure_icons,
            cx,
        ));
    }

    region
}

#[allow(clippy::too_many_arguments)]
fn render_collapsed_rail_region(
    nodes: Vec<RenderedNavNode>,
    theme: &Arc<dyn SidebarTheme>,
    row_bounds: &mut impl Iterator<Item = SidebarPanelBoundsHandler>,
    row_hovers: &mut impl Iterator<Item = SidebarPanelHoverHandler>,
    row_mouse_downs: &mut impl Iterator<Item = SidebarPanelMouseDownHandler>,
    row_mouse_ups: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_mouse_up_outs: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_clicks: &mut impl Iterator<Item = SidebarPanelClickHandler>,
    disclosure_icons: &DisclosureIcons,
    cx: &mut App,
) -> Div {
    let mut region = div().flex().flex_col().items_center().gap(px(theme.geometry().region_gap));

    for node in nodes {
        region = region.child(render_collapsed_rail_node(
            node,
            RowHandlers {
                bounds: row_bounds.next(),
                hover: row_hovers.next(),
                mouse_down: row_mouse_downs.next(),
                mouse_up: row_mouse_ups.next(),
                mouse_up_out: row_mouse_up_outs.next(),
                click: row_clicks.next(),
            },
            theme,
            disclosure_icons,
            cx,
        ));
    }

    region
}

#[allow(clippy::too_many_arguments)]
fn render_node(
    node: RenderedNavNode,
    theme: &Arc<dyn SidebarTheme>,
    row_hovers: &mut impl Iterator<Item = SidebarPanelHoverHandler>,
    row_mouse_downs: &mut impl Iterator<Item = SidebarPanelMouseDownHandler>,
    row_mouse_ups: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_mouse_up_outs: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_clicks: &mut impl Iterator<Item = SidebarPanelClickHandler>,
    children_height_reports: &std::collections::HashMap<SharedString, SidebarPanelChildrenHeightHandler>,
    disclosure_icons: &DisclosureIcons,
    cx: &mut App,
) -> AnyElement {
    let RenderedNavNode {
        drop_layout,
        drag_drop,
        id,
        kind,
        label,
        icon,
        state,
        custom_element,
        focus_handle,
        has_children,
        expansion_progress,
        children_height_px,
        children,
        tooltip,
    } = node;
    let row = render_row(
        RowRenderInput {
            id: id.clone(),
            tooltip,
            kind,
            label,
            icon,
            state,
            custom_element,
            focus_handle,
            has_children,
            expansion_progress,
            disclosure_icons: disclosure_icons.clone(),
        },
        RowHandlers {
            bounds: None,
            hover: row_hovers.next(),
            mouse_down: row_mouse_downs.next(),
            mouse_up: row_mouse_ups.next(),
            mouse_up_out: row_mouse_up_outs.next(),
            click: row_clicks.next(),
        },
        theme,
        cx,
    );
    let row = if let Some(binding) = drag_drop {
        binding
            .bind(
                div().id(gpui::SharedString::from(format!("{id}-drag-row"))).w_full().child(row),
                theme.resolve_drag(ControlSize::Md),
                cx,
            )
            .into_any_element()
    } else {
        row
    };
    let row = if let Some(motion) = drop_layout {
        motion.item(id.clone(), row)
    } else {
        row
    };
    let mut root = div()
        .id(id.clone())
        .w_full()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(theme.geometry().region_gap))
        .child(row);

    if !children.is_empty() {
        let height_report = children_height_reports.get(&id).cloned();
        let mut children_region = div().w_full().min_w(px(0.0)).flex().flex_col().gap(px(theme.geometry().region_gap));
        for child in children {
            children_region = children_region.child(render_node(
                child,
                theme,
                row_hovers,
                row_mouse_downs,
                row_mouse_ups,
                row_mouse_up_outs,
                row_clicks,
                children_height_reports,
                disclosure_icons,
                cx,
            ));
        }

        let mut clip = div().w_full().overflow_hidden().opacity(expansion_progress).flex_shrink_0();
        if children_height_px > f32::EPSILON {
            clip = clip.h(px(children_height_px * expansion_progress));
        } else if expansion_progress < 1.0 - f32::EPSILON {
            clip = clip.h(px(0.0));
        }
        if let Some(height_report) = height_report {
            let clip = clip.on_children_prepainted(move |bounds, window, cx| {
                let height = bounds.iter().map(|child| child.size.height.as_f32()).fold(0.0_f32, f32::max);
                if height > f32::EPSILON {
                    height_report(&height, window, cx);
                }
            });
            root = root.child(clip.child(children_region));
        } else {
            root = root.child(clip.child(children_region));
        }
    }

    root.into_any_element()
}

struct RowRenderInput {
    tooltip: Option<crate::infra::attachments::TooltipHandle>,
    id: SharedString,
    kind: NavNodeKind,
    label: Option<SharedString>,
    icon: Option<LucideIcon>,
    state: super::NavNodeState,
    custom_element: Option<AnyElement>,
    focus_handle: Option<FocusHandle>,
    has_children: bool,
    expansion_progress: f32,
    disclosure_icons: DisclosureIcons,
}

struct RowHandlers {
    bounds: Option<SidebarPanelBoundsHandler>,
    hover: Option<SidebarPanelHoverHandler>,
    mouse_down: Option<SidebarPanelMouseDownHandler>,
    mouse_up: Option<SidebarPanelMouseUpHandler>,
    mouse_up_out: Option<SidebarPanelMouseUpHandler>,
    click: Option<SidebarPanelClickHandler>,
}

fn render_row(input: RowRenderInput, handlers: RowHandlers, theme: &Arc<dyn SidebarTheme>, cx: &mut App) -> AnyElement {
    if let Some(element) = input.custom_element {
        return div().w_full().child(element).into_any_element();
    }

    match input.kind {
        NavNodeKind::Section => render_section_row(input.label.unwrap_or(input.id), theme).into_any_element(),
        NavNodeKind::Item => render_item_row(input, handlers, theme, cx),
    }
}

fn render_section_row(label: SharedString, theme: &Arc<dyn SidebarTheme>) -> Div {
    let look = theme.resolve_section();

    div()
        .min_h(px(look.height))
        .pt(px(theme.geometry().section_padding_top))
        .text_size(px(look.typography.size))
        .line_height(px(look.typography.line_height))
        .font_weight(look.typography.weight)
        .font_features(theme.section_font_features())
        .text_color(look.label_color)
        .child(label)
}

fn render_item_row(
    input: RowRenderInput,
    handlers: RowHandlers,
    theme: &Arc<dyn SidebarTheme>,
    cx: &mut App,
) -> AnyElement {
    let RowRenderInput {
        id,
        label,
        icon,
        state,
        focus_handle,
        has_children,
        expansion_progress,
        disclosure_icons,
        tooltip,
        ..
    } = input;
    let interaction = InteractionState {
        hovered: state.hovered,
        pressed: state.pressed,
        focused: state.focused,
        disabled: !state.enabled,
        invalid: false,
    };
    let look = if has_children {
        theme.resolve_branch(interaction, ControlSize::Md)
    } else {
        theme.resolve_item(state.selected, interaction, ControlSize::Md)
    };
    let depth_indent = (look.icon_size + look.gap) * CHILD_DEPTH_INDENT_MULTIPLIER;
    let padding_left = look.padding_x + state.depth as f32 * depth_indent;
    let mut row = div()
        .id((id.clone(), 0usize))
        .role(gpui::Role::Button)
        .aria_label(label.clone().unwrap_or(id.clone()))
        .w_full()
        .min_h(px(look.height))
        .flex()
        .items_center()
        .gap(px(look.gap))
        .pl(px(padding_left))
        .pr(px(look.padding_x))
        .rounded(px(look.radius))
        .text_size(px(look.typography.size))
        .line_height(px(look.typography.line_height))
        .text_color(look.foreground)
        .font_weight(look.typography.weight);

    if let Some(icon) = icon {
        row = row.child(render_lucide_icon(icon, look.icon_color, look.icon_size));
    }

    row = row.child(div().flex_1().child(label.unwrap_or(id)));

    if has_children {
        row = row.child(render_disclosure_icon(
            &disclosure_icons,
            expansion_progress,
            look.icon_color,
            theme.geometry().disclosure_icon_size,
        ));
    }

    if let Some(background) = look.background {
        row = row.bg(background);
    }

    row = row.relative().child(crate::infra::focus_adorner::FocusAdorner::new(theme.metrics().focus).overlay(
        look.focus_border.unwrap_or_else(gpui::transparent_black),
        !interaction.disabled,
        look.radius,
        true,
    ));

    if state.enabled {
        row = row.cursor_pointer();
    } else {
        row = row.opacity(theme.geometry().disabled_opacity);
    }

    let RowHandlers { hover, mouse_down, mouse_up, mouse_up_out, click, .. } = handlers;
    let mut row = row;
    if state.enabled {
        if let Some(focus_handle) = focus_handle.as_ref() {
            row = row.track_focus(focus_handle);
        }
        if let Some(hover) = hover {
            row = row.on_hover(move |event, window, cx| hover(event, window, cx));
        }
        if let Some(mouse_down) = mouse_down {
            row = row.on_mouse_down(MouseButton::Left, move |event, window, cx| mouse_down(event, window, cx));
        }
        if let Some(mouse_up) = mouse_up {
            row = row.on_mouse_up(MouseButton::Left, move |event, window, cx| mouse_up(event, window, cx));
        }
        if let Some(mouse_up_out) = mouse_up_out {
            row = row.on_mouse_up_out(MouseButton::Left, move |event, window, cx| mouse_up_out(event, window, cx));
        }
        if let Some(click) = click {
            row = row.on_click(move |event, window, cx| click(event, window, cx));
        }
    }

    if let Some(tip) = tooltip {
        row = tip.render(row, interaction, cx);
    }
    row.into_any_element()
}

fn render_collapsed_rail_node(
    node: RenderedNavNode,
    handlers: RowHandlers,
    theme: &Arc<dyn SidebarTheme>,
    disclosure_icons: &DisclosureIcons,
    cx: &mut App,
) -> AnyElement {
    let RenderedNavNode { id, label, icon, state, focus_handle, has_children, tooltip, .. } = node;
    let interaction = InteractionState {
        hovered: state.hovered,
        pressed: state.pressed,
        focused: state.focused,
        disabled: !state.enabled,
        invalid: false,
    };
    let look = if has_children {
        theme.resolve_branch(interaction, ControlSize::Md)
    } else {
        theme.resolve_item(state.selected, interaction, ControlSize::Md)
    };
    let row_width = if has_children {
        rail_branch_button_width(look.height, &theme.geometry())
    } else {
        look.height
    };
    let mut row = div()
        .id((id.clone(), 1usize))
        .role(gpui::Role::Button)
        .aria_label(label.unwrap_or(id.clone()))
        .w(px(row_width))
        .h(px(look.height))
        .flex_none()
        .flex()
        .relative()
        .items_center()
        .justify_center()
        .rounded(px(look.radius))
        .text_color(look.foreground);

    if has_children {
        row = row.child(
            div()
                .absolute()
                .left(px(centered_icon_left(look.height, look.icon_size)))
                .top(px(centered_icon_left(look.height, look.icon_size)))
                .child(render_lucide_icon(
                    icon.expect("collapsed rail nodes always have icons"),
                    look.icon_color,
                    look.icon_size,
                )),
        );
        row = row.child(
            div()
                .absolute()
                .left(px(rail_branch_indicator_left(look.height, &theme.geometry())))
                .top(px(centered_icon_left(look.height, theme.geometry().rail_branch_indicator_size)))
                .child(render_icon_source(
                    &disclosure_icons.collapsed,
                    look.icon_color,
                    theme.geometry().rail_branch_indicator_size,
                )),
        );
    } else {
        row = row.child(render_lucide_icon(
            icon.expect("collapsed rail nodes always have icons"),
            look.icon_color,
            look.icon_size,
        ));
    }

    if let Some(background) = look.background {
        row = row.bg(background);
    }

    row = row.relative().child(crate::infra::focus_adorner::FocusAdorner::new(theme.metrics().focus).overlay(
        look.focus_border.unwrap_or_else(gpui::transparent_black),
        !interaction.disabled,
        look.radius,
        true,
    ));

    if state.enabled {
        row = row.cursor_pointer();
    } else {
        row = row.opacity(theme.geometry().disabled_opacity);
    }

    let RowHandlers { bounds, hover, mouse_down, mouse_up, mouse_up_out, click } = handlers;
    if state.enabled {
        if let Some(focus_handle) = focus_handle.as_ref() {
            row = row.track_focus(focus_handle);
        }
        if let Some(hover) = hover {
            row = row.on_hover(move |event, window, cx| hover(event, window, cx));
        }
        if let Some(mouse_down) = mouse_down {
            row = row.on_mouse_down(MouseButton::Left, move |event, window, cx| mouse_down(event, window, cx));
        }
        if let Some(mouse_up) = mouse_up {
            row = row.on_mouse_up(MouseButton::Left, move |event, window, cx| mouse_up(event, window, cx));
        }
        if let Some(mouse_up_out) = mouse_up_out {
            row = row.on_mouse_up_out(MouseButton::Left, move |event, window, cx| mouse_up_out(event, window, cx));
        }
        if let Some(click) = click {
            row = row.on_click(move |event, window, cx| click(event, window, cx));
        }
    }

    if let Some(tip) = tooltip {
        row = tip.render(row, interaction, cx);
    }
    if let Some(bounds) = bounds {
        div()
            .on_children_prepainted(move |child_bounds, window, cx| {
                if let Some(child_bounds) = child_bounds.first() {
                    bounds(child_bounds, window, cx);
                }
            })
            .child(row)
            .into_any_element()
    } else {
        row.into_any_element()
    }
}

fn render_rail_submenu_overlay(
    submenu: RenderedRailSubmenu,
    look: FloatingMenuLook,
    geometry: crate::theme::stylesheet::SidebarGeometry,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    bounds_handler: Option<SidebarPanelBoundsHandler>,
    mouse_down_out: SidebarPanelMouseDownHandler,
) -> impl IntoElement {
    let menu = render_floating_menu(
        &submenu.id,
        &submenu.items,
        submenu.open_submenu,
        submenu.active_path,
        look.clone(),
        item_hovers,
        item_clicks,
    );

    let content_size = submenu.content_size.unwrap_or_default();
    let offset = submenu.presence.adjust_offset(point(px(geometry.rail_submenu_offset_x), px(0.0)), content_size);
    let content = if let Some(bounds_handler) = bounds_handler {
        div()
            .on_mouse_down_out(move |event, window, cx| mouse_down_out(event, window, cx))
            .on_prepaint(move |bounds, window, cx| {
                bounds_handler(&bounds, window, cx);
            })
            .opacity(submenu.presence.opacity())
            .child(menu)
    } else {
        div().opacity(submenu.presence.opacity()).child(menu)
    };

    anchored()
        .snap_to_window_with_margin(px(geometry.window_margin))
        .anchor(Anchor::TopLeft)
        .position(point(submenu.parent_bounds.right(), submenu.parent_bounds.top()))
        .offset(offset)
        .child(content)
}

fn rail_branch_indicator_left(button_height: f32, geometry: &crate::theme::stylesheet::SidebarGeometry) -> f32 {
    button_height + geometry.rail_branch_overhang - geometry.rail_branch_indicator_size
}

fn rail_branch_button_width(button_height: f32, geometry: &crate::theme::stylesheet::SidebarGeometry) -> f32 {
    button_height.max(rail_branch_indicator_left(button_height, geometry) + geometry.rail_branch_indicator_size)
}

fn centered_icon_left(button_height: f32, icon_size: f32) -> f32 {
    (button_height - icon_size) * 0.5
}

fn render_lucide_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .child(crate::infra::icon::lucide_icon(icon, color, size))
        .into_any_element()
}

#[cfg(all(test, feature = "test-support"))]
mod focus_tests {
    use super::*;

    #[test]
    fn sidebar_rows_keep_bounds_when_the_focus_border_appears() {
        for collapsed in [false, true] {
            for has_children in [false, true] {
                let theme = default_sidebar_theme();
                crate::infra::focus_adorner::testing::assert_stable_layout(move |interaction, selected, _, cx| {
                    let id: SharedString = "node".into();
                    let state = super::super::NavNodeState {
                        id: id.clone(),
                        depth: 0,
                        sidebar_collapsed: collapsed,
                        index: 0,
                        sibling_count: 1,
                        selected,
                        hovered: interaction.hovered,
                        pressed: interaction.pressed,
                        focused: interaction.focused,
                        expanded: false,
                        enabled: !interaction.disabled,
                    };
                    let handlers = RowHandlers {
                        bounds: None,
                        hover: None,
                        mouse_down: None,
                        mouse_up: None,
                        mouse_up_out: None,
                        click: None,
                    };
                    if collapsed {
                        render_collapsed_rail_node(
                            RenderedNavNode {
                                drop_layout: None,
                                drag_drop: None,
                                tooltip: None,
                                id,
                                kind: NavNodeKind::Item,
                                label: Some("Node".into()),
                                icon: Some(LucideIcon::Folder),
                                state,
                                custom_element: None,
                                focus_handle: None,
                                has_children,
                                expansion_progress: 0.,
                                children_height_px: 0.,
                                children: Vec::new(),
                            },
                            handlers,
                            &theme,
                            &DisclosureIcons::default(),
                            cx,
                        )
                    } else {
                        render_item_row(
                            RowRenderInput {
                                tooltip: None,
                                id,
                                kind: NavNodeKind::Item,
                                label: Some("Node".into()),
                                icon: None,
                                state,
                                custom_element: None,
                                focus_handle: None,
                                has_children,
                                expansion_progress: 0.,
                                disclosure_icons: DisclosureIcons::default(),
                            },
                            handlers,
                            &theme,
                            cx,
                        )
                    }
                });
            }
        }
    }
}
