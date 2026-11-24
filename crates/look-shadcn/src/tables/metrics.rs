//! Metric tables derived from the same paint helpers and SDK scales used by rendering.
//! Inspection DTOs and display formatting remain in the companion crate.

pub(crate) mod helpers;
pub use crate::controls::accordion::{AccordionMetricTable};
pub use crate::controls::progress::{ProgressMetricTable, resolve_progress_metrics, resolve_progress_metrics_for_size};
pub use crate::controls::sidebar::{SidebarMetricTable, resolve_sidebar_metrics};
pub use crate::controls::stepper::{StepperMetricTable, resolve_stepper_metrics, resolve_stepper_metrics_for_size};
