use std::sync::Arc;

use gpui::{App, Div, Hsla, Stateful, Window, div, prelude::*, px};
use xgpui_luma::infra::ElementExt;
use xgpui_luma::controls::control_group::ControlGroupItemHandlerExt;
use xgpui_luma::controls::tabs::{
    TabsIndicatorMotion, TabsRenderModel, TabsTemplate, TabsTemplateHandlers, TabsTheme, render_tab_button,
    resolve_tabs_uniform_item_width,
};
use xgpui_luma::theme::{ControlSize, InteractionState};
use xgpui_luma_look_shadcn::ShadcnLook;

pub fn luma_studio_tabs_template(look: Arc<ShadcnLook>, tab_size: ControlSize) -> Arc<dyn TabsTemplate> {
    let full_bar_color = look.token_color("border").unwrap_or(look.chrome().border);
    Arc::new(LumaStudioTabsTemplate { theme: look.tabs_theme(), full_bar_color, tab_size })
}

struct LumaStudioTabsTemplate {
    theme: Arc<dyn TabsTheme>,
    full_bar_color: Hsla,
    tab_size: ControlSize,
}

impl TabsTemplate for LumaStudioTabsTemplate {
    fn render(
        &self,
        model: &TabsRenderModel<'_>,
        handlers: TabsTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let list_look = self.theme.resolve_list(model.enabled, self.tab_size);
        let uniform_width = resolve_tabs_uniform_item_width(model, self.theme.as_ref(), self.tab_size, window);

        let active_item = model.items.iter().find(|item| item.active);
        let active_look =
            active_item.map(|item| self.theme.resolve_item(true, item.state.interaction_state(), self.tab_size));
        if let Some(motion) = model.indicator_motion {
            let look = active_look
                .unwrap_or_else(|| self.theme.resolve_item(true, InteractionState::default(), self.tab_size));
            motion.set_metrics(look.padding_x, look.indicator_height, look.indicator);
        }

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .px(px(24.0))
            .gap(px(list_look.gap))
            .p(px(list_look.padding))
            .rounded(px(list_look.radius))
            .child(div().absolute().left(px(0.0)).right(px(0.0)).bottom(px(0.0)).h(px(1.0)).bg(self.full_bar_color));

        if let Some(motion) = model.indicator_motion.cloned() {
            root = root.on_prepaint(move |bounds, _, _| {
                motion.set_list_bounds(bounds);
            });
        }

        if let Some(background) = list_look.background {
            root = root.bg(background);
        }

        if let Some(border) = list_look.border {
            root = root.border_1().border_color(border);
        }

        for (item, item_handlers) in model.items.iter().zip(handlers.into_item_handlers()) {
            let look = self.theme.resolve_item(item.active, item.state.interaction_state(), self.tab_size);
            let mut tab = render_tab_button(model.id, item, look, self.theme.font_family(), self.tab_size, window, cx)
                .control_group_item_handlers(item_handlers);

            if let Some(width) = uniform_width {
                tab = tab.w(px(width)).flex_none();
            }

            if !item.state.disabled {
                tab = tab.cursor_pointer();
            }

            tab = tab.role(gpui::Role::Tab).aria_label(item.label.clone()).debug_selector({
                let id = item.id.clone();
                move || format!("content-tab-{id}")
            });
            root = root.child(tab);
        }

        if let Some(indicator) = model.indicator.or_else(|| model.indicator_motion.and_then(TabsIndicatorMotion::paint))
        {
            root = root.child(
                div()
                    .absolute()
                    .left(px(indicator.left))
                    .bottom(px(0.0))
                    .w(px(indicator.width))
                    .h(px(indicator.height))
                    .rounded(px(indicator.height))
                    .bg(indicator.color),
            );
        }

        root
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use xgpui_luma::controls::tabs::TabsItem;

    #[test]
    fn content_tab_targets_preserve_navigation() {
        let mut app = gpui::TestAppContext::single();
        let look = Arc::new(ShadcnLook::built_in());
        struct Page {
            tabs: gpui::Entity<xgpui_luma::controls::tabs::Tabs>,
        }
        impl gpui::Render for Page {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
                div().child(self.tabs.clone())
            }
        }
        let (page, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            let builder = xgpui_luma::controls::tabs::Tabs::new("luma-studio-content-tabs")
                .template(luma_studio_tabs_template(look.clone(), ControlSize::Lg))
                .items([TabsItem::new("cards").label("Cards"), TabsItem::new("dashboard").label("Dashboard")])
                .active("cards");
            Page { tabs: builder.spawn(cx) }
        });
        cx.run_until_parked();
        let tabs = cx.update(|_, app| page.read(app).tabs.clone());
        let cards = cx.debug_bounds("content-tab-cards").expect("cards tab target");
        let dashboard = cx.debug_bounds("content-tab-dashboard").expect("dashboard tab target");
        assert!(cards.origin.x + cards.size.width <= dashboard.origin.x);
        cx.simulate_event(gpui::MouseMoveEvent { position: dashboard.center(), ..Default::default() });
        cx.executor().advance_clock(std::time::Duration::from_millis(500));
        cx.run_until_parked();
        cx.simulate_click(dashboard.center(), Default::default());
        cx.run_until_parked();
        assert_eq!(cx.update(|_, app| tabs.read(app).active_id().map(ToString::to_string)), Some("dashboard".into()));
    }
}
