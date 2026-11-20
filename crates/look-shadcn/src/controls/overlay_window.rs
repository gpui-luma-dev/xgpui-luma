/// Native common geometry owned by this control.
pub(crate) fn native_overlay_window_stylesheet() -> xgpui_luma::theme::stylesheet::OverlayWindowStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    OverlayWindowStylesheet {
        geometry: OverlayWindowGeometryStylesheet { backdrop_opacity: Some(0.42), ..Default::default() },
        sizes: xgpui_luma::theme::ControlSizes::from([
            ("sm".into(), OverlayWindowGeometryStylesheet { max_width: Some(420.0), ..Default::default() }),
            ("md".into(), OverlayWindowGeometryStylesheet::default()),
            ("lg".into(), OverlayWindowGeometryStylesheet { max_width: Some(620.0), ..Default::default() }),
        ]),
    }
}
