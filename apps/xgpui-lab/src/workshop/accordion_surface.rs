//! The SDK accordion on the design surface page (issue #383). Its `.lss` (`accordion`) styles the trigger
//! rows: hover, pressed and disabled. Design mode pins the picked states on every trigger row; which rows
//! are expanded stays live.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window, div, prelude::*};
use xgpui_luma::controls::accordion::{
    self, Accordion, AccordionContent, AccordionContentPalette, AccordionEvent, AccordionItem,
    AccordionItemRenderModel, AccordionItemState, AccordionPalette, AccordionRenderModel, AccordionScale,
    AccordionSelectionMode, AccordionTemplate, AccordionTemplateHandlers, AccordionTheme, AccordionTrigger,
    ControlFocusState, StyledAccordion, ThemedAccordionTemplate,
};
use xgpui_luma::infra::icon::DisclosureIcons;
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::theme::{InteractionState, MetricTokens};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The sections: id, trigger and content, with ascenders and descenders.
const SECTIONS: [(&str, &str, &str); 2] = [
    ("shipping", "Shipping policy", "Packages ship quickly, typically by the next day."),
    ("quality", "Quality guarantee", "Faulty goods qualify for a prompt replacement."),
];

/// The `.lss` states the trigger rows read, and `focused` for the keyboard focus affordance.
const STATES: [&str; 4] = ["hover", "pressed", "focused", "disabled"];

pub(super) struct AccordionSurface {
    accordion: Accordion,
}

impl AccordionSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let items = SECTIONS.iter().enumerate().map(|(index, (id, trigger, content))| {
            let content = *content;
            AccordionItem::new(
                *id,
                AccordionTrigger::new(*trigger),
                AccordionContent::custom(move |_, _| div().child(content).into_any_element()),
            )
            .expanded(index == 0)
        });
        let accordion = accordion::new("workshop-sdk-accordion").items(items).collapsible(true).spawn(cx);
        Box::new(Self { accordion })
    }

    /// The SDK accordion, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn accordion(&self) -> Accordion {
        self.accordion.clone()
    }
}

/// The accordion from a theme's `accordion` control.
struct LssAccordionTheme(StyledAccordion);

impl AccordionTheme for LssAccordionTheme {
    fn resolve_trigger(&self, state: InteractionState) -> AccordionPalette {
        self.0.trigger(state)
    }

    fn resolve_content(&self, _expanded: bool) -> AccordionContentPalette {
        self.0.content()
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }

    fn resolve_scale(&self, scale_factor: f32) -> AccordionScale {
        self.0.scale(scale_factor)
    }
}

/// The accordion template for the `accordion` control in `theme`.
fn template(theme: Theme) -> ThemedAccordionTemplate {
    let styled = StyledAccordion::new(StyleBinding::new("accordion", ThemeSource::Fixed(theme)));
    ThemedAccordionTemplate::new(Arc::new(LssAccordionTheme(styled)))
}

/// A trigger row's state for the picked states, keeping whether the row is current.
fn pinned_item_state(live: AccordionItemState, states: &StateSet) -> AccordionItemState {
    let focused = states.has("focused");
    AccordionItemState {
        disabled: states.has("disabled"),
        hovered: states.has("hover"),
        pressed: states.has("pressed"),
        active: focused,
        focus_visible: focused,
        ..live
    }
}

/// `model` with every trigger row in the picked states.
fn pinned_model<'a>(model: &AccordionRenderModel<'a>, states: &StateSet) -> AccordionRenderModel<'a> {
    let enabled = !states.has("disabled");
    let focused = states.has("focused");
    AccordionRenderModel {
        id: model.id,
        items: model
            .items
            .iter()
            .map(|item| AccordionItemRenderModel {
                id: item.id,
                trigger: item.trigger,
                content: item.content,
                expanded: item.expanded,
                progress: item.progress,
                content_height_px: item.content_height_px,
                enabled,
                state: pinned_item_state(item.state, states),
            })
            .collect(),
        selection_mode: model.selection_mode,
        collapsible: model.collapsible,
        enabled,
        item_dividers: model.item_dividers,
        content_padding_y: model.content_padding_y,
        content_padding_top: model.content_padding_top,
        content_padding_bottom: model.content_padding_bottom,
        trigger_min_height: model.trigger_min_height,
        trigger_padding_y: model.trigger_padding_y,
        disclosure_icons: model.disclosure_icons,
        focus: ControlFocusState { focused, focus_visible: focused },
    }
}

/// Draws the base template with the picked states (Design mode), or as the accordion is.
struct PinnedAccordionTemplate {
    base: ThemedAccordionTemplate,
    pinned: Option<StateSet>,
}

impl AccordionTemplate for PinnedAccordionTemplate {
    fn drag_tooltip_theme(&self) -> Arc<dyn xgpui_luma::controls::tooltip::TooltipTheme> {
        self.base.drag_tooltip_theme()
    }

    fn resolve_drag(&self) -> xgpui_luma::infra::drag_drop::DragDropLook {
        self.base.resolve_drag()
    }

    fn render(
        &self,
        model: &AccordionRenderModel<'_>,
        handlers: AccordionTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        match &self.pinned {
            Some(states) => self.base.render(&pinned_model(model, states), handlers, window, cx),
            None => self.base.render(model, handlers, window, cx),
        }
    }
}

/// Handlers that do nothing, one per row, for a State matrix cell.
fn inert_handlers(rows: usize) -> AccordionTemplateHandlers {
    AccordionTemplateHandlers {
        drag: None,
        trigger_hovers: (0..rows).map(|_| Box::new(|_: &bool, _: &mut Window, _: &mut App| {}) as _).collect(),
        trigger_mouse_downs: (0..rows).map(|_| Box::new(|_: &_, _: &mut Window, _: &mut App| {}) as _).collect(),
        trigger_mouse_ups: (0..rows).map(|_| Box::new(|_: &_, _: &mut Window, _: &mut App| {}) as _).collect(),
        trigger_clicks: (0..rows).map(|_| Box::new(|_: &_, _: &mut Window, _: &mut App| {}) as _).collect(),
        content_height_reports: (0..rows).map(|_| Box::new(|_: f32, _: &mut Window, _: &mut App| {}) as _).collect(),
    }
}

impl SurfaceControl for AccordionSurface {
    fn title(&self) -> &'static str {
        "Accordion"
    }

    fn view(&self) -> AnyView {
        self.accordion.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let template = PinnedAccordionTemplate { base: template(theme), pinned: pinned.cloned() };
        self.accordion.update(cx, |accordion, cx| accordion.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.accordion.update(cx, |accordion, cx| accordion.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        STATES.to_vec()
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.accordion, move |side: &mut ControlSide, _, event: &AccordionEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, AccordionEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                AccordionEvent::ItemHoverChanged { hovered, .. } => Some(("hover", *hovered)),
                AccordionEvent::FocusChanged { focused } => Some(("focused", *focused)),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    /// Both sections, collapsed, their triggers in `states`.
    fn cell(
        &self,
        theme: &Theme,
        id: SharedString,
        states: &StateSet,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let sections: Vec<(SharedString, AccordionTrigger, AccordionContent)> = SECTIONS
            .iter()
            .map(|(id, trigger, _)| {
                (
                    SharedString::from(*id),
                    AccordionTrigger::new(*trigger),
                    AccordionContent::custom(|_, _| div().into_any_element()),
                )
            })
            .collect();
        let icons = DisclosureIcons::default();
        let model = AccordionRenderModel {
            id: &id,
            items: sections
                .iter()
                .map(|(id, trigger, content)| AccordionItemRenderModel {
                    id,
                    trigger,
                    content,
                    expanded: false,
                    progress: 0.,
                    content_height_px: 0.,
                    enabled: true,
                    state: AccordionItemState::default(),
                })
                .collect(),
            selection_mode: AccordionSelectionMode::default(),
            collapsible: true,
            enabled: true,
            item_dividers: true,
            content_padding_y: None,
            content_padding_top: None,
            content_padding_bottom: None,
            trigger_min_height: None,
            trigger_padding_y: None,
            disclosure_icons: &icons,
            focus: ControlFocusState::default(),
        };
        let model = pinned_model(&model, states);
        div()
            .w(gpui::px(220.))
            .child(template(theme.clone()).render(&model, inert_handlers(sections.len()), window, cx))
            .into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{Focusable, TestAppContext};

    use super::*;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_accordion_page_expands_and_pins_its_trigger_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "accordion");
        let side = cx.update(|_, cx| page.read(cx).side());
        let accordion =
            cx.update(|_, cx| side.read(cx).control_as::<AccordionSurface>().expect("an accordion").accordion());

        cx.update(|window, cx| accordion.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("end enter");
        cx.run_until_parked();
        cx.update(|_, cx| assert!(accordion.read(cx).is_expanded(&"quality".into())));
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("ExpandedChanged"), "expansion is logged: {text}");

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("hover", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("hover"), "{pinned:?}");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw accordions");
    }
}
