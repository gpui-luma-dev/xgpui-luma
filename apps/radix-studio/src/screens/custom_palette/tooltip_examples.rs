//! Optional tooltip customization examples, kept out of ordinary control setup.
use std::time::Duration;
use xgpui_luma::controls::tooltip::{Tooltip, bubble};

/// Once-only help with a shortcut and a local template override.
pub fn search_help() -> Tooltip {
    Tooltip::new("Search the component collection")
        .shortcut("Enter")
        .delay(Duration::from_millis(300))
        .show_once()
        .template(|model| {
            let mut model = model.clone();
            model.radius = 12.0;
            model.padding = 16.0;
            model.max_width = model.max_width.min(200.0);
            bubble(&model)
        })
}
