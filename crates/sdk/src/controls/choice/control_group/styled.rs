//! The control-group frame read from `.lss` (issue #285): `control_group.lss`. Its rows are the `.lss`
//! toggle, through [`StyledButtonTemplate::look`].

use crate::style::PartRead;
use std::sync::Arc;

use xgpui_luma_style::PartId;

use super::theme::{ControlGroupItemPalette, ControlGroupListLook, ControlGroupTheme};
use crate::controls::button::StyledButtonTemplate;
use crate::controls::button_family::ButtonFamilyRole;
use crate::style::StyleBinding;
use crate::theme::{InteractionState, MetricTokens, StandardBoxScale};

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
    fn resolve_list(&self, enabled: bool) -> ControlGroupListLook {
        let resolved = self.binding.resolve(&[("disabled", !enabled)], 1.0);
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
            item_disabled_opacity: self.binding.resolve(&[("disabled", true)], 1.0).part(PartId::new("item")).opacity(),
        }
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
        let look = self.rows.look(ButtonFamilyRole::Toggle { selected }, state, 1.0);
        ControlGroupItemPalette {
            foreground: look.foreground,
            background: look.background,
            muted_foreground: look.muted_foreground,
            typography: look.typography,
            font_family: look.font_family,
        }
    }
}
