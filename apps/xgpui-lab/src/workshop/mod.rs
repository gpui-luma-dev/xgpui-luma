//! Control Workshop. Catalog and complete sample compositions are copied from Luma Studio;
//! inspector and documentation hosts intentionally stay out of this application.
pub(crate) mod catalog;
mod button;
mod custom_button;
mod toolbar;
mod color_arc;
mod color_field;
mod color_ring;
mod color_slider;
mod color_slider_revealed;
mod color_multi_mixer;
mod color_exposition_common;
mod event_stream;
mod event_log_view;

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{
    accordion::{Accordion, AccordionItem, AccordionTrigger, AccordionContent},
    button::Button,
};
use xgpui_luma::theme::ThemeMode;
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma_look_shadcn::ShadcnLook;
use crate::theme::{ColorVizLookExt, Look};
use catalog::COMPONENT_CATALOG;

// Keep routing independent of sample implementation, with typed retained entities.
enum Sample {
    Button(Entity<button::ButtonControlExposition>),
    CustomButton(Entity<custom_button::CustomButtonControlExposition>),
    Toolbar(Entity<toolbar::ToolbarControlExposition>),
    Arc(Entity<color_arc::ColorArcExpositionLeftPane>),
    Field(Entity<color_field::ColorFieldExpositionLeftPane>),
    Ring(Entity<color_ring::ColorRingExpositionLeftPane>),
    Slider(Entity<color_slider::ColorSliderExpositionLeftPane>),
    Revealed(Entity<color_slider_revealed::ColorSliderRevealedPane>),
    Mixer(Entity<color_multi_mixer::ColorMultiMixerControlExposition>),
}

impl Sample {
    fn view(&self) -> gpui::AnyView {
        match self {
            Self::Button(v) => v.clone().into(),
            Self::CustomButton(v) => v.clone().into(),
            Self::Toolbar(v) => v.clone().into(),
            Self::Arc(v) => v.clone().into(),
            Self::Field(v) => v.clone().into(),
            Self::Ring(v) => v.clone().into(),
            Self::Slider(v) => v.clone().into(),
            Self::Revealed(v) => v.clone().into(),
            Self::Mixer(v) => v.clone().into(),
        }
    }

    fn sync_look(&self, look: Arc<ShadcnLook>, cx: &mut gpui::App) {
        match self {
            Self::Button(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::CustomButton(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Toolbar(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Arc(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Field(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Ring(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Slider(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Revealed(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Mixer(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
        }
    }
}

struct NavigationEntry {
    id: &'static str,
    selected: Arc<AtomicBool>,
    button: Entity<Button>,
}

pub(crate) struct Workshop {
    pub(crate) sidebar: Accordion,
    selected: &'static str,
    navigation: Vec<NavigationEntry>,
    samples: HashMap<&'static str, Sample>,
    look: Arc<ShadcnLook>,
    shell_look: Arc<Look>,
}

impl Workshop {
    pub(crate) fn new(shell_look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let look =
            Arc::new(ShadcnLook::from_css_str(xgpui_luma_look_shadcn::FALLBACK_CSS).expect("bundled Workshop theme"));
        look.set_mode(shell_look.mode());
        let mut navigation = Vec::new();
        let mut groups = Vec::new();
        for group in COMPONENT_CATALOG {
            let mut rows = Vec::new();
            for entry in group.entries {
                let id = entry.id;
                let label = entry.label;
                let selected = Arc::new(AtomicBool::new(id == "color-field"));
                let paint_selected = selected.clone();
                let row_look = shell_look.clone();
                let label_look = shell_look.clone();
                let label_selected = selected.clone();
                let button = xgpui_luma_look_radix::Button::new(format!("workshop-{id}"))
                    .look(shell_look.as_ref())
                    .ghost()
                    .gray()
                    .size(xgpui_luma_look_radix::ButtonSize::One)
                    .content(move |_, _| {
                        div()
                            .debug_selector(move || format!("workshop-label-{id}"))
                            .text_color(
                                label_look
                                    .resolve_step(
                                        xgpui_luma_look_radix::ScaleFamily::Gray,
                                        if label_selected.load(Ordering::Relaxed) { 12 } else { 11 },
                                    )
                                    .hsla(),
                            )
                            .child(label)
                    })
                    .with_template_modifier(|row, _| row.w_full())
                    .on_click(cx, move |this: &mut Self, cx| this.select(id, cx))
                    .spawn(cx);
                // Like Explorer, modify the SDK's inner button surface so content
                // alignment and selected paint apply to the actual interactive row.
                let template = xgpui_luma::controls::button::DefaultButtonTemplate::new(
                    xgpui_luma_look_radix::button_family_theme_with(
                        &shell_look,
                        xgpui_luma_look_radix::ButtonVariant::Ghost,
                        xgpui_luma_look_radix::Paint::gray(),
                    ),
                )
                .with_modifier(move |row, model| {
                    row.debug_selector(move || format!("workshop-{id}"))
                        .w_full()
                        .justify_start()
                        .text_align(gpui::TextAlign::Left)
                        .when_some(
                            row_look
                                .interaction_colors()
                                .background(paint_selected.load(Ordering::Relaxed), model.state),
                            |row, color| row.bg(color),
                        )
                });
                button.update(cx, |button, cx| button.set_template(Arc::new(template), cx));
                rows.push(button.clone());
                navigation.push(NavigationEntry { id, selected, button });
            }
            groups.push(
                AccordionItem::new(
                    group.id,
                    AccordionTrigger::new(group.label),
                    AccordionContent::custom(move |_, _| {
                        div().w_full().flex().flex_col().pl(px(20.)).gap_1().children(rows.clone()).into_any_element()
                    }),
                )
                .expanded(group.id == "color"),
            );
        }
        let sidebar = xgpui_luma_look_radix::accordion_builder(&shell_look, "control-workshop-catalog")
            .size(xgpui_luma::theme::ControlSize::Sm)
            .template(crate::app::nav_accordion::template_with_labels(
                &shell_look,
                COMPONENT_CATALOG.iter().map(|g| (g.id, g.label)).collect(),
            ))
            .multiple()
            .collapsible(true)
            .item_dividers(false)
            .content_padding_y(4.)
            .items(groups)
            .spawn(cx);
        let mut this = Self { sidebar, selected: "color-field", navigation, samples: HashMap::new(), look, shell_look };
        this.select("color-field", cx);
        this
    }

    fn select(&mut self, id: &'static str, cx: &mut Context<Self>) {
        if !COMPONENT_CATALOG.iter().any(|g| g.entries.iter().any(|e| e.id == id)) {
            return;
        }
        self.selected = id;
        if !self.samples.contains_key(id) {
            let look = self.look.clone();
            let sample = match id {
                "button" => Some(Sample::Button(cx.new(|cx| button::ButtonControlExposition::new(cx, look)))),
                "custom-button" => {
                    Some(Sample::CustomButton(cx.new(|cx| custom_button::CustomButtonControlExposition::new(cx, look))))
                }
                "toolbar" => Some(Sample::Toolbar(cx.new(|cx| toolbar::ToolbarControlExposition::new(cx, look)))),
                "color-arc" => Some(Sample::Arc(cx.new(|cx| color_arc::ColorArcExpositionLeftPane::new(cx, look)))),
                "color-field" => {
                    Some(Sample::Field(cx.new(|cx| color_field::ColorFieldExpositionLeftPane::new(cx, look))))
                }
                "color-ring" => Some(Sample::Ring(cx.new(|cx| color_ring::ColorRingExpositionLeftPane::new(cx, look)))),
                "color-slider" => {
                    Some(Sample::Slider(cx.new(|cx| color_slider::ColorSliderExpositionLeftPane::new(cx, look))))
                }
                "color-slider-revealed" => {
                    Some(Sample::Revealed(cx.new(|cx| color_slider_revealed::ColorSliderRevealedPane::new(cx, look))))
                }
                "color-multi-mixer" => {
                    Some(Sample::Mixer(cx.new(|cx| color_multi_mixer::ColorMultiMixerControlExposition::new(cx, look))))
                }
                _ => None,
            };
            if let Some(sample) = sample {
                self.samples.insert(id, sample);
            }
        }
        for entry in &self.navigation {
            entry.selected.store(entry.id == id, Ordering::Relaxed);
            entry.button.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }

    pub(crate) fn sync_theme(&mut self, mode: ThemeMode, cx: &mut Context<Self>) {
        self.look.set_mode(mode);
        for sample in self.samples.values() {
            sample.sync_look(self.look.clone(), cx);
        }
        cx.notify();
    }
}

impl Render for Workshop {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.shell_look.chrome();
        let mut root =
            div().id("control-workshop-content").size_full().min_w_0().min_h_0().text_color(chrome.body_text);
        if let Some(sample) = self.samples.get(self.selected) {
            root = root.child(sample.view());
        } else if let Some((group, entry)) = COMPONENT_CATALOG
            .iter()
            .find_map(|g| g.entries.iter().find(|e| e.id == self.selected).map(|e| (g, e)))
        {
            root = root.child(
                div()
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(div().text_size(px(22.)).child(entry.label))
                    .child(div().text_color(chrome.muted_text).child(group.label))
                    .child("Sample coming soon"),
            );
        }
        root
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext};
    use xgpui_luma::controls::button::ButtonEvent;

    #[test]
    fn workshop_catalog_matches_studio_and_has_unique_routes() {
        let studio = include_str!("../../../luma-studio/src/studio/components/catalog.rs");
        let expected: Vec<_> = studio
            .lines()
            .filter_map(|line| {
                let (_, entry) = line.split_once("ComponentCatalogEntry { id: \"")?;
                let (id, rest) = entry.split_once('"')?;
                let (_, rest) = rest.split_once("label: \"")?;
                Some((id, rest.split('"').next().unwrap()))
            })
            .collect();
        let actual: Vec<_> = COMPONENT_CATALOG.iter().flat_map(|g| g.entries.iter().map(|e| (e.id, e.label))).collect();
        assert_eq!(actual, expected);
        assert_eq!(
            COMPONENT_CATALOG.iter().map(|g| g.label).collect::<Vec<_>>(),
            [
                "Command",
                "Choice",
                "Collection",
                "Input",
                "Color",
                "Menu",
                "Layout",
                "Navigation",
                "Feedback",
                "Selection",
                "Prototypes"
            ]
        );
        assert_eq!(actual.len(), 49);
        assert_eq!(
            COMPONENT_CATALOG.iter().map(|g| g.entries.len()).collect::<Vec<_>>(),
            [3, 8, 4, 4, 6, 3, 5, 2, 7, 6, 1]
        );
        assert_eq!(actual.iter().map(|(id, _)| id).collect::<std::collections::HashSet<_>>().len(), 49);
        assert_eq!(
            COMPONENT_CATALOG.iter().map(|g| g.id).collect::<Vec<_>>(),
            [
                "command",
                "choice",
                "collection",
                "input",
                "color",
                "menu",
                "layout",
                "navigation",
                "feedback",
                "selection",
                "prototypes"
            ]
        );
    }

    #[test]
    fn workshop_every_route_is_intentional_and_samples_are_retained() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (view, cx) = app.add_window_view(|_, cx| Workshop::new(crate::theme::default_look(), cx));
        cx.update(|_, cx| {
            let workshop = view.read(cx);
            assert_eq!(workshop.selected, "color-field");
            assert!(workshop.sidebar.read(cx).is_expanded(&"color".into()));
            assert!(!workshop.sidebar.read(cx).is_expanded(&"command".into()));
        });
        let field = cx.update(|_, cx| view.read(cx).samples["color-field"].view().entity_id());
        for group in COMPONENT_CATALOG {
            for entry in group.entries {
                let button = cx
                    .update(|_, cx| view.read(cx).navigation.iter().find(|n| n.id == entry.id).unwrap().button.clone());
                button.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
                cx.run_until_parked();
                cx.update(|_, cx| {
                    let workshop = view.read(cx);
                    assert_eq!(workshop.selected, entry.id);
                    assert_eq!(workshop.samples.contains_key(entry.id), matches!(group.id, "color" | "command"));
                    assert_eq!(workshop.navigation.iter().filter(|n| n.selected.load(Ordering::Relaxed)).count(), 1);
                });
            }
        }
        view.update(cx, |v, cx| {
            v.select("color-field", cx);
            v.sync_theme(ThemeMode::Light, cx);
        });
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).samples.len(), 9);
            assert_eq!(view.read(cx).samples["color-field"].view().entity_id(), field);
            assert_eq!(view.read(cx).look.mode(), ThemeMode::Light);
        });
    }

    struct NavigationHost {
        workshop: Entity<Workshop>,
    }
    impl Render for NavigationHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .child(div().w(px(260.)).child(self.workshop.read(cx).sidebar.clone()))
                .child(div().flex_1().child(self.workshop.clone()))
        }
    }

    #[test]
    fn workshop_group_links_support_keyboard_activation() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (host, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            NavigationHost { workshop: cx.new(|cx| Workshop::new(crate::theme::default_look(), cx)) }
        });
        cx.update(|_, cx| xgpui_luma::key_handling::bind_default_control_keys(cx));
        cx.run_until_parked();
        let folder = cx.debug_bounds("nav-folder-Color-open").unwrap();
        let folder_label = cx.debug_bounds("nav-label-Color").unwrap();
        assert!((folder_label.left() - folder.right() - px(6.)).abs() < px(1.));
        let field_label = cx.debug_bounds("workshop-label-color-field").unwrap();
        for (label_id, row_id) in [
            ("workshop-label-color-arc", "workshop-color-arc"),
            ("workshop-label-color-ring", "workshop-color-ring"),
            ("workshop-label-color-slider", "workshop-color-slider"),
            ("workshop-label-color-slider-revealed", "workshop-color-slider-revealed"),
            ("workshop-label-color-multi-mixer", "workshop-color-multi-mixer"),
        ] {
            let label = cx.debug_bounds(label_id).unwrap();
            let row = cx.debug_bounds(row_id).unwrap();
            assert!((label.left() - field_label.left()).abs() < px(1.));
            assert!(label.left() - row.left() < px(16.), "control label must sit at row start");
        }
        let view = cx.update(|_, cx| host.read(cx).workshop.clone());
        let button =
            cx.update(|_, cx| view.read(cx).navigation.iter().find(|n| n.id == "color-slider").unwrap().button.clone());
        cx.update(|window, cx| button.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-color-slider").is_some());
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(view.read(cx).selected, "color-slider"));
    }
}
