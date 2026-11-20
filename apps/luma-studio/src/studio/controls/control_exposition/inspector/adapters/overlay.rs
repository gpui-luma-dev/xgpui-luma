//! Inspector adapters for the overlay control family.

mod split_button {
    use super::super::super::schema::SharedInspectorResolver;
    use super::super::super::schema::ControlInspectorSpec;
    use super::super::super::specs::{CHOICE_SIZES, SPLIT_BUTTON_PARTS, POPUP_MENU_STATES};

    pub static SPLIT_BUTTON_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Split Button",
        id_prefix: "split-button-theme-inspector",
        parts: &SPLIT_BUTTON_PARTS,
        variants: &[],
        states: &POPUP_MENU_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "trigger",
        default_variant_id: "outline",
        default_size_id: "md",
        default_value_id: "",
    };

    pub fn split_button_inspector_resolver() -> SharedInspectorResolver {
        super::super::PopupMenuInspectorAdapter::shared()
    }
}
pub use split_button::{SPLIT_BUTTON_INSPECTOR_SPEC, split_button_inspector_resolver};

mod selection_panel {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::control_size;
    use super::super::super::metrics::floating_menu_layout_section;
    use super::super::super::provenance::color_row;
    use super::super::super::specs::{CHOICE_SIZES, FLOATING_MENU_ITEM_STATES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub static SELECTION_PANEL_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Selection Panel",
        id_prefix: "selection-panel-theme-inspector",
        parts: &[],
        variants: &[],
        states: &FLOATING_MENU_ITEM_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct SelectionPanelInspectorAdapter;

    impl SelectionPanelInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for SelectionPanelInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(floating_menu_layout_section(
                    look,
                    "selection-panel-theme-inspector-box-model",
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        let palette = ShadcnInspect::new(look).inspect_floating_menu_color_palette(control_size(selection.size_id));
        match selection.state_id {
            "hover" => vec![
                color_row("item hover background", &palette.item_hover_background),
                color_row("item hover foreground", &palette.item_hover_foreground),
            ],
            "disabled" => vec![color_row("item disabled foreground", &palette.item_disabled_foreground)],
            _ => vec![
                color_row("background", &palette.background),
                color_row("foreground", &palette.foreground),
                color_row("border", &palette.border),
            ],
        }
    }
}
pub use selection_panel::{SELECTION_PANEL_INSPECTOR_SPEC, SelectionPanelInspectorAdapter};
