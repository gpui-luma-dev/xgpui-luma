//! Accordion property mappings:
//!
//! | Part              | Token                              |
//! |-------------------|------------------------------------|
//! | Trigger label     | `foreground`                       |
//! | Trigger hover bg  | `accent`                           |
//! | Trigger hover fg  | `accent-foreground` (paired)       |
//! | Disabled label    | `muted-foreground`                 |
//! | Chevron           | `muted-foreground` / hover fg      |
//! | Item border       | `border`                           |
//! | Content label     | `foreground`                       |

use crate::provenance::ResolvedColor;

#[derive(Clone, Debug)]
pub struct AccordionTriggerColorTable {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
    pub border_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl AccordionTriggerColorTable {
    pub fn fallback() -> Self {
        Self {
            foreground: ResolvedColor::fallback_foreground(),
            icon_color: ResolvedColor::fallback_foreground(),
            chevron_color: ResolvedColor::fallback_foreground(),
            border_color: ResolvedColor::fallback_foreground(),
            background: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AccordionContentColorTable {
    pub foreground: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl AccordionContentColorTable {
    pub fn fallback() -> Self {
        Self { foreground: ResolvedColor::fallback_foreground(), background: None }
    }
}

#[derive(Clone, Debug)]
pub struct AccordionMetricTable {
    pub disabled_opacity: crate::ResolvedMetric,
    pub drag_handle_padding: crate::ResolvedMetric,
    pub trigger_height: crate::ResolvedMetric,
    pub padding_x: crate::ResolvedMetric,
    pub padding_y: crate::ResolvedMetric,
    pub content_padding_y: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
    pub item_gap: crate::ResolvedMetric,
    pub inner_gap: crate::ResolvedMetric,
    pub icon_size: crate::ResolvedMetric,
    pub chevron_size: crate::ResolvedMetric,
}

impl AccordionMetricTable {
    /// Converts the complete resolved table to the SDK layout contract.
    pub fn scale(&self) -> xgpui_luma::controls::accordion::AccordionScale {
        xgpui_luma::controls::accordion::AccordionScale {
            disabled_opacity: self.disabled_opacity.value_px,
            drag_handle_padding: self.drag_handle_padding.value_px,
            trigger_height: self.trigger_height.value_px,
            padding_x: self.padding_x.value_px,
            padding_y: self.padding_y.value_px,
            content_padding_y: self.content_padding_y.value_px,
            radius: self.radius.value_px,
            item_gap: self.item_gap.value_px,
            inner_gap: self.inner_gap.value_px,
            icon_size: self.icon_size.value_px,
            chevron_size: self.chevron_size.value_px,
        }
    }
}
