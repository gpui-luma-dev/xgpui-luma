//! Navigation sidebar control exposition — `SidebarControl` preview and event log.

use xgpui_luma::controls::sidebar::SidebarEventExt;
use xgpui_luma::controls::popup_menu::PopupMenuEventExt;
use std::cell::Cell;
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::motion::VisualTransition;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{ButtonContentContext, ControlIcon};
use xgpui_luma::controls::icon_button::IconButton;
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::controls::popup_menu::{HasPresenter, PopupMenu, PopupMenuEvent, PopupMenuPlacement};
use xgpui_luma::infra::presenter::ControlPresenter;
use xgpui_luma::controls::scroll_container::ScrollbarAutoHideActivate;
use xgpui_luma::controls::sidebar::{SidebarPresentation, SidebarControl, SidebarEvent};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize};
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::collection_theme_inspectors::SidebarThemeInspector;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::ControlExpositionLayout;
use super::inspector::{SidebarInspectorAdapter, SIDEBAR_INSPECTOR_SPEC, sidebar_motion_report};
use super::template::render_control_exposition_card;

#[derive(Clone, Copy)]
struct PropertyLeaf {
    id: &'static str,
    label: &'static str,
    icon: Option<LucideIcon>,
    enabled: bool,
}

#[derive(Clone, Copy)]
struct PropertyGroup {
    id: &'static str,
    label: &'static str,
    icon: LucideIcon,
    expanded: bool,
    leaves: &'static [PropertyLeaf],
}

const INITIAL_PROPERTY_SELECTION_ID: &str = "dimensions";

const PINNED_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "summary", label: "Summary", icon: Some(LucideIcon::Info), enabled: true },
    PropertyLeaf { id: "tokens", label: "Design Tokens", icon: Some(LucideIcon::Tags), enabled: true },
];

const LAYOUT_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "position", label: "Position", icon: None, enabled: true },
    PropertyLeaf { id: INITIAL_PROPERTY_SELECTION_ID, label: "Dimensions", icon: None, enabled: true },
    PropertyLeaf { id: "constraints", label: "Constraints", icon: None, enabled: true },
    PropertyLeaf { id: "grid", label: "Grid", icon: None, enabled: true },
];

const LOOK_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "fill", label: "Fill", icon: None, enabled: true },
    PropertyLeaf { id: "stroke", label: "Stroke", icon: None, enabled: true },
    PropertyLeaf { id: "typography", label: "Typography", icon: None, enabled: true },
    PropertyLeaf { id: "effects", label: "Effects", icon: None, enabled: true },
];

const BEHAVIOR_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "interactions", label: "Interactions", icon: None, enabled: true },
    PropertyLeaf { id: "conditions", label: "Conditions", icon: None, enabled: true },
    PropertyLeaf { id: "validation", label: "Validation", icon: None, enabled: true },
    PropertyLeaf { id: "data-binding", label: "Data Binding", icon: None, enabled: false },
];

const PROPERTY_GROUPS: &[PropertyGroup] = &[
    PropertyGroup { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded: true, leaves: LAYOUT_PROPERTIES },
    PropertyGroup { id: "look", label: "Look", icon: LucideIcon::Palette, expanded: true, leaves: LOOK_PROPERTIES },
    PropertyGroup {
        id: "behavior",
        label: "Behavior",
        icon: LucideIcon::MousePointer2,
        expanded: false,
        leaves: BEHAVIOR_PROPERTIES,
    },
];

const USER_MENU_NAME: &str = "shadcn";
const USER_MENU_EMAIL: &str = "m@example.com";

pub struct SidebarControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<SidebarExpositionLeftPane>,
    theme_inspector: Entity<SidebarThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct SidebarExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    sidebar_control: Entity<SidebarControl>,
    sidebar_toggle: IconButton,
    user_menu: Entity<PopupMenu>,
    control_open: Rc<Cell<bool>>,
    width_transition: VisualTransition,
    event_stream: Entity<ControlEventStream>,
}

impl SidebarExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.sidebar_control.update(cx, |_, cx| cx.notify());
        self.sidebar_toggle.update(cx, |_, cx| cx.notify());
        self.user_menu.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }

    fn sync_sidebar_toggle_icon(&self, cx: &mut Context<Self>) {
        let icon = if self.control_open.get() {
            ControlIcon::Lucide(LucideIcon::PanelLeft)
        } else {
            ControlIcon::Lucide(LucideIcon::PanelLeftOpen)
        };
        self.sidebar_toggle.update(cx, |button, cx| {
            button.set_presenter(sidebar_toggle_presenter(icon), cx);
        });
    }

    fn sync_user_menu_trigger(&self, cx: &mut Context<Self>) {
        let expanded = self.control_open.get();
        self.user_menu.update(cx, |menu, cx| {
            if expanded {
                menu.set_presenter(user_menu_content(), cx);
                menu.set_end_icon(Some(LucideIcon::EllipsisVertical), cx);
            } else {
                menu.set_icon(LucideIcon::CircleUser, cx);
                menu.set_end_icon(None, cx);
            }
        });
    }
}

impl Render for SidebarExpositionLeftPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.width_transition.set_target(if self.control_open.get() { 1.0 } else { 0.0 });
        self.width_transition.sync();
        self.width_transition.schedule_frame(window, cx);
        self.sync_sidebar_toggle_icon(cx);
        self.sync_user_menu_trigger(cx);

        with_look(&self.look, || {
            let look = &self.look;
            let metrics = look.sidebar_metric_scale();
            let control_width =
                self.width_transition.interpolate_pixels(metrics.width_icon_rail, metrics.width_expanded);

            let preview = div()
                .w_full()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(div().text_sm().text_color(look.chrome().muted_text).child("Drag row edges before/after; branch centers move into branches. Hold near an edge to scroll; Escape cancels."))
                .child(preview_shell(
                    look,
                    control_width,
                    shadcn::Frame::sidebar("exposition-nav-frame")
                        .look(look)
                        .child(self.sidebar_control.clone())
                        .render(cx)
                        .into_any_element(),
                    self.sidebar_toggle.clone(),
                    self.user_menu.clone(),
                ))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-sidebar-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl SidebarControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("sidebar").expect("sidebar catalog entry");

        let sidebar_control = spawn_sidebar_control(&look, cx);

        let sidebar_toggle = shadcn::Button::icon_button("controls-doc-sidebar-toggle", LucideIcon::PanelLeft)
            .look(look.as_ref())
            .content_only()
            .size(shadcn::ShadcnSize::Sm)
            .spawn(cx);

        let user_menu = spawn_user_menu(&look, cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-sidebar-event-log",
                "Interact with the sidebar to inspect events.",
            )
        });

        let control_open = Rc::new(Cell::new(true));
        let left_pane = cx.new(|_| SidebarExpositionLeftPane {
            look: look.clone(),
            entry,
            sidebar_control: sidebar_control.clone(),
            sidebar_toggle: sidebar_toggle.clone(),
            user_menu: user_menu.clone(),
            control_open: control_open.clone(),
            width_transition: VisualTransition::default(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-sidebar-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &SIDEBAR_INSPECTOR_SPEC,
            SidebarInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(sidebar_motion_report([("Properties", sidebar_control.clone())]), cx);
        });

        let control_open_cell = control_open.clone();
        let observe_subscription = cx.observe(&sidebar_control, {
            let left_pane = left_pane.clone();
            move |_, _, cx| {
                left_pane.update(cx, |_, cx| cx.notify());
            }
        });
        sidebar_control.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            let left_pane = left_pane.downgrade();
            move |_, event: &SidebarEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };
                if let SidebarEvent::PresentationChanged { presentation } = event {
                    control_open_cell.set(*presentation == SidebarPresentation::Expanded);
                    left_pane.update(cx, |_, cx| cx.notify());
                }
                if let Some(line) = format_sidebar_control_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        });
        sidebar_toggle.clone().on_click(cx, {
            let sidebar_control = sidebar_control.downgrade();
            move |_, cx| {
                let Some(sidebar_control) = sidebar_control.upgrade() else {
                    return;
                };

                sidebar_control.update(cx, |sidebar, cx| {
                    let presentation = if sidebar.presentation() == SidebarPresentation::Expanded {
                        SidebarPresentation::Icons
                    } else {
                        SidebarPresentation::Expanded
                    };
                    sidebar.set_presentation(presentation, cx);
                });
            }
        });
        user_menu.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |_, event: &PopupMenuEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                let line = format_user_menu_event(event);
                event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
            }
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: vec![observe_subscription] }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for SidebarControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-sidebar-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

/// App-shell stage: outer canvas, left sidebar rail, inset content pane on the right.
const PREVIEW_SHELL_HEIGHT_PX: f32 = 560.0;
const PREVIEW_SHELL_RADIUS_PX: f32 = 12.0;
const PREVIEW_CONTENT_INSET_PX: f32 = 10.0;

fn preview_shell(
    look: &ShadcnLook,
    sidebar_width: gpui::Pixels,
    sidebar: gpui::AnyElement,
    sidebar_toggle: IconButton,
    user_menu: Entity<PopupMenu>,
) -> gpui::AnyElement {
    let chrome = look.chrome();
    let title_style = look.typography_scale(ShadcnTextSize::Lg);
    let muted_style = look.typography_scale(ShadcnTextSize::Sm);
    let muted = look.token_color("muted-foreground").unwrap_or(chrome.muted_text);
    let sidebar_bg = look.token_color("sidebar").unwrap_or(chrome.panel_background);
    let content_radius = look.mode_tokens().metrics.radius.lg;

    div()
        .id("controls-doc-sidebar-preview-shell")
        .w_full()
        .min_w(px(0.0))
        .h(px(PREVIEW_SHELL_HEIGHT_PX))
        .flex()
        .flex_row()
        .overflow_hidden()
        .rounded(px(PREVIEW_SHELL_RADIUS_PX))
        .border_1()
        .border_color(chrome.border)
        .bg(sidebar_bg)
        .child(
            div()
                .id("controls-doc-sidebar-preview-rail")
                .flex_none()
                .w(sidebar_width)
                .h_full()
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(div().flex_1().min_h(px(0.0)).w_full().overflow_hidden().child(sidebar))
                .child(
                    div()
                        .id("controls-doc-sidebar-user-menu")
                        .flex_none()
                        .w_full()
                        .px(px(8.0))
                        .pb(px(8.0))
                        .child(user_menu),
                ),
        )
        .child(
            div()
                .id("controls-doc-sidebar-preview-content")
                .flex_1()
                .min_w(px(0.0))
                .h_full()
                .p(px(PREVIEW_CONTENT_INSET_PX))
                .child(
                    div()
                        .size_full()
                        .min_h(px(0.0))
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .rounded(px(content_radius))
                        .border_1()
                        .border_color(chrome.border)
                        .bg(chrome.content_background)
                        .child(
                            div()
                                .id("controls-doc-sidebar-preview-content-header")
                                .w_full()
                                .flex_shrink_0()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .h(px(44.0))
                                .px(px(12.0))
                                .border_b_1()
                                .border_color(chrome.border)
                                .child(sidebar_toggle)
                                .child(div().h(px(16.0)).w(px(1.0)).bg(chrome.border))
                                .child(
                                    div()
                                        .typography_style(title_style)
                                        .text_color(chrome.title_text)
                                        .child("Documents"),
                                ),
                        )
                        .child(
                            div().flex_1().min_h(px(0.0)).w_full().p(px(16.0)).child(
                                div()
                                    .size_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(8.0))
                                    .border_1()
                                    .border_color(chrome.border)
                                    .bg(chrome.panel_background)
                                    .child(
                                        div()
                                            .typography_style(muted_style)
                                            .text_color(muted)
                                            .child("Main content area"),
                                    ),
                            ),
                        ),
                ),
        )
        .into_any_element()
}

fn sidebar_toggle_presenter(icon: ControlIcon) -> ControlPresenter<ButtonContentContext<()>> {
    Arc::new(move |model, _| {
        let color = model.look.foreground;
        match &icon {
            ControlIcon::Lucide(lucide) => div()
                .text_size(px(16.0))
                .child(xgpui_luma::infra::icon::lucide_icon(*lucide, color, 16.0))
                .into_any_element(),
            ControlIcon::SvgPath(path) => {
                gpui::svg().size(px(16.0)).text_color(color).path(path.clone()).into_any_element()
            }
        }
    })
}

fn spawn_sidebar_control(look: &Arc<ShadcnLook>, cx: &mut Context<SidebarControlExposition>) -> Entity<SidebarControl> {
    let mut pinned_menu = shadcn::Sidebar::menu("pinned_menu");
    for leaf in PINNED_PROPERTIES {
        pinned_menu = pinned_menu.item(property_leaf_menu_item(look, leaf).draggable(leaf.id != "summary"));
    }

    let mut properties_menu = shadcn::Sidebar::menu("properties_menu");
    for group in PROPERTY_GROUPS {
        let mut sub = shadcn::Sidebar::menu_sub();
        for leaf in group.leaves {
            sub = sub.item(property_leaf_menu_item(look, leaf));
        }
        properties_menu = properties_menu
            .item(shadcn::Sidebar::menu_item(group.id, group.label).icon(group.icon).expanded(group.expanded).sub(sub));
    }

    shadcn::Sidebar::new("controls-doc-sidebar-control")
        .look(look.as_ref())
        .drag_drop(xgpui_luma::controls::sidebar::SidebarDragDrop::new())
        .auto_hide_scrollbar(true)
        .auto_hide_scrollbar_activate(ScrollbarAutoHideActivate::Move)
        .sidebar(
            shadcn::Sidebar::panel("workbench_sidebar")
                .header(shadcn::Sidebar::header().title("Properties").subtitle("Rectangle / Prominent card"))
                .content(
                    shadcn::Sidebar::content()
                        .group(shadcn::Sidebar::group().id("pinned").label("Pinned").menu(pinned_menu))
                        .group(shadcn::Sidebar::group().id("properties").label("Properties").menu(properties_menu)),
                ),
        )
        .overlay_scrollbar(true)
        .spawn(cx)
}

fn user_menu_content() -> ControlPresenter<xgpui_luma::controls::popup_menu::PopupMenuTriggerModel> {
    Arc::new(|model, _| {
        div()
            .flex_1()
            .min_w(px(0.0))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(div().truncate().child(model.label.clone()))
            .child(div().truncate().text_size(px(12.0)).opacity(0.65).child(USER_MENU_EMAIL))
            .into_any_element()
    })
}

fn spawn_user_menu(look: &Arc<ShadcnLook>, cx: &mut Context<SidebarControlExposition>) -> Entity<PopupMenu> {
    let mut builder = shadcn::PopupMenu::new("controls-doc-sidebar-user-menu")
        .look(look.as_ref())
        .label(USER_MENU_NAME)
        .end_icon(LucideIcon::EllipsisVertical)
        .full_width(true)
        .ghost()
        .without_elevation()
        .placement(PopupMenuPlacement::RightEnd)
        .items(user_menu_items());
    builder.set_presenter(user_menu_content());
    builder.spawn(cx)
}

fn user_menu_items() -> [MenuItem; 4] {
    [
        MenuItem::new("account").label("Account").icon(LucideIcon::CircleUser),
        MenuItem::new("billing").label("Billing").icon(LucideIcon::CreditCard),
        MenuItem::new("notifications").label("Notifications").icon(LucideIcon::Bell),
        MenuItem::new("log-out").label("Log out").icon(LucideIcon::LogOut),
    ]
}

fn format_user_menu_event(event: &PopupMenuEvent) -> String {
    match event {
        PopupMenuEvent::Select { item_id, label } => {
            format!("PopupMenuEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        PopupMenuEvent::OpenChanged { open } => format!("PopupMenuEvent::OpenChanged {{ open: {open} }}"),
        PopupMenuEvent::Dismiss => "PopupMenuEvent::Dismiss".to_string(),
        PopupMenuEvent::FocusChanged { focused } => {
            format!("PopupMenuEvent::FocusChanged {{ focused: {focused} }}")
        }
        PopupMenuEvent::HoverChanged { hovered } => {
            format!("PopupMenuEvent::HoverChanged {{ hovered: {hovered} }}")
        }
        PopupMenuEvent::EnabledChanged { enabled } => {
            format!("PopupMenuEvent::EnabledChanged {{ enabled: {enabled} }}")
        }
        _ => "PopupMenuEvent::(unknown)".to_string(),
    }
}

fn property_leaf_menu_item(
    _look: &Arc<ShadcnLook>,
    leaf: &PropertyLeaf,
) -> xgpui_luma::controls::sidebar::SidebarMenuItemBuilder {
    let mut item = shadcn::Sidebar::menu_item(leaf.id, leaf.label)
        .disabled(!leaf.enabled)
        .active(leaf.id == INITIAL_PROPERTY_SELECTION_ID);
    if let Some(icon) = leaf.icon {
        item = item.icon(icon);
    }
    item
}

fn format_sidebar_control_event(event: &SidebarEvent) -> Option<String> {
    match event {
        SidebarEvent::DragDrop(event) => Some(format!("SidebarEvent::DragDrop {event:?}")),
        SidebarEvent::PresentationChanged { presentation } => {
            Some(format!("SidebarEvent::PresentationChanged {{ presentation: {presentation:?} }}"))
        }
        SidebarEvent::Select { id } => Some(format!("SidebarEvent::Select {{ id: \"{id}\" }}")),
        SidebarEvent::ItemFocused { id } => Some(format!("SidebarEvent::ItemFocused {{ id: \"{id}\" }}")),
        SidebarEvent::HoverChanged { id } => Some(match id {
            Some(id) => format!("SidebarEvent::HoverChanged {{ id: Some(\"{id}\") }}"),
            None => "SidebarEvent::HoverChanged { id: None }".to_string(),
        }),
        SidebarEvent::SubMenuToggle { id, open } => {
            Some(format!("SidebarEvent::SubMenuToggle {{ id: \"{id}\", open: {open} }}"))
        }
        SidebarEvent::EnabledChanged { enabled } => {
            Some(format!("SidebarEvent::EnabledChanged {{ enabled: {enabled} }}"))
        }
        _ => None,
    }
}
