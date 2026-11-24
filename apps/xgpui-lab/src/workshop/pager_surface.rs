//! The SDK pager on the design surface page (issue #383). Its `.lss` (`pager`) has one state,
//! `:disabled`; the page buttons inside keep the button's `.lss` and stay live. The pager's style
//! (Minimal · Edge · Numeric) is the page's one option.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window, prelude::*};
use xgpui_luma::controls::button::default_button_template;
use xgpui_luma::controls::control_group::{ControlGroup, ControlGroupEvent, ControlGroupItem};
use xgpui_luma::controls::pager::{
    self, Pager, PagerEvent, PagerIcons, PagerRenderModel, PagerStyle, PagerTemplate, PagerTemplateHandlers,
    PagerTemplateParameters, StyledPagerTheme, ThemedPagerTemplate,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

const PAGE_COUNT: usize = 12;
const PAGE_SIZE: usize = 10;
const PAGE_SIZE_OPTIONS: [usize; 3] = [10, 25, 50];

/// The style option's items.
const STYLES: [(&str, &str, PagerStyle); 3] = [
    ("minimal", "Minimal", PagerStyle::Minimal),
    ("edge", "Edge", PagerStyle::MinimalEdge),
    ("numeric", "Numeric", PagerStyle::Numeric),
];

pub(super) struct PagerSurface {
    pager: Pager,
    /// Radix toggle group: Minimal · Edge · Numeric.
    style_picker: ControlGroup<ControlGroupItem>,
}

impl PagerSurface {
    pub(super) fn make(look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let pager = pager::new("workshop-sdk-pager")
            .page_count(PAGE_COUNT)
            .page_size(PAGE_SIZE)
            .page_size_options(PAGE_SIZE_OPTIONS)
            .spawn(cx);
        let current = pager.read(cx).style();
        let selected = STYLES.iter().find(|(_, _, style)| *style == current).map_or("edge", |(id, _, _)| *id);
        let style_picker = xgpui_luma_look_radix::control_group_builder(
            look.as_ref(),
            "workshop-surface-pager-style",
            xgpui_luma_look_radix::RadixSize::One,
        )
        .horizontal()
        .single_required()
        .items(STYLES.map(|(id, label, _)| ControlGroupItem::new(id).label(label)))
        .selected_ids([selected])
        .spawn(cx);
        let target = pager.clone();
        cx.subscribe(&style_picker, move |_: &mut ControlSide, _, event: &ControlGroupEvent, cx| {
            if let ControlGroupEvent::Change { selected_ids, .. } = event
                && let Some((_, _, style)) =
                    selected_ids.first().and_then(|id| STYLES.iter().find(|(item, _, _)| *item == id.as_ref()))
            {
                let style = *style;
                target.update(cx, |pager, cx| pager.set_style(style, cx));
            }
        })
        .detach();
        Box::new(Self { pager, style_picker })
    }

    /// The SDK pager, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn pager(&self) -> Pager {
        self.pager.clone()
    }

    /// The style option, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn style_picker(&self) -> ControlGroup<ControlGroupItem> {
        self.style_picker.clone()
    }
}

/// The pager template for the `pager` control in `theme`, with the SDK's page buttons.
fn template(theme: Theme) -> ThemedPagerTemplate {
    let binding = StyleBinding::new("pager", ThemeSource::Fixed(theme));
    ThemedPagerTemplate::new(Arc::new(StyledPagerTheme::new(binding, Arc::new(|_| default_button_template()))))
}

/// Draws the base template enabled as Design mode picks it, or as the pager is.
struct PinnedPagerTemplate {
    base: ThemedPagerTemplate,
    enabled: Option<bool>,
}

impl PagerTemplate for PinnedPagerTemplate {
    fn render(
        &self,
        model: &PagerRenderModel<'_>,
        handlers: PagerTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let Some(enabled) = self.enabled else {
            return self.base.render(model, handlers, window, cx);
        };
        let pinned = PagerRenderModel {
            id: model.id,
            current_page: model.current_page,
            page_count: model.page_count,
            page_size: model.page_size,
            page_size_options: model.page_size_options,
            page_size_open: model.page_size_open,
            enabled,
            style: model.style,
            info_text: model.info_text,
            info_slot: model.info_slot,
            template_parameters: model.template_parameters,
            icons: model.icons,
        };
        self.base.render(&pinned, handlers, window, cx)
    }
}

/// Handlers that do nothing, for a State matrix cell.
fn inert_handlers() -> PagerTemplateHandlers {
    PagerTemplateHandlers {
        outside_mouse_down: Arc::new(|_, _, _| {}),
        toggle_page_size: Arc::new(|_, _, _| {}),
        set_page: Arc::new(|_, _, _, _| {}),
        set_page_size: Arc::new(|_, _, _, _| {}),
    }
}

impl SurfaceControl for PagerSurface {
    fn title(&self) -> &'static str {
        "Pager"
    }

    fn view(&self) -> AnyView {
        self.pager.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let enabled = pinned.map(|states| !states.has("disabled"));
        let template = PinnedPagerTemplate { base: template(theme), enabled };
        self.pager.update(cx, |pager, cx| pager.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.pager.update(cx, |pager, cx| pager.set_enabled(enabled, cx));
    }

    /// The pager's one state.
    fn template_states(&self) -> Vec<&'static str> {
        vec!["disabled"]
    }

    /// Page and page-size changes; none sets a pager state.
    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.pager, move |side: &mut ControlSide, _, event: &PagerEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, PagerEvent::EnabledChanged { .. }) {
                return;
            }
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state: None }, cx);
        })
    }

    /// The pager in the live pager's style, on its first page.
    fn cell(
        &self,
        theme: &Theme,
        id: SharedString,
        states: &StateSet,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let style = self.pager.read(cx).style();
        let (parameters, icons) = (PagerTemplateParameters::default(), PagerIcons::default());
        let model = PagerRenderModel {
            id: &id,
            current_page: 0,
            page_count: PAGE_COUNT,
            page_size: PAGE_SIZE,
            page_size_options: &PAGE_SIZE_OPTIONS,
            page_size_open: false,
            enabled: !states.has("disabled"),
            style,
            info_text: None,
            info_slot: None,
            template_parameters: &parameters,
            icons: &icons,
        };
        template(theme.clone()).render(&model, inert_handlers(), window, cx)
    }

    fn options(&self) -> Option<AnyElement> {
        Some(self.style_picker.clone().into_any_element())
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::TestAppContext;

    use super::*;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_pager_page_pages_and_pins_disabled_and_offers_its_styles() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "pager");
        let side = cx.update(|_, cx| page.read(cx).side());
        let (pager, picker) = cx.update(|_, cx| {
            let surface = side.read(cx).control_as::<PagerSurface>().expect("a pager");
            (surface.pager(), surface.style_picker())
        });
        assert!(cx.debug_bounds("workshop-surface-options").is_some(), "the style option is shown");

        let bounds = cx.debug_bounds("workshop-surface-control").expect("control");
        cx.simulate_click(gpui::point(bounds.right() - gpui::px(12.), bounds.center().y), Default::default());
        cx.run_until_parked();
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("PageChanged"), "page changes are logged: {text}");

        picker.update(cx, |_, cx| {
            cx.emit(ControlGroupEvent::Change {
                changed_id: "numeric".into(),
                selected: true,
                selected_ids: vec!["numeric".into()],
            })
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(pager.read(cx).style(), PagerStyle::Numeric));

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("disabled", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert_eq!(pinned, StateSet::new(["disabled"]));

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw pagers");
    }
}
