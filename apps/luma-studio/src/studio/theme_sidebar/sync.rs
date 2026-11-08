use gpui::Context;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::controls::tabs::TabsWidthMode;

use super::ThemeSidebar;
use super::theme_selector::theme_selector_state;
use crate::studio::content_tabs::luma_studio_tabs_template;
use crate::studio::overrides::StudioOverrides;
use xgpui_luma_look_shadcn::ShadcnLook;

impl ThemeSidebar {
    /// Full sidebar refresh for theme/template changes.
    ///
    /// Use this path when the active `ShadcnLook` changes or when control templates need to be
    /// re-resolved from the current theme. Unlike the lightweight override sync below, this method
    /// is allowed to rebuild accordion entities because structure/styling inputs may have changed.
    ///
    /// We capture and restore the expanded category ids before rebuilding so a theme change does
    /// not collapse the user's working context in the sidebar.
    pub fn apply_theme_snapshot(
        &mut self,
        look: std::sync::Arc<ShadcnLook>,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) {
        self.look = look;
        self.global_overrides = overrides.global_color_overrides.clone();
        let theme = self.look.clone();
        self.sync_theme_selector_items(&theme, cx);
        self.sync_tabs_template(&theme, cx);
        self.colors_panel.update(cx, |panel, cx| panel.apply_theme_snapshot(theme.clone(), overrides, cx));
        self.typography_panel.update(cx, |panel, cx| panel.apply_theme_snapshot(theme.clone(), cx));
        self.other_panel.update(cx, |panel, cx| panel.apply_theme_snapshot(theme, overrides, cx));
        cx.notify();
    }

    fn sync_theme_selector_items(&self, theme: &std::sync::Arc<ShadcnLook>, cx: &mut Context<Self>) {
        let (items, swatch_cache) = theme_selector_state(theme.as_ref());
        *self.theme_selector_swatches.write().expect("theme selector swatches lock") = swatch_cache;
        self.theme_selector.update(cx, |selector, cx| {
            selector.set_items(items, cx);
            selector.set_template(
                super::theme_selector::theme_search_selector_template(
                    theme,
                    self.theme_selector_swatches.clone(),
                    self.theme_selector_selected_id.clone(),
                ),
                cx,
            );
        });
    }

    fn sync_tabs_template(&self, theme: &std::sync::Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.tabs.update(cx, |tabs, cx| {
            tabs.set_size(ControlSize::Lg, cx);
            tabs.set_width_mode(TabsWidthMode::Uniform, cx);
            tabs.set_template(luma_studio_tabs_template(theme.clone(), ControlSize::Lg), cx);
        });
    }

    /// Lightweight value sync for live override edits.
    ///
    /// This path is called frequently while the user types or drags controls. It intentionally
    /// updates existing fields/sliders in place and does *not* rebuild the token/other accordions.
    /// Replacing those entities during a drag can invalidate the active slider subtree mid-gesture,
    /// which manifests as the thumb moving slightly and then "stopping" until the UI settles.
    ///
    /// In short:
    /// - `apply_theme_snapshot` may rebuild structure when theme/template inputs change.
    /// - `sync_global_overrides` must stay incremental so live interactions remain stable.
    pub fn sync_global_overrides(&mut self, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        self.global_overrides = overrides.global_color_overrides.clone();
        self.colors_panel.update(cx, |panel, cx| panel.sync_global_overrides(overrides, cx));
        self.other_panel.update(cx, |panel, cx| panel.sync_global_overrides(overrides, cx));
        cx.notify();
    }

    pub fn sync_typography_selectors(&mut self, look: std::sync::Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.typography_panel.update(cx, |panel, cx| panel.sync_font_selectors(look, cx));
        cx.notify();
    }

    pub fn sync_theme_selector(&mut self, active_theme_id: impl Into<gpui::SharedString>, cx: &mut Context<Self>) {
        let active_theme_id = active_theme_id.into();
        *self.theme_selector_selected_id.write().expect("theme selector selected id lock") = active_theme_id.clone();
        self.theme_selector.update(cx, |selector, cx| {
            selector.set_selected_id(active_theme_id, cx);
        });
        cx.notify();
    }
}
