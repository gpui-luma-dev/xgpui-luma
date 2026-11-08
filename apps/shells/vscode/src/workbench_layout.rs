//! VS Code workbench layout regions (Customize Layout dialog):
//! - Activity Bar
//! - Secondary Activity Bar
//! - Primary Side Bar
//! - Secondary Side Bar
//! - Status Bar

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Pixels, SharedString, div, px, prelude::*};
use xgpui_luma::controls::resizable_panels::{
    PanelId, ResizablePanelSpec, ResizablePanels, ResizeHandleSize, ResizeHandleVisibility,
};
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextRole};

use crate::layout_config::{PanelAlignment, PrimarySideBarPosition};

pub const WORKBENCH_PANEL_H: f32 = 180.0;
// These opaque IDs belong to this demo; the SDK only knows them as stable panel identities.
pub const PRIMARY_SIDE_BAR_REGION: &str = "vscode.primary-side-bar";
pub const EDITOR_REGION: &str = "vscode.editor";
pub const SECONDARY_SIDE_BAR_REGION: &str = "vscode.secondary-side-bar";

/// Resizable panel index for the physical left edge slot.
pub const LEFT_EDGE_PANEL_INDEX: usize = 0;

/// Resizable panel index for the physical right edge slot.
pub const RIGHT_EDGE_PANEL_INDEX: usize = 2;

#[derive(Clone)]
pub struct SideBarSpec {
    render: Rc<dyn Fn() -> AnyElement>,
    width_px: f32,
    min_px: f32,
    max_px: f32,
}

impl SideBarSpec {
    pub fn new(id: &'static str, look: Arc<ShadcnLook>) -> Self {
        Self { render: Rc::new(move || side_bar_region(id, &look)), width_px: 360.0, min_px: 240.0, max_px: 460.0 }
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width_px = width.as_f32();
        self
    }

    pub fn min(mut self, min: Pixels) -> Self {
        self.min_px = min.as_f32();
        self
    }

    pub fn max(mut self, max: Pixels) -> Self {
        self.max_px = max.as_f32();
        self
    }
}

pub type PrimarySideBar = SideBarSpec;
pub type SecondarySideBar = SideBarSpec;

pub struct WorkbenchLayout {
    panels: Entity<ResizablePanels>,
    primary_side_bar_position: Rc<Cell<PrimarySideBarPosition>>,
    panel_visible: Rc<Cell<bool>>,
    panel_alignment: Rc<Cell<PanelAlignment>>,
    panel_height_px: Rc<Cell<f32>>,
    panel_splitter: Rc<RefCell<Rc<dyn Fn() -> AnyElement>>>,
}

impl WorkbenchLayout {
    pub fn new<T: 'static>(
        id: impl Into<SharedString>,
        look: Arc<ShadcnLook>,
        primary_side_bar: PrimarySideBar,
        secondary_side_bar: SecondarySideBar,
        cx: &mut Context<T>,
    ) -> Self {
        let id = id.into();
        let look_for_panels = look.clone();
        let primary_side_bar_position = Rc::new(Cell::new(PrimarySideBarPosition::Left));
        let panel_visible = Rc::new(Cell::new(true));
        let panel_alignment = Rc::new(Cell::new(PanelAlignment::Center));
        let panel_height_px = Rc::new(Cell::new(WORKBENCH_PANEL_H));
        let editor_panel_visible = panel_visible.clone();
        let editor_panel_alignment = panel_alignment.clone();
        let editor_panel_height_px = panel_height_px.clone();
        let panel_splitter: Rc<RefCell<Rc<dyn Fn() -> AnyElement>>> =
            Rc::new(RefCell::new(Rc::new(|| gpui::Empty.into_any_element())));
        let editor_panel_splitter = panel_splitter.clone();

        // The workbench has three physical slots. Logical IDs let the shell swap
        // sidebar ownership without rebuilding the panel tree or losing sizes.
        let panels = look
            .resizable_panels(format!("{id}-panels"))
            .handle_visibility(ResizeHandleVisibility::Hover)
            .resize_handle(ResizeHandleSize::Sm)
            .handle_grip(true)
            .show_border(false)
            .region(
                PanelId::new(PRIMARY_SIDE_BAR_REGION),
                side_bar_panel(edge_side_bar(
                    primary_side_bar_position.clone(),
                    PrimarySideBarPosition::Left,
                    primary_side_bar.clone(),
                    secondary_side_bar.clone(),
                )),
            )
            .region(
                PanelId::new(EDITOR_REGION),
                ResizablePanelSpec::new_render(move || {
                    render_region_slot(editor_region(
                        &look_for_panels,
                        editor_panel_visible.clone(),
                        editor_panel_alignment.clone(),
                        editor_panel_height_px.clone(),
                        editor_panel_splitter.clone(),
                    ))
                })
                .weight(1.0)
                .min(px(320.0)),
            )
            .region(
                PanelId::new(SECONDARY_SIDE_BAR_REGION),
                side_bar_panel(edge_side_bar(
                    primary_side_bar_position.clone(),
                    PrimarySideBarPosition::Right,
                    primary_side_bar,
                    secondary_side_bar,
                )),
            )
            .spawn(cx);

        Self { panels, primary_side_bar_position, panel_visible, panel_alignment, panel_height_px, panel_splitter }
    }

    pub fn panels(&self) -> Entity<ResizablePanels> {
        self.panels.clone()
    }

    pub fn sync_chrome<T: 'static>(&self, look: &ShadcnLook, cx: &mut Context<T>) {
        let chrome = look.chrome();
        self.panels.update(cx, |panels, cx| {
            panels.set_panel_background(0, Some(chrome.panel_background), cx);
            panels.set_panel_background(1, Some(chrome.content_background), cx);
            panels.set_panel_background(2, Some(chrome.panel_background), cx);
        });
    }

    pub fn sync_theme<T>(
        &self,
        theme: &Arc<dyn xgpui_luma::controls::resizable_panels::ResizablePanelsTheme>,
        cx: &mut Context<T>,
    ) {
        self.panels.update(cx, |panels, cx| panels.set_theme(theme.clone(), cx));
    }

    pub fn set_frame_size<T: 'static>(&self, width: Pixels, height: Pixels, cx: &mut Context<T>) {
        self.panels.update(cx, |panels, cx| panels.set_frame_size(width, height, cx));
    }

    pub fn panel_sizes_px<T: 'static>(&self, cx: &mut Context<T>) -> Vec<f32> {
        self.panels.read(cx).panel_sizes_px()
    }

    pub fn set_primary_side_bar_position<T: 'static>(&self, position: PrimarySideBarPosition, cx: &mut Context<T>) {
        if self.primary_side_bar_position.get() == position {
            return;
        }
        self.primary_side_bar_position.set(position);
        self.panels.update(cx, |panels, cx| {
            // Visibility is applied by VscodeShellApp after this identity swap.
            panels.swap_regions(&PanelId::new(PRIMARY_SIDE_BAR_REGION), &PanelId::new(SECONDARY_SIDE_BAR_REGION), cx);
        });
    }

    pub fn set_panel_layout<T: 'static>(&self, visible: bool, alignment: PanelAlignment, cx: &mut Context<T>) {
        if self.panel_visible.get() == visible && self.panel_alignment.get() == alignment {
            return;
        }
        self.panel_visible.set(visible);
        self.panel_alignment.set(alignment);
        self.panels.update(cx, |_, cx| cx.notify());
    }

    pub fn set_panel_height<T: 'static>(&self, height: Pixels, cx: &mut Context<T>) {
        let height = height.as_f32();
        if (self.panel_height_px.get() - height).abs() < f32::EPSILON {
            return;
        }
        self.panel_height_px.set(height);
        self.panels.update(cx, |_, cx| cx.notify());
    }

    pub fn set_panel_splitter<T: 'static>(&self, panel_splitter: Rc<dyn Fn() -> AnyElement>, cx: &mut Context<T>) {
        *self.panel_splitter.borrow_mut() = panel_splitter;
        self.panels.update(cx, |_, cx| cx.notify());
    }
}

fn side_bar_panel(side_bar: SideBarSpec) -> ResizablePanelSpec {
    ResizablePanelSpec::new_render(move || render_region_slot((side_bar.render)()))
        .size(px(side_bar.width_px))
        .min(px(side_bar.min_px))
        .max(px(side_bar.max_px))
}

fn edge_side_bar(
    primary_side_bar_position: Rc<Cell<PrimarySideBarPosition>>,
    edge_position: PrimarySideBarPosition,
    primary_side_bar: PrimarySideBar,
    secondary_side_bar: SecondarySideBar,
) -> SideBarSpec {
    let width_px = primary_side_bar.width_px;
    let min_px = primary_side_bar.min_px;
    let max_px = primary_side_bar.max_px;

    SideBarSpec {
        render: Rc::new(move || {
            if primary_side_bar_position.get() == edge_position {
                (primary_side_bar.render)()
            } else {
                (secondary_side_bar.render)()
            }
        }),
        width_px,
        min_px,
        max_px,
    }
}

fn side_bar_region(id: &'static str, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    let label_style = look.typography_role(ShadcnTextRole::P);
    let label = match id {
        "primary-side-bar" => "Primary Side Bar",
        "secondary-side-bar" => "Secondary Side Bar",
        _ => "",
    };

    div()
        .id(id)
        .size_full()
        .bg(chrome.panel_background)
        .p(px(12.0))
        .child(div().typography_style(label_style).text_color(chrome.muted_text).child(label))
        .into_any_element()
}

fn editor_region(
    look: &ShadcnLook,
    panel_visible: Rc<Cell<bool>>,
    panel_alignment: Rc<Cell<PanelAlignment>>,
    panel_height_px: Rc<Cell<f32>>,
    panel_splitter: Rc<RefCell<Rc<dyn Fn() -> AnyElement>>>,
) -> AnyElement {
    let chrome = look.chrome();
    let editor = div().id("editor").flex_1().min_h_0().w_full().bg(chrome.content_background);

    if panel_visible.get() && panel_alignment.get() == PanelAlignment::Center {
        return div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .child(editor)
            .child(panel_splitter.borrow()())
            .child(output_panel(look, panel_height_px.get()))
            .into_any_element();
    }

    editor.h_full().into_any_element()
}

fn render_region_slot(content: impl IntoElement) -> AnyElement {
    div().size_full().min_h_0().flex().flex_col().overflow_hidden().child(content).into_any_element()
}

fn output_panel(look: &ShadcnLook, height_px: f32) -> impl IntoElement {
    let chrome = look.chrome();
    let label_style = look.typography_role(ShadcnTextRole::P);

    div()
        .id("panel")
        .h(px(height_px))
        .w_full()
        .flex_shrink_0()
        .bg(chrome.panel_background)
        .border_t_1()
        .border_color(chrome.border)
        .p(px(12.0))
        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("Panel"))
}
