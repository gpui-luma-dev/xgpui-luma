use std::time::Duration;
use gpui::{AnyElement, App, Bounds, Context, Pixels, Window, div, point, px, size, prelude::*};
use xgpui_luma::{
    infra::attachments::{AttachmentHost, AttachmentTarget, TooltipEntityExt},
    theme::InteractionState,
};
use super::super::points::motion::AnchorMotion;
struct TooltipOwner {
    attachments: AttachmentHost,
    anchor: Bounds<Pixels>,
}
impl AttachmentTarget for TooltipOwner {
    fn attachments(&self) -> &AttachmentHost {
        &self.attachments
    }
    fn attachments_mut(&mut self) -> &mut AttachmentHost {
        &mut self.attachments
    }
    fn attachment_anchor(&self, _: Bounds<Pixels>) -> Bounds<Pixels> {
        self.anchor
    }
}
#[derive(Default)]
struct TooltipState {
    current: Option<crate::RadialBar>,
    owner: Option<gpui::Entity<TooltipOwner>>,
    motion: AnchorMotion,
}
pub(super) fn tooltip(
    slice: Option<&crate::RadialBar>,
    position: gpui::Point<Pixels>,
    bounds: Bounds<Pixels>,
    factory: &std::sync::Arc<crate::views::radial::RadialTooltip>,
    duration: Duration,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    let state = window.use_state(cx, |_, _| TooltipState::default());
    let target = state.update(cx, |state, cx: &mut Context<TooltipState>| {
        let Some(slice) = slice else {
            state.current = None;
            state.owner = None;
            state.motion.clear();
            return None;
        };
        if state.current.as_ref() != Some(slice) {
            let config = factory(slice);
            let owner = state.owner.clone().unwrap_or_else(|| {
                cx.new(|_| TooltipOwner { attachments: AttachmentHost::default(), anchor: Bounds::default() })
            });
            state.owner = Some(owner.tooltip(config, cx));
            state.current = Some(slice.clone());
        }
        let anchor = state.motion.update(position, bounds, duration);
        state.motion.schedule_frame(window, cx);
        Some((anchor, state.owner.clone()?))
    });
    let (anchor, owner) = target?;
    let mut element = owner.update(cx, |owner, cx| {
        owner.anchor = Bounds::new(anchor - point(px(4.0), px(4.0)), size(px(8.0), px(8.0)));
        owner
            .attachments
            .render(
                div().id("radial-tooltip").w(bounds.size.width).h(bounds.size.height),
                InteractionState { hovered: true, ..InteractionState::default() },
                cx,
            )
            .into_any_element()
    });
    element.prepaint_as_root(bounds.origin, bounds.size.map(gpui::AvailableSpace::Definite), window, cx);
    Some(element)
}
