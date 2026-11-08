use xgpui_luma::controls::search_selector::SearchSelectorEventExt;
use xgpui_luma::controls::context_menu::ContextMenuEventExt;
use gpui::{Context, Entity, Subscription};
use xgpui_luma::controls::{context_menu::ContextMenuEvent, search_selector::SearchSelectorEvent};

use super::panels::{ColorsPanel, OtherPanel, TypographyPanel};
use super::ThemeSidebar;
use crate::studio::app::LumaStudioApp;

impl ThemeSidebar {
    /// Wire selector and token field events on the app entity.
    ///
    /// Subscriptions must not be registered on `ThemeSidebar` itself: `cx.subscribe` re-enters
    /// the subscriber entity, and handlers call `theme_sidebar.update`, which panics.
    pub fn wire_subscriptions(
        sidebar: &Entity<Self>,
        cx: &mut Context<LumaStudioApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        let theme_selector = sidebar.read(cx).theme_selector.clone();
        theme_selector.clone().on_event(cx, |app, event: &SearchSelectorEvent, cx| {
            let theme_id = match event {
                SearchSelectorEvent::Select { item_id, .. } | SearchSelectorEvent::Complete { item_id, .. } => item_id,
                _ => return,
            };
            app.change_theme(theme_id.as_ref(), cx);
        });

        let colors_host = sidebar.read(cx).colors_host.clone();
        let colors_panel = sidebar.read(cx).colors_panel.clone();
        let colors_context_menu = colors_host.read(cx).context_menu();
        let colors_panel_for_menu = colors_panel.clone();
        colors_context_menu.clone().on_event(cx, move |app, event: &ContextMenuEvent, cx| {
            let ContextMenuEvent::Select { item_id, .. } = event else {
                return;
            };
            match item_id.as_ref() {
                "expand-all" => {
                    colors_panel_for_menu.update(cx, |panel, cx| panel.set_all_categories_expanded(true, cx))
                }
                "collapse-all" => {
                    colors_panel_for_menu.update(cx, |panel, cx| panel.set_all_categories_expanded(false, cx))
                }
                "reset" => app.reload_active_theme(cx),
                "toggle-mode" => app.toggle_mode(cx),
                _ => {}
            }
        });

        ColorsPanel::wire_subscriptions(&colors_panel, cx, subscriptions);

        let other_panel = sidebar.read(cx).other_panel.clone();
        let other_host = sidebar.read(cx).other_host.clone();
        let other_context_menu = other_host.read(cx).context_menu();
        other_context_menu.clone().on_event(cx, |app, event: &ContextMenuEvent, cx| {
            let ContextMenuEvent::Select { item_id, .. } = event else {
                return;
            };
            if item_id.as_ref() == "reset" {
                app.reload_active_theme(cx);
            }
        });
        OtherPanel::wire_subscriptions(&other_panel, cx, subscriptions);

        let typography_panel = sidebar.read(cx).typography_panel.clone();
        TypographyPanel::wire_subscriptions(&typography_panel, cx);
    }
}
