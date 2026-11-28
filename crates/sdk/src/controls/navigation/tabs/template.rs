use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::cell::Cell;
use std::sync::{Arc, Mutex, OnceLock};

use gpui::{
    AnyElement, App, Bounds, Div, MouseDownEvent, Pixels, SharedString, Size, Stateful, TextRun, Window, anchored,
    deferred, div, font, hsla, px, prelude::*,
};

use super::indicator::TabsIndicatorMotion;
use super::{TabsItem, TabsItemAccessory, TabsRenderItem, TabsRenderModel, model::TabsWidthMode};
use crate::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
use crate::infra::ElementExt;
use crate::infra::popup_placement::{PopupAlign, PopupPlacementRequest, PopupSide, PopupSpace, ResolvedPopupPlacement};
use crate::infra::lock;
use crate::controls::button::{ButtonRenderModel, ButtonTemplate, StyledButtonTemplate};
use crate::controls::control_group::{
    ControlGroupBoundsHandler, ControlGroupClickHandler, ControlGroupHoverHandler, ControlGroupItemHandlerExt,
    ControlGroupMouseDownHandler, ControlGroupMouseUpHandler, ControlGroupRenderModel, ControlGroupTemplate,
    ControlGroupTemplateHandlers,
};
use crate::infra::icon::{DisclosureIcons, IconSource, render_icon_source, render_disclosure_icon};
use crate::motion::overlay_presence::OverlayPresence;
use crate::controls::tabs::{TabsTheme, default_tabs_theme, resolve_tab, resolve_tabs_list};
use crate::style::PartRead;
use xgpui_luma_style::{PartId, Resolved};
use crate::theme::InteractionState;

pub type TabsClickHandler = ControlGroupClickHandler;
pub type TabsBoundsHandler = ControlGroupBoundsHandler;
pub type TabsHoverHandler = ControlGroupHoverHandler;
pub type TabsMouseDownHandler = ControlGroupMouseDownHandler;
pub type TabsMouseUpHandler = ControlGroupMouseUpHandler;
pub type TabsTemplateHandlers = ControlGroupTemplateHandlers;

#[derive(Clone)]
pub struct TabsOverlayState {
    motion: Arc<Mutex<crate::motion::MotionConfig<crate::motion::PresenceMotion>>>,
    trigger_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    presence: Arc<Mutex<OverlayPresence>>,
    last_overlay: Arc<Mutex<Option<TabsCachedOverlay>>>,
    /// The window size and layout direction, read when the trigger is laid out.
    space: Arc<Mutex<PopupSpace>>,
}

#[derive(Clone)]
struct TabsCachedOverlay {
    content_size: Size<Pixels>,
    placement: TabsOverlayPlacement,
    offset_y: Pixels,
    window_margin: Pixels,
}

impl Default for TabsOverlayState {
    fn default() -> Self {
        Self {
            motion: Arc::new(Mutex::new(Default::default())),
            trigger_bounds: Arc::new(Mutex::new(None)),
            presence: Arc::new(Mutex::new(OverlayPresence::new(false, true))),
            last_overlay: Arc::new(Mutex::new(None)),
            space: Arc::new(Mutex::new(PopupSpace::default())),
        }
    }
}

impl TabsOverlayState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure before first use; clones share this host's snapshot.
    pub fn motion(self, configure: impl Fn(&mut crate::motion::PresenceMotion) + Send + Sync + 'static) -> Self {
        lock::mutex(&self.motion).configure(configure);
        self
    }
    pub fn motion_defaults(self, profile: crate::motion::PresenceMotion) -> Self {
        lock::mutex(&self.motion).defaults(profile);
        self
    }
    pub fn motion_enabled(self, enabled: bool) -> Self {
        self.set_motion_enabled(enabled);
        self
    }

    pub fn trigger_bounds(&self) -> Option<Bounds<Pixels>> {
        *lock::mutex(&self.trigger_bounds)
    }

    pub fn set_motion_enabled(&self, motion_enabled: bool) {
        lock::mutex(&self.presence).set_motion_enabled(motion_enabled);
    }

    /// Sync presence for the current frame. Returns `true` while animating.
    /// Hosting controls should call this from `Render` and `cx.notify()` while it returns true.
    pub fn sync_for_frame(&self) -> bool {
        lock::mutex(&self.presence).sync()
    }

    pub fn is_animating(&self) -> bool {
        lock::mutex(&self.presence).is_animating()
    }

    pub fn should_paint(&self) -> bool {
        lock::mutex(&self.presence).should_paint()
    }

    fn set_overlay_open(&self, open: bool) {
        let mut config = lock::mutex(&self.motion);
        let mut presence = lock::mutex(&self.presence);
        if !config.is_frozen() {
            config.freeze();
            *presence = OverlayPresence::with_motion(
                presence.is_logical_open(),
                presence.motion_enabled(),
                config.profile().clone(),
            );
        }
        presence.set_open(open);
    }

    fn presence_snapshot(&self) -> OverlayPresence {
        lock::mutex(&self.presence).clone()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TabsOverlayPlacement {
    BelowStart,
    BelowCenter,
}

pub struct TabsItemOverlay {
    pub content: AnyElement,
    pub content_size: Size<Pixels>,
    pub placement: TabsOverlayPlacement,
    pub offset_y: Pixels,
    pub window_margin: Pixels,
}

#[derive(Clone)]
struct TabsButtonData {
    /// The tab's resolved style in its state (#433).
    style: Arc<Resolved>,
    /// The tab's horizontal padding, less any inset label background's.
    padding_x: f32,
    /// The tab's fill, unless an inset label background took it.
    background: Option<gpui::Hsla>,
    disclosure_icons: DisclosureIcons,
}

impl TabsButtonData {
    fn label_color(&self) -> gpui::Hsla {
        self.style.part(PartId::new("item")).color("foreground")
    }

    fn accessory_size(&self) -> f32 {
        self.style.root().number("accessory_size")
    }

    fn accessory_gap(&self) -> f32 {
        self.style.root().number("accessory_gap")
    }
}

#[derive(Clone)]
pub struct TabsItemButtonStyle {
    /// The tab's resolved style in its state (#433; see [`resolve_tab`]): its root and `item` part.
    pub style: Arc<Resolved>,
    pub font_family: SharedString,
    pub disclosure_icons: DisclosureIcons,
}

impl TabsItemButtonStyle {
    pub fn new(style: Arc<Resolved>, font_family: SharedString) -> Self {
        Self {
            style,
            font_family,
            disclosure_icons: DisclosureIcons::menu_from_stylesheet(
                crate::theme::stylesheet::CommonStylesheet::sdk_default(),
            ),
        }
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.disclosure_icons = icons;
        self
    }
}

pub trait TabsTemplate: Send + Sync {
    fn render(
        &self,
        model: &TabsRenderModel<'_>,
        handlers: TabsTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

impl RenderModel for dyn TabsTemplate {
    type Model<'a> = TabsRenderModel<'a>;
}

pub struct ThemedTabsTemplate {
    theme: Arc<dyn TabsTheme>,
    modifiers: RootModifiers<dyn TabsTemplate>,
}

impl ThemedTabsTemplate {
    pub fn new(theme: Arc<dyn TabsTheme>) -> Self {
        Self { theme, modifiers: RootModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TabsRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

pub fn default_tabs_template() -> Arc<dyn TabsTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn TabsTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedTabsTemplate::new(default_tabs_theme()))).clone()
}

pub(super) fn template_with_modifier<F>(template: Arc<dyn TabsTemplate>, modifier: F) -> Arc<dyn TabsTemplate>
where
    F: Fn(Stateful<Div>, &TabsRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl TabsTemplate for ModifiedTemplate<dyn TabsTemplate> {
    fn render(
        &self,
        model: &TabsRenderModel<'_>,
        handlers: TabsTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply(root, model)
    }
}

impl TabsTemplate for ThemedTabsTemplate {
    fn render(
        &self,
        model: &TabsRenderModel<'_>,
        handlers: TabsTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let list_style = resolve_tabs_list(&self.theme.style(), model.enabled);
        let list = list_style.part(PartId::new("list"));
        let (list_gap, list_padding, list_background, list_border) =
            (list.number("gap"), list.number("padding"), list.painted("background"), list.painted("border"));
        let uniform_width = resolve_tabs_uniform_item_width(model, self.theme.as_ref(), window);

        let active_item = model.items.iter().find(|item| item.active);
        if let Some(motion) = model.indicator_motion {
            let state = active_item.map_or_else(InteractionState::default, |item| item.state.interaction_state());
            let active = resolve_tab(&self.theme.item_style(true, state), true, state);
            let root = active.root();
            motion.set_metrics(
                self.theme.indicator_inset().unwrap_or(active.part(PartId::new("item")).number("padding_x")),
                root.number("indicator_height"),
                root.painted("indicator"),
            );
        }

        let drop_scope = model.drag_drop.as_ref().and_then(|owner| {
            owner
                .update(cx, |tabs, _| {
                    tabs.drop_layout.configure_transition(&tabs.motion_profile().drop_layout);
                    tabs.drop_layout.set_motion_enabled(tabs.motion_enabled);
                    (tabs.drop_layout.clone(), tabs.drag_scroll.clone())
                })
                .ok()
        });
        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(list_gap))
            .p(px(list_padding))
            .rounded(px(list.number("radius")));
        if let Some((motion, scroll)) = &drop_scope {
            let scroll = scroll.clone();
            root = motion.begin_surface(root, move || scroll.offset());
        }

        if let Some(motion) = model.indicator_motion.cloned() {
            root = root.on_prepaint(move |bounds, _, _| {
                motion.set_list_bounds(bounds);
            });
        }

        if let Some(background) = list_background {
            root = root.bg(background);
        }

        if let Some(border) = list_border {
            root = root.border(px(list.number("border_width"))).border_color(border);
        }

        if let Some(baseline) = self.theme.baseline(model.enabled) {
            root = root.child(
                div()
                    .absolute()
                    .left(px(baseline.inset_x.max(0.0)))
                    .right(px(baseline.inset_x.max(0.0)))
                    .bottom(px(baseline.bottom))
                    .h(px(baseline.height.max(0.0)))
                    .bg(baseline.color),
            );
        }

        if let Some(owner) = &model.drag_drop {
            if let Ok(bound) = owner.update(cx, |tabs, cx| tabs.bind_drag_strip(root, cx)) {
                root = bound;
            } else {
                return div().id(model.id.clone());
            }
        }
        for (item, item_handlers) in model.items.iter().zip(handlers.into_item_handlers()) {
            let state = item.state.interaction_state();
            let style = resolve_tab(&self.theme.item_style(item.active, state), item.active, state);
            let mut tab = render_tab_button_with_content_padding(
                model.id,
                item,
                TabsItemButtonStyle::new(style, self.theme.font_family())
                    .disclosure_icons(model.disclosure_icons.clone()),
                self.theme.content_padding(),
                window,
                cx,
            )
            .control_group_item_handlers(item_handlers);

            if let Some(width) = uniform_width {
                tab = tab.w(px(width)).flex_none();
            }

            if !item.state.disabled {
                tab = tab.cursor_pointer();
            }

            if let Some(owner) = &model.drag_drop {
                if let Ok(bound) = owner.update(cx, |tabs, cx| {
                    let bound = tabs.bind_tab_drag(tab, item.id, item.label.clone(), self.theme.drag_style(), cx);
                    tabs.drop_layout.item(item.id.clone(), bound)
                }) {
                    root = root.child(bound);
                }
            } else {
                root = root.child(tab);
            }
        }

        if let Some(owner) = &model.drag_drop {
            if let Ok(bound) = owner.update(cx, |tabs, cx| {
                tabs.bind_tab_drop_targets(
                    root,
                    list_gap,
                    list_padding + if list_border.is_some() { 1.0 } else { 0.0 },
                    self.theme.drag_style(),
                    cx,
                )
            }) {
                root = bound;
            } else {
                return div().id(model.id.clone());
            }
        }
        if let Some(indicator) = model.indicator_motion.and_then(TabsIndicatorMotion::paint).or(model.indicator) {
            root = root.child(
                div()
                    .absolute()
                    .left(px(indicator.left))
                    .bottom(px(0.0))
                    .w(px(indicator.width))
                    .h(px(indicator.height))
                    .rounded(px(indicator.height))
                    .bg(indicator.color),
            );
        }

        if let Some((motion, _)) = drop_scope {
            root = motion.end_surface(root);
        }
        self.modifiers.apply(root, model)
    }
}

pub fn resolve_tabs_uniform_item_width(
    model: &TabsRenderModel<'_>,
    theme: &dyn TabsTheme,
    window: &mut Window,
) -> Option<f32> {
    if model.width_mode != TabsWidthMode::Uniform {
        return None;
    }

    let font_family = theme.font_family();
    let mut max_width = 0.0_f32;

    for item in &model.items {
        let state = item.state.interaction_state();
        let style = resolve_tab(&theme.item_style(item.active, state), item.active, state);
        let (root, part) = (style.root(), style.part(PartId::new("item")));
        let typography = part.typography(14.0, 20.0);
        let run = TextRun {
            len: item.label.len(),
            font: {
                let mut font = font(font_family.clone());
                font.weight = typography.weight;
                font
            },
            color: part.color("foreground"),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window.text_system().shape_line(item.label.clone(), px(typography.size), &[run], None);
        let accessory_count =
            usize::from(item.leading_accessory.is_some()) + usize::from(item.trailing_accessory.is_some());
        let accessory_width = accessory_count as f32 * root.number("accessory_size")
            + if accessory_count > 0 {
                accessory_count as f32 * root.number("accessory_gap")
            } else {
                0.0
            };
        let width = line.x_for_index(item.label.len()).as_f32() + part.number("padding_x") * 2.0 + accessory_width;
        max_width = max_width.max(width);
    }

    Some(max_width)
}

pub fn render_tabs_overlay_host<F>(
    id: impl Into<SharedString>,
    mut trigger: Stateful<Div>,
    width: Option<Pixels>,
    overlay_state: TabsOverlayState,
    overlay: Option<TabsItemOverlay>,
    on_mouse_down_out: F,
) -> AnyElement
where
    F: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
{
    let id = id.into();
    let mut root = div()
        .on_children_prepainted({
            let trigger_bounds = overlay_state.trigger_bounds.clone();
            let space = overlay_state.space.clone();
            move |bounds, window, cx| {
                if let Some(bounds) = bounds.first() {
                    *lock::mutex(&trigger_bounds) = Some(*bounds);
                }
                *lock::mutex(&space) = PopupSpace::of(window, cx);
            }
        })
        .id(id)
        .relative()
        .on_mouse_down_out(on_mouse_down_out);

    if let Some(width) = width {
        trigger = trigger.w_full();
        root = root.w(width).flex_none();
    }

    root = root.child(trigger);

    match overlay {
        Some(overlay) => {
            *lock::mutex(&overlay_state.last_overlay) = Some(TabsCachedOverlay {
                content_size: overlay.content_size,
                placement: overlay.placement,
                offset_y: overlay.offset_y,
                window_margin: overlay.window_margin,
            });
            overlay_state.set_overlay_open(true);
            let presence = overlay_state.presence_snapshot();
            let placement = resolve_tabs_overlay_placement(
                overlay_state.trigger_bounds(),
                *lock::mutex(&overlay_state.space),
                overlay.placement,
                overlay.content_size,
                overlay.offset_y,
                overlay.window_margin,
            );
            let offset = presence.adjust_offset(placement.offset, overlay.content_size);
            // Taller than the space on its side, the overlay scrolls (#433).
            let content = div().opacity(presence.opacity()).child(
                div()
                    .id("tabs-overlay-scroll")
                    .max_h(tabs_overlay_max_height(&placement, overlay.content_size))
                    .overflow_y_scroll()
                    .child(overlay.content),
            );
            let anchored_overlay = anchored()
                .snap_to_window_with_margin(overlay.window_margin)
                .anchor(placement.anchor)
                .position(placement.position)
                .offset(offset)
                .child(content);
            root = root.child(deferred(anchored_overlay).with_priority(1));
        }
        None => {
            overlay_state.set_overlay_open(false);
            let presence = overlay_state.presence_snapshot();
            if presence.should_paint()
                && let Some(cached) = lock::mutex(&overlay_state.last_overlay).clone()
            {
                let placement = resolve_tabs_overlay_placement(
                    overlay_state.trigger_bounds(),
                    *lock::mutex(&overlay_state.space),
                    cached.placement,
                    cached.content_size,
                    cached.offset_y,
                    cached.window_margin,
                );
                let offset = presence.adjust_offset(placement.offset, cached.content_size);
                // Exit keeps a sized shell while content is unavailable (`AnyElement` is not cloneable).
                let content = div()
                    .w(cached.content_size.width)
                    .h(tabs_overlay_max_height(&placement, cached.content_size))
                    .opacity(presence.opacity());
                let anchored_overlay = anchored()
                    .snap_to_window_with_margin(cached.window_margin)
                    .anchor(placement.anchor)
                    .position(placement.position)
                    .offset(offset)
                    .child(content);
                root = root.child(deferred(anchored_overlay).with_priority(1));
            }
        }
    }

    root.into_any_element()
}
fn resolve_tabs_overlay_placement(
    trigger_bounds: Option<Bounds<Pixels>>,
    space: impl Into<PopupSpace>,
    placement: TabsOverlayPlacement,
    content_size: Size<Pixels>,
    offset_y: Pixels,
    window_margin: Pixels,
) -> ResolvedPopupPlacement {
    let trigger_bounds = trigger_bounds.unwrap_or_default();
    let space: PopupSpace = space.into();
    let align = match placement {
        TabsOverlayPlacement::BelowStart => PopupAlign::Start,
        TabsOverlayPlacement::BelowCenter => PopupAlign::Center,
    };
    // Until the trigger is laid out the window size is unknown: open below, unlimited.
    let measured = space.viewport.height > px(0.0);
    let mut placement = PopupPlacementRequest::new(trigger_bounds, PopupSide::Below, space)
        .align(align)
        .flip(measured)
        .gap(offset_y)
        .content_size(Some(content_size))
        .window_margin(window_margin)
        .resolve();
    placement.scrolling &= measured;
    placement
}

/// The height the overlay may take: unlimited until the window size is known.
fn tabs_overlay_max_height(placement: &ResolvedPopupPlacement, content_size: Size<Pixels>) -> Pixels {
    if placement.scrolling {
        placement.max_size.height
    } else {
        content_size.height
    }
}

fn tab_button_template() -> Arc<dyn ButtonTemplate<TabsButtonData>> {
    static TEMPLATE: OnceLock<Arc<dyn ButtonTemplate<TabsButtonData>>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(StyledButtonTemplate::<TabsButtonData>::sdk())).clone()
}

pub fn render_tab_button(
    navigation_id: &SharedString,
    item: &TabsRenderItem<'_>,
    style: Arc<Resolved>,
    font_family: SharedString,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    render_tab_button_with_style(navigation_id, item, TabsItemButtonStyle::new(style, font_family), window, cx)
}

pub fn render_tab_button_with_style(
    navigation_id: &SharedString,
    item: &TabsRenderItem<'_>,
    style: TabsItemButtonStyle,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    render_tab_button_with_content_padding(navigation_id, item, style, None, window, cx)
}

fn render_tab_button_with_content_padding(
    navigation_id: &SharedString,
    item: &TabsRenderItem<'_>,
    style: TabsItemButtonStyle,
    content_padding: Option<(f32, f32)>,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    tab_button_template().render(
        &tab_button_model(navigation_id, item, style.style, style.font_family, style.disclosure_icons, content_padding),
        window,
        cx,
    )
}

fn tab_button_model(
    navigation_id: &SharedString,
    item: &TabsRenderItem<'_>,
    style: Arc<Resolved>,
    font_family: SharedString,
    disclosure_icons: DisclosureIcons,
    content_padding: Option<(f32, f32)>,
) -> ButtonRenderModel<TabsButtonData> {
    let part = style.part(PartId::new("item"));
    let (mut padding_x, mut background, radius) =
        (part.number("padding_x"), part.painted("background"), part.number("radius"));
    let inner_background = content_padding.map(|(x, y)| {
        let x = x.clamp(0.0, padding_x.max(0.0));
        padding_x -= x;
        (x, y.max(0.0), background.take(), radius)
    });
    let label = item.label.clone();
    let leading_accessory = item.leading_accessory.cloned();
    let trailing_accessory = item.trailing_accessory.cloned();
    let disclosure_progress = item.disclosure_progress;

    ButtonRenderModel {
        id: format!("{}-tab-{}", navigation_id, item.id).into(),
        data: TabsButtonData { style, padding_x, background, disclosure_icons },
        icon: None,
        content: Arc::new(move |model, _| {
            let mut label_content = div().flex().items_center().gap(px(model.data.accessory_gap()));
            if let Some((x, y, background, radius)) = inner_background {
                label_content = label_content.px(px(x)).py(px(y)).rounded(px(radius));
                if let Some(background) = background {
                    label_content = label_content.bg(background);
                }
            }
            if let Some(accessory) = &leading_accessory {
                label_content = label_content.child(render_accessory(
                    accessory,
                    model.data.label_color(),
                    &model.data.disclosure_icons,
                    disclosure_progress,
                    model.data.accessory_size(),
                ));
            }
            label_content = label_content.child(label.clone());
            if let Some(accessory) = &trailing_accessory {
                label_content = label_content.child(render_accessory(
                    accessory,
                    model.data.label_color(),
                    &model.data.disclosure_icons,
                    disclosure_progress,
                    model.data.accessory_size(),
                ));
            }
            label_content.into_any_element()
        }),
        role: ButtonFamilyRole::Toggle { selected: item.active },
        state: item.state.interaction_state(),
        round: false,
        radius_override: Cell::new(Some(radius)),
        elevation: false,
        // Tabs own their focus indicator; retain the dense item footprint.
        compact: true,
        switch_track_width_extra: 0.0,
        switch_track_width: None,
        switch_track_height: None,
        switch_thumb_size: None,
        switch_orientation: crate::controls::switch::SwitchOrientation::Horizontal,
        switch_track_content: None,
        switch_thumb_content: None,
        look: Some(Arc::new(move |model| tab_button_look(&model.data, font_family.clone()))),
    }
}

fn tab_button_look(data: &TabsButtonData, font_family: SharedString) -> ButtonFamilyLook {
    let part = data.style.part(PartId::new("item"));
    ButtonFamilyLook {
        focus_border: gpui::transparent_black(),
        background: data.background.unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.0)),
        foreground: data.label_color(),
        muted_foreground: data.label_color(),
        border: None,
        typography: part.typography(14.0, 20.0),
        font_family,
        radius: part.number("radius"),
        padding_x: data.padding_x,
        padding_y: 0.0,
        gap: data.accessory_gap(),
        height: part.number("height"),
        icon_size: data.accessory_size(),
        shadow: None,
    }
}

fn render_accessory(
    accessory: &TabsItemAccessory,
    color: gpui::Hsla,
    disclosure_icons: &crate::infra::icon::DisclosureIcons,
    disclosure_progress: f32,
    accessory_size: f32,
) -> gpui::AnyElement {
    match accessory {
        TabsItemAccessory::Icon(icon) => render_tab_icon_source(icon, color, accessory_size),
        TabsItemAccessory::Disclosure { .. } => {
            render_disclosure_icon(disclosure_icons, disclosure_progress, color, accessory_size)
        }
    }
}

fn render_tab_icon_source(icon: &IconSource, color: gpui::Hsla, accessory_size: f32) -> gpui::AnyElement {
    render_icon_source(icon, color, accessory_size)
}

pub(crate) fn tabs_control_group_template(
    width_mode: TabsWidthMode,
    template: Arc<dyn TabsTemplate>,
    indicator_motion: TabsIndicatorMotion,
    disclosure_icons: crate::infra::icon::DisclosureIcons,
    disclosure_progress: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<SharedString, f32>>>,
    owner: gpui::WeakEntity<super::Tabs>,
) -> ControlGroupTemplate<TabsItem> {
    Arc::new(move |model, handlers, window, cx| {
        let mut tabs_model =
            tabs_render_model(model, width_mode, &indicator_motion, &disclosure_icons, &disclosure_progress);
        tabs_model.drag_drop = Some(owner.clone());
        template.render(&tabs_model, handlers, window, cx)
    })
}

fn tabs_render_model<'a>(
    model: &'a ControlGroupRenderModel<'a, TabsItem>,
    width_mode: TabsWidthMode,
    indicator_motion: &'a TabsIndicatorMotion,
    disclosure_icons: &'a crate::infra::icon::DisclosureIcons,
    disclosure_progress: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<SharedString, f32>>>,
) -> TabsRenderModel<'a> {
    let progress = lock::mutex(disclosure_progress);
    TabsRenderModel {
        drag_drop: None,
        id: model.id,
        width_mode,
        items: model
            .items
            .iter()
            .map(|item| TabsRenderItem {
                id: item.item.id(),
                label: item.item.label_text(),
                trigger_kind: item.item.trigger_kind_value(),
                leading_accessory: item.item.leading_accessory_ref(),
                trailing_accessory: item.item.trailing_accessory_ref(),
                active: item.selected,
                enabled: item.enabled,
                state: item.state,
                disclosure_progress: progress.get(item.item.id()).copied().unwrap_or(item.item.disclosure_progress),
            })
            .collect(),
        active_id: model.selected_ids.first(),
        enabled: model.enabled,
        focus: model.focus,
        indicator: indicator_motion.paint(),
        indicator_motion: Some(indicator_motion),
        disclosure_icons,
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Anchor, Bounds, point, px, size};

    use super::*;

    #[test]
    fn overlay_below_start_anchors_to_trigger_bottom_left() {
        let placement = resolve_tabs_overlay_placement(
            Some(Bounds::new(point(px(40.0), px(10.0)), size(px(80.0), px(32.0)))),
            size(px(800.0), px(600.0)),
            TabsOverlayPlacement::BelowStart,
            size(px(200.0), px(120.0)),
            px(6.0),
            px(8.0),
        );

        assert_eq!(placement.anchor, Anchor::TopLeft);
        assert_eq!(placement.position, point(px(40.0), px(42.0)));
        assert_eq!(placement.offset, point(px(0.0), px(6.0)));
    }

    #[test]
    fn overlay_below_center_anchors_to_trigger_center_with_content_offset() {
        let placement = resolve_tabs_overlay_placement(
            Some(Bounds::new(point(px(40.0), px(10.0)), size(px(80.0), px(32.0)))),
            size(px(800.0), px(600.0)),
            TabsOverlayPlacement::BelowCenter,
            size(px(200.0), px(120.0)),
            px(6.0),
            px(8.0),
        );

        assert_eq!(placement.anchor, Anchor::TopLeft);
        assert_eq!(placement.position, point(px(80.0), px(42.0)));
        assert_eq!(placement.offset, point(px(-100.0), px(6.0)));
    }

    #[test]
    fn poisoned_overlay_locks_do_not_abort_queries() {
        let state = TabsOverlayState::new();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = state.presence.lock().unwrap();
            panic!("poison");
        }));
        assert!(!state.should_paint());
        assert!(!state.is_animating());
        assert!(state.trigger_bounds().is_none());
    }
}

#[cfg(test)]
mod height_tests {
    use gpui::{Bounds, point, px, size};

    use super::*;

    #[test]
    fn overlay_near_the_window_bottom_flips_and_limits_its_height() {
        let trigger = Some(Bounds::new(point(px(40.0), px(500.0)), size(px(80.0), px(32.0))));
        let content = size(px(200.0), px(900.0));
        let placement = resolve_tabs_overlay_placement(
            trigger,
            size(px(800.0), px(600.0)),
            TabsOverlayPlacement::BelowStart,
            content,
            px(6.0),
            px(8.0),
        );
        // Below: 600 - 8 - 538 = 54; above: 500 - 6 - 8 = 486.
        assert_eq!(placement.side, PopupSide::Above);
        assert!(placement.scrolling);
        assert_eq!(tabs_overlay_max_height(&placement, content), px(486.0));
    }

    #[test]
    fn overlay_before_layout_opens_below_unlimited() {
        let content = size(px(200.0), px(900.0));
        let placement = resolve_tabs_overlay_placement(
            None,
            gpui::Size::default(),
            TabsOverlayPlacement::BelowStart,
            content,
            px(6.0),
            px(8.0),
        );
        assert_eq!(placement.side, PopupSide::Below);
        assert_eq!(tabs_overlay_max_height(&placement, content), content.height);
    }
}
