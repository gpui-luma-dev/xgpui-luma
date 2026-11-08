use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement, SharedString};
use xgpui_luma_look_shadcn::ShadcnLook;

use super::inspector::{ControlInspectorSpec, SharedInspectorResolver};
use super::inspector_split::InspectorSplitShell;
use super::theme_inspector::ThemeInspector;

pub struct ViewportInspectorPane {
    pub theme_inspector: Entity<ThemeInspector>,
    pub inspector_split: Entity<InspectorSplitShell>,
}

pub fn spawn_viewport_inspector<T: 'static>(
    cx: &mut Context<T>,
    look: Arc<ShadcnLook>,
    split_id: impl Into<SharedString>,
    left: impl Fn() -> AnyElement + 'static + Clone,
    spec: &'static ControlInspectorSpec,
    resolver: SharedInspectorResolver,
) -> ViewportInspectorPane {
    let theme_inspector =
        cx.new(|cx| ThemeInspector::for_embedded_pane_with_spec(look.clone(), spec, resolver.clone(), cx));
    let inspector_split = cx.new(|cx| {
        let left = left.clone();
        let theme_inspector = theme_inspector.clone();
        InspectorSplitShell::new(cx, look, split_id, left, move || theme_inspector.clone().into_any_element())
    });
    ViewportInspectorPane { theme_inspector, inspector_split }
}

pub fn sync_viewport_inspector(
    look: Arc<ShadcnLook>,
    theme_inspector: &Entity<ThemeInspector>,
    inspector_split: &Entity<InspectorSplitShell>,
    cx: &mut App,
) {
    theme_inspector.update(cx, |inspector, cx| inspector.sync_look(look.clone(), cx));
    inspector_split.update(cx, |split, cx| split.sync_look(look, cx));
}
