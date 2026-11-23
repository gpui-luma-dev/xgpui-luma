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
