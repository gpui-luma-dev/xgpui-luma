//! Control group list chrome — muted surface + border.
//!
//! | Part            | Token                |
//! |-----------------|----------------------|
//! | Enabled bg      | `muted`              |
//! | Disabled bg     | `muted-foreground`   |
//! | Border          | `border`             |

use crate::provenance::ResolvedColor;

#[derive(Clone, Debug)]
pub struct ControlGroupListColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
}

impl ControlGroupListColorTable {
    pub fn fallback() -> Self {
        Self { background: ResolvedColor::transparent(), border: ResolvedColor::fallback_foreground() }
    }
}
