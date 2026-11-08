use xgpui_luma::controls::{
    tooltip::{Tooltip, TooltipPlacement},
};
use crate::theme::AppTheme;
use xgpui_luma_plot::{MarkerVisibility, PlotPoint, PlotSlice};

pub(crate) fn visibility(id: &str) -> MarkerVisibility {
    match id {
        "all" => MarkerVisibility::Always,
        "slice" => MarkerVisibility::HoverSlice,
        _ => MarkerVisibility::Hover,
    }
}

pub(crate) fn tooltip(look: &AppTheme, point: &PlotPoint, coordinate_label: String) -> Tooltip {
    Tooltip::new(format!("{} · {}", coordinate_label, point.label))
        .shortcut(point.sample.y.to_string())
        .theme(std::sync::Arc::new(look.clone()))
}

pub(crate) fn slice_tooltip(look: &AppTheme, slice: &PlotSlice, coordinate_label: String) -> Tooltip {
    if let Some(point) = slice.points.first().filter(|_| slice.points.len() == 1) {
        return tooltip(look, point, coordinate_label).delay(std::time::Duration::ZERO);
    }
    let values = slice
        .points
        .iter()
        .map(|point| format!("{}: {}", point.label, point.sample.y))
        .collect::<Vec<_>>()
        .join("\n");
    Tooltip::new(format!("{}\n{}", coordinate_label, values))
        .delay(std::time::Duration::ZERO)
        .theme(std::sync::Arc::new(look.clone()))
}

pub(crate) fn placement(id: &str) -> TooltipPlacement {
    if id == "below" {
        TooltipPlacement::Below
    } else {
        TooltipPlacement::Above
    }
}

pub(crate) fn motion(id: &str) -> std::time::Duration {
    if id == "smooth" {
        std::time::Duration::from_millis(250)
    } else {
        std::time::Duration::ZERO
    }
}
