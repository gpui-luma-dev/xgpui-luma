//! Developer tab screen: live tuning tools that are not part of the design docs.

use xgpui_luma::controls::tabs::TabsEventExt;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, App, div, prelude::*, px};
use xgpui_luma::controls::tabs::{Tabs, TabsEvent};
use super::tooltip_playground::TooltipPlayground;
use xgpui_luma::vstack;
use xgpui_luma_look_radix::{Look, SemanticRole};

use super::section::section;
use crate::controls::ClassicShadowEditor;

pub struct State {
    tabs: Entity<Tabs>,
}
impl State {
    pub fn new<M: 'static>(look: &Arc<Look>, shadow_editor: Entity<ClassicShadowEditor>, cx: &mut Context<M>) -> Self {
        let tooltips = cx.new(|cx| TooltipPlayground::new(look, cx));
        let button_look = Arc::clone(look);
        let tabs = xgpui_luma_look_radix::Tabs::new("developer-tabs")
            .look(look)
            .line()
            .with_template_modifier(|root, _| root.flex_none().debug_selector(|| "developer-tabs".into()))
            .tab_with("buttons", "Buttons", move |_, _| button_page(&button_look, shadow_editor.clone()))
            .tab("tooltips", "Tooltips", tooltips)
            .active("buttons")
            .spawn(cx);
        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| cx.notify());
        Self { tabs }
    }
    pub fn render(&self, cx: &App) -> AnyElement {
        div()
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(24.0))
            .w_full()
            .child(self.tabs.read(cx).tab_list())
            .child(self.tabs.read(cx).body())
            .into_any_element()
    }
}

fn button_page(look: &Arc<Look>, shadow_editor: Entity<ClassicShadowEditor>) -> AnyElement {
    let fg = look.resolve_role(SemanticRole::Foreground).hsla();
    let muted = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let border = look.resolve_role(SemanticRole::Border).hsla();

    vstack! {
        gap=28;
        section(
            "Classic Button",
            "Retunes the raised bubble live. Every Classic button in the app follows.",
            fg,
            muted,
            border,
            shadow_editor.into_any_element(),
        ),
    }
    .w_full()
    .into_any_element()
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use gpui::{Bounds, Pixels, Render, TestAppContext, Window};

    struct Page {
        state: State,
        bounds: HashMap<String, Bounds<Pixels>>,
    }
    impl Render for Page {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().flex().flex_col().child(
                div()
                    .id("developer-test-scroll")
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .overflow_y_scroll()
                    .child(self.state.render(cx)),
            )
        }
    }
    #[test]
    fn developer_tabs_switch_to_and_from_tooltip_preview() {
        let mut app = TestAppContext::single();
        let look = Arc::new(Look::built_in());
        let (page, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            let shadow = cx.new(|cx| ClassicShadowEditor::new(&look, cx));
            let state = State::new(&look, shadow, cx);
            state.tabs.clone().on_event(cx, |this: &mut Page, event: &TabsEvent, _| {
                if let TabsEvent::ItemBoundsChanged { tab_id, bounds } = event {
                    this.bounds.insert(tab_id.to_string(), *bounds);
                }
            });
            Page { state, bounds: HashMap::new() }
        });
        cx.simulate_resize(gpui::size(px(800.0), px(600.0)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("tooltip-preview").is_none());
        for tab in ["tooltips", "buttons", "tooltips"] {
            let bounds = cx.update(|_, app| page.read(app).bounds[tab]);
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
            assert_eq!(
                cx.update(|_, app| page.read(app).state.tabs.read(app).active_id().cloned()),
                Some(tab.into()),
                "clicked {tab} at {bounds:?}"
            );
            assert_eq!(cx.debug_bounds("tooltip-preview").is_some(), tab == "tooltips");
        }
    }
}
