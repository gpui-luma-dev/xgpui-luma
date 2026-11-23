//! Resizable panels property mappings:
//!
//! | Part          | Token              |
//! |---------------|--------------------|
//! | Divider line  | `border`           |
//! | Panel border  | `border`           |
//! | Handle grip   | `border`           |
//! | Grip emphasis | `accent` (layer)   |
//! | Disabled      | `muted-foreground` |

use crate::provenance::ResolvedColor;

#[derive(Clone, Debug)]
pub struct ResizablePanelsColorTable {
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
    pub grip: ResolvedColor,
    pub grip_emphasis: ResolvedColor,
}

impl ResizablePanelsColorTable {
    pub fn fallback() -> Self {
        Self {
            border: ResolvedColor::fallback_foreground(),
            divider: ResolvedColor::fallback_foreground(),
            grip: ResolvedColor::fallback_foreground(),
            grip_emphasis: ResolvedColor::fallback_foreground(),
        }
    }
}
