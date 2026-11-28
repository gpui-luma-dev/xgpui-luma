//! The SDK listbox on the design surface page (issue #393). The SDK owns the listbox's state, input and
//! scrolling, and draws its surface and rows from `listbox.lss` (`ThemedListBoxTemplate`, #397); looks can supply
//! their own `ListBoxTemplate`. The page draws the SDK's, in the theme it binds. Design mode pins the picked
//! states on the surface and the first row; each State matrix cell draws a short list.

use std::sync::Arc;

use gpui::{
    AnyElement, AnyView, App, Context, Div, ElementId, Entity, SharedString, Stateful, Subscription, Window, div,
    prelude::*, px,
};
use xgpui_luma::controls::listbox::{
    ListBoxBuilder, ListBoxControl, ListBoxDragLook, ListBoxFlow, ListBoxInput, ListBoxItemState, ListBoxMotion,
    ListBoxRenderModel, ListBoxRowRenderModel, ListBoxSnapshot, ListBoxState, ListBoxTemplate, SelectionMode,
    StyledListBox, ThemedListBoxTemplate,
};
use xgpui_luma::style::PartRead;
use xgpui_luma::style::lss::{PartId, Resolved, Theme};
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The rows: a key, a label with ascenders and descenders, and whether it is enabled.
type Item = (&'static str, &'static str, bool);
const ITEMS: [Item; 5] = [
    ("typography", "Typography", true),
    ("display", "Display", true),
    ("legacy", "Legacy layout", false),
    ("spacing", "Spacing", true),
    ("quality", "Quality", true),
];
const WIDTH: f32 = 240.;
const ROW_HEIGHT: f32 = 32.;
const ROW_GAP: f32 = 2.;

fn styled(theme: &Theme) -> StyledListBox {
    StyledListBox::new(StyleBinding::new("listbox", ThemeSource::Fixed(theme.clone())))
}

fn item_id(key: &&'static str) -> ElementId {
    (*key).into()
}

/// A row's chrome in fixed states, from its resolved style's `row` part: the pinned first row and the matrix
/// cells' rows.
fn fixed_row(resolved: Arc<Resolved>, id: impl Into<ElementId>) -> Stateful<Div> {
    let row = resolved.part(PartId::new("row"));
    let font_family = row.font_family_name();
    div()
        .id(id)
        .bg(row.color("background"))
        .text_color(row.color("foreground"))
        .text_size(px(row.font_size().unwrap_or(14.0)))
        .line_height(px(row.line_height().unwrap_or(20.0)))
        .font_weight(row.font_weight())
        .border(px(row.number("border_width")))
        .border_color(row.color("border"))
        .rounded(px(row.number("radius")))
        .px(px(row.number("padding_x")))
        .when(!font_family.is_empty(), |row| row.font_family(font_family))
}

/// A row's resolved style in picked states.
fn picked_row(styled: &StyledListBox, states: &StateSet) -> Arc<Resolved> {
    let state = ListBoxItemState { selected: states.has("selected"), active: true, enabled: !states.has("disabled") };
    styled.resolve_row(state, states.has("focused"), states.has("hover"), states.has("pressed"))
}

/// The SDK's listbox chrome (`ThemedListBoxTemplate`, from `listbox.lss`); in Design mode the picked states draw on
/// the surface and the first row.
struct PageTemplate {
    themed: ThemedListBoxTemplate,
    pinned: Option<StateSet>,
}

impl ListBoxTemplate for PageTemplate {
    fn render(&self, model: &ListBoxRenderModel, cx: &mut App) -> Stateful<Div> {
        let surface = self.themed.render(model, cx);
        let Some(states) = &self.pinned else {
            return surface;
        };
        let picked = self.themed.styled().resolve_surface(!states.has("disabled"), states.has("focused"));
        let picked = picked.root();
        surface.bg(picked.color("background")).border_color(picked.color("border"))
    }

    fn render_row(&self, model: &ListBoxRowRenderModel, cx: &mut App) -> Stateful<Div> {
        match &self.pinned {
            Some(states) if model.id == item_id(&ITEMS[0].0) => {
                fixed_row(picked_row(self.themed.styled(), states), model.id.clone())
                    .w_full()
                    .when(!model.content_sized, |row| row.h_full())
            }
            _ => self.themed.render_row(model, cx),
        }
    }

    fn drag_look(&self) -> ListBoxDragLook {
        self.themed.drag_look()
    }

    fn motion_defaults(&self) -> ListBoxMotion {
        self.themed.motion_defaults()
    }
}

/// What the live listbox reports, as log lines.
struct ListBoxLine(String);

/// The live listbox: the SDK control and the page's template, in the theme last bound.
struct ListBoxHost {
    list: ListBoxControl<Self, Item, &'static str>,
    theme: Option<Theme>,
    pinned: Option<StateSet>,
    enabled: bool,
}

impl gpui::EventEmitter<ListBoxLine> for ListBoxHost {}

impl ListBoxHost {
    fn new(cx: &mut Context<Self>) -> Self {
        let snapshot = ListBoxSnapshot::try_with_enabled(ITEMS, |item: &Item| item.0, |item: &Item| item.2)
            .expect("the rows have unique keys");
        let state = ListBoxState::from_snapshot(snapshot, SelectionMode::SingleRequired);
        let list = ListBoxControl::new(state, Self::on_input, item_id, |item: &Item| item.1.into(), cx);
        Self { list, theme: None, pinned: None, enabled: true }
    }

    fn on_input(&mut self, input: ListBoxInput<&'static str>, _: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled {
            return;
        }
        let update = self.list.apply(input, cx);
        for event in update.events {
            cx.emit(ListBoxLine(format!("{event:?}")));
        }
        cx.notify();
    }
}

impl Render for ListBoxHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(theme) = self.theme.clone() else {
            return div().into_any_element();
        };
        let template = PageTemplate { themed: ThemedListBoxTemplate::new(styled(&theme)), pinned: self.pinned.clone() };
        let layout = ListBoxFlow::Vertical { visible_items: ITEMS.len(), item_height: ROW_HEIGHT, gap: ROW_GAP }
            .layout((0., 0.));
        let list = ListBoxBuilder::from_control("workshop-sdk-listbox", &mut self.list, layout, |surface, item, _| {
            // The SDK row draws state chrome only; this page's content is a label, centered in the row.
            surface.flex().items_center().child(item.item.1)
        })
        .build(&template, window, cx);
        div().debug_selector(|| "workshop-sdk-listbox".into()).w_full().child(list).into_any_element()
    }
}

pub(super) struct ListBoxSurface {
    host: Entity<ListBoxHost>,
}

impl ListBoxSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Box::new(Self { host: cx.new(ListBoxHost::new) })
    }
}

impl SurfaceControl for ListBoxSurface {
    fn title(&self) -> &'static str {
        "ListBox"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    /// The list fills the width the stage gives it, which can be dragged; its height is its rows' (#397).
    fn stage_width(&self) -> Option<f32> {
        Some(WIDTH)
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        self.host.update(cx, |host, cx| {
            host.theme = Some(theme);
            host.pinned = pinned.cloned();
            cx.notify();
        });
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.host.update(cx, |host, cx| {
            host.enabled = enabled;
            cx.notify();
        });
    }

    fn template_states(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.host, move |side: &mut ControlSide, _, line: &ListBoxLine, cx| {
            on_event(side, SurfaceEvent { line: line.0.clone(), state: None }, cx);
        })
    }

    /// A short list: the first row in the cell's states, the second at rest.
    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, _: &mut App) -> AnyElement {
        let styled = styled(theme);
        let surface = styled.resolve_surface(!states.has("disabled"), states.has("focused"));
        let surface = surface.root();
        let rest =
            styled.resolve_row(ListBoxItemState { selected: false, active: false, enabled: true }, false, false, false);
        let row = |look, key: &str, label: &'static str| {
            fixed_row(look, ElementId::Name(format!("{id}-{key}").into()))
                .h(px(ROW_HEIGHT))
                .flex()
                .items_center()
                .child(label)
        };
        div()
            .w(px(180.))
            .flex()
            .flex_col()
            .gap(px(ROW_GAP))
            .p_1()
            .bg(surface.color("background"))
            .border(px(surface.number("border_width")))
            .border_color(surface.color("border"))
            .rounded(px(surface.number("radius")))
            .child(row(picked_row(&styled, states), "first", ITEMS[0].1))
            .child(row(rest, "second", ITEMS[1].1))
            .into_any_element()
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
    fn the_listbox_page_selects_rows_and_pins_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "listbox");
        let side = cx.update(|_, cx| page.read(cx).side());
        assert!(cx.update(|_, cx| side.read(cx).control_as::<ListBoxSurface>().is_some()));
        let list = cx.debug_bounds("workshop-sdk-listbox").expect("the listbox is drawn");

        // Test: clicking the second row selects it, and the change is logged.
        let second = list.origin + gpui::point(px(40.), px(4. + ROW_HEIGHT + ROW_GAP + ROW_HEIGHT / 2.));
        cx.simulate_click(second, gpui::Modifiers::none());
        cx.run_until_parked();
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("SelectionChanged"), "{text}");

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("hover", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("hover"), "{pinned:?}");
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "the matrix draws lists");
    }
}
