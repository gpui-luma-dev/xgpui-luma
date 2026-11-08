//! Toggle-style page navigation backed by the SDK screen content slots.

use xgpui_luma::controls::toggle::ToggleEventExt;
use std::sync::Arc;

use gpui::{
    AnyElement, Hsla, FontWeight, px, Context, Entity, EventEmitter, Focusable, IntoElement, Render, Subscription,
    Window, div, prelude::*,
};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::tabs::{Tabs};
use xgpui_luma::controls::toggle::{Toggle, ToggleEvent};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::prelude::TooltipEntityExt;
use xgpui_luma::controls::tooltip::{Tooltip, TooltipSettings};
use xgpui_luma::theme::ThemeMode;
use xgpui_luma_look_radix::Look;
use xgpui_luma_look_radix as radix;

use crate::assets::{icon_named, react_icon};
use crate::tabs::RadixStudioTab;

/// Radix icons are drawn on a native 15x15 grid.
const THEME_ICON_SIZE: f32 = 15.0;

pub(crate) const SCREEN_TABS: [(RadixStudioTab, &str, &str); 5] = [
    (RadixStudioTab::CustomPalette, "page-custom-palette", "Custom Palette"),
    (RadixStudioTab::Colors, "page-colors", "Colors"),
    (RadixStudioTab::Icons, "page-icons", "Icons"),
    (RadixStudioTab::StyleGuide, "page-style-guide", "Style Guide"),
    (RadixStudioTab::Developer, "page-developer", "Developer"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScreenNavEvent {
    Change { tab: RadixStudioTab },
    ModeChange { mode: ThemeMode },
    ResetTheme,
}

pub struct ScreenNav {
    look: Arc<Look>,
    active: RadixStudioTab,
    tabs: Entity<Tabs>,
    pages: Vec<(RadixStudioTab, Toggle)>,
    theme_toggle: Entity<Button>,
    theme_reset: Entity<Button>,
    tooltip_toggle: Entity<Button>,
    github: Entity<Button>,
    radix_link: Entity<Button>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ScreenNavEvent> for ScreenNav {}

impl ScreenNav {
    /// Page navigation uses the app look; theme actions follow the editable palette.
    pub fn new(look: &Arc<Look>, action_look: &Arc<Look>, tabs: Entity<Tabs>, cx: &mut Context<Self>) -> Self {
        // Top-right actions must have no background or hover fill in any state.
        let icon_look = Arc::clone(action_look);
        let theme_toggle = super::icon_button::ghost_no_hover("screen-nav-theme", action_look, ())
            .content(move |model, _| {
                let name = match icon_look.mode() {
                    ThemeMode::Dark => "moon",
                    ThemeMode::Light => "sun",
                };
                icon_named(name)
                    .map(|icon| react_icon(icon, model.look.foreground, THEME_ICON_SIZE))
                    .unwrap_or_else(|| div().into_any_element())
            })
            .spawn(cx)
            .help("Switch between light and dark mode", cx);

        let tooltip_toggle = super::icon_button::ghost_no_hover("screen-nav-tooltips", action_look, ())
            .content(|model, cx| {
                let enabled = cx.try_global::<TooltipSettings>().is_none_or(|settings| settings.enabled);
                icon_named(if enabled { "eye-open" } else { "eye-closed" })
                    .map(|icon| react_icon(icon, model.look.foreground, THEME_ICON_SIZE))
                    .unwrap_or_else(|| div().into_any_element())
            })
            .spawn(cx)
            .tooltip(Tooltip::new("Enable or disable app tooltips").always_enabled(), cx);

        let theme_reset = super::icon_button::ghost_no_hover("screen-nav-reset-theme", action_look, ())
            .content(|model, _| {
                icon_named("reset")
                    .map(|icon| react_icon(icon, model.look.foreground, THEME_ICON_SIZE))
                    .unwrap_or_else(|| div().into_any_element())
            })
            .spawn(cx)
            .help("Reset the custom palette", cx);

        let github = super::icon_button::ghost_no_hover("screen-nav-github", action_look, ())
            .content(|model, _| {
                icon_named("github-logo")
                    .map(|icon| react_icon(icon, model.look.foreground, THEME_ICON_SIZE))
                    .unwrap_or_else(|| div().into_any_element())
            })
            .spawn(cx)
            .help("View xgpui-luma on GitHub", cx);

        let radix_link = radix::Button::new("screen-nav-radix")
            .look(look)
            .ghost_quiet()
            .gray()
            .size(radix::ButtonSize::One)
            .compact()
            .content(|model, _| radix_lockup(model.look.foreground))
            .with_template_modifier(|root, _| root.aria_label("Radix website"))
            .spawn(cx)
            .help("Visit the original Radix website", cx);

        let mut subscriptions = Vec::new();
        let pages = SCREEN_TABS
            .iter()
            .map(|&(tab, id, label)| {
                let toggle = radix::Toggle::new(format!("{id}-toggle"))
                    .look(look)
                    .page()
                    .label(label)
                    .selected(tab == RadixStudioTab::CustomPalette)
                    .motion_enabled(false)
                    .spawn(cx);
                {
                    let callback_control = (&toggle).downgrade();
                    toggle.clone().on_event(cx, move |this, event: &ToggleEvent, cx| {
                        let Some(toggle) = callback_control.upgrade() else {
                            return;
                        };

                        if matches!(event, ToggleEvent::Change { .. }) {
                            if this.active == tab {
                                toggle.update(cx, |toggle, cx| toggle.set_data(true, cx));
                            } else {
                                this.select(tab, cx);
                            }
                        }
                    })
                };
                (tab, toggle)
            })
            .collect();
        tooltip_toggle.clone().on_click(cx, |this, cx| {
            let enabled = cx.try_global::<TooltipSettings>().is_none_or(|settings| settings.enabled);
            cx.set_global(TooltipSettings { enabled: !enabled });
            this.tooltip_toggle.update(cx, |_, cx| cx.notify());
            cx.refresh_windows();
        });
        radix_link.clone().on_click(cx, |_, cx| {
            cx.open_url("https://www.radix-ui.com/primitives/docs/overview/introduction");
        });
        github.clone().on_click(cx, |_, cx| {
            cx.open_url("https://github.com/gpui-luma-dev/xgpui-luma");
        });
        theme_reset.clone().on_click(cx, |_, cx| {
            cx.emit(ScreenNavEvent::ResetTheme);
        });
        subscriptions.push(cx.observe(&tabs, |this, tabs, cx| {
            let active_id = tabs.read(cx).active_id().cloned();
            if let Some((tab, _, _)) = SCREEN_TABS.iter().find(|(_, id, _)| Some(*id) == active_id.as_deref()) {
                this.select(*tab, cx);
            }
        }));
        theme_toggle.clone().on_click(cx, |this, cx| {
            let mode = match this.look.mode() {
                ThemeMode::Dark => ThemeMode::Light,
                ThemeMode::Light => ThemeMode::Dark,
            };
            cx.emit(ScreenNavEvent::ModeChange { mode });
        });

        Self {
            look: Arc::clone(look),
            active: RadixStudioTab::CustomPalette,
            tabs,
            pages,
            theme_toggle,
            theme_reset,
            tooltip_toggle,
            github,
            radix_link,
            _subscriptions: subscriptions,
        }
    }

    pub fn page_focus_handle(&self, id: &str, cx: &gpui::App) -> Option<gpui::FocusHandle> {
        let tab = SCREEN_TABS.iter().find(|(_, page, _)| *page == id)?.0;
        self.pages.iter().find(|(page, _)| *page == tab).map(|(_, toggle)| toggle.read(cx).focus_handle(cx))
    }

    #[cfg(all(test, feature = "test-support"))]
    pub fn active(&self) -> RadixStudioTab {
        self.active
    }

    pub fn select(&mut self, tab: RadixStudioTab, cx: &mut Context<Self>) {
        if self.active == tab {
            return;
        }
        self.active = tab;
        for (page, toggle) in &self.pages {
            toggle.update(cx, |toggle, cx| toggle.set_data(*page == tab, cx));
        }
        if let Some((_, id, _)) = SCREEN_TABS.iter().find(|(screen, _, _)| *screen == tab) {
            self.tabs.update(cx, |tabs, cx| {
                // Keep the screen content slots in sync with the page toggles.
                if tabs.active_id().map(|active| active.as_ref()) != Some(*id) {
                    tabs.set_active(*id, cx);
                }
            });
        }
        cx.emit(ScreenNavEvent::Change { tab });
        cx.notify();
    }

    /// Refresh child controls after mutating their shared Look.
    pub fn theme_changed(&mut self, cx: &mut Context<Self>) {
        self.tabs.update(cx, |tabs, cx| tabs.set_template(radix::tabs_template(&self.look), cx));
        for (_, toggle) in &self.pages {
            toggle.update(cx, |_, cx| cx.notify());
        }
        self.theme_toggle.update(cx, |_, cx| cx.notify());
        self.theme_reset.update(cx, |_, cx| cx.notify());
        self.tooltip_toggle.update(cx, |_, cx| cx.notify());
        self.github.update(cx, |_, cx| cx.notify());
        self.radix_link.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

/// The Radix square, circle, and half-circle mark from the website header.
fn radix_lockup(foreground: Hsla) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .text_color(foreground)
        .child(
            div()
                .relative()
                .w(px(16.5))
                .h(px(24.0))
                .flex_none()
                .child(div().absolute().left_0().top_0().size(px(7.5)).bg(foreground))
                .child(div().absolute().left(px(9.0)).top_0().size(px(7.5)).rounded_full().bg(foreground))
                .child(div().absolute().left_0().top(px(9.0)).w(px(7.5)).h(px(15.0)).rounded_l(px(7.5)).bg(foreground)),
        )
        .child(div().text_size(px(21.0)).line_height(px(24.0)).font_weight(FontWeight::SEMIBOLD).child("Radix"))
        .into_any_element()
}

impl Render for ScreenNav {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(self.radix_link.clone())
                    .child(
                        div()
                            .w(px(1.0))
                            .h(px(24.0))
                            .flex_none()
                            .bg(self.look.resolve_role(radix::SemanticRole::Border).hsla()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(self.look.resolve_role(radix::SemanticRole::Foreground).hsla())
                            .child("Made with XGPUI-LUMA"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                        if event.keystroke.modifiers != Default::default() {
                            return;
                        }
                        let Some(index) = this
                            .pages
                            .iter()
                            .position(|(_, toggle)| toggle.read(cx).focus_handle(cx).is_focused(window))
                        else {
                            return;
                        };
                        let next = match event.keystroke.key.as_str() {
                            "left" => (index + this.pages.len() - 1) % this.pages.len(),
                            "right" => (index + 1) % this.pages.len(),
                            _ => return,
                        };
                        let (tab, toggle) = this.pages[next].clone();
                        this.select(tab, cx);
                        toggle.read(cx).focus_handle(cx).focus(window, cx);
                        cx.stop_propagation();
                    }))
                    .children(self.pages.iter().zip(SCREEN_TABS).map(|((_, toggle), (_, id, _))| {
                        div().debug_selector(move || id.into()).child(toggle.clone())
                    })),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .child(self.github.clone())
                    .child(self.tooltip_toggle.clone())
                    .child(self.theme_reset.clone())
                    .child(self.theme_toggle.clone()),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[test]
    fn clicked_page_keeps_focus_and_arrows_select_neighboring_screens() {
        let mut app = TestAppContext::single();
        let (nav, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(Look::built_in());
            let mut tabs = radix::Tabs::new("test-screens").look(&look);
            for (_, id, label) in SCREEN_TABS {
                tabs = tabs.tab_with(id, label, |_, _| div());
            }
            let tabs = tabs.spawn(cx);
            ScreenNav::new(&look, &look, tabs, cx)
        });
        cx.simulate_resize(gpui::size(px(1400.0), px(100.0)));
        cx.run_until_parked();
        let colors = cx.debug_bounds("page-colors").expect("Colors toggle");
        cx.simulate_click(colors.center(), Default::default());
        cx.run_until_parked();
        assert_page_focused(&nav, RadixStudioTab::Colors, cx);
        cx.simulate_keystrokes("right");
        cx.run_until_parked();
        assert_page_focused(&nav, RadixStudioTab::Icons, cx);
        cx.simulate_keystrokes("left");
        cx.run_until_parked();
        assert_page_focused(&nav, RadixStudioTab::Colors, cx);
    }

    fn assert_page_focused(nav: &Entity<ScreenNav>, expected: RadixStudioTab, cx: &mut gpui::VisualTestContext) {
        cx.update(|window, cx| {
            let nav = nav.read(cx);
            assert_eq!(nav.active(), expected);
            let (_, toggle) = nav.pages.iter().find(|(tab, _)| *tab == expected).unwrap();
            assert!(toggle.read(cx).focus_handle(cx).is_focused(window));
        });
    }
}
