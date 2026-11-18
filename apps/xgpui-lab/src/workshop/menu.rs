//! Native SDK menu owners and their default templates.
use std::sync::Arc;
use gpui::{AnyView, Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{
    context_menu::{self, MenuItem},
    popup_menu,
};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};
use super::{
    event_stream::ControlEventStream,
    sample_layout::{card, page},
};

#[derive(Clone, Copy)]
pub(super) enum MenuKind {
    Context,
    Popup,
}
impl MenuKind {
    fn title(self) -> &'static str {
        match self {
            Self::Context => "Context Menu",
            Self::Popup => "Popup Menu",
        }
    }
    fn description(self) -> &'static str {
        match self {
            Self::Context => {
                "Right-click the SDK target, or focus it and press Shift+F10. Arrow keys navigate the menu."
            }
            Self::Popup => {
                "Open the SDK menu trigger. Arrow keys navigate items and submenus; Enter selects and Escape dismisses."
            }
        }
    }
}
fn items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("new").label("New document"),
        MenuItem::new("export")
            .label("Export")
            .submenu([MenuItem::new("pdf").label("PDF"), MenuItem::new("text").label("Plain text")]),
        MenuItem::separator("separator"),
        MenuItem::new("unavailable").label("Unavailable action").enabled(false),
        MenuItem::new("copy").label("Copy"),
    ]
}
enum Control {
    Context(Entity<context_menu::ContextMenu>),
    Popup(Entity<popup_menu::PopupMenu>),
}
impl Control {
    fn view(&self) -> AnyView {
        match self {
            Self::Context(v) => v.clone().into(),
            Self::Popup(v) => v.clone().into(),
        }
    }
}
pub(super) struct MenuSample {
    kind: MenuKind,
    look: Arc<Look>,
    control: Control,
    selected: String,
    open: bool,
    events: Entity<ControlEventStream>,
}
impl MenuSample {
    pub(super) fn new(kind: MenuKind, look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let control = match kind {
            MenuKind::Context => Control::Context(crate::l10n::localize_context(
                context_menu::ContextMenuBuilder::new("workshop-sdk-context-menu")
                    .label("Right-click here")
                    .items(items())
                    .spawn(cx),
                cx,
            )),
            MenuKind::Popup => Control::Popup(crate::l10n::localize_popup(
                popup_menu::PopupMenuBuilder::new("workshop-sdk-popup-menu")
                    .label("Document actions")
                    .items(items())
                    .spawn(cx),
                cx,
            )),
        };
        match &control {
            Control::Context(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &context_menu::ContextMenuEvent, cx| {
                    match event {
                        context_menu::ContextMenuEvent::Select { item_id, label } => {
                            this.selected = format!("{label} ({item_id})")
                        }
                        context_menu::ContextMenuEvent::OpenChanged { open } => this.open = *open,
                        context_menu::ContextMenuEvent::HoverChanged { .. }
                        | context_menu::ContextMenuEvent::FocusChanged { .. } => return,
                        _ => {}
                    }
                    this.record(event, cx);
                })
                .detach();
            }
            Control::Popup(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &popup_menu::PopupMenuEvent, cx| {
                    match event {
                        popup_menu::PopupMenuEvent::Select { item_id, label } => {
                            this.selected = format!("{label} ({item_id})")
                        }
                        popup_menu::PopupMenuEvent::OpenChanged { open } => this.open = *open,
                        popup_menu::PopupMenuEvent::HoverChanged { .. }
                        | popup_menu::PopupMenuEvent::FocusChanged { .. } => return,
                        _ => {}
                    }
                    this.record(event, cx);
                })
                .detach();
            }
        }
        let events = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "workshop-menu-events",
                "Open, navigate, and select a native SDK menu item to see its events.",
            )
        });
        Self { kind, look, control, selected: "none".into(), open: false, events }
    }
    fn record(&mut self, event: &impl std::fmt::Debug, cx: &mut Context<Self>) {
        self.events.update(cx, |events, cx| events.append_line(&format!("{event:?}"), cx));
        cx.notify();
    }
    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for MenuSample {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        page(
            "workshop-menu-page",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(card(
                    &self.look,
                    "workshop-menu-example",
                    self.kind.title(),
                    self.kind.description(),
                    700.,
                    div().w_full().flex().flex_col().gap(px(16.)).child(self.control.view()).child(
                        div()
                            .typography_style(self.look.typography_scale(TextSize::Xs))
                            .text_color(self.look.chrome().muted_text)
                            .child(crate::l10n::ui({
                                let __l10n_0 = &(if self.open { "open" } else { "closed" });
                                let __l10n_1 = &(self.selected);
                                crate::l10n::UiCopy::compose(
                                    |args| crate::l10n::messages::msg_ui_d236e3233293(args[0].clone(), args[1].clone()),
                                    vec![
                                        crate::l10n::UiCopy::from(*__l10n_0),
                                        crate::l10n::UiCopy::numeric(__l10n_1.to_string()),
                                    ],
                                )
                            })),
                    ),
                ))
                .child(self.events.clone()),
        )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext};
    use crate::workshop::{Sample, Workshop};

    #[test]
    fn native_menus_select_submenus_skip_disabled_items_and_retain_selection() {
        let mut app = TestAppContext::single();
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            // The app supplies Escape through its enclosing LumaFocus scope.
            // This test mounts Workshop directly, so bind that same action here.
            cx.bind_keys([gpui::KeyBinding::new("escape", xgpui_luma::focus::EscapeFocus, None)]);
        });
        let (workshop, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        });
        for (id, open_keys) in [("context-menu", "shift-f10"), ("popup-menu", "space")] {
            workshop.update(cx, |v, cx| v.select(id, cx));
            cx.run_until_parked();
            let sample = cx.update(|window, cx| {
                let Sample::Menu(sample) = &workshop.read(cx).samples[id] else {
                    panic!("missing native menu")
                };
                let sample = sample.clone();
                let focus = match &sample.read(cx).control {
                    Control::Context(v) => v.read(cx).focus_handle(cx),
                    Control::Popup(v) => v.read(cx).focus_handle(cx),
                };
                focus.focus(window, cx);
                sample
            });
            cx.run_until_parked();
            cx.simulate_keystrokes(open_keys);
            cx.run_until_parked();
            cx.update(|_, cx| assert!(sample.read(cx).open));
            cx.simulate_keystrokes("home down right enter");
            cx.run_until_parked();
            cx.update(|_, cx| {
                assert_eq!(sample.read(cx).selected, "PDF (pdf)");
                assert!(!sample.read(cx).open);
            });
            cx.simulate_keystrokes(open_keys);
            cx.simulate_keystrokes("home down down enter");
            cx.run_until_parked();
            cx.update(|_, cx| {
                assert_eq!(sample.read(cx).selected, "Copy (copy)");
                assert!(!sample.read(cx).open);
            });
            cx.simulate_keystrokes(open_keys);
            cx.simulate_keystrokes("escape");
            cx.run_until_parked();
            cx.update(|_, cx| assert!(!sample.read(cx).open));
            workshop.update(cx, |v, cx| {
                v.select("button", cx);
                v.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
                v.select(id, cx);
            });
            cx.run_until_parked();
            cx.update(|_, cx| {
                let Sample::Menu(retained) = &workshop.read(cx).samples[id] else {
                    unreachable!()
                };
                assert_eq!(retained, &sample);
                assert_eq!(sample.read(cx).selected, "Copy (copy)");
            });
        }
        workshop.update(cx, |v, cx| v.select("floating-menu", cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-sdk-unavailable").is_some());
        cx.update(|_, cx| assert!(!workshop.read(cx).samples.contains_key("floating-menu")));
    }
}
