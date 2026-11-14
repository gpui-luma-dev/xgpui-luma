use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Div, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, Pixels,
    SharedString, Stateful, StatefulInteractiveElement, Window, div, prelude::*,
};

use super::model::{
    ControlGroupChromeModel, ControlGroupItemLike, ControlGroupItemRenderModel, ControlGroupLayout,
    ControlGroupRenderModel, ControlSelectionMode,
};
use super::theme::{ControlGroupTheme, default_control_group_theme};
use super::themed_template::{ThemedControlGroupTemplate, themed_control_group_template};
use crate::infra::ElementExt;

pub type ControlGroupClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ControlGroupHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type ControlGroupMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type ControlGroupMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type ControlGroupBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;

pub struct ControlGroupTemplateHandlers {
    pub item_bounds: Vec<ControlGroupBoundsHandler>,
    pub item_hovers: Vec<ControlGroupHoverHandler>,
    pub item_mouse_downs: Vec<ControlGroupMouseDownHandler>,
    pub item_mouse_ups: Vec<ControlGroupMouseUpHandler>,
    pub item_mouse_up_outs: Vec<ControlGroupMouseUpHandler>,
    pub item_clicks: Vec<ControlGroupClickHandler>,
}

pub struct ControlGroupItemHandlers {
    pub bounds: ControlGroupBoundsHandler,
    pub hover: ControlGroupHoverHandler,
    pub mouse_down: ControlGroupMouseDownHandler,
    pub mouse_up: ControlGroupMouseUpHandler,
    pub mouse_up_out: ControlGroupMouseUpHandler,
    pub click: ControlGroupClickHandler,
}

impl ControlGroupTemplateHandlers {
    pub fn into_item_handlers(self) -> impl Iterator<Item = ControlGroupItemHandlers> {
        let Self { item_bounds, item_hovers, item_mouse_downs, item_mouse_ups, item_mouse_up_outs, item_clicks } = self;

        item_bounds
            .into_iter()
            .zip(item_hovers)
            .zip(item_mouse_downs)
            .zip(item_mouse_ups)
            .zip(item_mouse_up_outs)
            .zip(item_clicks)
            .map(|(((((bounds, hover), mouse_down), mouse_up), mouse_up_out), click)| ControlGroupItemHandlers {
                bounds,
                hover,
                mouse_down,
                mouse_up,
                mouse_up_out,
                click,
            })
    }
}

pub trait ControlGroupItemHandlerExt: ParentElement + StatefulInteractiveElement + Sized {
    fn control_group_item_handlers(self, handlers: ControlGroupItemHandlers) -> Self;
}

impl<E> ControlGroupItemHandlerExt for E
where
    E: ParentElement + StatefulInteractiveElement + Sized,
{
    fn control_group_item_handlers(self, handlers: ControlGroupItemHandlers) -> Self {
        self.on_prepaint(move |bounds, window, cx| {
            (handlers.bounds)(&bounds, window, cx);
        })
        .on_hover(handlers.hover)
        .on_mouse_down(MouseButton::Left, handlers.mouse_down)
        .on_mouse_up(MouseButton::Left, handlers.mouse_up)
        .on_mouse_up_out(MouseButton::Left, handlers.mouse_up_out)
        .on_click(handlers.click)
    }
}

pub type ControlGroupItemTemplate<T> = Arc<
    dyn for<'a> Fn(&ControlGroupItemRenderModel<'a, T>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
>;

pub type ControlGroupItemElementTemplate<T> = Arc<
    dyn for<'a> Fn(
            &ControlGroupItemRenderModel<'a, T>,
            Option<&ControlGroupItemTemplate<T>>,
            &mut Window,
            &mut App,
        ) -> Stateful<Div>
        + Send
        + Sync
        + 'static,
>;

pub struct ControlGroupItemElement {
    pub id: SharedString,
    pub label: SharedString,
    pub index: usize,
    pub selected: bool,
    pub enabled: bool,
    element: Option<AnyElement>,
}

impl ControlGroupItemElement {
    pub fn into_element(mut self) -> AnyElement {
        self.element.take().unwrap_or_else(|| div().into_any_element())
    }
}

pub struct ControlGroupItemElements {
    items: Vec<ControlGroupItemElement>,
}

impl ControlGroupItemElements {
    pub fn take(&mut self, id: &str) -> AnyElement {
        let Some(index) = self.items.iter().position(|item| item.id.as_ref() == id) else {
            return div().into_any_element();
        };

        self.items.remove(index).into_element()
    }

    pub fn take_index(&mut self, index: usize) -> AnyElement {
        let Some(position) = self.items.iter().position(|item| item.index == index) else {
            return div().into_any_element();
        };

        self.items.remove(position).into_element()
    }

    pub fn into_elements(self) -> impl Iterator<Item = AnyElement> {
        self.items.into_iter().map(ControlGroupItemElement::into_element)
    }
}

pub fn make_control_group_item_template<T, F, E>(template: F) -> ControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + 'static,
    F: for<'a> Fn(&ControlGroupItemRenderModel<'a, T>, &mut Window, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, window, cx| template(model, window, cx).into_any_element())
}

pub fn make_control_group_item_element_template<T, F>(template: F) -> ControlGroupItemElementTemplate<T>
where
    T: ControlGroupItemLike + 'static,
    F: for<'a> Fn(
            &ControlGroupItemRenderModel<'a, T>,
            Option<&ControlGroupItemTemplate<T>>,
            &mut Window,
            &mut App,
        ) -> Stateful<Div>
        + Send
        + Sync
        + 'static,
{
    Arc::new(template)
}

pub type ControlGroupItemTemplateModifier<T> = Box<
    dyn for<'a> Fn(AnyElement, &ControlGroupItemRenderModel<'a, T>, &mut Window, &mut App) -> AnyElement
        + Send
        + Sync
        + 'static,
>;

struct ModifiedControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    base: ControlGroupItemTemplate<T>,
    modifiers: Vec<ControlGroupItemTemplateModifier<T>>,
}

impl<T> ModifiedControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    fn new(base: ControlGroupItemTemplate<T>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(AnyElement, &ControlGroupItemRenderModel<'a, T>, &mut Window, &mut App) -> AnyElement
            + Send
            + Sync
            + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn into_template(self) -> ControlGroupItemTemplate<T> {
        let base = self.base;
        let modifiers = self.modifiers;
        Arc::new(move |model, window, cx| {
            let mut element = base(model, window, cx);
            for modifier in &modifiers {
                element = modifier(element, model, window, cx);
            }
            element
        })
    }
}

pub(super) fn item_template_with_modifier<T, F>(
    template: ControlGroupItemTemplate<T>,
    modifier: F,
) -> ControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + 'static,
    F: for<'a> Fn(AnyElement, &ControlGroupItemRenderModel<'a, T>, &mut Window, &mut App) -> AnyElement
        + Send
        + Sync
        + 'static,
{
    ModifiedControlGroupItemTemplate::new(template).with_modifier(modifier).into_template()
}

pub type ControlGroupTemplate<T> = Arc<
    dyn for<'a> Fn(
            &ControlGroupRenderModel<'a, T>,
            ControlGroupTemplateHandlers,
            &mut Window,
            &mut App,
        ) -> Stateful<Div>
        + Send
        + Sync
        + 'static,
>;

pub type ControlGroupItemLayout<T> = Arc<
    dyn for<'a> Fn(ControlGroupItemElements, &ControlGroupRenderModel<'a, T>, &mut Window, &mut App) -> AnyElement
        + Send
        + Sync
        + 'static,
>;

pub fn default_control_group_template<T>() -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    control_group_template_with_theme(default_control_group_theme())
}

pub fn control_group_template_with_theme<T>(theme: Arc<dyn ControlGroupTheme>) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    wrap_themed_control_group_template(ThemedControlGroupTemplate::new(theme))
}

fn wrap_themed_control_group_template<T>(themed: ThemedControlGroupTemplate) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    let themed = Arc::new(themed);
    Arc::new(move |model, handlers, window, cx| themed.render(model, handlers, window, cx))
}

pub(super) fn modified_control_group_template<T, F>(
    template: ControlGroupTemplate<T>,
    modifier: F,
) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
    F: Fn(Stateful<Div>, &ControlGroupChromeModel) -> Stateful<Div> + Send + Sync + 'static,
{
    let modifier = Arc::new(modifier);
    Arc::new(move |model, handlers, window, cx| {
        let chrome = ControlGroupChromeModel::from(model);
        let root = template(model, handlers, window, cx);
        modifier(root, &chrome)
    })
}

fn default_themed_control_group_template() -> Arc<ThemedControlGroupTemplate> {
    static TEMPLATE: OnceLock<Arc<ThemedControlGroupTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(themed_control_group_template())).clone()
}

pub fn shared_control_group_template<T>() -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    let themed = default_themed_control_group_template();
    Arc::new(move |model, handlers, window, cx| themed.render(model, handlers, window, cx))
}

pub fn control_group_item_layout_template<T, F, E>(layout: F) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
    F: for<'a> Fn(ControlGroupItemElements, &ControlGroupRenderModel<'a, T>, &mut Window, &mut App) -> E
        + Send
        + Sync
        + 'static,
    E: IntoElement + 'static,
{
    let layout: ControlGroupItemLayout<T> =
        Arc::new(move |options, model, window, cx| layout(options, model, window, cx).into_any_element());

    Arc::new(move |model, handlers, window, cx| {
        let options = render_control_group_item_elements(model, handlers, window, cx);
        div().id(model.id.clone()).child(layout(options, model, window, cx))
    })
}

fn default_item_content<T>(model: &ControlGroupItemRenderModel<'_, T>) -> AnyElement
where
    T: ControlGroupItemLike + 'static,
{
    let active_marker = if model.state.focus_visible {
        ">"
    } else if model.active {
        "*"
    } else {
        " "
    };

    let selection_marker = match model.selection_mode {
        ControlSelectionMode::SingleRequired | ControlSelectionMode::SingleAllowNone => {
            if model.selected {
                "(*)"
            } else {
                "( )"
            }
        }
        ControlSelectionMode::Multiple => {
            if model.selected {
                "[x]"
            } else {
                "[ ]"
            }
        }
    };

    // Glyph advances in proportional fonts must not move the label when focus or
    // selection changes. Keep the two marker columns independent of their paint.
    div()
        .flex()
        .items_center()
        .child(div().w(gpui::rems(1.0)).flex_none().overflow_hidden().whitespace_nowrap().child(active_marker))
        .child(div().w(gpui::rems(2.0)).flex_none().overflow_hidden().whitespace_nowrap().child(selection_marker))
        .child(model.item.label().clone())
        .into_any_element()
}

fn default_item_element<T>(
    model: &ControlGroupItemRenderModel<'_, T>,
    item_template: Option<&ControlGroupItemTemplate<T>>,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div>
where
    T: ControlGroupItemLike + 'static,
{
    let content = if let Some(item_template) = item_template {
        item_template(model, window, cx)
    } else {
        default_item_content(model)
    };

    div().id((model.item.id().clone(), 0usize)).flex().child(content)
}

pub fn render_control_group_item_elements<T>(
    model: &ControlGroupRenderModel<'_, T>,
    handlers: ControlGroupTemplateHandlers,
    window: &mut Window,
    cx: &mut App,
) -> ControlGroupItemElements
where
    T: ControlGroupItemLike + 'static,
{
    let opacity = model.stylesheet.control_group.geometry.disabled_opacity.unwrap_or_default();
    render_control_group_item_elements_with_opacity(model, handlers, window, cx, opacity)
}

fn render_control_group_item_elements_with_opacity<T>(
    model: &ControlGroupRenderModel<'_, T>,
    handlers: ControlGroupTemplateHandlers,
    window: &mut Window,
    cx: &mut App,
    disabled_opacity: f32,
) -> ControlGroupItemElements
where
    T: ControlGroupItemLike + 'static,
{
    let mut items = Vec::with_capacity(model.items.len());

    for (item, item_handlers) in model.items.iter().zip(handlers.into_item_handlers()) {
        let mut element = if let Some(item_element_template) = model.item_element_template {
            item_element_template(item, model.item_template, window, cx)
        } else {
            default_item_element(item, model.item_template, window, cx)
        }
        .control_group_item_handlers(item_handlers);

        if item.enabled {
            element = element.cursor_pointer();
        } else if model.item_element_template.is_none() {
            element = element.opacity(disabled_opacity);
        }

        if model.scroll_handle.is_some() {
            element = element.flex_none();
        }

        if model.layout == ControlGroupLayout::Horizontal {
            element = if model.scroll_handle.is_some() {
                element.flex_none().w_auto().h_full()
            } else {
                element.flex_1().h_full()
            };
        }

        items.push(ControlGroupItemElement {
            id: item.item.id().clone(),
            label: item.item.label().clone(),
            index: item.index,
            selected: item.selected,
            enabled: item.enabled,
            element: Some(element.into_any_element()),
        });
    }

    ControlGroupItemElements { items }
}

pub(crate) fn render_control_group_items<T>(
    model: &ControlGroupRenderModel<'_, T>,
    handlers: ControlGroupTemplateHandlers,
    window: &mut Window,
    cx: &mut App,
    disabled_opacity: f32,
) -> Vec<AnyElement>
where
    T: ControlGroupItemLike + 'static,
{
    render_control_group_item_elements_with_opacity(model, handlers, window, cx, disabled_opacity)
        .into_elements()
        .collect()
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn default_group_markers_keep_intrinsic_bounds_across_focus_states() {
    use super::model::ControlGroupItem;
    use crate::infra::state::CompositeItemState;
    let item = ControlGroupItem::new("fallback").label("Fallback item");
    let id = "group".into();
    let stylesheet = crate::theme::stylesheet::CommonStylesheet::sdk_default();
    crate::infra::focus_adorner::testing::assert_stable_layout(move |state, selected, _, _| {
        default_item_content(&ControlGroupItemRenderModel {
            group_id: &id,
            stylesheet,
            layout: ControlGroupLayout::Vertical,
            item: &item,
            index: 0,
            sibling_count: 1,
            selected,
            selection_progress: if selected { 1.0 } else { 0.0 },
            active: state.focused,
            enabled: !state.disabled,
            state: CompositeItemState {
                active: state.focused,
                focus_visible: state.focused,
                selected,
                ..Default::default()
            },
            selection_mode: ControlSelectionMode::Multiple,
            foreground: None,
        })
    });
}
