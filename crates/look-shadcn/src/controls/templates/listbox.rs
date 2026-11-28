//! Shadcn chrome for the shared SDK listbox builder.
use std::sync::Arc;
use xgpui_luma::style::PartRead;
use gpui::{App, Div, Stateful, prelude::*, px};
use xgpui_luma::controls::frame::FrameBuilder;
use xgpui_luma::controls::listbox::{
    ListBoxTemplate, ListBoxRenderModel, ListBoxRowRenderModel, ListBoxDragLook, ListBoxMotion,
};
use crate::ShadcnLook;
struct ShadcnListBoxTemplate {
    look: ShadcnLook,
}
impl ListBoxTemplate for ShadcnListBoxTemplate {
    fn render(&self, model: &ListBoxRenderModel, cx: &mut App) -> Stateful<Div> {
        let surface = self.look.styled_listbox().resolve_surface(true, model.focused);
        let surface = surface.root();
        FrameBuilder::new(format!("{}-surface", model.id))
            // The viewport plus inset determines height, not Frame's full-size default.
            .h_auto()
            // Keep flex parents from shrinking the surface below its fixed viewport.
            .min_h(gpui::Length::Auto)
            .bg(surface.color("background"))
            .border(px(surface.number("border_width")))
            .border_color(surface.color("border"))
            .rounded(px(surface.number("radius")))
            .px(px(model.layout.inset_x))
            .py(px(model.layout.inset_y))
            .render(cx)
            .debug_selector(|| format!("{}-surface", model.id))
            .relative()
            .aria_label(model.label)
    }
    fn render_row(&self, model: &ListBoxRowRenderModel, _: &mut App) -> Stateful<Div> {
        self.look.listbox_row(model.id.clone(), model.state, model.focus_visible)
    }
    fn drag_look(&self) -> ListBoxDragLook {
        let color = self.look.token_color("ring").unwrap_or(self.look.chrome().border);
        ListBoxDragLook {
            marker_color: color,
            marker_width: 2.0,
            surface_color: Some(color),
            preview: crate::tooltip_theme(&self.look).resolve(),
        }
    }
    fn motion_defaults(&self) -> ListBoxMotion {
        self.look.motion_defaults().listbox
    }
}

/// Live Shadcn chrome for the SDK listbox builder.
pub fn listbox_template(look: ShadcnLook) -> Arc<dyn ListBoxTemplate> {
    Arc::new(ShadcnListBoxTemplate { look })
}
