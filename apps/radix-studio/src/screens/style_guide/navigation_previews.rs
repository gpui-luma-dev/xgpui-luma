//! Presentation-only SDK template matrices for groups and navigation.
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};
use gpui::{AnyElement, App, Context, Entity, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::{accordion, control_group as group, dock_splitter, split_view, pager, sidebar, stepper};
use xgpui_luma::controls::{progress::ProgressDirection, scroll_container::ScrollContainer, tabs::Tabs};
use xgpui_luma::infra::{
    icon::{DisclosureIcons, SelectionStatusIcons},
    state::{CompositeItemState, ControlFocusState},
};
use xgpui_luma::theme::{ControlSize, InteractionState};
use xgpui_luma::GridTrack;
use xgpui_luma_look_radix::{self as radix, Accent, Gray, Look, SemanticRole};
use super::{
    matrix_grid::{column_header, empty_corner, equal_data_columns, fixed_grid, preview_tabbed, row_label},
    preview_handlers, states,
};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
    IconGroup,
    DockSplitter,
    SplitView,
    Accordion,
    Pager,
    Stepper,
    Sidebar,
}
impl Kind {
    pub(super) const ALL: [Self; 7] = [
        Self::IconGroup,
        Self::DockSplitter,
        Self::SplitView,
        Self::Accordion,
        Self::Pager,
        Self::Stepper,
        Self::Sidebar,
    ];
    fn key(self) -> &'static str {
        match self {
            Self::IconGroup => "icon-group",
            Self::DockSplitter => "dock-splitter",
            Self::SplitView => "split-view",
            Self::Accordion => "accordion",
            Self::Pager => "pager",
            Self::Stepper => "stepper",
            Self::Sidebar => "sidebar",
        }
    }
    fn preview_width(self) -> f32 {
        match self {
            Self::Accordion => 170.0,
            Self::DockSplitter | Self::SplitView => 160.0,
            Self::Pager => 300.0,
            _ => 220.0,
        }
    }
    fn rows(self) -> &'static [&'static str] {
        match self {
            Self::IconGroup => &["Single", "Multiple"],
            Self::DockSplitter => &["Vertical", "Horizontal"],
            Self::SplitView => &["Expanded", "Collapsed"],
            Self::Accordion => &["Collapsed", "Expanded"],
            Self::Pager => &["Minimal", "Minimal edge", "Numeric"],
            Self::Stepper => &["Horizontal", "Vertical"],
            Self::Sidebar => &["Expanded", "Icon rail"],
        }
    }
}
#[derive(Clone)]
pub(super) struct Previews {
    tabs: Vec<Entity<Tabs>>,
    splitter_focus: gpui::FocusHandle,
    // Sidebar templates require retained scroll surfaces, even for static samples.
    sidebar_scrolls: Rc<RefCell<HashMap<SharedString, ScrollContainer>>>,
}
impl Previews {
    pub(super) fn new<M: 'static>(look: &Look, cx: &mut Context<M>) -> Self {
        Self {
            tabs: Kind::ALL.iter().map(|kind| super::state::preview_navigation(kind.key(), look, cx)).collect(),
            sidebar_scrolls: Rc::default(),
            splitter_focus: cx.focus_handle(),
        }
    }
    pub(super) fn render(&self, kind: Kind, look: &Arc<Look>, window: &mut Window, cx: &mut App) -> AnyElement {
        let index = Kind::ALL.iter().position(|candidate| *candidate == kind).unwrap_or(0);
        let tabs = self.tabs[index].clone();
        let active = tabs.read(cx).active_id().cloned().unwrap_or_else(|| "template-preview".into());
        let fg = look.resolve_role(SemanticRole::Foreground).hsla();
        let muted = look.resolve_role(SemanticRole::MutedForeground).hsla();
        let border = look.resolve_role(SemanticRole::Border).hsla();
        if kind == Kind::Pager && active.as_ref() == "template-preview" {
            return preview_tabbed(tabs, border, pager_matrix(look, fg, muted, window, cx));
        }
        let labels = kind.rows();
        let width = kind.preview_width();
        let states = states::samples();
        let columns = match active.as_ref() {
            "colors" => labels.len(),
            "all-sizes" => 3,
            _ => states.len(),
        };
        let (label_width, gap) = if matches!(kind, Kind::Accordion | Kind::DockSplitter | Kind::SplitView) {
            (104.0, 12.0)
        } else {
            (120.0, 20.0)
        };
        let mut grid =
            fixed_grid(equal_data_columns(label_width, width, columns), gap, 24.0).child(empty_corner(), 0, 0);
        match active.as_ref() {
            "colors" => {
                for (col, label) in labels.iter().enumerate() {
                    grid = grid.child(column_header(*label, muted), 0, col + 1);
                }
                for (row, accent) in Accent::ALL.iter().enumerate() {
                    let fork = look.fork();
                    fork.set_palettes(*accent, Gray::Auto);
                    grid = grid.child(row_label(super::palettes::title_case(accent.as_str()), fg), row + 1, 0);
                    for col in 0..labels.len() {
                        grid = grid.child(
                            self.sample(
                                &fork,
                                Cell {
                                    kind,
                                    variant: col,
                                    size: ControlSize::Md,
                                    state: InteractionState { focused: true, ..Default::default() },
                                    scenario: 1,
                                    id: format!("color-{row}-{col}"),
                                },
                                window,
                                cx,
                            ),
                            row + 1,
                            col + 1,
                        );
                    }
                }
            }
            "all-sizes" => {
                let names = match kind {
                    Kind::Pager => ["First", "Middle", "Last"],
                    Kind::DockSplitter | Kind::SplitView => ["Compact", "Default", "Wide"],
                    Kind::Sidebar => ["Default", "Selected", "Disabled"],
                    _ => ["Size 1", "Size 2", "Size 3"],
                };
                for (col, label) in names.iter().enumerate() {
                    grid = grid.child(column_header(*label, muted), 0, col + 1);
                }
                for (row, label) in labels.iter().enumerate() {
                    grid = grid.child(row_label(*label, fg), row + 1, 0);
                    for (col, size) in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg].into_iter().enumerate() {
                        grid = grid.child(
                            self.sample(
                                look,
                                Cell {
                                    kind,
                                    variant: row,
                                    size,
                                    state: InteractionState {
                                        disabled: kind == Kind::Sidebar && col == 2,
                                        ..Default::default()
                                    },
                                    scenario: col,
                                    id: format!("size-{row}-{col}"),
                                },
                                window,
                                cx,
                            ),
                            row + 1,
                            col + 1,
                        );
                    }
                }
            }
            _ => {
                for (col, state) in states.iter().enumerate() {
                    let header = if kind == Kind::Pager {
                        column_header(["First", "Middle", "Last", "Page size menu", "Disabled"][col], muted)
                    } else if kind == Kind::SplitView {
                        column_header(["Default", "Hover", "Resize cue", "Hover-only", "Disabled"][col], muted)
                    } else if kind == Kind::Stepper {
                        column_header(
                            ["Not started", "In progress", "Complete", "State override", "Disabled"][col],
                            muted,
                        )
                    } else {
                        states::header_cell(state, muted)
                    };
                    grid = grid.child(header, 0, col + 1);
                }
                for (row, label) in labels.iter().enumerate() {
                    grid = grid.child(row_label(*label, fg), row + 1, 0);
                    for (col, state) in states.iter().enumerate() {
                        grid = grid.child(
                            self.sample(
                                look,
                                Cell {
                                    kind,
                                    variant: row,
                                    size: ControlSize::Md,
                                    state: state.state,
                                    scenario: col,
                                    id: format!("state-{row}-{col}"),
                                },
                                window,
                                cx,
                            ),
                            row + 1,
                            col + 1,
                        );
                    }
                }
            }
        }
        preview_tabbed(tabs, border, grid.into_any_element())
    }
    fn sample(&self, look: &Look, cell: Cell, window: &mut Window, cx: &mut App) -> AnyElement {
        let id = SharedString::from(format!("guide-{}-{}", cell.kind.key(), cell.id));
        let content = match cell.kind {
            Kind::IconGroup => group_sample(look, &id, &cell, window, cx),
            Kind::DockSplitter | Kind::SplitView => splitter_sample(look, &id, &cell, &self.splitter_focus, window, cx),
            Kind::Accordion => accordion_sample(look, &id, &cell, window, cx),
            Kind::Pager => pager_sample(look, &id, &cell, window, cx),
            Kind::Stepper => stepper_sample(look, &id, &cell, window, cx),
            Kind::Sidebar => {
                let mut scrolls = self.sidebar_scrolls.borrow_mut();
                let scroll = scrolls.entry(id.clone()).or_insert_with(|| {
                    ScrollContainer::new(format!("{id}-scroll"), radix::scrollbar_template(look), cx)
                });
                sidebar_sample(look, &id, &cell, scroll, window, cx)
            }
        };
        let cell_id = format!("{id}-cell");
        // Grid tracks align the comparison columns. Their width is not a control width:
        // a flex row measures groups intrinsically instead of block-stretching their shells.
        let cell_root = div().w_full().flex().items_center().justify_center().debug_selector(move || cell_id.clone());
        if cell.kind == Kind::IconGroup {
            cell_root.child(div().flex_none().flex().items_start().child(content)).into_any_element()
        } else {
            cell_root
                .child(div().w(px(cell.kind.preview_width())).flex_none().child(content))
                .into_any_element()
        }
    }
}
struct Cell {
    kind: Kind,
    variant: usize,
    size: ControlSize,
    state: InteractionState,
    scenario: usize,
    id: String,
}
fn focus(state: InteractionState) -> ControlFocusState {
    ControlFocusState { focused: state.focused, focus_visible: state.focused }
}
fn composite(state: InteractionState, selected: bool) -> CompositeItemState {
    CompositeItemState {
        disabled: state.disabled,
        hovered: state.hovered,
        pressed: state.pressed,
        selected,
        active: state.focused,
        focus_visible: state.focused,
    }
}
fn group_sample(look: &Look, id: &SharedString, cell: &Cell, window: &mut Window, cx: &mut App) -> AnyElement {
    let items: Vec<_> = ["Bold", "Italic", "Underline"]
        .into_iter()
        .enumerate()
        .map(|(i, label)| group::ControlGroupItem::new(i.to_string()).label(label).enabled(i != 2))
        .collect();
    let layout = group::ControlGroupLayout::Horizontal;
    let sheet = look.common_stylesheet();
    let item_template = radix::icon_group_item_template(look, cell.size);
    let mode = if cell.variant == 1 {
        group::ControlSelectionMode::Multiple
    } else {
        group::ControlSelectionMode::SingleAllowNone
    };
    let glyphs = group::make_control_group_item_template(
        |item: &group::ControlGroupItemRenderModel<'_, group::ControlGroupItem>, _, _| {
            let glyph = match item.index {
                0 => "B",
                1 => "I",
                _ => "U",
            };
            div().child(glyph).into_any_element()
        },
    );
    let selected = |index| index == 0 || (cell.variant == 1 && index == 1);
    let selected_ids: Vec<SharedString> =
        (0..items.len()).filter(|index| selected(*index)).map(|index| index.to_string().into()).collect();
    let model = group::ControlGroupRenderModel {
        id,
        stylesheet: &sheet,
        shell_stylesheet: None,
        items: items
            .iter()
            .enumerate()
            .map(|(index, item)| group::ControlGroupItemRenderModel {
                group_id: id,
                stylesheet: &sheet,
                layout,
                item,
                index,
                sibling_count: items.len(),
                selected: selected(index),
                selection_progress: if selected(index) { 1.0 } else { 0.0 },
                active: index == 1,
                enabled: !cell.state.disabled && index != 2,
                state: composite(
                    InteractionState { disabled: cell.state.disabled || index == 2, ..cell.state },
                    selected(index),
                ),
                selection_mode: mode,
                foreground: None,
            })
            .collect(),
        selected_ids: &selected_ids,
        active_id: None,
        selection_mode: mode,
        state_mode: group::ControlGroupStateMode::Unmanaged,
        enabled: !cell.state.disabled,
        layout,
        scroll_handle: None,
        focus: focus(cell.state),
        focus_strategy: group::ControlGroupFocusStrategy::ActiveDescendant,
        item_template: Some(&glyphs),
        item_element_template: Some(&item_template),
    };
    let template = radix::icon_group_template(look);
    let control_id = format!("{id}-control");
    template(&model, preview_handlers::control_group_handlers(items.len()), window, cx)
        .debug_selector(move || control_id.clone())
        .into_any_element()
}
fn splitter_sample(
    look: &Look,
    id: &SharedString,
    cell: &Cell,
    focus: &gpui::FocusHandle,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let fork = look.fork();
    if cell.id.starts_with("size-") {
        let mut sheet = fork.common_stylesheet();
        let hit = [8.0, 12.0, 20.0][cell.scenario];
        sheet.dock_splitter.geometry.hit_target = Some(hit);
        sheet.split_view.geometry.hitbox_width = Some(hit);
        sheet.split_view.geometry.cue_width = Some([2.0, 4.0, 6.0][cell.scenario]);
        sheet.split_view.geometry.cue_hovered_width = Some([4.0, 6.0, 8.0][cell.scenario]);
        fork.set_common_stylesheet(sheet);
    }
    let surface = fork.resolve_role(SemanticRole::Surface).hsla();
    let fg = fork.resolve_role(SemanticRole::MutedForeground).hsla();
    let control_id = format!("{id}-control");
    let frame = div()
        .w_full()
        .h(px(96.0))
        .flex()
        .items_center()
        .justify_center()
        .bg(surface)
        .text_color(fg)
        .text_xs();
    if cell.kind == Kind::DockSplitter {
        let model = dock_splitter::DockSplitterRenderModel {
            id,
            orientation: if cell.variant == 0 {
                dock_splitter::SplitterOrientation::Vertical
            } else {
                dock_splitter::SplitterOrientation::Horizontal
            },
            enabled: !cell.state.disabled,
            hovered: cell.state.hovered && !cell.state.disabled,
            dragging: cell.state.pressed && !cell.state.disabled,
            focused: cell.state.focused && !cell.state.disabled,
            focus_handle: focus,
        };
        return frame
            .child(
                radix::dock_splitter_template(true)
                    .render(
                        &model,
                        &radix::dock_splitter_theme(&fork).resolve(!cell.state.disabled),
                        preview_handlers::dock_splitter_handlers(),
                        window,
                        cx,
                    )
                    .debug_selector(move || control_id.clone()),
            )
            .into_any_element();
    }
    let theme = radix::split_view_theme(&fork);
    let collapsed = cell.variant == 1;
    let model = split_view::SplitViewRenderModel {
        id,
        sidebar_width: px(48.0),
        sidebar_collapsed_width: px(0.0),
        effective_sidebar_width: px(if collapsed { 0.0 } else { 48.0 }),
        collapsed,
        resizable: true,
        enabled: !cell.state.disabled,
        separator_hovered: !cell.state.disabled && (cell.state.hovered || cell.state.pressed || cell.state.focused),
        separator_visibility: if cell.scenario == 3 {
            split_view::SplitViewSeparatorVisibility::Hover
        } else {
            split_view::SplitViewSeparatorVisibility::Always
        },
        theme: &theme,
        separator_color_override: None,
        separator_hover_color_override: None,
    };
    frame
        .child(
            radix::split_view_template()
                .render(
                    &model,
                    div().size_full().flex().items_center().justify_center().child("Side").into_any_element(),
                    div().size_full().flex().items_center().justify_center().child("Main").into_any_element(),
                    preview_handlers::split_view_handlers(),
                    window,
                    cx,
                )
                .debug_selector(move || control_id.clone()),
        )
        .into_any_element()
}
fn accordion_sample(look: &Look, id: &SharedString, cell: &Cell, window: &mut Window, cx: &mut App) -> AnyElement {
    let trigger = accordion::AccordionTrigger::new("Account settings");
    let content = accordion::AccordionContent::new("Manage your profile and preferences.");
    let icons = DisclosureIcons::from_stylesheet(&look.common_stylesheet());
    let expanded = cell.variant == 1;
    let model = accordion::AccordionRenderModel {
        id,
        items: vec![accordion::AccordionItemRenderModel {
            id,
            trigger: &trigger,
            content: &content,
            expanded,
            progress: if expanded { 1.0 } else { 0.0 },
            content_height_px: 48.0,
            enabled: !cell.state.disabled,
            state: composite(cell.state, expanded),
        }],
        selection_mode: accordion::AccordionSelectionMode::Single,
        collapsible: true,
        enabled: !cell.state.disabled,
        size: cell.size,
        item_dividers: true,
        content_padding_y: None,
        content_padding_top: None,
        content_padding_bottom: None,
        trigger_min_height: None,
        trigger_padding_y: None,
        disclosure_icons: &icons,
        focus: focus(cell.state),
    };
    radix::accordion_template(look)
        .render(&model, preview_handlers::accordion_handlers(1), window, cx)
        .into_any_element()
}
// Compare full SDK pagers at widths that fit their navigation controls.
fn pager_matrix(look: &Look, fg: gpui::Hsla, muted: gpui::Hsla, window: &mut Window, cx: &mut App) -> AnyElement {
    let widths = [200.0, 240.0, 360.0];
    let mut grid = fixed_grid(
        std::iter::once(GridTrack::Px(104.0)).chain(widths.map(GridTrack::Px)).collect(),
        12.0,
        24.0,
    )
    .child(empty_corner(), 0, 0);
    for (col, label) in Kind::Pager.rows().iter().enumerate() {
        grid = grid.child(column_header(*label, muted), 0, col + 1);
    }
    for (row, (label, scenario)) in [("First", 0), ("Middle", 1), ("Last", 2), ("Disabled", 4)].into_iter().enumerate()
    {
        grid = grid.child(row_label(label, fg), row + 1, 0);
        for (col, width) in widths.into_iter().enumerate() {
            let id = SharedString::from(format!("guide-pager-state-{col}-{scenario}"));
            let cell = Cell {
                kind: Kind::Pager,
                variant: col,
                size: ControlSize::Md,
                state: InteractionState { disabled: scenario == 4, ..Default::default() },
                scenario,
                id: String::new(),
            };
            let selector = format!("{id}-cell");
            grid = grid.child(
                div()
                    .w(px(width))
                    .flex_none()
                    .debug_selector(move || selector.clone())
                    .child(pager_sample(look, &id, &cell, window, cx)),
                row + 1,
                col + 1,
            );
        }
    }
    grid.into_any_element()
}
fn pager_sample(look: &Look, id: &SharedString, cell: &Cell, window: &mut Window, cx: &mut App) -> AnyElement {
    let icons = pager::PagerIcons::from_stylesheet(&look.common_stylesheet());
    let params = pager::PagerTemplateParameters::default().show_first_last(false).numeric_slot_count(3);
    let model = pager::PagerRenderModel {
        id,
        current_page: match cell.scenario {
            0 => 0,
            2 => 9,
            _ => 4,
        },
        page_count: 10,
        page_size: 20,
        page_size_options: &[10, 20, 50],
        page_size_open: cell.scenario == 3 && !cell.state.disabled,
        enabled: !cell.state.disabled,
        style: [pager::PagerStyle::Minimal, pager::PagerStyle::MinimalEdge, pager::PagerStyle::Numeric][cell.variant],
        info_text: None,
        info_slot: None,
        template_parameters: &params,
        icons: &icons,
    };
    radix::pager_template(look).render(&model, preview_handlers::pager_handlers(), window, cx)
}
fn stepper_sample(look: &Look, id: &SharedString, cell: &Cell, window: &mut Window, cx: &mut App) -> AnyElement {
    let icons = SelectionStatusIcons::from_stylesheet(&look.common_stylesheet());
    let labels = ["Account".into(), "Details".into(), "Review".into()];
    use stepper::StepState::{Complete, InProgress, Incomplete};
    let (current, status) = match cell.scenario {
        0 => (0, [InProgress, Incomplete, Incomplete]),
        2 => (2, [Complete, Complete, Complete]),
        3 => (1, [Complete, Incomplete, Incomplete]),
        _ => (1, [Complete, InProgress, Incomplete]),
    };
    let model = stepper::StepperRenderModel {
        id,
        step_count: 3,
        current_step: current,
        display_step: current as f32,
        transition_progress: 1.0,
        from_step: 0.0,
        to_step: current,
        step_states: &status,
        labels: &labels,
        label_placement: stepper::StepperLabelPlacement::End,
        direction: if cell.variant == 0 {
            ProgressDirection::LeftToRight
        } else {
            ProgressDirection::TopToBottom
        },
        size: cell.size,
        enabled: !cell.state.disabled,
        step_contents: &[],
        content_height: None,
        icons: &icons,
    };
    radix::stepper_template(look).render(&model, window, cx).into_any_element()
}
fn sidebar_sample(
    look: &Look,
    id: &SharedString,
    cell: &Cell,
    scroll: &ScrollContainer,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let collapsed = cell.variant == 1;
    let node = |index: usize, label: &str, enabled: bool| {
        let node_id = SharedString::from(format!("{id}-{index}"));
        sidebar::SidebarRenderedNode {
            drop_layout: None,
            drag_drop: None,
            tooltip: None,
            id: node_id.clone(),
            kind: sidebar::SidebarNodeKind::Item,
            label: Some(label.to_owned().into()),
            icon: Some(
                [lucide_svg_static::Icon::House, lucide_svg_static::Icon::Folder, lucide_svg_static::Icon::Settings]
                    [index],
            ),
            state: sidebar::SidebarNodeState {
                id: node_id,
                depth: 0,
                sidebar_collapsed: collapsed,
                index,
                sibling_count: 3,
                selected: index == 0 && cell.scenario != 0,
                hovered: index == 1 && cell.state.hovered,
                pressed: index == 1 && cell.state.pressed,
                focused: index == 1 && cell.state.focused,
                expanded: false,
                enabled: enabled && !cell.state.disabled,
            },
            custom_element: None,
            focus_handle: None,
            has_children: false,
            expansion_progress: 0.0,
            children_height_px: 0.0,
            children: Vec::new(),
        }
    };
    let nodes = || {
        let mut branch = node(1, "Projects", true);
        branch.has_children = true;
        if !collapsed {
            branch.state.expanded = true;
            branch.expansion_progress = 1.0;
            branch.children_height_px = 36.0;
            let mut child = node(0, "Recent", true);
            child.id = format!("{id}-recent").into();
            child.state.id = child.id.clone();
            child.state.depth = 1;
            child.state.selected = false;
            child.state.sibling_count = 1;
            branch.children.push(child);
        }
        vec![node(0, "Overview", true), branch, node(2, "Settings", false)]
    };
    let mut section = node(0, "Workspace", false);
    section.id = format!("{id}-section").into();
    section.state.id = section.id.clone();
    section.kind = sidebar::SidebarNodeKind::Section;
    section.icon = None;
    let model = sidebar::SidebarPanelRenderModel {
        id: id.clone(),
        title: None,
        subtitle: None,
        header_nodes: if collapsed { Vec::new() } else { vec![section] },
        nodes: if collapsed { Vec::new() } else { nodes() },
        footer_nodes: Vec::new(),
        rail_nodes: if collapsed { nodes() } else { Vec::new() },
        rail_footer_nodes: Vec::new(),
        rail_submenu: None,
        selected_id: (cell.scenario != 0).then(|| format!("{id}-0").into()),
        collapsed,
        disclosure_icons: DisclosureIcons::from_stylesheet(&look.common_stylesheet()),
    };
    div()
        .w_full()
        .h(px(220.0))
        .child(radix::sidebar_template(look).render(model, scroll, Default::default(), window, cx))
        .into_any_element()
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Render, TestAppContext};
    use xgpui_luma::theme::ThemeMode;
    struct Harness {
        look: Arc<Look>,
        previews: Previews,
        kind: Kind,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.previews.render(self.kind, &self.look, window, cx)
        }
    }
    #[test]
    fn every_navigation_preview_tab_renders_in_both_modes_without_live_controls() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(Look::built_in());
            Harness { previews: Previews::new(&look, cx), look, kind: Kind::IconGroup }
        });
        cx.simulate_resize(gpui::size(px(1600.0), px(900.0)));
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for (index, kind) in Kind::ALL.into_iter().enumerate() {
                for tab in ["template-preview", "colors", "all-sizes"] {
                    cx.update(|_, app| {
                        view.update(app, |view, cx| {
                            view.look.set_mode(mode);
                            view.kind = kind;
                            let tabs = view.previews.tabs[index].clone();
                            tabs.update(cx, |tabs, cx| tabs.set_active(tab, cx));
                            cx.notify();
                        })
                    });
                    cx.run_until_parked();
                }
            }
        }
    }
    #[test]
    fn icon_group_preview_shells_keep_intrinsic_width_inside_grid_cells() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(Look::built_in());
            Harness { previews: Previews::new(&look, cx), look, kind: Kind::IconGroup }
        });
        cx.simulate_resize(gpui::size(px(1600.0), px(900.0)));
        cx.run_until_parked();
        let icons = cx.debug_bounds("guide-icon-group-state-0-0-control").unwrap();
        let cell = cx.debug_bounds("guide-icon-group-state-0-0-cell").unwrap();
        assert!(icons.size.width < cell.size.width * 0.7, "icon shell must hug its three hit targets");
        assert!((icons.center().x - cell.center().x).abs() < px(1.0));
        cx.update(|_, app| {
            view.update(app, |view, cx| {
                let tabs = view.previews.tabs[0].clone();
                tabs.update(cx, |tabs, cx| tabs.set_active("all-sizes", cx));
                cx.notify();
            })
        });
        cx.run_until_parked();
        let largest = cx.debug_bounds("guide-icon-group-size-0-2-control").unwrap();
        let largest_cell = cx.debug_bounds("guide-icon-group-size-0-2-cell").unwrap();
        assert!(
            largest.size.width <= largest_cell.size.width,
            "largest group must fit without clipping: {:?} > {:?}",
            largest.size.width,
            largest_cell.size.width
        );
    }
    #[test]
    fn accordion_states_fit_in_a_compact_preview() {
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(Look::built_in());
            Harness { previews: Previews::new(&look, cx), look, kind: Kind::Accordion }
        });
        cx.simulate_resize(gpui::size(px(1040.0), px(600.0)));
        cx.run_until_parked();
        for selector in [
            "guide-accordion-state-0-0-cell",
            "guide-accordion-state-0-4-cell",
            "guide-accordion-state-1-0-cell",
            "guide-accordion-state-1-4-cell",
        ] {
            let bounds = cx.debug_bounds(selector).unwrap();
            assert_eq!(bounds.size.width, px(170.0));
            assert!(bounds.left() >= px(0.0));
            assert!(bounds.right() <= px(1040.0));
        }
    }
    #[test]
    fn pager_matrix_fits_all_styles_without_empty_menu_height() {
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(Look::built_in());
            Harness { previews: Previews::new(&look, cx), look, kind: Kind::Pager }
        });
        cx.simulate_resize(gpui::size(px(1040.0), px(600.0)));
        cx.run_until_parked();
        let first = cx.debug_bounds("guide-pager-state-0-0-cell").unwrap();
        let numeric = cx.debug_bounds("guide-pager-state-2-0-cell").unwrap();
        let middle = cx.debug_bounds("guide-pager-state-0-1-cell").unwrap();
        let last = cx.debug_bounds("guide-pager-state-2-4-cell").unwrap();
        assert_eq!(first.top(), numeric.top());
        assert!(first.left() >= px(0.0));
        assert!(numeric.right() <= px(1040.0));
        assert_eq!(numeric.size.width, px(360.0));
        assert!(first.size.height < px(80.0));
        assert!(middle.top() > first.bottom());
        assert!(last.right() <= px(1040.0));
        assert!(last.bottom() < px(600.0));
    }
    #[test]
    fn splitter_preview_frames_fit_without_stretching_the_controls() {
        for (kind, first_id, last_id) in [
            (Kind::DockSplitter, "guide-dock-splitter-state-0-0-cell", "guide-dock-splitter-state-1-4-cell"),
            (Kind::SplitView, "guide-split-view-state-0-0-control", "guide-split-view-state-1-4-control"),
        ] {
            let mut app = TestAppContext::single();
            let (_, cx) = app.add_window_view(|_, cx| {
                let look = Arc::new(Look::built_in());
                Harness { previews: Previews::new(&look, cx), look, kind }
            });
            cx.simulate_resize(gpui::size(px(1040.0), px(600.0)));
            cx.run_until_parked();
            let first = cx.debug_bounds(first_id).unwrap();
            let last = cx.debug_bounds(last_id).unwrap();
            assert_eq!(first.size.width, px(160.0));
            assert!(first.left() >= px(0.0));
            assert!(last.right() <= px(1040.0));
            assert!(last.bottom() <= px(600.0));
        }
    }
}
