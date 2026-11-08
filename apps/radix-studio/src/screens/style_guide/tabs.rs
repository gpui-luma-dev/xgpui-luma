//! Tabs examples using the same three preview pages as other controls.

use xgpui_luma::controls::tabs::TabsEventExt;
use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, Hsla, IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma::controls::tabs::{
    Tabs, TabsEvent, TabsItem, TabsTheme, TabsListLook, TabsItemLook, TabsBaseline, ThemedTabsTemplate,
};
use xgpui_luma::theme::{ControlSize, InteractionState};
use xgpui_luma_look_radix::{Accent, Gray, Look, ScaleFamily, TabsSize};

use super::matrix_grid::{preview_tabbed, fixed_grid, equal_data_columns};

#[derive(Clone)]
pub struct TabsExamples {
    navigation: Entity<Tabs>,
    preview: Vec<(String, Entity<Tabs>)>,
    sizes: Vec<(String, Entity<Tabs>)>,
    colors: Vec<(String, Look, Entity<Tabs>, Entity<Tabs>)>,
}

fn example<M: 'static>(id: String, look: &Look, size: TabsSize, enabled: bool, cx: &mut Context<M>) -> Entity<Tabs> {
    xgpui_luma_look_radix::Tabs::new(id)
        .look(look)
        .line()
        .size(size)
        .enabled(enabled)
        .with_template_modifier(|root, _| root.w_full())
        .items([
            TabsItem::new("themes").label("Themes"),
            TabsItem::new("primitives").label("Primitives"),
            TabsItem::new("icons").label("Icons"),
            TabsItem::new("colors").label("Colors").enabled(false),
        ])
        .active("themes")
        .spawn(cx)
}

impl TabsExamples {
    pub fn spawn<M: 'static>(look: &Look, cx: &mut Context<M>) -> Self {
        let navigation = xgpui_luma_look_radix::Tabs::new("guide-tabs-preview-navigation")
            .look(look)
            .with_template_modifier(|root, _| root.w_full())
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .spawn(cx);
        navigation.clone().on_event(cx, |_, event: &TabsEvent, cx| {
            if matches!(event, TabsEvent::Change { .. }) {
                cx.notify();
            }
        });
        let preview = [("Interactive", true), ("Disabled", false)]
            .into_iter()
            .map(|(label, enabled)| {
                (label.to_owned(), example(format!("guide-tabs-preview-{label}"), look, TabsSize::Two, enabled, cx))
            })
            .collect();
        let sizes = TabsSize::ALL
            .into_iter()
            .map(|size| {
                (size.label().to_owned(), example(format!("guide-tabs-size-{}", size.as_str()), look, size, true, cx))
            })
            .collect();
        let colors = Accent::ALL
            .iter()
            .map(|accent| {
                let row_look = look.fork();
                row_look.set_palettes(*accent, Gray::Auto);
                let tabs = color_example(format!("guide-tabs-color-{}", accent.as_str()), &row_look, false, cx);
                let contrast = color_example(format!("guide-tabs-contrast-{}", accent.as_str()), &row_look, true, cx);
                (super::palettes::title_case(accent.as_str()), row_look, tabs, contrast)
            })
            .collect();
        Self { navigation, preview, sizes, colors }
    }

    pub fn render(&self, look: &Look, muted: Hsla, border: Hsla, cx: &mut App) -> AnyElement {
        let active = self.navigation.read(cx).active_id().cloned();
        if active.as_deref() == Some("colors") {
            let mut grid = fixed_grid(equal_data_columns(88.0, 240.0, 2), 24.0, 24.0);
            for (row, (label, row_look, tabs, contrast)) in self.colors.iter().enumerate() {
                if row_look.mode() != look.mode() {
                    row_look.set_mode(look.mode());
                    tabs.update(cx, |_, cx| cx.notify());
                    contrast.update(cx, |_, cx| cx.notify());
                }
                grid = grid
                    .child(
                        div().h(px(40.0)).flex().items_center().text_sm().text_color(muted).child(label.clone()),
                        row,
                        0,
                    )
                    .child(tabs.clone(), row, 1)
                    .child(contrast.clone(), row, 2);
            }
            return preview_tabbed(self.navigation.clone(), border, grid.into_any_element());
        }
        let controls = match active.as_deref() {
            Some("all-sizes") => self.sizes.clone(),
            _ => self.preview.clone(),
        };
        let body = div()
            .flex()
            .flex_wrap()
            .gap(px(24.0))
            .children(controls.into_iter().map(|(label, tabs)| {
                div()
                    .flex()
                    .flex_col()
                    .w(px(400.0))
                    .gap(px(8.0))
                    .child(div().text_sm().text_color(muted).child(label))
                    .child(tabs)
            }))
            .into_any_element();
        preview_tabbed(self.navigation.clone(), border, body)
    }
}

// Local comparison treatment: reuse Radix tabs and only change the selected
// indicator to accent 12 for the high-contrast column.
struct ContrastTabsTheme {
    base: Arc<dyn TabsTheme>,
    look: Look,
}

impl TabsTheme for ContrastTabsTheme {
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> TabsListLook {
        self.base.resolve_list(enabled, size)
    }

    fn resolve_item(&self, active: bool, state: InteractionState, size: ControlSize) -> TabsItemLook {
        let mut item = self.base.resolve_item(active, state, size);
        if item.indicator.is_some() {
            item.indicator = Some(self.look.resolve_step(ScaleFamily::Color, 12).hsla());
        }
        item
    }

    fn content_padding(&self, size: ControlSize) -> Option<(f32, f32)> {
        self.base.content_padding(size)
    }

    fn font_family(&self) -> SharedString {
        self.base.font_family()
    }
    fn baseline(&self, enabled: bool, size: ControlSize) -> Option<TabsBaseline> {
        self.base.baseline(enabled, size)
    }
    fn indicator_inset(&self, size: ControlSize) -> Option<f32> {
        self.base.indicator_inset(size)
    }
}

fn color_example<M: 'static>(id: String, look: &Look, contrast: bool, cx: &mut Context<M>) -> Entity<Tabs> {
    let base = xgpui_luma_look_radix::tabs_theme(look);
    let theme: Arc<dyn TabsTheme> = if contrast {
        Arc::new(ContrastTabsTheme { base, look: look.clone() })
    } else {
        base
    };
    let template = ThemedTabsTemplate::new(theme).with_modifier(|root, _| root.w_full());
    Tabs::new(id)
        .template(Arc::new(template))
        .items([TabsItem::new("account").label("Account"), TabsItem::new("documents").label("Documents")])
        .active("account")
        .spawn(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn contrast_indicator_tracks_palette_and_mode_without_changing_labels() {
        let look = Look::built_in();
        let base = xgpui_luma_look_radix::tabs_theme(&look);
        let contrast = ContrastTabsTheme { base: base.clone(), look: look.clone() };
        for accent in [Accent::Gold, Accent::Bronze] {
            look.set_palettes(accent, Gray::Auto);
            for mode in [ThemeMode::Light, ThemeMode::Dark] {
                look.set_mode(mode);
                let normal = base.resolve_item(true, InteractionState::default(), ControlSize::Md);
                let high = contrast.resolve_item(true, InteractionState::default(), ControlSize::Md);
                assert_eq!(high.indicator, Some(look.resolve_step(ScaleFamily::Color, 12).hsla()));
                assert_ne!(normal.indicator, high.indicator);
                assert_eq!(normal.label_color, high.label_color);
                assert_eq!(normal.height, high.height);
                assert!(contrast.resolve_item(false, InteractionState::default(), ControlSize::Md).indicator.is_none());
            }
        }
    }
}
