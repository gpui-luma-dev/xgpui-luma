mod styled;
pub use styled::{StyledSliderTheme, styled_slider_look};
mod constraints;
mod control;
mod domain;
mod input;
mod layout;
mod model;
mod segments;
mod template;
pub(crate) mod theme;
mod thumbs;

pub use constraints::{clamp_and_snap_value, normalize_intervals, step_allowed_value};
pub use segments::build_track_segments;
pub use control::{SliderControl, SliderDrag, SliderEvent};
pub use domain::DomainTrackRenderer;
pub use input::{SliderInputStrategy, angle_for_percentage, wrap_and_snap, wrap_value};
pub use model::{
    RadialHitTarget, SliderBuilder, SliderModel, SliderOrientation, SliderRenderModel, SliderThumbPolicy,
    SliderThumbRole, SliderThumbValue, SliderValueMapping, ThumbId, TrackPresentation, TrackSegment, TrackSegmentKind,
};
pub use template::{
    SliderBoundsHandler, SliderDragMoveHandler, SliderHoverHandler, SliderMouseDownHandler, SliderMouseMoveHandler,
    SliderMouseUpHandler, SliderTemplate, SliderTemplateHandlers, SliderTemplateModifier, SliderThumbMouseDownHandler,
    ThemedAngularDialTemplate, ThemedCircularRingTemplate, ThemedSliderTemplate, default_angular_dial_template,
    default_circular_ring_template, default_slider_template,
};
pub use theme::{DefaultSliderTheme, SliderLook, SliderTheme, default_slider_theme};

pub use layout::{display_position, segment_corner_radii, segment_display_span};
pub use template::{
    SliderInteractionHandlers, attach_linear_interaction, attach_radial_interaction, attach_thumb_drag,
    render_domain_track_layer, track_bounds_canvas,
};

use gpui::{Entity, SharedString};

pub type Slider = Entity<SliderControl>;

pub fn new(id: impl Into<SharedString>) -> SliderBuilder {
    SliderBuilder::new(id)
}

mod callback;
pub use callback::SliderEventExt;

pub use model::SliderMotion;
