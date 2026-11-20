//! Inspector adapters for the input control family.

mod textfield_menu {
    use std::sync::Arc;

    use xgpui_luma::controls::textfield::TextFieldState;
    use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnTextFieldStyle};
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::input::{floating_menu_color_rows, textfield_color_rows_prefixed};
    use super::super::super::metrics::textfield_menu_layout_section;
    use super::super::super::schema::{
        ControlInspectorResolver, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub struct TextFieldMenuInspectorAdapter {
        diagram_id: &'static str,
    }

    impl TextFieldMenuInspectorAdapter {
        pub fn shared(diagram_id: &'static str) -> SharedInspectorResolver {
            Arc::new(Self { diagram_id })
        }
    }

    impl ControlInspectorResolver for TextFieldMenuInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look)),
                "layout" => InspectorCategoryContent::Layout(textfield_menu_layout_section(
                    look,
                    self.diagram_id,
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(look: &ShadcnLook) -> Vec<InspectColorRow> {
        let palette = ShadcnInspect::new(look).inspect_textfield_color_palette(
            ShadcnTextFieldStyle::Input,
            TextFieldState::default(),
            true,
        );
        let mut rows = textfield_color_rows_prefixed("textfield", &palette);
        rows.extend(floating_menu_color_rows(look));
        rows
    }
}

mod combobox {
    use super::super::super::specs::{CHOICE_SIZES, DEFAULT_INTERACTION_STATES};
    use super::super::super::schema::ControlInspectorSpec;
    use super::textfield_menu::TextFieldMenuInspectorAdapter;

    pub static COMBOBOX_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "ComboBox",
        id_prefix: "combobox-theme-inspector",
        parts: &[],
        variants: &[],
        states: &DEFAULT_INTERACTION_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "md",
        default_value_id: "",
    };

    pub fn combobox_inspector_adapter() -> super::super::super::schema::SharedInspectorResolver {
        TextFieldMenuInspectorAdapter::shared("combobox-theme-inspector-box-model")
    }
}
pub use combobox::{COMBOBOX_INSPECTOR_SPEC, combobox_inspector_adapter};

mod search_selector {
    use super::super::super::specs::{CHOICE_SIZES, DEFAULT_INTERACTION_STATES};
    use super::super::super::schema::ControlInspectorSpec;
    use super::textfield_menu::TextFieldMenuInspectorAdapter;

    pub static SEARCH_SELECTOR_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "SearchSelector",
        id_prefix: "search-selector-theme-inspector",
        parts: &[],
        variants: &[],
        states: &DEFAULT_INTERACTION_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "md",
        default_value_id: "",
    };

    pub fn search_selector_inspector_adapter() -> super::super::super::schema::SharedInspectorResolver {
        TextFieldMenuInspectorAdapter::shared("search-selector-theme-inspector-box-model")
    }
}
pub use search_selector::{SEARCH_SELECTOR_INSPECTOR_SPEC, search_selector_inspector_adapter};
