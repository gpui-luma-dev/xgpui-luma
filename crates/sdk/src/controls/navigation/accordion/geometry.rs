//! Measure whole item layout boxes, including dividers and scroll positioning.
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels,
    SharedString, Window,
};
pub(super) type ItemBounds = Rc<RefCell<HashMap<SharedString, Bounds<Pixels>>>>;
struct MeasuredItem {
    child: AnyElement,
    key: SharedString,
    bounds: ItemBounds,
}
pub(super) fn measure(bounds: ItemBounds, key: SharedString, child: impl IntoElement) -> AnyElement {
    MeasuredItem { child: child.into_any_element(), key, bounds }.into_any_element()
}
impl Element for MeasuredItem {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.bounds.borrow_mut().insert(self.key.clone(), bounds);
        self.child.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}
