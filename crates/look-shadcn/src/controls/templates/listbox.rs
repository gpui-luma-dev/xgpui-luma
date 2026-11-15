//! Shadcn chrome for the shared SDK listbox builder.
use std::sync::Arc;
use gpui::{App, Div, Stateful, prelude::*, px};
use xgpui_luma::controls::frame::FrameBuilder;
use xgpui_luma::controls::listbox::{
    ListBoxTemplate, ListBoxRenderModel, ListBoxRowRenderModel, ListBoxDragLook, ListBoxMotion,
};
use crate::ShadcnLook;
const CORNER_RADIUS: f32 = 8.0;
struct ShadcnListBoxTemplate {
    look: ShadcnLook,
}
impl ListBoxTemplate for ShadcnListBoxTemplate {
    fn render(&self, model: &ListBoxRenderModel, cx: &mut App) -> Stateful<Div> {
        let mode = self.look.mode_tokens();
        let palette = crate::controls::listbox::listbox_surface_palette(&mode, true);
        let border = if model.focused {
            crate::focus::focus_ring_or_fallback(&mode.catalog)
        } else {
            palette.border
        };
        FrameBuilder::new(format!("{}-surface", model.id))
            // The viewport plus inset determines height, not Frame's full-size default.
            .h_auto()
            // Keep flex parents from shrinking the surface below its fixed viewport.
            .min_h(gpui::Length::Auto)
            .bg(palette.background)
            .border_1()
            .border_color(border)
            .rounded(px(CORNER_RADIUS))
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
