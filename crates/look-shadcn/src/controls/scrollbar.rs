//! Scrollbar property mappings for shadcn-inspired controls:
//!
//! | Part  | Token              |
//! |-------|--------------------|
//! | Track | transparent (`Ghost`) / muted (`Soft`) |
//! | Thumb | `border`           |
//! | Hover | `border` (no change) |
//! | Press | `border` (darkened)  |

#[cfg(all(test, feature = "test-support"))]
#[path = "scroll_container_tests.rs"]
mod scroll_container_tests;

/// Native common geometry owned by this control.
pub(crate) fn native_scrollbar_stylesheet() -> xgpui_luma::theme::stylesheet::ScrollbarStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ScrollbarStylesheet {
        geometry: ScrollbarGeometryStylesheet { length_h: Some(260.0), length_v: Some(180.0), ..Default::default() },
        sizes: ["sm", "md", "lg"]
            .into_iter()
            .map(|key| (key.into(), ScrollbarGeometryStylesheet::default()))
            .collect(),
    }
}
