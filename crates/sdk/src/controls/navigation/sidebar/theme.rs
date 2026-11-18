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
    fn default() -> Self {
        Self::from_stylesheet(crate::theme::stylesheet::CommonStylesheet::sdk_default())
    }
}

impl SidebarMetricScale {
    /// Capture resolved sidebar dimensions; explicit host metrics can be edited afterward.
    pub fn from_stylesheet(stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        let base = crate::theme::stylesheet::CommonStylesheet::sdk_default().sidebar.authored_geometry();
        let geometry = stylesheet.sidebar.authored_geometry().with_fallback(&base);
        Self {
            width_expanded: px(geometry.width_expanded.unwrap_or_default()),
            width_icon_rail: px(geometry.width_icon_rail.unwrap_or_default()),
            width_mobile: px(geometry.width_mobile.unwrap_or_default()),
            item_height: px(geometry.item_height.unwrap_or_default()),
            icon_size: px(geometry.item_icon_size.unwrap_or_default()),
            rail_hit_width: px(geometry.rail_hit_width.unwrap_or_default()),
            popover_offset: px(geometry.popover_offset.unwrap_or_default()),
        }
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    SidebarStylesheet,
    SidebarGeometryStylesheet,
    SidebarGeometry,
    ResolvedSidebarGeometry,
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
        engine_item_padding_x, rail_branch_indicator_size, rail_branch_overhang
    ],
    unit[subtitle_opacity, disabled_opacity]
);

impl SidebarStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: SidebarGeometryStylesheet {
                rail_branch_indicator_size: Some(18.0),
                rail_branch_overhang: Some(10.0),
                container_gap: Some(8.0),
                region_gap: Some(4.0),
                header_padding_bottom: Some(8.0),
                footer_padding_top: Some(8.0),
                title_gap: Some(2.0),
                title_padding_bottom: Some(4.0),
                section_padding_top: Some(8.0),
                disclosure_icon_size: Some(14.0),
                rail_submenu_offset_x: Some(12.0),
                window_margin: Some(8.0),
                engine_item_height: Some(30.0),
                engine_section_height: Some(20.0),
                engine_item_padding_x: Some(8.0),
                subtitle_opacity: Some(0.72),
                disabled_opacity: Some(0.56),
                width_expanded: Some(256.0),
                width_icon_rail: Some(48.0),
                width_mobile: Some(288.0),
                item_height: Some(32.0),
                item_icon_size: Some(16.0),
                rail_hit_width: Some(6.0),
                popover_offset: Some(8.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
