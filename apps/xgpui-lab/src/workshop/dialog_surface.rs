//! The SDK overlay window on the design surface page (issue #383): the four Dialog pages (Modal, Modeless,
//! Positioning, Draggable). Its `.lss` (`overlay_window`) has no states; `variant modal` dims the page and
//! casts the dialog shadow. The live control is an Open button and the window it opens; the matrix draws the
//! window's shell in place.

use std::sync::{Arc, Mutex};

use gpui::{
    AnyElement, AnyView, App, Context, Div, Entity, SharedString, Stateful, Subscription, Window, div, prelude::*, px,
};
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::overlay_window::{
    OverlayWindowBuilder, OverlayWindowControl, OverlayWindowElementRenderer, OverlayWindowEvent, OverlayWindowLook,
    OverlayWindowMode, OverlayWindowPosition, OverlayWindowRenderModel, OverlayWindowTemplate,
    OverlayWindowTemplateHandlers, OverlayWindowTemplateParts, StyledOverlayWindowTheme, ThemedOverlayWindowTemplate,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The window's heading and body, with ascenders and descenders.
const HEADING: &str = "Shipping preferences";
const BODY: &str = "Pick a delivery day, and we'll keep you posted by email.";

#[derive(Clone, Copy, PartialEq)]
enum DialogKind {
    Modal,
    Modeless,
    Positioning,
    Draggable,
}

impl DialogKind {
    fn title(self) -> &'static str {
        match self {
            Self::Modal => "Dialog Modal",
            Self::Modeless => "Dialog Modeless",
            Self::Positioning => "Dialog Positioning",
            Self::Draggable => "Dialog Draggable",
        }
    }

    fn mode(self) -> OverlayWindowMode {
        match self {
            Self::Modal | Self::Draggable => OverlayWindowMode::Modal,
            Self::Modeless | Self::Positioning => OverlayWindowMode::Modeless,
        }
    }
}

/// The window's template, swapped when the page binds a theme (the window takes its template once).
#[derive(Clone)]
struct SwappableTemplate(Arc<Mutex<Arc<ThemedOverlayWindowTemplate>>>);

impl SwappableTemplate {
    fn current(&self) -> Arc<ThemedOverlayWindowTemplate> {
        self.0
            .lock()
            .map(|template| template.clone())
            .unwrap_or_else(|poisoned| poisoned.into_inner().clone())
    }

    fn set(&self, template: ThemedOverlayWindowTemplate) {
        let mut current = self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = Arc::new(template);
    }
}

impl OverlayWindowTemplate for SwappableTemplate {
    fn resolve_look(&self, mode: OverlayWindowMode) -> OverlayWindowLook {
        self.current().resolve_look(mode)
    }

    fn render(
        &self,
        model: &OverlayWindowRenderModel<'_>,
        look: &OverlayWindowLook,
        parts: OverlayWindowTemplateParts,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        self.current().render(model, look, parts, window, cx)
    }
}

/// The overlay window template for the `overlay_window` control in `theme`.
fn template(theme: Theme) -> ThemedOverlayWindowTemplate {
    let binding = StyleBinding::new("overlay_window", ThemeSource::Fixed(theme));
    ThemedOverlayWindowTemplate::new(Arc::new(StyledOverlayWindowTheme::new(binding)))
}

/// The window's content: heading, body and a Close button (none in a matrix cell).
fn content(close: Option<Entity<Button>>) -> OverlayWindowElementRenderer {
    Arc::new(move |_, _, _| {
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(div().font_weight(gpui::FontWeight::SEMIBOLD).child(HEADING))
            .child(BODY)
            .when_some(close.clone(), |content, close| content.child(div().flex().justify_end().child(close)))
            .into_any_element()
    })
}

/// The Open button and the window it opens.
struct DialogHost {
    window: Entity<OverlayWindowControl>,
    open: Entity<Button>,
}

impl DialogHost {
    fn new(kind: DialogKind, look: &Arc<Look>, template: SwappableTemplate, cx: &mut Context<Self>) -> Self {
        let close = xgpui_luma_look_radix::Button::new("workshop-sdk-dialog-close")
            .look(look.as_ref())
            .soft()
            .size(xgpui_luma_look_radix::ButtonSize::One)
            .content(|_, _| div().child("Close"))
            .on_click(cx, |this: &mut Self, cx| this.window.update(cx, |window, cx| window.dismiss(cx)))
            .spawn(cx);
        let mut builder = OverlayWindowBuilder::new("workshop-sdk-dialog")
            .mode(kind.mode())
            .template(Arc::new(template))
            .with_template_modifier(|root, _| root.debug_selector(|| "workshop-sdk-dialog-bounds".into()));
        match kind {
            DialogKind::Positioning => builder = builder.position(OverlayWindowPosition::BottomRight),
            // Centered dragging estimates the height; an explicit origin avoids a jump when a drag starts.
            DialogKind::Draggable => builder = builder.draggable(true).absolute_position(100., 100.),
            DialogKind::Modal | DialogKind::Modeless => {}
        }
        let content = content(Some(close));
        let window = builder.content(move |model, window, cx| content(model, window, cx)).spawn(cx);
        let open = xgpui_luma_look_radix::Button::new("workshop-sdk-dialog-open")
            .look(look.as_ref())
            .size(xgpui_luma_look_radix::ButtonSize::Two)
            .content(|_, _| div().child("Open shipping dialog"))
            .on_click(cx, |this: &mut Self, cx| {
                use gpui::Focusable;
                let focus = this.open.read(cx).focus_handle(cx);
                this.window.update(cx, |window, cx| window.open_from(Some(focus), cx));
            })
            .spawn(cx);
        Self { window, open }
    }
}

impl Render for DialogHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.open.clone()).child(self.window.clone())
    }
}

pub(super) struct DialogSurface {
    kind: DialogKind,
    host: Entity<DialogHost>,
    template: SwappableTemplate,
}

impl DialogSurface {
    fn make(kind: DialogKind, look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let sdk = ThemedOverlayWindowTemplate::new(Arc::new(StyledOverlayWindowTheme::sdk()));
        let template = SwappableTemplate(Arc::new(Mutex::new(Arc::new(sdk))));
        let shared = template.clone();
        let host = cx.new(|cx| DialogHost::new(kind, look, shared, cx));
        Box::new(Self { kind, host, template })
    }

    pub(super) fn make_modal(look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Self::make(DialogKind::Modal, look, cx)
    }

    pub(super) fn make_modeless(look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Self::make(DialogKind::Modeless, look, cx)
    }

    pub(super) fn make_positioning(look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Self::make(DialogKind::Positioning, look, cx)
    }

    pub(super) fn make_draggable(look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Self::make(DialogKind::Draggable, look, cx)
    }

    /// The SDK overlay window and its Open button, for tests.
    #[cfg(all(test, feature = "test-support"))]
    fn parts(&self, cx: &App) -> (Entity<OverlayWindowControl>, Entity<Button>) {
        let host = self.host.read(cx);
        (host.window.clone(), host.open.clone())
    }
}

/// Handlers that do nothing, for a State matrix cell.
fn inert_handlers() -> OverlayWindowTemplateHandlers {
    OverlayWindowTemplateHandlers {
        key_down: Box::new(|_, _, _| {}),
        header_mouse_down: Box::new(|_, _, _| {}),
        shell_mouse_down: Box::new(|_, _, _| {}),
        shell_mouse_down_out: Box::new(|_, _, _| {}),
    }
}

impl SurfaceControl for DialogSurface {
    fn title(&self) -> &'static str {
        self.kind.title()
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    fn bind(&self, theme: Theme, _pinned: Option<&StateSet>, cx: &mut App) {
        self.template.set(template(theme));
        let window = self.host.read(cx).window.clone();
        window.update(cx, |_, cx| cx.notify());
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        let open = self.host.read(cx).open.clone();
        open.update(cx, |open, cx| open.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        let window = self.host.read(cx).window.clone();
        cx.subscribe(&window, move |side: &mut ControlSide, _, event: &OverlayWindowEvent, cx| {
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state: None }, cx);
        })
    }

    /// The window's shell, in place.
    fn cell(
        &self,
        theme: &Theme,
        id: SharedString,
        _states: &StateSet,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let template = template(theme.clone());
        let mode = self.kind.mode();
        let look = template.resolve_look(mode);
        let content = content(None);
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
        template.render(&model, &look, parts, window, cx).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{Focusable, TestAppContext};

    use super::*;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_dialog_pages_open_and_dismiss_and_draw_a_matrix_cell() {
        for id in ["dialog-modal", "dialog-modeless", "dialog-positioning", "dialog-draggable"] {
            let mut app = TestAppContext::single();
            let (page, cx) = open_surface(&mut app, id);
            let side = cx.update(|_, cx| page.read(cx).side());
            let (window, open) =
                cx.update(|_, cx| side.read(cx).control_as::<DialogSurface>().expect("a dialog").parts(cx));

            cx.update(|w, cx| open.read(cx).focus_handle(cx).focus(w, cx));
            cx.run_until_parked();
            cx.simulate_keystrokes("space");
            cx.run_until_parked();
            cx.update(|_, cx| assert!(window.read(cx).is_open(), "{id} opens"));
            assert!(cx.debug_bounds("workshop-sdk-dialog-bounds").is_some(), "{id} draws its window");
            let text = cx.update(|_, cx| side.read(cx).events_text(cx));
            assert!(text.contains("Opened"), "{id} logs opening: {text}");

            window.update(cx, |window, cx| window.dismiss(cx));
            cx.run_until_parked();
            cx.update(|_, cx| assert!(!window.read(cx).is_open(), "{id} dismisses"));

            side.update(cx, |side, cx| side.show_matrix(cx));
            cx.run_until_parked();
            assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "{id}'s matrix draws the window");
        }
    }
}
