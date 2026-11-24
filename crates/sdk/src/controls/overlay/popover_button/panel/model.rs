crate::motion_profile!(PopoverMotion => popover {
    presence: crate::motion::PresenceMotion,
});

use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Bounds, Context, Entity, IntoElement, Pixels, SharedString, Size, Window};

use super::control::PopoverPanel;

pub type PopoverPanelContent = Arc<dyn Fn(&PopoverPanelRenderModel<'_>, &mut Window, &mut App) -> AnyElement>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopoverPanelPlacement {
    #[default]
    BelowStart,
    BelowCenter,
    AboveStart,
    SmartStart,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopoverPanelDismissPolicy {
    #[default]
    KeepOpen,
    CloseOnClickAway,
    CloseOnFocusLoss,
    CloseOnClickAwayOrFocusLoss,
}

#[derive(Clone)]
pub struct PopoverPanelModel {
    pub(crate) motion: crate::motion::MotionConfig<PopoverMotion>,
    pub(crate) id: SharedString,
    pub(crate) content: PopoverPanelContent,
    pub(crate) open: bool,
    pub(crate) motion_enabled: bool,
    pub(crate) anchor_bounds: Option<Bounds<Pixels>>,
    pub(crate) placement: PopoverPanelPlacement,
    pub(crate) dismiss_policy: PopoverPanelDismissPolicy,
    pub(crate) dismissible: bool,
    pub(crate) offset_y: Pixels,
    pub(crate) window_margin: Pixels,
    pub(crate) focus_on_open: bool,
    pub(crate) initial_content_size: Option<Size<Pixels>>,
}

pub struct PopoverPanelRenderModel<'a> {
    pub id: &'a SharedString,
    pub open: bool,
    pub anchor_bounds: Option<Bounds<Pixels>>,
    pub placement: PopoverPanelPlacement,
    pub dismiss_policy: PopoverPanelDismissPolicy,
    pub dismissible: bool,
    pub focused: bool,
    pub content_bounds: Option<Bounds<Pixels>>,
}

pub struct PopoverPanelBuilder {
    explicit_offset_y: bool,
    explicit_window_margin: bool,
    pub(crate) model: PopoverPanelModel,
}

pub fn new(id: impl Into<SharedString>) -> PopoverPanelBuilder {
    PopoverPanelBuilder::new(id)
}

impl PopoverPanelBuilder {
    crate::motion_methods!(PopoverMotion, model.motion);

    pub fn new(id: impl Into<SharedString>) -> Self {
        let geometry = crate::theme::stylesheet::PopoverGeometry::sdk();
        Self {
            explicit_offset_y: false,
            explicit_window_margin: false,
            model: PopoverPanelModel {
                motion: Default::default(),
                id: id.into(),
                content: Arc::new(|_, _, _| gpui::div().into_any_element()),
                open: false,
                motion_enabled: true,
                anchor_bounds: None,
                placement: PopoverPanelPlacement::BelowStart,
                dismiss_policy: PopoverPanelDismissPolicy::KeepOpen,
                dismissible: true,
                offset_y: gpui::px(geometry.panel_offset_y),
                window_margin: gpui::px(geometry.window_margin),
                focus_on_open: true,
                initial_content_size: None,
            },
        }
    }

    pub fn content<F>(mut self, content: F) -> Self
    where
        F: Fn(&PopoverPanelRenderModel<'_>, &mut Window, &mut App) -> AnyElement + 'static,
    {
        self.model.content = Arc::new(content);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.model.open = open;
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.model.motion_enabled = motion_enabled;
        self
    }

    pub fn anchor_bounds(mut self, bounds: Bounds<Pixels>) -> Self {
        self.model.anchor_bounds = Some(bounds);
        self
    }

    pub fn placement(mut self, placement: PopoverPanelPlacement) -> Self {
        self.model.placement = placement;
        self
    }

    pub fn dismiss_policy(mut self, dismiss_policy: PopoverPanelDismissPolicy) -> Self {
        self.model.dismiss_policy = dismiss_policy;
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.model.dismissible = dismissible;
        self
    }

    pub fn offset_y(mut self, offset_y: Pixels) -> Self {
        self.explicit_offset_y = true;
        self.model.offset_y = offset_y;
        self
    }

    pub fn window_margin(mut self, margin: Pixels) -> Self {
        self.explicit_window_margin = true;
        self.model.window_margin = margin;
        self
    }

    pub fn focus_on_open(mut self, focus_on_open: bool) -> Self {
        self.model.focus_on_open = focus_on_open;
        self
    }

    pub fn initial_content_size(mut self, size: Size<Pixels>) -> Self {
        self.model.initial_content_size = Some(size);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<PopoverPanel> {
        cx.new(|cx| PopoverPanel::from_builder(self, cx))
    }
}

#[cfg(test)]
mod stylesheet_tests {
    use super::*;
    use gpui::px;

    /// Placement defaults are `popover.lss`'s (#287), and an explicit zero is kept.
    #[test]
    fn panel_placement_reads_popover_lss_and_preserves_explicit_zero() {
        let sdk = |name| crate::style::sdk_number("popover", name);
        let baseline = PopoverPanelBuilder::new("baseline");
        assert_eq!(
            (baseline.model.offset_y, baseline.model.window_margin),
            (px(sdk("panel_offset_y")), px(sdk("window_margin")))
        );
        let panel = PopoverPanelBuilder::new("explicit").offset_y(px(0.0)).window_margin(px(0.0));
        assert_eq!((panel.model.offset_y, panel.model.window_margin), (px(0.0), px(0.0)));
    }
}
