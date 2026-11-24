//! Split view property mappings:
//!
//! | Part              | Token              |
//! |-------------------|--------------------|
//! | Separator         | `border`           |
//! | Separator (hover) | `border-hover`     |
//! | Disabled          | `muted-foreground` |

use crate::provenance::ResolvedColor;

#[derive(Clone, Debug)]
pub struct SplitViewColorTable {
    pub separator: ResolvedColor,
    pub separator_hover: ResolvedColor,
}

impl SplitViewColorTable {
    pub fn fallback() -> Self {
        Self { separator: ResolvedColor::transparent(), separator_hover: ResolvedColor::fallback_foreground() }
    }
}
