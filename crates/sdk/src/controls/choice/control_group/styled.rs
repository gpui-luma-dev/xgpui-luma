//! The control-group frame read from `.lss` (issue #285): `control_group.lss`. Its rows are the `.lss`
//! toggle, through [`StyledButtonTemplate::look`].

use crate::style::PartRead;
use std::sync::Arc;

use xgpui_luma_style::{PartId, Resolved};

#[allow(deprecated)]
use super::theme::ControlGroupListLook;
use super::theme::{ControlGroupItemPalette, ControlGroupTheme};
use crate::controls::button::StyledButtonTemplate;
use crate::controls::button_family::ButtonFamilyRole;
use crate::style::StyleBinding;
use crate::theme::{InteractionState, MetricTokens, StandardBoxScale};

/// The control group's style for `binding` (the `control_group` control), enabled or not (#433): templates draw from
/// its root; a disabled row's opacity is the `item` part's in the disabled style.
pub fn resolve_control_group(binding: &StyleBinding, enabled: bool) -> Arc<Resolved> {
    binding.resolve(&[("disabled", !enabled)], 1.0)
}

#[allow(deprecated)]
impl ControlGroupListLook {
    /// The look `resolved` describes, with `disabled` the same group resolved disabled (for the row opacity): the
    /// compatibility path for code that still reads a `ControlGroupListLook` (#433).
    pub fn from_resolved(resolved: &Resolved, disabled: &Resolved) -> Self {
        let root = resolved.root();
        ControlGroupListLook {
            background: root.color("background"),
            border: root.color("border"),
            radius: root.number("radius"),
            padding_x: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            gap: root.number("gap"),
            vertical_gap: root.number("vertical_gap"),
            border_width: root.number("border_width"),
            item_disabled_opacity: disabled.part(PartId::new("item")).opacity(),
        }
    }
}

/// Supplies the control metrics the rows are laid out with.
pub type MetricsSource = Arc<dyn Fn() -> MetricTokens + Send + Sync>;

/// A control-group theme whose frame and rows are read from `.lss`.
pub struct StyledControlGroupTheme {
    binding: StyleBinding,
    rows: StyledButtonTemplate<()>,
    metrics: MetricsSource,
}

impl StyledControlGroupTheme {
    /// The `control_group` frame from `binding`, rows drawn by `rows` (a ghost-like toggle), laid out with
    /// `metrics`.
    pub fn new(binding: StyleBinding, rows: StyledButtonTemplate<()>, metrics: MetricsSource) -> Self {
        Self { binding, rows, metrics }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self {
            binding: StyleBinding::sdk("control_group"),
            rows: StyledButtonTemplate::sdk(),
            metrics: Arc::new(MetricTokens::default),
        }
    }
}

impl ControlGroupTheme for StyledControlGroupTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }

    fn metrics(&self) -> MetricTokens {
        (self.metrics)()
    }

    fn default_item_palette(
        &self,
        selected: bool,
        state: InteractionState,
        _scale: &StandardBoxScale,
    ) -> ControlGroupItemPalette {
        let face = self.rows.resolve_face(ButtonFamilyRole::Toggle { selected }, state, 1.0);
        let root = face.root();
        ControlGroupItemPalette {
            foreground: root.color("foreground"),
            background: root.color("background"),
            muted_foreground: root.color("muted_foreground"),
            typography: root.typography(14.0, 20.0),
            font_family: root.font_family_name(),
        }
    }
}
