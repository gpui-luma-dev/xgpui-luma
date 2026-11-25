//! The SDK overlay window on the design surface page: one Dialog page (issues #383, #393). Its `.lss`
//! (`overlay_window`) has no states and one variant, `modal`, which dims the page behind the window and
//! casts the dialog shadow. Where a dialog opens and whether it drags are behavior, not look, so the page
//! draws the window open, inline in a frame that stands for the page behind it, and offers Modeless ·
//! Modal: Modal fills the frame with the backdrop. The matrix draws the same frame.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::control_group::{ControlGroup, ControlGroupEvent, ControlGroupItem};
use xgpui_luma::controls::overlay_window::{
    OverlayWindowElementRenderer, OverlayWindowMode, OverlayWindowPosition, OverlayWindowRenderModel,
    OverlayWindowTemplate, OverlayWindowTemplateHandlers, OverlayWindowTemplateParts, StyledOverlayWindowTheme,
    ThemedOverlayWindowTemplate,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The window's heading and body, with ascenders and descenders.
const HEADING: &str = "Shipping preferences";
const BODY: &str = "Pick a delivery day, and we'll keep you posted by email.";

/// The frame standing for the page behind the window: room around a window at its widest.
const FRAME_WIDTH: f32 = 600.;
const FRAME_HEIGHT: f32 = 280.;

/// The variant option's items.
const MODELESS: &str = "modeless";
const MODAL: &str = "modal";

/// The overlay window template for the `overlay_window` control in `theme`.
fn template(theme: Theme) -> ThemedOverlayWindowTemplate {
    let binding = StyleBinding::new("overlay_window", ThemeSource::Fixed(theme));
    ThemedOverlayWindowTemplate::new(Arc::new(StyledOverlayWindowTheme::new(binding)))
}

/// Handlers that do nothing: the window is drawn, not opened.
fn inert_handlers() -> OverlayWindowTemplateHandlers {
    OverlayWindowTemplateHandlers {
        key_down: Box::new(|_, _, _| {}),
        header_mouse_down: Box::new(|_, _, _| {}),
        shell_mouse_down: Box::new(|_, _, _| {}),
        shell_mouse_down_out: Box::new(|_, _, _| {}),
    }
}

/// The window open in its frame, drawn from `theme` in `mode`: Modal fills the frame with the backdrop.
fn dialog(id: SharedString, theme: &Theme, mode: OverlayWindowMode, window: &mut Window, cx: &mut App) -> AnyElement {
    let template = template(theme.clone());
    let look = template.resolve_look(mode);
    let content: OverlayWindowElementRenderer = Arc::new(|_, _, _| {
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(div().font_weight(gpui::FontWeight::SEMIBOLD).child(HEADING))
            .child(BODY)
            .into_any_element()
    });
    let model = OverlayWindowRenderModel {
        id: &id,
        mode,
        position: OverlayWindowPosition::Center,
        dismissible: true,
        draggable: false,
        width: None,
        focused: false,
        content: &content,
    };
    let parts = OverlayWindowTemplateParts { header_drag_handle: None, handlers: inert_handlers() };
    let backdrop = match mode {
        OverlayWindowMode::Modal => look.backdrop_background,
        OverlayWindowMode::Modeless => gpui::transparent_black(),
    };
    div()
        .w(px(FRAME_WIDTH))
        .h(px(FRAME_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .bg(backdrop)
        .child(template.render(&model, &look, parts, window, cx))
        .into_any_element()
}

/// The live dialog: the window open in its frame, in the theme last bound and the picked variant.
struct DialogHost {
    theme: Option<Theme>,
    mode: OverlayWindowMode,
}

impl Render for DialogHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        div()
            .debug_selector(|| "workshop-sdk-dialog".into())
            .children(theme.map(|theme| dialog("workshop-sdk-dialog".into(), &theme, self.mode, window, cx)))
    }
}

pub(super) struct DialogSurface {
    host: Entity<DialogHost>,
    /// Radix toggle group: Modeless · Modal (the `.lss` variant).
    variant_picker: ControlGroup<ControlGroupItem>,
}

impl DialogSurface {
    pub(super) fn make(look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let host = cx.new(|_| DialogHost { theme: None, mode: OverlayWindowMode::Modeless });
        let variant_picker = xgpui_luma_look_radix::control_group_builder(
            look.as_ref(),
            "workshop-surface-dialog-variant",
            xgpui_luma_look_radix::RadixSize::One,
        )
        .horizontal()
        .single_required()
        .items([ControlGroupItem::new(MODELESS).label("Modeless"), ControlGroupItem::new(MODAL).label("Modal")])
        .selected_ids([MODELESS])
        .spawn(cx);
        let target = host.clone();
        cx.subscribe(&variant_picker, move |_: &mut ControlSide, _, event: &ControlGroupEvent, cx| {
            if let ControlGroupEvent::Change { selected_ids, .. } = event
                && let Some(id) = selected_ids.first()
            {
                let mode = if id.as_ref() == MODAL {
                    OverlayWindowMode::Modal
                } else {
                    OverlayWindowMode::Modeless
                };
                target.update(cx, |host, cx| {
                    host.mode = mode;
                    cx.notify();
                });
                // The matrix draws the picked variant too.
                cx.notify();
            }
        })
        .detach();
        Box::new(Self { host, variant_picker })
    }

    /// The picked variant, for tests.
    #[cfg(all(test, feature = "test-support"))]
    fn mode(&self, cx: &App) -> OverlayWindowMode {
        self.host.read(cx).mode
    }

    /// The variant option, for tests.
    #[cfg(all(test, feature = "test-support"))]
    fn variant_picker(&self) -> ControlGroup<ControlGroupItem> {
        self.variant_picker.clone()
    }
}

impl SurfaceControl for DialogSurface {
    fn title(&self) -> &'static str {
        "Dialog"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    fn bind(&self, theme: Theme, _pinned: Option<&StateSet>, cx: &mut App) {
        self.host.update(cx, |host, cx| {
            host.theme = Some(theme);
            cx.notify();
        });
    }

    /// The window is drawn, not opened, so there is no input to turn off.
    fn set_enabled(&self, _enabled: bool, _cx: &mut App) {}

    fn template_states(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// A drawn window emits no events.
    fn subscribe(&self, cx: &mut Context<ControlSide>, _on_event: OnSurfaceEvent) -> Subscription {
        cx.observe(&self.host, |_, _, cx| cx.notify())
    }

    fn cell(
        &self,
        theme: &Theme,
        id: SharedString,
        _states: &StateSet,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let mode = self.host.read(cx).mode;
        dialog(id, theme, mode, window, cx)
    }

    fn options(&self) -> Option<AnyElement> {
        Some(self.variant_picker.clone().into_any_element())
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::TestAppContext;

    use super::*;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_dialog_page_draws_the_open_window_and_switches_to_modal() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "dialog");
        let side = cx.update(|_, cx| page.read(cx).side());
        assert!(cx.debug_bounds("workshop-sdk-dialog").is_some(), "the window is drawn open");
        let picker = cx.update(|_, cx| side.read(cx).control_as::<DialogSurface>().expect("a dialog").variant_picker());
        let mode = |cx: &mut gpui::VisualTestContext| {
            cx.update(|_, cx| side.read(cx).control_as::<DialogSurface>().expect("a dialog").mode(cx))
        };
        assert!(matches!(mode(cx), OverlayWindowMode::Modeless), "Modeless first");

        picker.update(cx, |_, cx| {
            cx.emit(ControlGroupEvent::Change {
                changed_id: MODAL.into(),
                selected: true,
                selected_ids: vec![MODAL.into()],
            })
        });
        cx.run_until_parked();
        assert!(matches!(mode(cx), OverlayWindowMode::Modal), "the option picks the modal variant");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "the matrix draws the window");
    }
}
