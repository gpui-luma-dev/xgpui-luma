mod model;
pub(crate) mod panels;
mod parsing;
mod subscriptions;
mod sync;
mod theme_selector;

use std::sync::{Arc, RwLock};

use gpui::{Context, Entity, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::search_selector::SearchSelector;
use xgpui_luma::controls::tabs::{Tabs, TabsWidthMode};

use xgpui_luma::theme::ControlSize;
use xgpui_luma_look_shadcn::{ShadcnLook};
use xgpui_luma_look_shadcn as shadcn;

use self::model::TOKEN_CATEGORIES;
use self::panels::{ColorsPanel, OtherPanel, PanelContextMenuHost, TypographyPanel};
use self::parsing::token_color_with_fallback;
use self::theme_selector::{
    ThemeSelectorSwatchCache, render_theme_search_selector_item, theme_search_selector_panel_template,
    theme_search_selector_template, theme_selector_state,
};
use super::content_tabs::luma_studio_tabs_template;
use crate::studio::app::LumaStudioApp;
use crate::studio::overrides::StudioOverrides;

pub struct ThemeSidebar {
    look: std::sync::Arc<ShadcnLook>,
    global_overrides: std::collections::HashMap<String, xgpui_luma::color::ColorValue>,
    theme_selector: SearchSelector,
    theme_selector_swatches: Arc<RwLock<ThemeSelectorSwatchCache>>,
    theme_selector_selected_id: Arc<RwLock<SharedString>>,
    tabs: Entity<Tabs>,
    colors_panel: Entity<ColorsPanel>,
    colors_host: Entity<PanelContextMenuHost>,
    other_host: Entity<PanelContextMenuHost>,
    typography_panel: Entity<TypographyPanel>,
    other_panel: Entity<OtherPanel>,
}

impl ThemeSidebar {
    pub fn new(
        _app: Entity<LumaStudioApp>,
        look: std::sync::Arc<ShadcnLook>,
        active_theme_id: impl Into<gpui::SharedString>,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) -> Self {
        let active_theme_id = active_theme_id.into();
        let global_overrides = overrides.global_color_overrides.clone();
        let colors_panel = cx.new(|cx| ColorsPanel::new(look.clone(), overrides, cx));
        let colors_target = colors_panel.clone();
        let colors_host = cx.new(|cx| {
            PanelContextMenuHost::new(
                "luma-studio-colors-context-menu",
                move |_| colors_target.clone(),
                panels::colors_context_menu_items(),
                look.clone(),
                cx,
            )
        });
        let typography_panel = cx.new(|cx| TypographyPanel::new(look.clone(), cx));
        let other_panel = cx.new(|cx| OtherPanel::new(look.clone(), overrides, cx));
        let other_target = other_panel.clone();
        let other_host = cx.new(|cx| {
            PanelContextMenuHost::new(
                "luma-studio-other-context-menu",
                move |_| other_target.clone(),
                panels::other_context_menu_items(),
                look.clone(),
                cx,
            )
        });
        let theme_selector_swatches = Arc::new(RwLock::new(ThemeSelectorSwatchCache::empty()));
        let theme_selector_selected_id = Arc::new(RwLock::new(active_theme_id.clone()));
        let (theme_items, swatch_cache) = theme_selector_state(look.as_ref());
        *theme_selector_swatches.write().expect("theme selector swatches lock") = swatch_cache;
        let swatches_for_template = theme_selector_swatches.clone();
        let theme_selector = shadcn::SearchSelector::new("luma-studio-theme-selector", theme_items)
            .look(look.as_ref())
            .placeholder("Theme")
            .search_placeholder("Search themes...")
            .selected_id(active_theme_id)
            .full_width(true)
            .fill_popup_viewport(true)
            .template(theme_search_selector_template(
                &look,
                theme_selector_swatches.clone(),
                theme_selector_selected_id.clone(),
            ))
            .panel_template(theme_search_selector_panel_template())
            .with_item_template(move |model, cx| render_theme_search_selector_item(model, &swatches_for_template, cx))
            .spawn(cx);

        let tabs = sidebar_tabs_builder(
            look.clone(),
            colors_host.clone(),
            typography_panel.clone(),
            other_host.clone(),
            colors_panel.read(cx).scroll_handle.clone(),
        )
        .spawn(cx);

        Self {
            look,
            global_overrides,
            theme_selector,
            theme_selector_swatches,
            theme_selector_selected_id,
            tabs,
            colors_panel,
            colors_host,
            other_host,
            typography_panel,
            other_panel,
        }
    }
}

pub(crate) fn palette_tokens() -> Vec<&'static str> {
    TOKEN_CATEGORIES.iter().flat_map(|(_, tokens)| tokens.iter().map(|(token, _)| *token)).collect()
}

impl Render for ThemeSidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let sidebar_bg =
            token_color_with_fallback(&self.look, &self.global_overrides, "sidebar", chrome.panel_background);

        div()
            .id("luma-studio-sidebar")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(sidebar_bg)
            .child(
                div()
                    .id("luma-studio-sidebar-theme-selector-shell")
                    .w_full()
                    .h(px(48.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .px(px(24.0))
                    .border_b_1()
                    .border_color(chrome.border)
                    .child(div().flex().items_center().w_full().h_full().child(self.theme_selector.clone())),
            )
            .child(div().flex_shrink_0().pt(px(8.0)).child(div().w_full().child(self.tabs.read(cx).tab_list())))
            .child(self.tabs.read(cx).body())
    }
}

fn sidebar_tabs_builder(
    look: Arc<ShadcnLook>,
    colors: Entity<impl Render + 'static>,
    typography: Entity<impl Render + 'static>,
    other: Entity<impl Render + 'static>,
    colors_scroll: gpui::ScrollHandle,
) -> shadcn::Tabs {
    shadcn::Tabs::new("luma-studio-sidebar-tabs")
        .look(look.as_ref())
        .size(shadcn::ShadcnSize::Lg)
        .width_mode(TabsWidthMode::Uniform)
        .template(luma_studio_tabs_template(look, ControlSize::Lg))
        .tab_with("colors", "Colors", move |_, _| {
            sidebar_scroll_body().track_scroll(&colors_scroll).child(colors.clone())
        })
        .tab_with("typography", "Typography", move |_, _| sidebar_scroll_body().child(typography.clone()))
        .tab_with("other", "Other", move |_, _| sidebar_scroll_body().child(other.clone()))
        .active("colors")
}

fn sidebar_scroll_body() -> gpui::Stateful<gpui::Div> {
    div()
        .id("luma-studio-sidebar-scroll")
        .flex_1()
        .min_h(px(0.0))
        .w_full()
        .overflow_y_scroll()
        .p(px(12.0))
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use std::time::Duration;
    use xgpui_luma::motion::{Transition, Easing};

    struct Panel(&'static str);
    impl Render for Panel {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let id = self.0;
            div().debug_selector(move || format!("sidebar-panel-{id}")).h(px(800.0)).child(id)
        }
    }
    struct Page {
        tabs: Entity<Tabs>,
    }
    impl Render for Page {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(self.tabs.read(cx).tab_list())
                .child(self.tabs.read(cx).body())
        }
    }

    #[test]
    fn sidebar_tabs_inherit_motion_and_retain_scrollable_panels() {
        let mut app = gpui::TestAppContext::single();
        let (page, cx) = app.add_window_view(|_, cx| {
            let colors = cx.new(|_| Panel("colors"));
            let typography = cx.new(|_| Panel("typography"));
            let other = cx.new(|_| Panel("other"));
            Page {
                tabs: sidebar_tabs_builder(
                    Arc::new(ShadcnLook::built_in()),
                    colors,
                    typography,
                    other,
                    gpui::ScrollHandle::default(),
                )
                .spawn(cx),
            }
        });
        cx.simulate_resize(gpui::size(px(400.0), px(500.0)));
        cx.run_until_parked();
        let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
        cx.update(|_, cx| {
            assert_eq!(
                tabs.read(cx).motion_profile().content.description(),
                Transition::timed(Duration::from_millis(300), Easing::OutCubic).description()
            );
        });
        for id in ["colors", "typography", "other", "colors"] {
            cx.update(|_, cx| tabs.update(cx, |tabs, cx| tabs.set_active(id, cx)));
            cx.run_until_parked();
            for (panel, selector) in [
                ("colors", "sidebar-panel-colors"),
                ("typography", "sidebar-panel-typography"),
                ("other", "sidebar-panel-other"),
            ] {
                assert_eq!(cx.debug_bounds(selector).is_some(), panel == id);
            }
        }
    }
}
