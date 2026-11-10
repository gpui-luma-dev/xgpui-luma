//! Declarative lab screen layout. A screen fills slots; this module decides where
//! they go and how they look. See `docs/lab-screen-api.md`.
//!
//! Slot order is fixed by [`LabScreen`], not by call order:
//! crumb, title, lede, hero, caption, controls, notices, cards, panels, note, credits.
//! Empty slots render nothing. Each slot carries the debug selector `"{id}-{slot}"`.
use std::sync::Arc;

use gpui::{
    AnyElement, App, Div, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window, div,
};
use xgpui_luma::hstack;
use xgpui_luma_look_radix::Look;

use crate::theme::{ColorVizLookExt, TypographyExt};

mod card;
pub(crate) mod data;
mod hero;
mod metrics;
mod panel;

pub(crate) use card::LabCard;
pub(crate) use hero::{Hero, Legend, Mark};
pub(crate) use metrics::{LabMetrics, MONO_FAMILY, TextRole, readout, text, value_field_look};
pub(crate) use panel::{Panel, Param, ParametersPanel, PreviewPanel};

#[derive(IntoElement)]
pub(crate) struct LabScreen {
    id: &'static str,
    look: Arc<Look>,
    crumb: Option<SharedString>,
    title: Option<SharedString>,
    lede: Option<SharedString>,
    hero: Option<Hero>,
    caption: Option<SharedString>,
    controls: Vec<AnyElement>,
    notices: Vec<SharedString>,
    cards: Vec<LabCard>,
    panels: Option<(Panel, Panel)>,
    note: Option<SharedString>,
    credits: Option<SharedString>,
}

impl LabScreen {
    pub fn new(id: &'static str, look: Arc<Look>) -> Self {
        Self {
            id,
            look,
            crumb: None,
            title: None,
            lede: None,
            hero: None,
            caption: None,
            controls: Vec::new(),
            notices: Vec::new(),
            cards: Vec::new(),
            panels: None,
            note: None,
            credits: None,
        }
    }

    /// Breadcrumb, e.g. `("xgpui-color", "Color Spaces")`. The separator is fixed.
    pub fn crumb(mut self, owner: &str, screen: &str) -> Self {
        self.crumb = Some(format!("{owner}  /  {screen}").into());
        self
    }

    pub fn title(mut self, text: impl Into<SharedString>) -> Self {
        self.title = Some(text.into());
        self
    }

    pub fn lede(mut self, text: impl Into<SharedString>) -> Self {
        self.lede = Some(text.into());
        self
    }

    pub fn hero(mut self, hero: Hero) -> Self {
        self.hero = Some(hero);
        self
    }

    /// Live state line under the hero.
    pub fn caption(mut self, text: impl Into<SharedString>) -> Self {
        self.caption = Some(text.into());
        self
    }

    /// Append one item to the wrapping control row.
    pub fn control(mut self, element: impl IntoElement) -> Self {
        self.controls.push(element.into_any_element());
        self
    }

    /// Errors and fallbacks, under the controls. `None` adds nothing; may be called repeatedly.
    pub fn notice(mut self, text: Option<impl Into<SharedString>>) -> Self {
        self.notices.extend(text.map(Into::into));
        self
    }

    pub fn cards(mut self, cards: impl IntoIterator<Item = LabCard>) -> Self {
        self.cards.extend(cards);
        self
    }

    /// The two-panel row. The wide/narrow ratio comes from [`LabMetrics`].
    pub fn panels(mut self, wide: impl Into<Panel>, narrow: impl Into<Panel>) -> Self {
        self.panels = Some((wide.into(), narrow.into()));
        self
    }

    /// Closing explanation at the end of the content.
    pub fn note(mut self, text: impl Into<SharedString>) -> Self {
        self.note = Some(text.into());
        self
    }

    /// Sources, licenses and provenance, last on the page.
    pub fn credits(mut self, text: impl Into<SharedString>) -> Self {
        self.credits = Some(text.into());
        self
    }
}

impl RenderOnce for LabScreen {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let metrics = LabMetrics::STANDARD;
        let look = &*self.look;
        let id = self.id;
        let slot = move |element: Div, name: &'static str| element.debug_selector(move || format!("{id}-{name}"));
        let caption = |text: SharedString, name| slot(metrics::text(look, TextRole::Caption, text), name);

        let crumb = self.crumb.map(|crumb| slot(metrics::text(look, TextRole::Crumb, crumb), "crumb"));
        let title = self.title.map(|title| slot(metrics::text(look, TextRole::Title, title), "title"));
        let lede = self.lede.map(|lede| slot(metrics::text(look, TextRole::Lede, lede), "lede"));
        let header = (title.is_some() || lede.is_some())
            .then(|| slot(div().flex().flex_col().gap(metrics.header_gap), "header").children(title).children(lede));

        let mut content: Vec<Div> = Vec::new();
        content.extend(self.hero.map(|hero| slot(hero.render(look, &metrics), "hero")));
        content.extend(self.caption.map(|text| caption(text, "caption")));
        if !self.controls.is_empty() {
            content.push(slot(control_row(), "controls").children(self.controls));
        }
        content.extend(self.notices.into_iter().map(|text| caption(text, "notice")));
        if !self.cards.is_empty() {
            let cards = self.cards.into_iter().map(|card| card.render(look, &metrics));
            content.push(slot(hstack! {}.flex_wrap().gap(metrics.section_gap), "cards").children(cards));
        }
        if let Some((wide, narrow)) = self.panels {
            let sized = |panel: Div, width: metrics::PanelWidth, name| {
                slot(panel, name).flex_grow(width.grow).flex_basis(width.basis).min_w(width.min)
            };
            let wide = sized(wide.render(look, &metrics), metrics.wide_panel, "panel-wide");
            // `flex_1` keeps the narrow panel's shrink and grow at 1, as before.
            let narrow = sized(narrow.render(look, &metrics).flex_1(), metrics.narrow_panel, "panel-narrow");
            content.push(slot(hstack! { wide, narrow }.flex_wrap().gap(metrics.section_gap), "panels"));
        }
        content.extend(self.note.map(|text| caption(text, "note")));
        content.extend(self.credits.map(|text| slot(metrics::text(look, TextRole::Fine, text), "credits")));
        let content = (!content.is_empty())
            .then(|| slot(div().flex().flex_col().gap(metrics.section_gap), "content").children(content));

        page(id, look).child(container().children(crumb).children(header).children(content))
    }
}

// Page chrome shared with the screens that still use `lab_ui`.

/// Scrolling page with the look's content background and body text.
pub(crate) fn page(id: &'static str, look: &Look) -> Stateful<Div> {
    let chrome = look.chrome();
    div()
        .id(id)
        .size_full()
        .overflow_y_scroll()
        .bg(chrome.content_background)
        .text_color(chrome.body_text)
}

/// Centered content column.
pub(crate) fn container() -> Div {
    let metrics = LabMetrics::STANDARD;
    div()
        .flex()
        .flex_col()
        .w_full()
        .max_w(metrics.max_width)
        .mx_auto()
        .p(metrics.page_padding)
        .gap(metrics.part_gap)
}

/// Wrapping, vertically centered control row.
pub(crate) fn control_row() -> Div {
    div().flex().flex_wrap().items_center().gap(LabMetrics::STANDARD.control_gap)
}

/// Apply the reference switch label typography to a labeled control.
pub(crate) fn labeled_control(look: &Look, element: Div) -> Div {
    let label = xgpui_luma_look_radix::switch_theme(look).resolve(
        false,
        xgpui_luma::theme::InteractionState::default(),
        xgpui_luma::theme::ControlSize::Md,
    );
    element
        .typography_style(label.label_typography)
        .font_family(label.label_font_family)
        .text_color(label.label_color)
}

/// Supporting panel surface.
pub(crate) fn panel_surface(look: &Look) -> Div {
    let chrome = look.chrome();
    div()
        .flex()
        .flex_col()
        .gap_3()
        .p_4()
        .rounded_lg()
        .bg(chrome.panel_background)
        .border_1()
        .border_color(chrome.border)
}

/// Card surface; also applied to Luma button template rows.
pub(crate) fn card_surface<V: Styled>(view: V, look: &Look) -> V {
    let chrome = look.chrome();
    view.p_3().gap_3().rounded_lg().bg(chrome.panel_background).border_1().border_color(chrome.border)
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::{Hero, LabScreen};
    use gpui::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*, px};

    struct Probe(bool);
    impl Render for Probe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let screen = LabScreen::new("probe", crate::theme::default_look());
            if !self.0 {
                return screen;
            }
            screen
                .crumb("xgpui-color", "Probe")
                .title("Probe")
                .hero(Hero::full(div().w_full().h(px(120.))))
                .caption("State")
        }
    }

    #[test]
    fn filled_slots_render_and_empty_slots_do_not() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (probe, cx) = app.add_window_view(|_, _| Probe(true));
        cx.run_until_parked();
        let filled = ["probe-crumb", "probe-title", "probe-hero", "probe-caption"];
        let tops = filled.map(|slot| cx.debug_bounds(slot).unwrap_or_else(|| panic!("{slot} is rendered")).top());
        assert!(tops.windows(2).all(|pair| pair[0] < pair[1]), "slots follow the fixed order");
        assert_eq!(cx.debug_bounds("probe-hero").unwrap().size.height, px(120.));
        for slot in ["probe-lede", "probe-controls", "probe-notice", "probe-cards", "probe-panels", "probe-note"] {
            assert!(cx.debug_bounds(slot).is_none(), "{slot} is empty");
        }

        probe.update(cx, |probe, cx| {
            probe.0 = false;
            cx.notify();
        });
        cx.run_until_parked();
        for slot in ["probe-crumb", "probe-header", "probe-content"] {
            assert!(cx.debug_bounds(slot).is_none(), "empty {slot} leaves nothing behind");
        }
    }
}
