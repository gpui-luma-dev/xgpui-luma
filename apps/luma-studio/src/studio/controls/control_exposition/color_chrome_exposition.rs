use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement, SharedString};
use xgpui_luma_look_shadcn::ShadcnLook;

use super::color_chrome_inspector::ColorChromeInspector;
use super::inspector::color_chrome::ColorChromeProfile;
use super::inspector_split::InspectorSplitShell;

pub struct ColorChromeViewportPane {
    pub chrome_inspector: Entity<ColorChromeInspector>,
    pub inspector_split: Entity<InspectorSplitShell>,
}

pub fn spawn_color_chrome_viewport<T: 'static>(
    cx: &mut Context<T>,
    look: Arc<ShadcnLook>,
    split_id: impl Into<SharedString>,
    inspector_id: impl Into<SharedString>,
    profiles: &'static [ColorChromeProfile],
    left: impl Fn() -> AnyElement + 'static + Clone,
) -> ColorChromeViewportPane {
    let chrome_inspector = cx.new(|cx| ColorChromeInspector::new(look.clone(), inspector_id, profiles, cx));
    let inspector_split = cx.new(|cx| {
        let left = left.clone();
        let chrome_inspector = chrome_inspector.clone();
        InspectorSplitShell::new(cx, look, split_id, left, move || chrome_inspector.clone().into_any_element())
    });
    ColorChromeViewportPane { chrome_inspector, inspector_split }
}

pub fn sync_color_chrome_viewport(
    look: Arc<ShadcnLook>,
    chrome_inspector: &Entity<ColorChromeInspector>,
    inspector_split: &Entity<InspectorSplitShell>,
    cx: &mut App,
) {
    chrome_inspector.update(cx, |inspector, cx| inspector.sync_look(look.clone(), cx));
    inspector_split.update(cx, |split, cx| split.sync_look(look, cx));
}

pub fn color_chrome_set_viewport_size(
    inspector_split: &Entity<InspectorSplitShell>,
    size: gpui::Size<gpui::Pixels>,
    cx: &mut App,
) {
    inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
}
