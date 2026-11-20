/// Native common geometry owned by this control.
pub(crate) fn native_pager_stylesheet() -> xgpui_luma::theme::stylesheet::PagerStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    PagerStylesheet {
        geometry: PagerGeometryStylesheet {
            padding_x: Some(8.0),
            padding_y: Some(8.0),
            gap: Some(4.0),
            ..Default::default()
        },
        sizes: xgpui_luma::theme::ControlSizes::from([
            (
                "minimal".into(),
                PagerGeometryStylesheet {
                    button_size: Some(28.0),
                    button_min_width: Some(28.0),
                    group_gap: Some(8.0),
                    ..Default::default()
                },
            ),
            (
                "minimal-edge".into(),
                PagerGeometryStylesheet { button_min_width: Some(32.0), group_gap: Some(16.0), ..Default::default() },
            ),
            (
                "numeric".into(),
                PagerGeometryStylesheet { button_min_width: Some(32.0), group_gap: Some(16.0), ..Default::default() },
            ),
        ]),
    }
}

#[derive(Clone, Debug)]
pub struct PagerMetricTable {
    pub control_height: crate::ResolvedMetric,
    pub button_size: crate::ResolvedMetric,
    pub button_min_width: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
    pub padding_x: crate::ResolvedMetric,
    pub padding_y: crate::ResolvedMetric,
    pub gap: crate::ResolvedMetric,
    pub group_gap: crate::ResolvedMetric,
    pub font_family: crate::ResolvedTypography,
    pub reserved_shadow_extent: crate::ResolvedMetric,
}
