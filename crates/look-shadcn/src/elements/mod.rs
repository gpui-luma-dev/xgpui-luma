#![allow(deprecated)] // Compatibility paths and parity tests still read the deprecated Looks (#433).
pub(crate) mod badge;

pub use badge::{BadgeLook, badge_look};
pub use badge::{Badge, BadgeIconPlacement, BadgeVariant, badge_style};
