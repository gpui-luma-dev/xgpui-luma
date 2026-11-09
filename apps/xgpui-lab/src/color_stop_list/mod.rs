//! A list of [`ColorStop`] controls: gradient stops, mesh points, palettes.
//!
//! Composition, in the style of the SDK selection controls:
//! - **List control (this module)**: the stop model (order, limits, add,
//!   remove, reverse, distribute, reorder), the SDK [`ListBoxControl`]
//!   (selection, keyboard, drag-to-reorder), header actions, and one
//!   [`ColorStop`] control per stop.
//! - **List template** ([`template`]): arranges header actions and rows.
//! - **Stop template** ([`crate::color_stop::template`]): each row's layout,
//!   set with [`ColorStopListBuilder::stop_template`].
//!
//! The list owns its stops and reports user edits as [`ColorStopListEvent`]s.
//! Host setters (`set_stops`, `set_color`, `select`) never emit.

mod model;
pub(crate) mod template;

use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, IntoElement, SharedString, Subscription, Window, div,
    prelude::*,
};
use lucide_svg_static::Icon as LucideIcon;
use xgpui_luma::color::ColorValue;
use xgpui_luma::controls::button::{Button, ButtonEvent};
use xgpui_luma::controls::listbox::{
    ListBoxControl, ListBoxDragDrop, ListBoxDropProposal, ListBoxEvent, ListBoxFlow, ListBoxInput,
    ListBoxItemRenderModel, ListBoxSnapshot, ListBoxState, ListBoxVirtualization, SelectionMode, SelectionPolicy,
    make_listbox_item_template,
};
use xgpui_luma::controls::tooltip::Tooltip;
use xgpui_luma::infra::attachments::TooltipEntityExt;
use xgpui_luma_look_radix as radix;

use crate::color_stop::{
    ColorStop, ColorStopBuilder, ColorStopEvent, ColorStopTemplate, DEFAULT_LINE_HEIGHT, default_color_stop_template,
    stop_label,
};
use crate::theme::Look;

pub(crate) use model::{CountLimits, StopData, StopKey};
use model::StopModel;
pub(crate) use template::{ColorStopListRenderModel, ColorStopListTemplate, default_color_stop_list_template};

/// The Radix ListBox row draws a 1px border around each stop.
const ROW_BORDER: f32 = 1.0;
const ROW_GAP: f32 = 2.0;
/// The viewport fits every row, so overscan only needs to cover small lists.
const MEASURED_OVERSCAN: usize = 32;

/// User edits, emitted after the list has applied them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ColorStopListEvent {
    SelectionChanged(Option<StopKey>),
    /// Enter / double-click on a row.
    Activated(StopKey),
    ColorChanged {
        key: StopKey,
        color: ColorValue,
    },
    PositionChanged {
        key: StopKey,
        position: f32,
    },
    Added(StopKey),
    Removed(StopKey),
    /// Rows were dragged into a new order. Positioned lists keep their position
    /// slots, so colors move between positions.
    Reordered,
    Reversed,
    Distributed,
}

/// Header actions offered by the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HeaderActions {
    pub reverse: bool,
    pub distribute: bool,
    pub add: bool,
}

impl HeaderActions {
    pub const NONE: Self = Self { reverse: false, distribute: false, add: false };
    pub const ALL: Self = Self { reverse: true, distribute: true, add: true };
}

pub(crate) struct ColorStopListBuilder {
    id: SharedString,
    look: Option<Arc<Look>>,
    entries: Vec<(ColorValue, Option<f32>)>,
    positioned: bool,
    limits: CountLimits,
    removable: bool,
    reorderable: bool,
    actions: HeaderActions,
    expand_selected: bool,
    template: Option<Arc<dyn ColorStopListTemplate>>,
    stop_template: Option<Arc<dyn ColorStopTemplate>>,
}

impl ColorStopListBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            entries: Vec::new(),
            positioned: false,
            limits: CountLimits::default(),
            removable: true,
            reorderable: true,
            actions: HeaderActions::NONE,
            expand_selected: false,
            template: None,
            stop_template: None,
        }
    }

    pub fn look(mut self, look: Arc<Look>) -> Self {
        self.look = Some(look);
        self
    }

    /// Gradient stops: each entry carries a normalized position.
    pub fn positioned_stops(mut self, stops: impl IntoIterator<Item = (ColorValue, f32)>) -> Self {
        self.positioned = true;
        self.entries = stops.into_iter().map(|(color, position)| (color, Some(position))).collect();
        self
    }

    /// Unpositioned colors (mesh points, palettes), kept in the given order.
    pub fn colors(mut self, colors: impl IntoIterator<Item = ColorValue>) -> Self {
        self.positioned = false;
        self.entries = colors.into_iter().map(|color| (color, None)).collect();
        self
    }

    pub fn count_limits(mut self, min: usize, max: usize) -> Self {
        self.limits = CountLimits { min, max: max.max(min) };
        self
    }

    #[allow(dead_code)]
    pub fn removable(mut self, removable: bool) -> Self {
        self.removable = removable;
        self
    }

    /// Drag rows to reorder (SDK ListBox drag/drop). Enabled by default.
    #[allow(dead_code)]
    pub fn reorderable(mut self, reorderable: bool) -> Self {
        self.reorderable = reorderable;
        self
    }

    pub fn header_actions(mut self, actions: HeaderActions) -> Self {
        self.actions = actions;
        self
    }

    /// Expand the selected stop's color editor inside its row.
    pub fn expand_selected(mut self, enabled: bool) -> Self {
        self.expand_selected = enabled;
        self
    }

    /// Layout of the header and rows.
    #[allow(dead_code)]
    pub fn template(mut self, template: Arc<dyn ColorStopListTemplate>) -> Self {
        self.template = Some(template);
        self
    }

    /// Layout of each row's [`ColorStop`].
    pub fn stop_template(mut self, template: Arc<dyn ColorStopTemplate>) -> Self {
        self.stop_template = Some(template);
        self
    }

    pub fn spawn(self, cx: &mut App) -> Entity<ColorStopList> {
        cx.new(|cx| ColorStopList::new(self, cx))
    }
}

struct HeaderButtons {
    reverse: Option<Entity<Button>>,
    distribute: Option<Entity<Button>>,
    add: Option<Entity<Button>>,
}

/// One row's control plus its event and layout subscriptions.
struct StopRow {
    control: Entity<ColorStop>,
    _subscriptions: [Subscription; 2],
}

pub(crate) struct ColorStopList {
    id: SharedString,
    look: Arc<Look>,
    model: StopModel,
    selected: Option<StopKey>,
    removable: bool,
    expand_selected: bool,
    list: ListBoxControl<Self, StopData, StopKey>,
    header: HeaderButtons,
    template: Arc<dyn ColorStopListTemplate>,
    stop_template: Arc<dyn ColorStopTemplate>,
    rows: HashMap<StopKey, StopRow>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ColorStopListEvent> for ColorStopList {}

impl ColorStopList {
    fn new(builder: ColorStopListBuilder, cx: &mut Context<Self>) -> Self {
        let look = builder.look.unwrap_or_else(crate::theme::default_look);
        let model = StopModel::new(builder.entries.iter().copied(), builder.limits, builder.positioned);

        let mut state =
            ListBoxState::try_new(model.stops().to_vec(), |stop: &StopData| stop.key, SelectionMode::SingleAllowNone)
                .expect("model allocates unique stop keys");
        state.set_selection_policy(SelectionPolicy {
            selection_follows_active: true,
            ..SelectionPolicy::new(SelectionMode::SingleAllowNone)
        });
        let mut list = ListBoxControl::new(
            state,
            Self::handle_list_input,
            |key: &StopKey| ("color-stop", *key as usize).into(),
            |stop: &StopData| stop_label(stop.color).into(),
            cx,
        )
        // Rows are content-sized so a stop template may change row height and
        // the selected stop can expand its editor in place.
        .virtualization(ListBoxVirtualization::Measured {
            estimated_height: DEFAULT_LINE_HEIGHT + ROW_BORDER * 2.0,
            overscan: MEASURED_OVERSCAN,
        });
        if builder.reorderable {
            list = list.drag_drop(ListBoxDragDrop::new(cx.entity_id(), Self::handle_drop));
        }

        let mut subscriptions = Vec::new();
        let mut header_button = |enabled: bool,
                                 name: &str,
                                 icon: LucideIcon,
                                 tip: &'static str,
                                 action: fn(&mut Self, &mut Context<Self>),
                                 cx: &mut Context<Self>| {
            enabled.then(|| {
                let button = crate::theme::icon_button(format!("{}-{name}", builder.id), icon)
                    .look(look.as_ref())
                    .content_only()
                    .radius(radix::Radius::Medium)
                    .spawn(cx)
                    .tooltip(Tooltip::new(tip), cx);
                subscriptions.push(cx.subscribe(&button, move |this, _, event: &ButtonEvent, cx| {
                    if matches!(event, ButtonEvent::Click) {
                        action(this, cx);
                    }
                }));
                button
            })
        };
        let actions = builder.actions;
        let header = HeaderButtons {
            reverse: header_button(
                actions.reverse,
                "reverse",
                LucideIcon::ArrowLeftRight,
                "Reverse",
                Self::reverse_by_user,
                cx,
            ),
            distribute: header_button(
                actions.distribute && builder.positioned,
                "distribute",
                LucideIcon::AlignHorizontalSpaceAround,
                "Distribute evenly",
                Self::distribute_by_user,
                cx,
            ),
            add: header_button(actions.add, "add", LucideIcon::Plus, "Add color", Self::add_by_user, cx),
        };

        let mut this = Self {
            id: builder.id,
            look,
            model,
            selected: None,
            removable: builder.removable,
            expand_selected: builder.expand_selected,
            list,
            header,
            template: builder.template.unwrap_or_else(default_color_stop_list_template),
            stop_template: builder.stop_template.unwrap_or_else(default_color_stop_template),
            rows: HashMap::new(),
            _subscriptions: subscriptions,
        };
        this.sync_stops(cx);
        this
    }

    // ---- Host API (never emits) ------------------------------------------
    // Screens adopt these as they migrate to the list (#131); until then only tests call them.

    #[allow(dead_code)]
    pub fn stops(&self) -> &[StopData] {
        self.model.stops()
    }

    #[allow(dead_code)]
    pub fn selected(&self) -> Option<StopKey> {
        self.selected
    }

    /// Replace every stop; keys are reallocated and selection is cleared.
    #[allow(dead_code)]
    pub fn set_stops(&mut self, entries: impl IntoIterator<Item = (ColorValue, Option<f32>)>, cx: &mut Context<Self>) {
        self.model.replace(entries);
        self.selected = None;
        self.sync_stops(cx);
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn set_color(&mut self, key: StopKey, color: ColorValue, cx: &mut Context<Self>) {
        if self.model.set_color(key, color) {
            self.sync_stops(cx);
            cx.notify();
        }
    }

    #[allow(dead_code)]
    pub fn select(&mut self, key: Option<StopKey>, cx: &mut Context<Self>) {
        let key = key.filter(|key| self.model.get(*key).is_some());
        if self.set_selected(key, cx) {
            cx.notify();
        }
    }

    pub fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for row in self.rows.values() {
            row.control.update(cx, |stop, cx| stop.sync_look(look.clone(), cx));
        }
        cx.notify();
    }

    // ---- User edits (emit after applying) ---------------------------------

    fn add_by_user(&mut self, cx: &mut Context<Self>) {
        if let Some(key) = self.model.add(self.selected) {
            self.selected = Some(key);
            self.sync_stops(cx);
            cx.emit(ColorStopListEvent::Added(key));
            cx.emit(ColorStopListEvent::SelectionChanged(Some(key)));
            cx.notify();
        }
    }

    fn remove_by_user(&mut self, key: StopKey, cx: &mut Context<Self>) {
        if !self.model.remove(key) {
            return;
        }
        let cleared = self.selected == Some(key);
        if cleared {
            self.selected = None;
        }
        self.sync_stops(cx);
        cx.emit(ColorStopListEvent::Removed(key));
        if cleared {
            cx.emit(ColorStopListEvent::SelectionChanged(None));
        }
        cx.notify();
    }

    fn reverse_by_user(&mut self, cx: &mut Context<Self>) {
        if self.model.reverse() {
            self.sync_stops(cx);
            cx.emit(ColorStopListEvent::Reversed);
            cx.notify();
        }
    }

    fn distribute_by_user(&mut self, cx: &mut Context<Self>) {
        if self.model.distribute() {
            self.sync_stops(cx);
            cx.emit(ColorStopListEvent::Distributed);
            cx.notify();
        }
    }

    fn reorder_by_user(&mut self, order: &[StopKey], cx: &mut Context<Self>) -> bool {
        if !self.model.reorder(order) {
            return false;
        }
        self.sync_stops(cx);
        cx.emit(ColorStopListEvent::Reordered);
        cx.notify();
        true
    }

    /// A row's control reported a user edit; apply it to the model.
    fn handle_stop_event(&mut self, key: StopKey, event: &ColorStopEvent, cx: &mut Context<Self>) {
        match *event {
            ColorStopEvent::ColorChanged(color) => {
                if self.model.set_color(key, color) {
                    self.sync_list(cx);
                    cx.emit(ColorStopListEvent::ColorChanged { key, color });
                }
            }
            ColorStopEvent::PositionChanged(position) => {
                if let Some(position) = self.model.set_position(key, position) {
                    cx.emit(ColorStopListEvent::PositionChanged { key, position });
                }
                // Re-sort, renumber, and push canonical positions back to the rows.
                self.sync_stops(cx);
            }
            ColorStopEvent::RemoveRequested => self.remove_by_user(key, cx),
            // Rows are built with `expand_on_swatch_click(false)`; selection drives expansion.
            ColorStopEvent::ExpandedChanged(_) => {}
        }
        cx.notify();
    }

    // ---- SDK ListBox plumbing ---------------------------------------------

    fn handle_list_input(&mut self, input: ListBoxInput<StopKey>, _window: &mut Window, cx: &mut Context<Self>) {
        let update = self.list.apply(input, cx);
        for event in update.events {
            match event {
                ListBoxEvent::SelectionChanged { selected, .. } => {
                    let key = selected.first().copied();
                    if self.set_selected(key, cx) {
                        cx.emit(ColorStopListEvent::SelectionChanged(key));
                    }
                }
                ListBoxEvent::ItemActivated { key } => cx.emit(ColorStopListEvent::Activated(key)),
                _ => {}
            }
        }
        cx.notify();
    }

    /// Same-list reorder only; the SDK computes `before`, the model applies it.
    fn handle_drop(&mut self, proposal: ListBoxDropProposal<StopKey>, _window: &mut Window, cx: &mut Context<Self>) {
        let list_id = self.list.drag_drop_id();
        if let Err(error) = proposal.validate() {
            proposal.rejected(error, cx);
            return;
        }
        if proposal.source() != list_id || proposal.target() != list_id {
            proposal.rejected("Colors can only be reordered within their own list", cx);
            return;
        }
        let current: Vec<StopKey> = self.model.stops().iter().map(|stop| stop.key).collect();
        let moving: Vec<StopKey> = current.iter().copied().filter(|key| proposal.keys().contains(key)).collect();
        let mut order: Vec<StopKey> = current.iter().copied().filter(|key| !moving.contains(key)).collect();
        let index = proposal
            .before()
            .and_then(|before| order.iter().position(|key| key == before))
            .unwrap_or(order.len());
        order.splice(index..index, moving.iter().copied());
        let changed = self.reorder_by_user(&order, cx);
        proposal.committed(changed, moving, cx);
    }

    // ---- Synchronization --------------------------------------------------

    /// Single write path for selection so the SDK list and rows always track it.
    fn set_selected(&mut self, key: Option<StopKey>, cx: &mut Context<Self>) -> bool {
        if self.selected == key {
            return false;
        }
        self.selected = key;
        self.sync_stops(cx);
        true
    }

    /// Push model order/content into the SDK list and mirror our selection into it.
    fn sync_list(&mut self, cx: &mut Context<Self>) {
        let snapshot = ListBoxSnapshot::try_new(self.model.stops().to_vec(), |stop: &StopData| stop.key)
            .expect("model allocates unique stop keys");
        let update = self.list.state.replace_snapshot(snapshot);
        self.list.handle_update(&update, cx);
        if self.list.state.selected_keys().next().copied() != self.selected {
            self.list.apply(
                match self.selected {
                    Some(key) => ListBoxInput::Select(key),
                    None => ListBoxInput::ClearSelection,
                },
                cx,
            );
        }
    }

    /// Keep the SDK list and one [`ColorStop`] per stop in step with the model.
    fn sync_stops(&mut self, cx: &mut Context<Self>) {
        self.sync_list(cx);
        if let Some(add) = &self.header.add {
            let can_add = self.model.can_add();
            add.update(cx, |button, cx| button.set_enabled(can_add, cx));
        }
        let can_remove = self.model.can_remove();
        let stops = self.model.stops().to_vec();
        self.rows.retain(|key, _| stops.iter().any(|stop| stop.key == *key));
        for (index, stop) in stops.into_iter().enumerate() {
            let key = stop.key;
            if !self.rows.contains_key(&key) {
                let row = self.spawn_row(&stop, cx);
                self.rows.insert(key, row);
            }
            let selected = self.selected == Some(key);
            let expanded = selected && self.expand_selected;
            let control = self.rows[&key].control.clone();
            control.update(cx, |row, cx| {
                row.set_index(Some(index), cx);
                row.set_color(stop.color, cx);
                if let Some(position) = stop.position {
                    row.set_position(position, cx);
                }
                row.set_selected(selected, cx);
                row.set_expanded(expanded, cx);
                row.set_can_remove(can_remove, cx);
            });
        }
        // Row content (expansion, template) changed; let the ListBox re-measure.
        self.list.invalidate_measurements(None, cx);
    }

    fn spawn_row(&mut self, stop: &StopData, cx: &mut Context<Self>) -> StopRow {
        let key = stop.key;
        let mut builder = ColorStopBuilder::new(format!("{}-stop-{key}", self.id), stop.color)
            .look(self.look.clone())
            .removable(self.removable)
            .expand_on_swatch_click(false)
            .template(self.stop_template.clone());
        if let Some(position) = stop.position {
            builder = builder.position(position);
        }
        let control = builder.spawn(cx);
        let events = cx.subscribe(&control, move |this, _, event: &ColorStopEvent, cx| {
            this.handle_stop_event(key, event, cx);
        });
        // Row height (from the stop template) sizes the viewport.
        let layout = cx.observe(&control, |_, _, cx| cx.notify());
        StopRow { control, _subscriptions: [events, layout] }
    }

    /// Total row height so the list shows every stop without scrolling.
    fn viewport_height(&self, cx: &App) -> f32 {
        let rows = self.model.stops();
        let content: f32 = rows
            .iter()
            .map(|stop| {
                let measured = self.rows.get(&stop.key).and_then(|row| row.control.read(cx).height());
                measured.unwrap_or(DEFAULT_LINE_HEIGHT) + ROW_BORDER * 2.0
            })
            .sum();
        content + rows.len().saturating_sub(1) as f32 * ROW_GAP
    }
}

impl Render for ColorStopList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let flow = ListBoxFlow::VerticalContent { viewport_height: self.viewport_height(cx), gap: ROW_GAP };
        let rows = &self.rows;
        let item = make_listbox_item_template(move |model: &ListBoxItemRenderModel<'_, StopData>, _: &mut App| {
            div().w_full().children(rows.get(&model.item.key).map(|row| row.control.clone()))
        });
        let list = radix::ListBoxBuilder::new("color-stop-rows", &mut self.list, item)
            .look(self.look.as_ref())
            .flow(flow)
            .build(window, cx)
            .into_any_element();
        let actions: Vec<AnyElement> = [&self.header.reverse, &self.header.distribute, &self.header.add]
            .into_iter()
            .flatten()
            .map(|button| button.clone().into_any_element())
            .collect();
        let model = ColorStopListRenderModel {
            id: &self.id,
            look: &self.look,
            count: self.model.len(),
            positioned: self.model.stops().iter().any(|stop| stop.position.is_some()),
            actions,
            list,
        };
        self.template.render(model, window, cx)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use gpui::TestAppContext;

    fn gray(v: f32) -> ColorValue {
        ColorValue::srgb(v, v, v, 1.0)
    }

    fn spawn_with_events(
        app: &mut TestAppContext,
        builder: ColorStopListBuilder,
    ) -> (Entity<ColorStopList>, Rc<RefCell<Vec<ColorStopListEvent>>>) {
        app.update(|cx| {
            xgpui_luma::init(cx).expect("initialize SDK");
            let list = builder.spawn(cx);
            let events = Rc::new(RefCell::new(Vec::new()));
            let sink = events.clone();
            cx.subscribe(&list, move |_, event: &ColorStopListEvent, _| sink.borrow_mut().push(event.clone()))
                .detach();
            (list, events)
        })
    }

    fn row(app: &TestAppContext, list: &Entity<ColorStopList>, key: StopKey) -> Entity<ColorStop> {
        list.read_with(app, |list, _| list.rows[&key].control.clone())
    }

    #[test]
    fn user_edits_emit_events_and_respect_limits() {
        let mut app = TestAppContext::single();
        let (list, events) = spawn_with_events(
            &mut app,
            ColorStopListBuilder::new("test-stops")
                .positioned_stops([(gray(0.0), 0.0), (gray(1.0), 1.0)])
                .count_limits(2, 3)
                .header_actions(HeaderActions::ALL),
        );

        list.update(&mut app, |list, cx| list.add_by_user(cx));
        let added = list.read_with(&app, |list, _| list.selected()).expect("new stop is selected");
        assert_eq!(
            *events.borrow(),
            vec![ColorStopListEvent::Added(added), ColorStopListEvent::SelectionChanged(Some(added))]
        );
        assert_eq!(list.read_with(&app, |list, _| list.rows.len()), 3, "one ColorStop control per stop");

        events.borrow_mut().clear();
        list.update(&mut app, |list, cx| list.add_by_user(cx));
        assert!(events.borrow().is_empty(), "max count reached");

        row(&app, &list, added).update(&mut app, |_, cx| cx.emit(ColorStopEvent::RemoveRequested));
        assert_eq!(
            *events.borrow(),
            vec![ColorStopListEvent::Removed(added), ColorStopListEvent::SelectionChanged(None)]
        );

        events.borrow_mut().clear();
        let first = list.read_with(&app, |list, _| list.stops()[0].key);
        list.update(&mut app, |list, cx| {
            list.set_color(first, gray(0.3), cx);
            list.select(Some(first), cx);
        });
        assert!(events.borrow().is_empty(), "host setters never emit");
        assert_eq!(row(&app, &list, first).read_with(&app, |stop, _| stop.color()), gray(0.3), "row control synced");
    }

    #[test]
    fn row_edits_flow_into_the_model_and_renumber_rows() {
        let mut app = TestAppContext::single();
        let (list, events) = spawn_with_events(
            &mut app,
            ColorStopListBuilder::new("test-row-edits").positioned_stops([
                (gray(0.0), 0.0),
                (gray(0.5), 0.5),
                (gray(1.0), 1.0),
            ]),
        );
        let first = list.read_with(&app, |list, _| list.stops()[0].key);

        let green = ColorValue::srgb(0.0, 1.0, 0.0, 1.0);
        row(&app, &list, first).update(&mut app, |_, cx| cx.emit(ColorStopEvent::ColorChanged(green)));
        row(&app, &list, first).update(&mut app, |_, cx| cx.emit(ColorStopEvent::PositionChanged(0.75)));
        assert_eq!(
            *events.borrow(),
            vec![
                ColorStopListEvent::ColorChanged { key: first, color: green },
                ColorStopListEvent::PositionChanged { key: first, position: 0.75 },
            ]
        );
        assert_eq!(list.read_with(&app, |list, _| list.model.index_of(first)), Some(1), "list re-sorts");
        assert_eq!(row(&app, &list, first).read_with(&app, |stop, _| stop.index()), Some(1), "row renumbered");
    }

    #[test]
    fn selection_expands_only_the_selected_row() {
        let mut app = TestAppContext::single();
        let (list, _) = spawn_with_events(
            &mut app,
            ColorStopListBuilder::new("test-expand").colors([gray(0.0), gray(1.0)]).expand_selected(true),
        );
        let keys: Vec<StopKey> = list.read_with(&app, |list, _| list.stops().iter().map(|stop| stop.key).collect());
        list.update(&mut app, |list, cx| list.select(Some(keys[1]), cx));
        let expanded = |app: &TestAppContext, key| row(app, &list, key).read_with(app, |stop, _| stop.expanded());
        assert!(!expanded(&app, keys[0]) && expanded(&app, keys[1]));
        list.update(&mut app, |list, cx| list.select(Some(keys[0]), cx));
        assert!(expanded(&app, keys[0]) && !expanded(&app, keys[1]));
    }

    #[test]
    fn reorder_emits_and_sdk_list_mirrors_order_and_selection() {
        let mut app = TestAppContext::single();
        let (list, events) = spawn_with_events(
            &mut app,
            ColorStopListBuilder::new("test-reorder").colors([gray(0.0), gray(0.5), gray(1.0)]),
        );
        let keys: Vec<StopKey> = list.read_with(&app, |list, _| list.stops().iter().map(|stop| stop.key).collect());
        let sdk_order = |app: &TestAppContext| {
            list.read_with(app, |list, _| {
                list.list.state.snapshot().items().iter().map(|stop| stop.key).collect::<Vec<_>>()
            })
        };

        list.update(&mut app, |list, cx| assert!(list.reorder_by_user(&[keys[2], keys[0], keys[1]], cx)));
        assert_eq!(*events.borrow(), vec![ColorStopListEvent::Reordered]);
        assert_eq!(sdk_order(&app), vec![keys[2], keys[0], keys[1]], "SDK snapshot follows the model");

        list.update(&mut app, |list, cx| list.select(Some(keys[0]), cx));
        let sdk_selected = list.read_with(&app, |list, _| list.list.state.selected_keys().copied().collect::<Vec<_>>());
        assert_eq!(sdk_selected, vec![keys[0]], "host selection is mirrored into the SDK list");
    }

    #[test]
    fn expanded_row_reports_its_height_to_size_the_viewport() {
        let mut app = TestAppContext::single();
        app.update(|cx| {
            xgpui_luma::init(cx).expect("initialize SDK");
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        let (list, cx) = app.add_window_view(|_, cx| {
            ColorStopList::new(
                ColorStopListBuilder::new("test-row-picker")
                    .positioned_stops([(gray(0.0), 0.0), (gray(1.0), 1.0)])
                    .expand_selected(true),
                cx,
            )
        });
        cx.run_until_parked();
        let collapsed = cx.update(|_, cx| list.read(cx).viewport_height(cx));
        let second = cx.update(|_, cx| list.read(cx).stops()[1].key);
        list.update(cx, |list, cx| list.select(Some(second), cx));
        cx.run_until_parked();
        let row_height = cx.update(|_, cx| list.read(cx).rows[&second].control.read(cx).height()).unwrap();
        let expanded = cx.update(|_, cx| list.read(cx).viewport_height(cx));
        assert!(row_height > 100.0, "editor laid out inside the row, got {row_height}");
        assert!(expanded > collapsed + 100.0, "viewport grew from {collapsed} to {expanded}");
    }
}
