#![allow(dead_code)]

mod control;
mod look;
mod model;

pub(super) use control::{PopoverPanel, PopoverPanelEvent};
pub(super) use model::PopoverPanelPlacement;

#[allow(deprecated)]
pub use look::PopoverPanelLook;
pub use look::{
    popover_panel_chrome_height, popover_panel_shown, resolve_popover_panel, show_popover_panel, wrap_popover_panel,
};
pub use model::PopoverMotion;

crate::theme::stylesheet::geometry::lss_geometry!(
    /// Where a popover opens, from `popover.lss`.
    PopoverGeometry,
    "popover",
    [button_offset_y, panel_offset_y, window_margin]
);
