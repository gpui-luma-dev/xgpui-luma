#![allow(deprecated)] // Still reads a deprecated Look until its family has a style-override replacement (#433).
pub(crate) mod badge;

pub use badge::{BadgeLook, badge_look};
pub use badge::{Badge, BadgeIconPlacement, BadgeVariant, badge_style};
