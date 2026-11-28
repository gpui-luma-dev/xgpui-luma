//! Navigation sidebar property mappings (shadcn / tweakcn sidebar tokens):
//!
//! | Part              | Token                              |
//! |-------------------|------------------------------------|
//! | Container bg      | `sidebar`                          |
//! | Container fg      | `sidebar-foreground`               |
//! | Container border  | `sidebar-border`                   |
//! | Section label     | `muted-foreground`                 |
//! | Item fg           | `sidebar-foreground`               |
//! | Item hover bg     | `accent`                           |
//! | Item hover fg     | `accent-foreground`                |
//! | Selected bg       | `sidebar-primary`                  |
//! | Selected fg       | `sidebar-primary-foreground`       |
//! | Focus ring        | `sidebar-ring`                     |

#[cfg(all(test, feature = "test-support"))]
use crate::mode::ShadcnModeTokens;

#[cfg(all(test, feature = "test-support"))]
/// A standalone sidebar frame's colors, from the look's `sidebar.lss` (`container_background`,
/// `container_border`), and the large radius.
pub fn sidebar_container_look(mode: &ShadcnModeTokens) -> SidebarContainerLook {
    let resolved =
        xgpui_luma::style::StyleBinding::new("sidebar", xgpui_luma::style::ThemeSource::Fixed(mode.style_theme()))
            .resolve(&[], 1.0);
    let color = |name| {
        resolved
            .root()
            .value(name)
            .and_then(xgpui_luma::style::lss::Value::as_color)
            .map(gpui::Hsla::from)
            .unwrap_or_else(gpui::transparent_black)
    };
    SidebarContainerLook {
        background: color("container_background"),
        border: color("container_border"),
        radius: mode.radii.large,
    }
}

#[cfg(all(test, feature = "test-support"))]
/// Theme colors used by standalone sidebar frames, separate from navigation items.
#[derive(Clone, Copy, Debug)]
pub struct SidebarContainerLook {
    pub background: gpui::Hsla,
    pub border: gpui::Hsla,
    pub radius: f32,
}
