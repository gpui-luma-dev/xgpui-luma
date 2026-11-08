use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, FocusHandle, Hsla, IntoElement, Pixels, SharedString, div, px, prelude::*};
use xgpui_luma::controls::resizable_panels::{
    ResizablePanelSpec, ResizablePanels, ResizeHandleSize, ResizeHandleVisibility,
};
use xgpui_luma::focus::LumaFocusScopeExt;
use xgpui_luma_look_shadcn::ShadcnLook;

pub const LEFT_SIDEBAR_PANEL_INDEX: usize = 0;
pub const CONTENT_PANEL_INDEX: usize = 1;

#[derive(Clone)]
pub struct WorkbenchSidebar {
    render: Rc<dyn Fn() -> AnyElement>,
    width_px: f32,
    min_px: f32,
    max_px: f32,
}

impl WorkbenchSidebar {
    pub fn new(render: impl Fn() -> AnyElement + 'static) -> Self {
        Self { render: Rc::new(render), width_px: 360.0, min_px: 360.0, max_px: 460.0 }
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

pub struct WorkbenchLayout {
    panels: Entity<ResizablePanels>,
}

impl WorkbenchLayout {
    pub fn new<T: 'static>(
        id: impl Into<SharedString>,
        look: Arc<ShadcnLook>,
        left_sidebar: WorkbenchSidebar,
        content: impl Fn() -> AnyElement + 'static,
        cx: &mut Context<T>,
    ) -> Self {
        let id = id.into();

        let panels = look
            .resizable_panels(format!("{id}-panels"))
            .motion_enabled(true)
            .handle_visibility(ResizeHandleVisibility::Hover)
            .resize_handle(ResizeHandleSize::Sm)
            .handle_grip(true)
            .show_border(false)
            .panel(sidebar_panel(left_sidebar))
            .panel(ResizablePanelSpec::new_render(move || render_panel_slot(content())).weight(1.0).min(px(320.0)))
            .spawn(cx);

        Self { panels }
    }

    pub fn panels(&self) -> Entity<ResizablePanels> {
        self.panels.clone()
    }

    pub fn sync_theme<T>(
        &self,
        theme: &Arc<dyn xgpui_luma::controls::resizable_panels::ResizablePanelsTheme>,
        cx: &mut Context<T>,
    ) {
        self.panels.update(cx, |panels, cx| panels.set_theme(theme.clone(), cx));
    }

    pub fn render_body(&self) -> AnyElement {
        self.panels.clone().into_any_element()
    }

    pub fn render_shell(
        focus_scope: &FocusHandle,
        font_family: SharedString,
        app_background: Hsla,
        title_bar: impl IntoElement,
        body: impl IntoElement,
        bottom_bar: impl IntoElement,
    ) -> AnyElement {
        div()
            .luma_focus_scope(focus_scope)
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .font_family(font_family)
            .bg(app_background)
            .child(title_bar)
            .child(div().id("workbench-layout-body").flex_1().min_h_0().w_full().overflow_hidden().child(body))
            .child(bottom_bar)
            .into_any_element()
    }
}

fn sidebar_panel(sidebar: WorkbenchSidebar) -> ResizablePanelSpec {
    ResizablePanelSpec::new_render(move || render_panel_slot((sidebar.render)()))
        .size(px(sidebar.width_px))
        .min(px(sidebar.min_px))
        .max(px(sidebar.max_px))
}

fn render_panel_slot(content: impl IntoElement) -> AnyElement {
    div().size_full().min_h_0().flex().flex_col().overflow_hidden().child(content).into_any_element()
}
