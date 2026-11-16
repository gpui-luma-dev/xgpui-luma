//! Control Workshop. Catalog and complete sample compositions are copied from Luma Studio;
//! inspector and documentation hosts intentionally stay out of this application.
pub(crate) mod catalog;
mod button;
mod choice;
mod collection;
mod input;
mod menu;
mod layout;
mod navigation;
mod feedback;
mod selection;
mod shadow_button;
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
mod sample_layout;
mod sdk_sample;
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
use crate::theme::{ColorVizLookExt, Look};
use catalog::COMPONENT_CATALOG;

// Keep routing independent of sample implementation, with typed retained entities.
enum Sample {
    ShadowButton(Entity<shadow_button::ShadowButtonSample>),
    Selection(Entity<selection::SelectionSample>),
    Feedback(Entity<feedback::FeedbackSample>),
    Navigation(Entity<navigation::NavigationSample>),
    Layout(Entity<layout::LayoutSample>),
    Menu(Entity<menu::MenuSample>),
    Input(Entity<input::InputSample>),
    Collection(Entity<collection::CollectionSample>),
    Choice(Entity<choice::ChoiceSample>),
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
            Self::ShadowButton(v) => v.clone().into(),
            Self::Selection(v) => v.clone().into(),
            Self::Feedback(v) => v.clone().into(),
            Self::Navigation(v) => v.clone().into(),
            Self::Layout(v) => v.clone().into(),
            Self::Menu(v) => v.clone().into(),
            Self::Input(v) => v.clone().into(),
            Self::Collection(v) => v.clone().into(),
            Self::Choice(v) => v.clone().into(),
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

    fn sync_look(&self, look: Arc<Look>, cx: &mut gpui::App) {
        match self {
            Self::ShadowButton(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Selection(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Feedback(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Navigation(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Layout(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Menu(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Input(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Collection(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
            Self::Choice(v) => v.update(cx, |v, cx| v.sync_look(look, cx)),
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
    pub(crate) color_sidebar: Vec<Entity<Button>>,
    selected: &'static str,
    navigation: Vec<NavigationEntry>,
    samples: HashMap<&'static str, Sample>,
    shell_look: Arc<Look>,
    page_look: Arc<Look>,
}

impl Workshop {
    pub(crate) fn new(shell_look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let mut navigation = Vec::new();
        let mut groups = Vec::new();
        let mut color_sidebar = Vec::new();
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
            if group.id == "color" {
                color_sidebar = rows;
                continue;
            }
            groups.push(AccordionItem::new(
                group.id,
                AccordionTrigger::new(group.label),
                AccordionContent::custom(move |_, _| {
                    div().w_full().flex().flex_col().pl(px(20.)).gap_1().children(rows.clone()).into_any_element()
                }),
            ));
        }
        let sidebar = xgpui_luma_look_radix::accordion_builder(&shell_look, "control-workshop-catalog")
            .size(xgpui_luma::theme::ControlSize::Sm)
            .template(crate::app::nav_accordion::template_with_labels(
                &shell_look,
                COMPONENT_CATALOG.iter().filter(|g| g.id != "color").map(|g| (g.id, g.label)).collect(),
            ))
            .multiple()
            .collapsible(true)
            .item_dividers(false)
            .content_padding_y(4.)
            .items(groups)
            .spawn(cx);
        let page_look = crate::theme::default_look();
        page_look.set_mode(ThemeMode::Light);
        let mut this = Self {
            sidebar,
            color_sidebar,
            selected: "color-field",
            navigation,
            samples: HashMap::new(),
            shell_look,
            page_look,
        };
        this.select("color-field", cx);
        this
    }

    fn select(&mut self, id: &'static str, cx: &mut Context<Self>) {
        if !COMPONENT_CATALOG.iter().any(|g| g.entries.iter().any(|e| e.id == id)) {
            return;
        }
        self.selected = id;
        if !self.samples.contains_key(id) {
            let look = self.page_look.clone();
            let sample = sdk_sample::with_sdk_defaults(|| match id {
                "shadow-button" => {
                    Some(Sample::ShadowButton(cx.new(|cx| shadow_button::ShadowButtonSample::new(look, cx))))
                }
                "split-button" => Some(Sample::Selection(
                    cx.new(|cx| selection::SelectionSample::new(selection::SelectionKind::SplitButton, look, cx)),
                )),
                "selection-panel" => Some(Sample::Selection(
                    cx.new(|cx| selection::SelectionSample::new(selection::SelectionKind::Panel, look, cx)),
                )),
                "popup-selector" => Some(Sample::Selection(
                    cx.new(|cx| selection::SelectionSample::new(selection::SelectionKind::Selector, look, cx)),
                )),
                "search-selector" => Some(Sample::Selection(
                    cx.new(|cx| selection::SelectionSample::new(selection::SelectionKind::SearchSelector, look, cx)),
                )),
                "combobox" => Some(Sample::Selection(
                    cx.new(|cx| selection::SelectionSample::new(selection::SelectionKind::ComboBox, look, cx)),
                )),
                "autocomplete-textfield" => Some(Sample::Selection(
                    cx.new(|cx| selection::SelectionSample::new(selection::SelectionKind::Autocomplete, look, cx)),
                )),
                "dialog-draggable" => Some(Sample::Feedback(
                    cx.new(|cx| feedback::FeedbackSample::new(feedback::FeedbackKind::Draggable, look, cx)),
                )),
                "dialog-positioning" => Some(Sample::Feedback(
                    cx.new(|cx| feedback::FeedbackSample::new(feedback::FeedbackKind::Positioning, look, cx)),
                )),
                "dialog-modeless" => Some(Sample::Feedback(
                    cx.new(|cx| feedback::FeedbackSample::new(feedback::FeedbackKind::Modeless, look, cx)),
                )),
                "dialog-modal" => Some(Sample::Feedback(
                    cx.new(|cx| feedback::FeedbackSample::new(feedback::FeedbackKind::Modal, look, cx)),
                )),
                "stepper" => Some(Sample::Feedback(
                    cx.new(|cx| feedback::FeedbackSample::new(feedback::FeedbackKind::Stepper, look, cx)),
                )),
                "progress" => Some(Sample::Feedback(
                    cx.new(|cx| feedback::FeedbackSample::new(feedback::FeedbackKind::Progress, look, cx)),
                )),
                "sidebar" => Some(Sample::Navigation(
                    cx.new(|cx| navigation::NavigationSample::new(navigation::NavigationKind::Sidebar, look, cx)),
                )),
                "tabs-navigation" => Some(Sample::Navigation(
                    cx.new(|cx| navigation::NavigationSample::new(navigation::NavigationKind::Tabs, look, cx)),
                )),
                "dock-panel" => {
                    Some(Sample::Layout(cx.new(|cx| layout::LayoutSample::new(layout::LayoutKind::Dock, look, cx))))
                }
                "resizable-panels" => Some(Sample::Layout(
                    cx.new(|cx| layout::LayoutSample::new(layout::LayoutKind::Resizable, look, cx)),
                )),
                "split-view-unified" => {
                    Some(Sample::Layout(cx.new(|cx| layout::LayoutSample::new(layout::LayoutKind::Split, look, cx))))
                }
                "context-menu" => {
                    Some(Sample::Menu(cx.new(|cx| menu::MenuSample::new(menu::MenuKind::Context, look, cx))))
                }
                "popup-menu" => Some(Sample::Menu(cx.new(|cx| menu::MenuSample::new(menu::MenuKind::Popup, look, cx)))),
                "textfield" => {
                    Some(Sample::Input(cx.new(|cx| input::InputSample::new(input::InputKind::TextField, look, cx))))
                }
                "textarea" => {
                    Some(Sample::Input(cx.new(|cx| input::InputSample::new(input::InputKind::TextArea, look, cx))))
                }
                "slider" => {
                    Some(Sample::Input(cx.new(|cx| input::InputSample::new(input::InputKind::Slider, look, cx))))
                }
                "scrollbar" => {
                    Some(Sample::Input(cx.new(|cx| input::InputSample::new(input::InputKind::Scrollbar, look, cx))))
                }
                "paging-table" => {
                    Some(Sample::Collection(cx.new(|cx| {
                        collection::CollectionSample::new(collection::CollectionKind::PagingTable, look, cx)
                    })))
                }
                "scrolling-table" => Some(Sample::Collection(cx.new(|cx| {
                    collection::CollectionSample::new(collection::CollectionKind::ScrollingTable, look, cx)
                }))),
                "tree-view" => Some(Sample::Collection(
                    cx.new(|cx| collection::CollectionSample::new(collection::CollectionKind::TreeView, look, cx)),
                )),
                "accordion" => Some(Sample::Choice(
                    cx.new(|cx| choice::ChoiceSample::new(choice::ChoiceKind::Accordion, look, cx)),
                )),
                "checkbox" => {
                    Some(Sample::Choice(cx.new(|cx| choice::ChoiceSample::new(choice::ChoiceKind::Checkbox, look, cx))))
                }
                "pager" => {
                    Some(Sample::Choice(cx.new(|cx| choice::ChoiceSample::new(choice::ChoiceKind::Pager, look, cx))))
                }
                "radio-button" => Some(Sample::Choice(
                    cx.new(|cx| choice::ChoiceSample::new(choice::ChoiceKind::RadioButton, look, cx)),
                )),
                "radio-group" => Some(Sample::Choice(
                    cx.new(|cx| choice::ChoiceSample::new(choice::ChoiceKind::RadioGroup, look, cx)),
                )),
                "switch" => {
                    Some(Sample::Choice(cx.new(|cx| choice::ChoiceSample::new(choice::ChoiceKind::Switch, look, cx))))
                }
                "toggle" => {
                    Some(Sample::Choice(cx.new(|cx| choice::ChoiceSample::new(choice::ChoiceKind::Toggle, look, cx))))
                }
                "button" => {
                    Some(Sample::Button(cx.new(|cx| button::ButtonControlExposition::new(cx, self.shell_look.clone()))))
                }
                "custom-button" => Some(Sample::CustomButton(
                    cx.new(|cx| custom_button::CustomButtonControlExposition::new(cx, self.shell_look.clone())),
                )),
                "toolbar" => Some(Sample::Toolbar(
                    cx.new(|cx| toolbar::ToolbarControlExposition::new(cx, self.shell_look.clone())),
                )),
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
            });
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
        self.shell_look.set_mode(mode);
        for sample in self.samples.values() {
            sample.sync_look(self.page_look.clone(), cx);
        }
        cx.notify();
    }
}

impl Render for Workshop {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.page_look.chrome();
        let unavailable_reason = match self.selected {
            "badge" => Some(
                "The base SDK does not provide a Badge control. Badge elements belong to the Radix and Shadcn look crates, so no SDK-default sample is available.",
            ),
            "slide-panel" => Some(
                "The SDK provides SlidePanelState and overlay rendering helpers, but no standalone Slide Panel control. A default control sample is unavailable.",
            ),
            "split-view-inset" => Some(
                "The SDK provides one SplitView control with no Inset variant. Its default template is demonstrated on the Split View: Unified screen.",
            ),
            "floating-menu" => Some(
                "The SDK provides Floating Menu rendering and navigation for Context Menu and Popup Menu, but no standalone Floating Menu control.",
            ),
            "toggle-group" => Some("The SDK does not provide a dedicated Toggle Group control."),
            "listbox" => Some(
                "The SDK provides ListBox state and behavior, but no default visual template. A native default sample is unavailable.",
            ),
            _ => None,
        };
        let mut root = div()
            .id("control-workshop-content")
            .relative()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(gpui::rgb(0xffffff))
            .text_color(chrome.body_text)
            .child(sample_layout::ruled_background());
        if let Some(sample) = self.samples.get(self.selected) {
            root = root.child(sdk_sample::isolated(sample.view()));
        } else if let Some((group, entry)) = COMPONENT_CATALOG
            .iter()
            .find_map(|g| g.entries.iter().find(|e| e.id == self.selected).map(|e| (g, e)))
        {
            root = root.child(
                div()
                    .text_color(gpui::rgb(0x202020))
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(div().text_size(px(22.)).child(entry.label))
                    .child(div().text_color(gpui::rgb(0x666666)).child(group.label))
                    .child(if let Some(reason) = unavailable_reason {
                        div().debug_selector(|| "workshop-sdk-unavailable".into()).child(reason)
                    } else {
                        div().child("Sample coming soon")
                    }),
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
            assert_eq!(workshop.page_look.mode(), ThemeMode::Light);
            assert!(!workshop.sidebar.read(cx).is_expanded(&"color".into()));
            assert_eq!(workshop.color_sidebar.len(), 6);
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
                    assert_eq!(
                        workshop.samples.contains_key(entry.id),
                        matches!(
                            group.id,
                            "color"
                                | "command"
                                | "choice"
                                | "collection"
                                | "input"
                                | "menu"
                                | "layout"
                                | "navigation"
                                | "feedback"
                                | "selection"
                                | "prototypes"
                        ) && !matches!(
                            entry.id,
                            "toggle-group" | "listbox" | "floating-menu" | "slide-panel" | "split-view-inset" | "badge"
                        )
                    );
                    assert_eq!(workshop.navigation.iter().filter(|n| n.selected.load(Ordering::Relaxed)).count(), 1);
                });
            }
        }
        view.update(cx, |v, cx| {
            v.select("color-field", cx);
            v.sync_theme(ThemeMode::Dark, cx);
            assert_eq!(v.page_look.mode(), ThemeMode::Light);
            v.sync_theme(ThemeMode::Light, cx);
        });
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).page_look.mode(), ThemeMode::Light);
            assert_eq!(view.read(cx).samples.len(), 43);
            assert_eq!(view.read(cx).samples["color-field"].view().entity_id(), field);
            assert_eq!(view.read(cx).shell_look.mode(), ThemeMode::Light);
        });
    }

    #[test]
    fn missing_sdk_toggle_group_shows_message_without_a_substitute_control() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| Workshop::new(crate::theme::default_look(), cx));
        view.update(cx, |v, cx| v.select("toggle-group", cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-sdk-unavailable").is_some());
        cx.update(|_, cx| {
            let workshop = view.read(cx);
            assert_eq!(workshop.selected, "toggle-group");
            assert!(!workshop.samples.contains_key("toggle-group"));
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
                .child(
                    div()
                        .w(px(260.))
                        .child(self.workshop.read(cx).sidebar.clone())
                        .child(div().flex().flex_col().children(self.workshop.read(cx).color_sidebar.clone())),
                )
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
        let folder = cx.debug_bounds("nav-folder-Command-closed").unwrap();
        let folder_label = cx.debug_bounds("nav-label-Command").unwrap();
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
