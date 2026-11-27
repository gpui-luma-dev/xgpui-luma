use gpui::{Pixels, px};

/// Navigation presentation; width and visibility belong to the host.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SidebarPresentation {
    #[default]
    Expanded,
    Icons,
}

/// Typed metric scale for sidebar layout. Resolved by look crates (e.g. Shadcn).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarMetricScale {
    pub width_expanded: Pixels,
    pub width_icon_rail: Pixels,
    pub width_mobile: Pixels,
    pub item_height: Pixels,
    pub icon_size: Pixels,
    pub rail_hit_width: Pixels,
    pub popover_offset: Pixels,
}

impl Default for SidebarMetricScale {
    /// The SDK's sidebar dimensions, from its `sidebar.lss`.
    fn default() -> Self {
        Self::from_geometry(&SidebarGeometry::sdk())
    }
}

impl SidebarMetricScale {
    /// Capture resolved sidebar dimensions; explicit host metrics can be edited afterward.
    pub fn from_geometry(geometry: &SidebarGeometry) -> Self {
        Self {
            width_expanded: px(geometry.width_expanded),
            width_icon_rail: px(geometry.width_icon_rail),
            width_mobile: px(geometry.width_mobile),
            item_height: px(geometry.item_height),
            icon_size: px(geometry.item_icon_size),
            rail_hit_width: px(geometry.rail_hit_width),
            popover_offset: px(geometry.popover_offset),
        }
    }
}

crate::theme::stylesheet::geometry::lss_geometry!(
    /// The sidebar's shell layout, from `sidebar.lss`.
    SidebarGeometry,
    "sidebar",
    [
        section_height,
        item_height,
        item_padding_x,
        item_gap,
        item_icon_size,
        width_expanded,
        width_icon_rail,
        width_mobile,
        rail_hit_width,
        popover_offset,
        container_gap,
        region_gap,
        header_padding_bottom,
        footer_padding_top,
        title_gap,
        title_padding_bottom,
        section_padding_top,
        disclosure_icon_size,
        rail_submenu_offset_x,
        window_margin,
        engine_item_height,
        engine_section_height,
        engine_item_padding_x,
        rail_branch_indicator_size,
        rail_branch_overhang,
        subtitle_opacity
    ]
);
