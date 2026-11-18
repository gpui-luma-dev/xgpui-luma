//! Toolbar control exposition — gallery-aligned editing toolbar with focus strategy toggles.

use xgpui_luma::controls::toolbar::ToolbarEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::control_group::ControlGroupFocusStrategy;
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::controls::selector::SelectorItem;
use xgpui_luma::controls::toolbar::{Toolbar, ToolbarEvent, ToolbarValue, ToolbarItem};
use crate::theme::{ColorVizLookExt, Look};

use lucide_svg_static::Icon as LucideIcon;

use super::event_stream::ControlEventStream;

pub struct ToolbarControlExposition {
    look: Arc<Look>,
    left_pane: Entity<ToolbarExpositionLeftPane>,
}

struct ToolbarExpositionLeftPane {
    look: Arc<Look>,
    toolbar: Toolbar,
    roving_focus_button: Entity<Button<()>>,
    sequential_focus_button: Entity<Button<()>>,
    event_stream: Entity<ControlEventStream>,
    focus_mode: &'static str,
}

impl ToolbarExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.toolbar.update(cx, |toolbar, cx| toolbar.notify_items(cx));
        self.roving_focus_button.update(cx, |_, cx| cx.notify());
        self.sequential_focus_button.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ToolbarExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        use super::sample_layout::{card, page};
        use crate::theme::{TextSize, TypographyExt};
        let chrome = self.look.chrome();
        page(
            "controls-doc-toolbar-left-pane",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(card(
                    &self.look,
                    "command-toolbar-editing",
                    "Editing toolbar",
                    "Menus, text styles, formatting, and insert commands.",
                    1120.,
                    div().id("command-toolbar-scroll").w_full().overflow_x_scroll().child(self.toolbar.clone()),
                ))
                .child(card(
                    &self.look,
                    "command-toolbar-focus",
                    "Keyboard navigation",
                    "Choose how focus moves between toolbar items.",
                    420.,
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .gap(px(8.))
                                .child(self.roving_focus_button.clone())
                                .child(self.sequential_focus_button.clone()),
                        )
                        .child(
                            div()
                                .typography_style(self.look.typography_scale(TextSize::Xs))
                                .text_color(chrome.muted_text)
                                .child(crate::l10n::ui({
                                    let __l10n_0 = &(self.focus_mode);
                                    crate::l10n::UiCopy::compose(
                                        |args| crate::l10n::messages::msg_ui_e7117ea9cdce(args[0].clone()),
                                        vec![crate::l10n::UiCopy::state(*__l10n_0)],
                                    )
                                })),
                        ),
                ))
                .child(self.event_stream.clone()),
        )
    }
}

impl ToolbarControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<Look>) -> Self {
        let roving_focus_button = Button::new("controls-doc-toolbar-focus-roving")
            .content(|_, _| div().child(crate::l10n::ui("Roving focus")))
            .spawn(cx);
        let sequential_focus_button = Button::new("controls-doc-toolbar-focus-sequential")
            .content(|_, _| div().child(crate::l10n::ui("Sequential focus")))
            .spawn(cx);

        let toolbar = xgpui_luma::controls::toolbar::ToolbarBuilder::new("controls-doc-toolbar")
            .item(toolbar_menu(
                "assistant",
                "Assistant",
                LucideIcon::WandSparkles,
                [
                    MenuItem::new("rewrite").label("Rewrite"),
                    MenuItem::new("summarize").label("Summarize"),
                    MenuItem::new("continue").label("Continue writing"),
                ],
                cx,
            ))
            .separator("assistant-separator")
            .item(toolbar_selector(
                "style",
                "Paragraph",
                [
                    SelectorItem::new("paragraph").label("Paragraph").icon(LucideIcon::Pilcrow),
                    SelectorItem::new("heading").label("Heading").icon(LucideIcon::TypeIcon),
                    SelectorItem::new("quote").label("Quote").icon(LucideIcon::BookType),
                ],
                "paragraph",
                cx,
            ))
            .separator("style-separator")
            .item(toolbar_toggle("bold", LucideIcon::Bold, cx))
            .item(toolbar_toggle("italic", LucideIcon::Italic, cx))
            .item(toolbar_toggle("underline", LucideIcon::Underline, cx))
            .item(toolbar_label_menu(
                "color",
                "A",
                [
                    MenuItem::new("default").label("Default color"),
                    MenuItem::new("muted").label("Muted"),
                    MenuItem::new("accent").label("Accent"),
                ],
                cx,
            ))
            .separator("format-separator")
            .item(toolbar_menu(
                "alignment",
                "Alignment",
                LucideIcon::TextAlignStart,
                [
                    MenuItem::new("left").label("Align left").icon(LucideIcon::TextAlignStart),
                    MenuItem::new("center").label("Align center").icon(LucideIcon::TextAlignCenter),
                    MenuItem::new("right").label("Align right").icon(LucideIcon::TextAlignEnd),
                    MenuItem::new("justify").label("Justify").icon(LucideIcon::TextAlignJustify),
                ],
                cx,
            ))
            .separator("insert-separator")
            .item(toolbar_button("link", LucideIcon::Link, cx))
            .item(toolbar_button("image", LucideIcon::Image, cx))
            .item(toolbar_button("video", LucideIcon::Video, cx))
            .item(toolbar_menu(
                "more",
                "More",
                LucideIcon::Ellipsis,
                [
                    MenuItem::new("clear").label("Clear formatting"),
                    MenuItem::new("divider").label("Insert divider"),
                    MenuItem::new("comment").label("Add comment"),
                ],
                cx,
            ))
            .separator("code-separator")
            .item(toolbar_search(cx))
            .item(toolbar_button("code", LucideIcon::Code, cx))
            .spawn(cx);

        toolbar.update(cx, |toolbar, cx| toolbar.set_focus_strategy(ControlGroupFocusStrategy::RovingItemFocus, cx));

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-toolbar-event-log",
                "Interact with toolbar items; ToolbarEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|_| ToolbarExpositionLeftPane {
            look: look.clone(),
            toolbar: toolbar.clone(),
            roving_focus_button: roving_focus_button.clone(),
            sequential_focus_button: sequential_focus_button.clone(),
            event_stream: event_stream.clone(),
            focus_mode: "roving",
        });
        toolbar.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |_, event: &ToolbarEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                if let Some(line) = format_toolbar_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        });
        roving_focus_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_focus_strategy(ControlGroupFocusStrategy::RovingItemFocus, cx);
                    });
                    pane.focus_mode = "roving";
                    cx.notify();
                });
            }
        });
        sequential_focus_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_focus_strategy(ControlGroupFocusStrategy::ActiveDescendant, cx);
                    });
                    pane.focus_mode = "sequential";
                    cx.notify();
                });
            }
        });

        Self { look, left_pane }
    }

    pub fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        cx.notify();
    }
}

impl Render for ToolbarControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .id("controls-doc-toolbar-exposition")
            .size_full()
            .min_h(px(0.0))
            .min_w(px(0.0))
            .child(self.left_pane.clone())
    }
}

fn format_toolbar_event(event: &ToolbarEvent) -> Option<String> {
    match event {
        ToolbarEvent::Click { id } => Some(format!("ToolbarEvent::Click {{ id: \"{id}\" }}")),
        ToolbarEvent::Change { id, value } => {
            let payload = match value {
                ToolbarValue::Bool(on) => format!("bool({on})"),
                ToolbarValue::String(value) => format!("string(\"{value}\")"),
            };
            Some(format!("ToolbarEvent::Change {{ id: \"{id}\", value: {payload} }}"))
        }
        _ => None,
    }
}

// Sample items use only SDK builders and their default themes.
fn toolbar_menu<T: 'static>(
    id: &'static str,
    label: &'static str,
    icon: LucideIcon,
    items: impl IntoIterator<Item = MenuItem>,
    cx: &mut Context<T>,
) -> ToolbarItem {
    let menu = crate::l10n::localize_popup(
        xgpui_luma::controls::popup_menu::PopupMenuBuilder::new(id)
            .label(label)
            .icon(icon)
            .items(items)
            .spawn(cx),
        cx,
    );
    ToolbarItem::menu_control(id, menu, cx).label(label)
}
fn toolbar_label_menu<T: 'static>(
    id: &'static str,
    label: &'static str,
    items: impl IntoIterator<Item = MenuItem>,
    cx: &mut Context<T>,
) -> ToolbarItem {
    let menu = crate::l10n::localize_popup(
        xgpui_luma::controls::popup_menu::PopupMenuBuilder::new(id).label(label).items(items).spawn(cx),
        cx,
    );
    ToolbarItem::menu_control(id, menu, cx).label(label)
}
fn toolbar_selector<T: 'static>(
    id: &'static str,
    label: &'static str,
    items: impl IntoIterator<Item = SelectorItem>,
    selected: &'static str,
    cx: &mut Context<T>,
) -> ToolbarItem {
    let selector = xgpui_luma::controls::selector::SelectorBuilder::new(id)
        .label(label)
        .items(items)
        .selected_id(selected)
        .spawn(cx);
    let selector = crate::l10n::localize_selector(selector, cx);
    ToolbarItem::selector_control(id, selector, cx).label(label)
}
fn toolbar_toggle<T: 'static>(id: &'static str, icon: LucideIcon, cx: &mut Context<T>) -> ToolbarItem {
    let toggle = xgpui_luma::controls::toggle::new(id)
        .content(move |model, _| {
            xgpui_luma::infra::icon::lucide_icon(icon, model.look.foreground, model.look.icon_size)
        })
        .spawn(cx);
    ToolbarItem::toggle_button(id, toggle, cx).label(id)
}
fn toolbar_button<T: 'static>(id: &'static str, icon: LucideIcon, cx: &mut Context<T>) -> ToolbarItem {
    let button = Button::new(id)
        .role(xgpui_luma::controls::button_family::ButtonFamilyRole::Icon)
        .icon(icon)
        .spawn(cx);
    ToolbarItem::command_button(id, button, cx).label(id)
}
fn toolbar_search<T: 'static>(cx: &mut Context<T>) -> ToolbarItem {
    let field = crate::l10n::localize_field(
        xgpui_luma::controls::textfield::TextFieldBuilder::new("search").placeholder("Search...").spawn(cx),
        cx,
    );
    ToolbarItem::textfield_control("search", field, cx)
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext};
    use xgpui_luma::controls::button::ButtonEvent;
    use crate::workshop::{Sample, Workshop};

    #[test]
    fn command_toolbar_focus_modes_work_and_survive_navigation() {
        let mut app = TestAppContext::single();
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        let (workshop, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        });
        workshop.update(cx, |workshop, cx| workshop.select("toolbar", cx));
        cx.run_until_parked();
        let pane = cx.update(|_, cx| match &workshop.read(cx).samples["toolbar"] {
            Sample::Toolbar(sample) => sample.read(cx).left_pane.clone(),
            _ => unreachable!(),
        });
        let sequential = cx.update(|_, cx| pane.read(cx).sequential_focus_button.clone());
        cx.update(|window, cx| sequential.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
        workshop.update(cx, |workshop, cx| {
            workshop.select("button", cx);
            workshop.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
            workshop.select("toolbar", cx);
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let pane = pane.read(cx);
            assert_eq!(pane.focus_mode, "sequential");
            assert_eq!(pane.toolbar.read(cx).focus_strategy(), ControlGroupFocusStrategy::ActiveDescendant);
        });
        let roving = cx.update(|_, cx| pane.read(cx).roving_focus_button.clone());
        roving.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let pane = pane.read(cx);
            assert_eq!(pane.focus_mode, "roving");
            assert_eq!(pane.toolbar.read(cx).focus_strategy(), ControlGroupFocusStrategy::RovingItemFocus);
        });
    }
}
