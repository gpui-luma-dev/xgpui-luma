//! Tooltip presentation bound to the control's mutable Shadcn look.
use std::sync::Arc;
use xgpui_luma::controls::tooltip::{TooltipLook, TooltipTheme};
use crate::{ShadcnLook, ShadcnToken, ShadcnRadius};
struct ShadcnTooltipTheme(ShadcnLook);
impl TooltipTheme for ShadcnTooltipTheme {
    fn resolve(&self) -> TooltipLook {
        let tokens = self.0.mode_tokens();
        let geometry = tokens.stylesheet().common.tooltip.resolve_geometry(
            "default",
            xgpui_luma::theme::stylesheet::TooltipGeometry {
                padding: 8.0,
                padding_y: 4.0,
                max_width: 260.0,
                font_size: 12.0,
                line_height: 16.0,
                ..Default::default()
            },
        );
        TooltipLook {
            placement: (&geometry).into(),
            shortcut_gap: geometry.shortcut_gap,
            shortcut_font_size: geometry.shortcut_font_size,
            shortcut_opacity: geometry.shortcut_opacity,
            background: self.0.color(ShadcnToken::Foreground),
            foreground: self.0.color(ShadcnToken::Background),
            padding: geometry.padding,
            padding_y: geometry.padding_y,
            radius: self.0.radius(ShadcnRadius::Md),
            max_width: geometry.max_width,
            text_size: geometry.font_size,
            line_height: geometry.line_height,
            shadow: Vec::new(),
        }
    }
}
/// Resolve tooltip tokens on each presentation, including mode changes.
pub fn tooltip_theme(look: &ShadcnLook) -> Arc<dyn TooltipTheme> {
    Arc::new(ShadcnTooltipTheme(look.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::ThemeMode;
    #[test]
    fn tooltip_tracks_bound_look_mode() {
        let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled CSS");
        let theme = tooltip_theme(&look);
        let light = theme.resolve();
        look.set_mode(ThemeMode::Dark);
        let dark = theme.resolve();
        assert_ne!(light.background, dark.background);
        assert_eq!(dark.background, look.color(ShadcnToken::Foreground));
        assert_eq!(dark.foreground, look.color(ShadcnToken::Background));
    }
}

#[cfg(all(test, feature = "test-support"))]
mod coverage_tests {
    use super::*;
    use gpui::{AnyView, Context, IntoElement, Render, TestAppContext, Window};
    use xgpui_luma::{
        controls::tooltip::{Tooltip, bubble},
        infra::attachments::TooltipEntityExt,
    };
    use xgpui_luma::theme::ThemeMode;
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    struct Page {
        target: AnyView,
    }
    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.target.clone()
        }
    }
    macro_rules! coverage {
        ($name:ident, $builder:expr) => {
            #[test]
            fn $name() {
                let mut app = TestAppContext::single();
                let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled CSS");
                let colors = Arc::new(Mutex::new(Vec::new()));
                let log = colors.clone();
                let (_, cx) = app.add_window_view(|window, cx| {
                    window.activate_window();
                    let target = $builder.look(&look).spawn(cx).tooltip(
                        Tooltip::new("Help").delay(Duration::ZERO).template(move |model| {
                            log.lock().unwrap().push((model.background, model.foreground));
                            bubble(model)
                        }),
                        cx,
                    );
                    Page { target: target.into() }
                });
                cx.simulate_event(gpui::MouseMoveEvent {
                    position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
                    ..Default::default()
                });
                cx.run_until_parked();
                assert_eq!(
                    colors.lock().unwrap().last(),
                    Some(&(look.color(ShadcnToken::Foreground), look.color(ShadcnToken::Background)))
                );
                look.set_mode(ThemeMode::Dark);
                cx.update(|window, _| window.refresh());
                cx.run_until_parked();
                // A theme change may resize the target and dismiss its current tip.
                cx.simulate_event(gpui::MouseMoveEvent {
                    position: gpui::point(gpui::px(600.0), gpui::px(600.0)),
                    ..Default::default()
                });
                cx.executor().advance_clock(Duration::from_millis(200));
                cx.run_until_parked();
                cx.simulate_event(gpui::MouseMoveEvent {
                    position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
                    ..Default::default()
                });
                cx.run_until_parked();
                assert_eq!(
                    colors.lock().unwrap().last(),
                    Some(&(look.color(ShadcnToken::Foreground), look.color(ShadcnToken::Background)))
                );
            }
        };
    }
    coverage!(button_help_tracks_look, crate::Button::new("button"));
    coverage!(icon_button_help_tracks_look, crate::Button::icon_button("icon", lucide_svg_static::Icon::Plus));
    coverage!(typed_button_help_tracks_look, crate::Button::new("typed").typed(42u32));
    coverage!(checkbox_help_tracks_look, crate::Checkbox::new("checkbox"));
    coverage!(switch_help_tracks_look, crate::Switch::new("switch"));
    coverage!(radio_help_tracks_look, crate::Radio::new("radio"));
    coverage!(toggle_help_tracks_look, crate::Toggle::new("toggle"));
    coverage!(textfield_help_tracks_look, crate::TextField::new("textfield"));
    coverage!(popup_menu_help_tracks_look, crate::PopupMenu::new("popup"));
    coverage!(split_button_help_tracks_look, crate::SplitButton::new("split"));
    coverage!(selector_help_tracks_look, crate::Selector::new("selector"));
    coverage!(
        search_selector_help_tracks_look,
        crate::SearchSelector::new("search", [xgpui_luma::controls::search_selector::SelectionItem::new("one", "One")])
    );
    coverage!(
        combobox_help_tracks_look,
        crate::ComboBox::new("combo", [xgpui_luma::controls::combobox::SelectionItem::new("one", "One")])
    );

    macro_rules! popup_behavior {
        ($name:ident, $builder:expr, $event:path, $selection:pat) => {
            #[test]
            fn $name() {
                use $event as ControlEvent;
                let mut app = TestAppContext::single();
                app.update(xgpui_luma::key_handling::bind_default_control_keys);
                let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled CSS");
                let tips = Arc::new(Mutex::new(Vec::new()));
                let openings = Arc::new(Mutex::new(Vec::new()));
                let tip_log = tips.clone();
                let open_log = openings.clone();
                let selected = Arc::new(Mutex::new(false));
                let selection_log = selected.clone();
                let (_, cx) = app.add_window_view(|window, cx| {
                    window.activate_window();
                    let target = $builder.look(&look).spawn(cx).tooltip(
                        Tooltip::new("Help")
                            .delay(Duration::ZERO)
                            .on_event(move |event| tip_log.lock().unwrap().push(event)),
                        cx,
                    );
                    cx.subscribe(&target, move |_, _, event: &ControlEvent, _| {
                        if matches!(event, $selection) {
                            *selection_log.lock().unwrap() = true;
                        }
                        if let ControlEvent::OpenChanged { open } = event {
                            open_log.lock().unwrap().push(*open);
                        }
                    })
                    .detach();
                    Page { target: target.into() }
                });
                let position = gpui::point(gpui::px(10.0), gpui::px(10.0));
                cx.simulate_event(gpui::MouseMoveEvent { position, ..Default::default() });
                cx.run_until_parked();
                assert_eq!(tips.lock().unwrap().as_slice(), &[xgpui_luma::controls::tooltip::TooltipEvent::Shown]);
                cx.simulate_click(position, Default::default());
                cx.run_until_parked();
                if openings.lock().unwrap().last() != Some(&true) {
                    // An editable ComboBox opens from its input with ArrowDown.
                    cx.simulate_keystrokes("down");
                    cx.run_until_parked();
                }
                assert_eq!(openings.lock().unwrap().last(), Some(&true));
                assert_eq!(tips.lock().unwrap().last(), Some(&xgpui_luma::controls::tooltip::TooltipEvent::Hidden));
                let count = tips.lock().unwrap().len();
                cx.simulate_event(gpui::MouseMoveEvent {
                    position: gpui::point(gpui::px(600.0), gpui::px(600.0)),
                    ..Default::default()
                });
                cx.executor().advance_clock(Duration::from_millis(200));
                cx.run_until_parked();
                cx.simulate_event(gpui::MouseMoveEvent { position, ..Default::default() });
                cx.executor().advance_clock(Duration::from_secs(1));
                cx.run_until_parked();
                assert_eq!(tips.lock().unwrap().len(), count);
                cx.simulate_keystrokes("down enter");
                cx.run_until_parked();
                assert_eq!(openings.lock().unwrap().last(), Some(&false));
                assert!(*selected.lock().unwrap());
            }
        };
    }
    popup_behavior!(
        popup_help_preserves_activation,
        crate::PopupMenu::new("popup").items([xgpui_luma::infra::menu_item::MenuItem::new("one").label("One")]),
        xgpui_luma::controls::popup_menu::PopupMenuEvent,
        ControlEvent::Select { .. }
    );
    popup_behavior!(
        selector_help_preserves_selection,
        crate::Selector::new("selector").items([xgpui_luma::controls::selector::SelectorItem::new("one").label("One")]),
        xgpui_luma::controls::selector::SelectorEvent,
        ControlEvent::Change { .. }
    );
    popup_behavior!(
        search_help_preserves_selection,
        crate::SearchSelector::new("search", [xgpui_luma::controls::search_selector::SelectionItem::new("one", "One")]),
        xgpui_luma::controls::search_selector::SearchSelectorEvent,
        ControlEvent::Select { .. } | ControlEvent::Complete { .. }
    );
    popup_behavior!(
        combo_help_preserves_selection,
        crate::ComboBox::new("combo", [xgpui_luma::controls::combobox::SelectionItem::new("one", "One")]),
        xgpui_luma::controls::combobox::ComboBoxEvent,
        ControlEvent::Select { .. } | ControlEvent::Complete { .. }
    );

    #[test]
    fn sidebar_item_help_tracks_look() {
        let mut app = TestAppContext::single();
        let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled CSS");
        let colors = Arc::new(Mutex::new(Vec::new()));
        let log = colors.clone();
        let (_, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            let target = crate::Sidebar::new("sidebar")
                .look(&look)
                .motion_enabled(false)
                .sidebar(crate::Sidebar::panel("content").content(crate::Sidebar::content().group(
                    crate::Sidebar::group().menu(crate::Sidebar::menu("main").item(
                        crate::Sidebar::menu_item("first", "First").icon(lucide_svg_static::Icon::House).tooltip(
                            Tooltip::new("First help").delay(Duration::ZERO).template(move |model| {
                                log.lock().unwrap().push((model.background, model.foreground));
                                bubble(model)
                            }),
                        ),
                    )),
                )))
                .spawn(cx);
            Page { target: target.into() }
        });
        cx.simulate_event(gpui::MouseMoveEvent {
            position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
            ..Default::default()
        });
        cx.run_until_parked();
        assert_eq!(
            colors.lock().unwrap().last(),
            Some(&(look.color(ShadcnToken::Foreground), look.color(ShadcnToken::Background)))
        );
        look.set_mode(ThemeMode::Dark);
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        assert_eq!(
            colors.lock().unwrap().last(),
            Some(&(look.color(ShadcnToken::Foreground), look.color(ShadcnToken::Background)))
        );
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_tooltip_stylesheet() -> xgpui_luma::theme::stylesheet::TooltipStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    TooltipStylesheet {
        geometry: TooltipGeometryStylesheet {
            padding: Some(8.0),
            padding_y: Some(4.0),
            font_size: Some(12.0),
            line_height: Some(16.0),
            ..Default::default()
        },
        ..Default::default()
    }
}
