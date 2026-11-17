//! Choice and composition presets.

pub mod checkbox;
pub mod control_group;
pub mod icon_group;
pub mod radio_button;
pub mod radio_group;
pub mod switch;
pub mod toggle;
pub mod toolbar;

#[cfg(all(test, feature = "test-support"))]
mod locale_tests;
