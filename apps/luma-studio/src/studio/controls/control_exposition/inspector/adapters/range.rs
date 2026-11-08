//! Inspector adapters for the range control family.

mod slider {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::interaction_state;
    use super::super::super::metrics::slider_layout_section;
    use super::super::super::provenance::color_row;
    use super::super::super::specs::COLOR_LAYOUT_INTERACTION_STATES;
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    static SLIDER_STATES: [super::super::super::schema::InspectorStateSpec; 4] = COLOR_LAYOUT_INTERACTION_STATES;

    pub static SLIDER_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Slider",
        id_prefix: "slider-theme-inspector",
        parts: &[],
        variants: &[],
        states: &SLIDER_STATES,
        sizes: &[],
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "",
        default_value_id: "",
    };

    pub struct SliderInspectorAdapter;

    impl SliderInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for SliderInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => {
                    InspectorCategoryContent::Layout(slider_layout_section(look, "slider-theme-inspector-box-model"))
                }
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        let palette = ShadcnInspect::new(look).inspect_slider_color_palette(interaction_state(selection.state_id));
        let rows = vec![
            color_row("track background", &palette.track_background),
            color_row("fill background", &palette.fill_background),
            color_row("thumb background", &palette.thumb_background),
            color_row("thumb border", &palette.thumb_border),
        ];
        rows
    }
}
pub use slider::{SLIDER_INSPECTOR_SPEC, SliderInspectorAdapter};

mod progress {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::progress_enabled;
    use super::super::super::metrics::{progress_circular_layout_section, progress_linear_layout_section};
    use super::super::super::provenance::color_row;
    use super::super::super::schema::InspectLayoutSection;
    use super::super::super::specs::{PROGRESS_STATES, PROGRESS_VARIANTS};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub static PROGRESS_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Progress",
        id_prefix: "progress-theme-inspector",
        parts: &[],
        variants: &PROGRESS_VARIANTS,
        states: &PROGRESS_STATES,
        sizes: &[],
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "circular",
        default_size_id: "",
        default_value_id: "",
    };

    pub struct ProgressInspectorAdapter;

    impl ProgressInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for ProgressInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(resolve_layout_section(look, selection)),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_layout_section(look: &ShadcnLook, selection: InspectorSelection<'_>) -> InspectLayoutSection {
        let metrics = ShadcnInspect::new(look).inspect_progress_metrics();
        let diagram_id = "progress-theme-inspector-box-model";
        if selection.variant_id == "linear" {
            progress_linear_layout_section(look, diagram_id, &metrics)
        } else {
            progress_circular_layout_section(look, diagram_id, &metrics)
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        let palette = ShadcnInspect::new(look).inspect_progress_color_palette(progress_enabled(selection.state_id));
        let mut rows = vec![color_row("track", &palette.track_color), color_row("progress", &palette.progress_color)];
        if selection.variant_id == "linear" {
            rows.push(color_row("thumb", &palette.thumb_color));
        }
        rows
    }
}
pub use progress::{PROGRESS_INSPECTOR_SPEC, ProgressInspectorAdapter};

mod stepper {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::stepper_enabled;
    use super::super::super::metrics::{stepper_horizontal_layout_section, stepper_vertical_layout_section};
    use super::super::super::provenance::color_row;
    use super::super::super::schema::InspectLayoutSection;
    use super::super::super::specs::{STEPPER_STATES, STEPPER_VARIANTS};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub static STEPPER_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Stepper",
        id_prefix: "stepper-theme-inspector",
        parts: &[],
        variants: &STEPPER_VARIANTS,
        states: &STEPPER_STATES,
        sizes: &[],
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "horizontal",
        default_size_id: "",
        default_value_id: "",
    };

    pub struct StepperInspectorAdapter;

    impl StepperInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for StepperInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(resolve_layout_section(look, selection)),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_layout_section(look: &ShadcnLook, selection: InspectorSelection<'_>) -> InspectLayoutSection {
        let metrics = ShadcnInspect::new(look).inspect_stepper_metrics();
        let diagram_id = "stepper-theme-inspector-box-model";
        if selection.variant_id == "vertical" {
            stepper_vertical_layout_section(look, diagram_id, &metrics)
        } else {
            stepper_horizontal_layout_section(look, diagram_id, &metrics)
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        let enabled = stepper_enabled(selection.state_id);
        let inspect = ShadcnInspect::new(look);
        let progress = inspect.inspect_progress_color_palette(enabled);
        let palette = inspect.inspect_stepper_color_palette(enabled);
        vec![
            color_row("track", &progress.track_color),
            color_row("progress", &progress.progress_color),
            color_row("incomplete bg", &palette.incomplete_bg),
            color_row("incomplete border", &palette.incomplete_border),
            color_row("incomplete fg", &palette.incomplete_fg),
        ]
    }
}
pub use stepper::{STEPPER_INSPECTOR_SPEC, StepperInspectorAdapter};

mod scrollbar {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::{interaction_state, scrollbar_style};
    use super::super::super::metrics::scrollbar_layout_section;
    use super::super::super::provenance::color_row;
    use super::super::super::specs::{COLOR_LAYOUT_INTERACTION_STATES, SCROLLBAR_STYLE_VARIANTS};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    static SCROLLBAR_STATES: [super::super::super::schema::InspectorStateSpec; 4] = COLOR_LAYOUT_INTERACTION_STATES;

    pub static SCROLLBAR_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Scrollbar",
        id_prefix: "scrollbar-theme-inspector",
        parts: &[],
        variants: &SCROLLBAR_STYLE_VARIANTS,
        states: &SCROLLBAR_STATES,
        sizes: &[],
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "ghost",
        default_size_id: "",
        default_value_id: "",
    };

    pub struct ScrollbarInspectorAdapter;

    impl ScrollbarInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for ScrollbarInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(scrollbar_layout_section(
                    look,
                    "scrollbar-theme-inspector-box-model",
                    selection.variant_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        let style = scrollbar_style(selection.variant_id);
        let palette =
            ShadcnInspect::new(look).inspect_scrollbar_color_palette(style, interaction_state(selection.state_id));
        let rows = vec![
            color_row("track background", &palette.track_background),
            color_row("thumb background", &palette.thumb_background),
        ];
        rows
    }
}
pub use scrollbar::{SCROLLBAR_INSPECTOR_SPEC, ScrollbarInspectorAdapter};
