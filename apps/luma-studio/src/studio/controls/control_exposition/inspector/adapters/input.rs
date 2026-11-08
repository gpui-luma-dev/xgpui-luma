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

mod textfield {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::{
        textfield_elevation_applies, textfield_enabled, textfield_state, textfield_variant_style,
    };
    use super::super::super::input::textfield_color_rows;
    use super::super::super::metrics::textfield_layout_section;
    use super::super::super::provenance::elevation_snapshot;
    use super::super::super::specs::{CHOICE_SIZES, TEXTFIELD_INTERACTION_STATES, TEXTFIELD_VARIANTS};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    static TEXTFIELD_STATES: [super::super::super::schema::InspectorStateSpec; 5] = TEXTFIELD_INTERACTION_STATES;

    pub static TEXTFIELD_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "TextField",
        id_prefix: "textfield-theme-inspector",
        parts: &[],
        variants: &TEXTFIELD_VARIANTS,
        states: &TEXTFIELD_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "primary",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct TextFieldInspectorAdapter;

    impl TextFieldInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for TextFieldInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            let style = textfield_variant_style(selection.variant_id);
            let enabled = textfield_enabled(selection.state_id);
            let state = textfield_state(selection.state_id);
            match category_id {
                "color" => {
                    let palette = ShadcnInspect::new(look).inspect_textfield_color_palette(style, state, enabled);
                    InspectorCategoryContent::Colors(textfield_color_rows(&palette))
                }
                "layout" => InspectorCategoryContent::Layout(textfield_layout_section(
                    look,
                    "textfield-theme-inspector-box-model",
                    selection.size_id,
                )),
                "elevation" => InspectorCategoryContent::Elevation(elevation_snapshot(
                    &ShadcnInspect::new(look).inspect_textfield_elevation(style, enabled),
                    "resolved textfield look",
                    "textfield.elevation_rules[].style",
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }

        fn category_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, category_id: &str) -> bool {
            if category_id == "elevation" {
                return textfield_elevation_applies(selection.variant_id, selection.state_id);
            }
            true
        }
    }
}
pub use textfield::{TEXTFIELD_INSPECTOR_SPEC, TextFieldInspectorAdapter};

mod textarea {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::{
        textfield_elevation_applies, textfield_enabled, textfield_variant_style, textarea_state,
    };
    use super::super::super::input::textfield_color_rows;
    use super::super::super::metrics::textarea_layout_section;
    use super::super::super::provenance::elevation_snapshot;
    use super::super::super::specs::{CHOICE_SIZES, TEXTAREA_INTERACTION_STATES, TEXTFIELD_VARIANTS};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    static TEXTAREA_STATES: [super::super::super::schema::InspectorStateSpec; 5] = TEXTAREA_INTERACTION_STATES;

    pub static TEXTAREA_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "TextArea",
        id_prefix: "textarea-theme-inspector",
        parts: &[],
        variants: &TEXTFIELD_VARIANTS,
        states: &TEXTAREA_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "primary",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct TextAreaInspectorAdapter;

    impl TextAreaInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for TextAreaInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            let style = textfield_variant_style(selection.variant_id);
            let enabled = textfield_enabled(selection.state_id);
            let state = textarea_state(selection.state_id);
            match category_id {
                "color" => {
                    let palette = ShadcnInspect::new(look).inspect_textarea_color_palette(style, state, enabled);
                    InspectorCategoryContent::Colors(textfield_color_rows(&palette))
                }
                "layout" => InspectorCategoryContent::Layout(textarea_layout_section(
                    look,
                    "textarea-theme-inspector-box-model",
                    selection.size_id,
                )),
                "elevation" => InspectorCategoryContent::Elevation(elevation_snapshot(
                    &ShadcnInspect::new(look).inspect_textfield_elevation(style, enabled),
                    "resolved textarea look",
                    "textfield.elevation_rules[].style",
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }

        fn category_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, category_id: &str) -> bool {
            if category_id == "elevation" {
                return textfield_elevation_applies(selection.variant_id, selection.state_id);
            }
            true
        }
    }
}
pub use textarea::{TEXTAREA_INSPECTOR_SPEC, TextAreaInspectorAdapter};

mod autocomplete {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::input::{autocomplete_chrome_color_rows, floating_menu_color_rows};
    use super::super::super::metrics::autocomplete_layout_section;
    use super::super::super::specs::{CHOICE_SIZES, DEFAULT_INTERACTION_STATES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    static AUTOCOMPLETE_STATES: [super::super::super::schema::InspectorStateSpec; 1] = DEFAULT_INTERACTION_STATES;

    pub static AUTOCOMPLETE_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Autocomplete",
        id_prefix: "autocomplete-theme-inspector",
        parts: &[],
        variants: &[],
        states: &AUTOCOMPLETE_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct AutocompleteInspectorAdapter;

    impl AutocompleteInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for AutocompleteInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look)),
                "layout" => InspectorCategoryContent::Layout(autocomplete_layout_section(
                    look,
                    "autocomplete-theme-inspector-box-model",
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(look: &ShadcnLook) -> Vec<InspectColorRow> {
        let chrome = ShadcnInspect::new(look).inspect_autocomplete_chrome_color_palette();
        let mut rows = autocomplete_chrome_color_rows(&chrome);
        rows.extend(floating_menu_color_rows(look));
        rows
    }
}
pub use autocomplete::{AUTOCOMPLETE_INSPECTOR_SPEC, AutocompleteInspectorAdapter};

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

mod selector {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;
    use lucide_svg_static::Icon as LucideIcon;
    use xgpui_luma::theme::InteractionState;

    use super::super::super::common::{control_size, interaction_state};
    use super::super::super::input::floating_menu_palette_rows;
    use super::super::super::metrics::selector_layout_section;
    use super::super::super::provenance::color_row;
    use super::super::super::specs::{CHOICE_SIZES, COLOR_LAYOUT_CATEGORIES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorPart,
        InspectorSelection, SharedInspectorResolver,
    };

    static SELECTOR_PARTS: [InspectorPart; 2] = [
        InspectorPart { id: "trigger", label: "Trigger", variants: &[], default_variant_id: "" },
        InspectorPart { id: "panel", label: "Popup / Items", variants: &[], default_variant_id: "" },
    ];

    static SELECTOR_STATES: [super::super::super::schema::InspectorStateSpec; 7] = [
        super::super::super::schema::InspectorStateSpec {
            id: "default",
            label: "Default",
            icon: LucideIcon::Circle,
            expanded_default: true,
            categories: &COLOR_LAYOUT_CATEGORIES,
        },
        super::super::super::schema::InspectorStateSpec {
            id: "disabled",
            label: "Disabled",
            icon: LucideIcon::CircleOff,
            expanded_default: false,
            categories: &COLOR_LAYOUT_CATEGORIES,
        },
        super::super::super::schema::InspectorStateSpec {
            id: "hover",
            label: "Hover",
            icon: LucideIcon::MousePointer2,
            expanded_default: false,
            categories: &COLOR_LAYOUT_CATEGORIES,
        },
        super::super::super::schema::InspectorStateSpec {
            id: "focus",
            label: "Focus",
            icon: LucideIcon::Focus,
            expanded_default: false,
            categories: &COLOR_LAYOUT_CATEGORIES,
        },
        super::super::super::schema::InspectorStateSpec {
            id: "open",
            label: "Open",
            icon: LucideIcon::ChevronUp,
            expanded_default: false,
            categories: &COLOR_LAYOUT_CATEGORIES,
        },
        super::super::super::schema::InspectorStateSpec {
            id: "invalid",
            label: "Invalid",
            icon: LucideIcon::CircleAlert,
            expanded_default: false,
            categories: &COLOR_LAYOUT_CATEGORIES,
        },
        super::super::super::schema::InspectorStateSpec {
            id: "pressed",
            label: "Pressed",
            icon: LucideIcon::MousePointerClick,
            expanded_default: false,
            categories: &COLOR_LAYOUT_CATEGORIES,
        },
    ];

    pub static SELECTOR_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Selector",
        id_prefix: "selector-theme-inspector",
        parts: &SELECTOR_PARTS,
        variants: &[],
        states: &SELECTOR_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "trigger",
        default_variant_id: "",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct SelectorInspectorAdapter;

    impl SelectorInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for SelectorInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(selector_layout_section(
                    look,
                    "selector-theme-inspector-box-model",
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        if selection.part_id == "panel" {
            return floating_menu_palette_rows(
                &ShadcnInspect::new(look).inspect_floating_menu_color_palette(control_size(selection.size_id)),
            );
        }

        let palette = ShadcnInspect::new(look).inspect_selector_color_palette(
            selector_interaction_state(selection.state_id),
            control_size(selection.size_id),
        );
        vec![
            color_row("trigger background", &palette.trigger_background),
            color_row("trigger foreground", &palette.trigger_foreground),
            color_row("trigger border", &palette.trigger_border),
        ]
    }

    fn selector_interaction_state(state_id: &str) -> InteractionState {
        let mut state = interaction_state(state_id);
        if state_id == "focus" || state_id == "open" {
            state.focused = true;
        }
        if state_id == "open" {
            state.hovered = true;
        }
        if state_id == "invalid" {
            state.invalid = true;
        }
        state
    }
}
pub use selector::{SELECTOR_INSPECTOR_SPEC, SelectorInspectorAdapter};
