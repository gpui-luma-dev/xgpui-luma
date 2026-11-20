//! Inspector adapters for the layout control family.

mod badge {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::badge_variant;
    use super::super::super::metrics::badge_layout_section;
    use super::super::super::provenance::color_row;
    use super::super::super::specs::{BADGE_VARIANTS, CHOICE_SIZES, DEFAULT_INTERACTION_STATES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub static BADGE_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Badge",
        id_prefix: "badge-theme-inspector",
        parts: &[],
        variants: &BADGE_VARIANTS,
        states: &DEFAULT_INTERACTION_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "default",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct BadgeInspectorAdapter;

    impl BadgeInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for BadgeInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(badge_layout_section(
                    look,
                    "badge-theme-inspector-box-model",
                    selection.variant_id,
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        let palette = ShadcnInspect::new(look).inspect_badge_color_palette(badge_variant(selection.variant_id));
        let mut rows = vec![color_row("background", &palette.background), color_row("foreground", &palette.foreground)];
        if let Some(border) = &palette.border {
            rows.push(color_row("border", border));
        }
        rows
    }
}
pub use badge::{BADGE_INSPECTOR_SPEC, BadgeInspectorAdapter};
