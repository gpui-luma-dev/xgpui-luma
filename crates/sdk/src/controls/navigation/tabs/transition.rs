//! Application-defined content transitions with SDK-owned scheduling and lifetime.
use std::sync::Arc;
use gpui::{AnyElement, App, Bounds, FocusHandle, IntoElement, Pixels, SharedString, Window};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TabsTransitionDirection {
    Forward,
    Backward,
}

/// Content sizing is independent of the chosen transition.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsContentLayout {
    /// Preserve the existing layout: body slots flex, stacked Tabs follows content.
    #[default]
    Auto,
    /// Let in-flow content determine the height, including separately placed slots.
    Content,
    /// Fill a bounded flex viewport (required for absolutely positioned effects).
    Fill,
}

/// Selection notification, delivered in order even if multiple selections happen
/// between renders. Use `previous_progress` to sample the old visual state before
/// retargeting (1 means the prior transition settled). A nonanimated change must
/// settle immediately; it is sent on initialization, item replacement and motion
/// disabling as well as zero-duration selection changes.
#[derive(Clone, Debug)]
pub struct TabsTransitionChange {
    pub previous_id: Option<SharedString>,
    pub active_id: Option<SharedString>,
    pub previous_progress: f32,
    pub direction: TabsTransitionDirection,
    pub animated: bool,
}

/// The SDK supplies one scoped element per retained panel. Use `id` for your
/// positioning wrapper; the internal focus scope has its own distinct ID.
pub struct TabsTransitionPanel {
    pub id: SharedString,
    pub element: AnyElement,
}

pub struct TabsTransitionFrame {
    pub incoming: Option<TabsTransitionPanel>,
    /// All retiring panels, including those retained across interruptions.
    /// Released on completion, cancellation, or content/item removal.
    pub outgoing: Vec<TabsTransitionPanel>,
    /// Eased progress in 0..=1, or 1 when settled/disabled.
    pub progress: f32,
    pub direction: TabsTransitionDirection,
    pub animating: bool,
    /// Last measured bounds, only available with `measure_viewport(true)`.
    /// None before first layout. Relative lengths need no measurement/redraw.
    pub viewport: Option<Bounds<Pixels>>,
}

/// One stateful renderer per Tabs instance. Selection notifications precede the
/// next render; retarget from the old poses using `previous_progress` rather than
/// resetting visual positions. Paint incoming last when panels overlap.
pub trait TabsTransitionRenderer: 'static {
    fn selection_changed(&mut self, _change: TabsTransitionChange) {}
    fn render(&mut self, frame: TabsTransitionFrame, window: &mut Window, cx: &mut App) -> AnyElement;
}

type Renderer = dyn Fn(TabsTransitionFrame, &mut Window, &mut App) -> AnyElement;
struct ClosureRenderer(Arc<Renderer>);
impl TabsTransitionRenderer for ClosureRenderer {
    fn render(&mut self, frame: TabsTransitionFrame, window: &mut Window, cx: &mut App) -> AnyElement {
        (self.0)(frame, window, cx)
    }
}
type FocusTarget = dyn Fn(&SharedString, &mut Window, &mut App) -> Option<FocusHandle>;

/// Pluggable content presentation. The SDK schedules only the body, clips only
/// Fill layouts, and shields retiring panels individually; incoming stays usable.
/// If focus is inside an inactive panel, it returns to navigation. Override the
/// destination with `focus_target`; unrelated focus is always preserved.
/// Without a custom transition the existing incoming fade is unchanged.
#[derive(Clone)]
pub struct TabsContentTransition {
    factory: Arc<dyn Fn() -> Box<dyn TabsTransitionRenderer>>,
    pub(super) measure: bool,
    pub(super) focus: Option<Arc<FocusTarget>>,
}
impl TabsContentTransition {
    /// Simple stateless effect. Use `with_factory` for interruption-aware effects.
    /// ```no_run
    /// use gpui::{div, prelude::*};
    /// use xgpui_luma::controls::tabs::{Tabs, TabsContentTransition};
    /// let effect = TabsContentTransition::new(|frame, _, _| {
    ///     div().opacity(frame.progress).children(frame.incoming.map(|panel| panel.element))
    /// });
    /// let builder = Tabs::new("pages").content_transition(effect);
    /// ```
    pub fn new<F, E>(renderer: F) -> Self
    where
        F: Fn(TabsTransitionFrame, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        let renderer: Arc<Renderer> = Arc::new(move |frame, window, cx| renderer(frame, window, cx).into_any_element());
        Self::with_factory(move || ClosureRenderer(renderer.clone()))
    }
    /// Creates independent effect state for every spawned Tabs, even when a
    /// transition configuration is cloned or reused.
    pub fn with_factory<R: TabsTransitionRenderer>(factory: impl Fn() -> R + 'static) -> Self {
        Self { factory: Arc::new(move || Box::new(factory())), measure: false, focus: None }
    }
    /// Opt into bounds measurement and the post-layout redraw it requires.
    pub fn measure_viewport(mut self, enabled: bool) -> Self {
        self.measure = enabled;
        self
    }
    /// Override where focus leaves an inactive panel. Returning None falls back
    /// to the native navigation focus handle. This provider may be queried each frame;
    /// querying it does not move unrelated focus.
    pub fn focus_target(
        mut self,
        target: impl Fn(&SharedString, &mut Window, &mut App) -> Option<FocusHandle> + 'static,
    ) -> Self {
        self.focus = Some(Arc::new(target));
        self
    }
    pub(super) fn create(&self) -> Box<dyn TabsTransitionRenderer> {
        (self.factory)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    type SelectionLog = Rc<RefCell<Vec<(Option<SharedString>, Option<SharedString>)>>>;
    struct Recorder {
        selected: Option<SharedString>,
        log: SelectionLog,
    }
    impl TabsTransitionRenderer for Recorder {
        fn selection_changed(&mut self, change: TabsTransitionChange) {
            self.log.borrow_mut().push((self.selected.clone(), change.active_id.clone()));
            self.selected = change.active_id;
        }
        fn render(&mut self, _: TabsTransitionFrame, _: &mut Window, _: &mut App) -> AnyElement {
            gpui::div().into_any_element()
        }
    }
    #[test]
    fn cloned_configuration_creates_independent_renderer_state() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let effect = TabsContentTransition::with_factory({
            let log = log.clone();
            move || Recorder { selected: None, log: log.clone() }
        });
        let mut first = effect.create();
        let mut second = effect.clone().create();
        let change = |id: &str| TabsTransitionChange {
            previous_id: None,
            active_id: Some(id.to_owned().into()),
            previous_progress: 1.0,
            direction: TabsTransitionDirection::Forward,
            animated: true,
        };
        first.selection_changed(change("a"));
        second.selection_changed(change("b"));
        first.selection_changed(change("c"));
        assert_eq!(
            *log.borrow(),
            vec![(None, Some("a".into())), (None, Some("b".into())), (Some("a".into()), Some("c".into()))]
        );
    }
}
