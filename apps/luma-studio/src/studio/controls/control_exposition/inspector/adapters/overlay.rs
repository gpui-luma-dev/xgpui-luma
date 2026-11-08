//! Inspector adapters for the overlay control family.

mod popup_menu {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::{control_size, interaction_state, popup_menu_trigger_style};
    use super::super::super::input::trigger_color_rows;
    use super::super::super::metrics::{floating_menu_layout_section, popup_menu_trigger_layout_section};
    use super::super::super::provenance::color_row;
    use super::super::super::specs::{CHOICE_SIZES, POPUP_MENU_PARTS, POPUP_MENU_STATES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        InspectorStateSpec, SharedInspectorResolver,
    };

    pub static POPUP_MENU_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Popup Menu",
        id_prefix: "popup-menu-theme-inspector",
        parts: &POPUP_MENU_PARTS,
        variants: &[],
        states: &POPUP_MENU_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "trigger",
        default_variant_id: "",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct PopupMenuInspectorAdapter;

    impl PopupMenuInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for PopupMenuInspectorAdapter {
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

        fn state_applies(
            &self,
            _look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            state: &InspectorStateSpec,
        ) -> bool {
            match selection.part_id {
                "trigger" => matches!(state.id, "default" | "hover" | "focused" | "pressed" | "disabled"),
                "panel" => matches!(state.id, "panel-default" | "panel-hover" | "panel-disabled"),
                _ => true,
            }
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        let size = control_size(selection.size_id);

        if selection.part_id == "panel" {
            let palette = ShadcnInspect::new(look).inspect_floating_menu_color_palette(size);
            return resolve_panel_color_rows(&palette, selection.state_id);
        }

        let palette = ShadcnInspect::new(look).inspect_popup_menu_color_palette(
            popup_menu_trigger_style(selection.variant_id),
            interaction_state(selection.state_id),
            size,
        );
        trigger_color_rows(&palette.trigger_background, &palette.trigger_foreground, &palette.trigger_border)
    }

    fn resolve_panel_color_rows(
        palette: &xgpui_luma_look_shadcn::inspect::FloatingMenuInspectPalette,
        state_id: &str,
    ) -> Vec<InspectColorRow> {
        match state_id {
            "panel-hover" => vec![
                color_row("item hover background", &palette.item_hover_background),
                color_row("item hover foreground", &palette.item_hover_foreground),
            ],
            "panel-disabled" => vec![color_row("item disabled foreground", &palette.item_disabled_foreground)],
            _ => vec![
                color_row("background", &palette.background),
                color_row("foreground", &palette.foreground),
                color_row("border", &palette.border),
            ],
        }
    }

    fn resolve_layout_section(
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
    ) -> super::super::super::schema::InspectLayoutSection {
        match selection.part_id {
            "panel" => {
                floating_menu_layout_section(look, "popup-menu-theme-inspector-panel-box-model", selection.size_id)
            }
            _ => popup_menu_trigger_layout_section(
                look,
                "popup-menu-theme-inspector-trigger-box-model",
                selection.variant_id,
                selection.size_id,
            ),
        }
    }
}
pub use popup_menu::{POPUP_MENU_INSPECTOR_SPEC, PopupMenuInspectorAdapter};

mod split_button {
    use std::sync::Arc;

    use super::super::super::schema::SharedInspectorResolver;
    use super::super::super::schema::ControlInspectorSpec;
    use super::popup_menu::PopupMenuInspectorAdapter;
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
        Arc::new(PopupMenuInspectorAdapter)
    }
}
pub use split_button::{SPLIT_BUTTON_INSPECTOR_SPEC, split_button_inspector_resolver};

mod context_menu {
    use std::sync::Arc;

    use xgpui_luma::theme::ControlSize;
    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::interaction_state;
    use super::super::super::input::{floating_menu_palette_rows, trigger_color_rows};
    use super::super::super::metrics::context_menu_layout_section;
    use super::super::super::specs::{CHOICE_SIZES, COLOR_LAYOUT_INTERACTION_STATES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    static CONTEXT_MENU_STATES: [super::super::super::schema::InspectorStateSpec; 4] = COLOR_LAYOUT_INTERACTION_STATES;

    pub static CONTEXT_MENU_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Context Menu",
        id_prefix: "context-menu-theme-inspector",
        parts: &[],
        variants: &[],
        states: &CONTEXT_MENU_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct ContextMenuInspectorAdapter;

    impl ContextMenuInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for ContextMenuInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(context_menu_layout_section(
                    look,
                    "context-menu-theme-inspector-box-model",
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        let size = match selection.size_id {
            "sm" => ControlSize::Sm,
            "lg" => ControlSize::Lg,
            _ => ControlSize::Md,
        };
        let palette =
            ShadcnInspect::new(look).inspect_context_menu_color_palette(interaction_state(selection.state_id), size);
        let mut rows =
            trigger_color_rows(&palette.target_background, &palette.target_foreground, &palette.target_border);
        rows.extend(floating_menu_palette_rows(&palette.menu));
        rows
    }
}
pub use context_menu::{CONTEXT_MENU_INSPECTOR_SPEC, ContextMenuInspectorAdapter};

mod floating_menu {
    use std::sync::Arc;

    use xgpui_luma::theme::ControlSize;
    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::input::floating_menu_palette_rows;
    use super::super::super::metrics::floating_menu_layout_section;
    use super::super::super::specs::{CHOICE_SIZES, DEFAULT_INTERACTION_STATES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub static FLOATING_MENU_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Floating Menu",
        id_prefix: "floating-menu-theme-inspector",
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

    pub struct FloatingMenuInspectorAdapter;

    impl FloatingMenuInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for FloatingMenuInspectorAdapter {
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
                    "floating-menu-theme-inspector-box-model",
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
    ) -> Vec<super::super::super::schema::InspectColorRow> {
        let size = match selection.size_id {
            "sm" => ControlSize::Sm,
            "lg" => ControlSize::Lg,
            _ => ControlSize::Md,
        };
        let palette = ShadcnInspect::new(look).inspect_floating_menu_color_palette(size);
        floating_menu_palette_rows(&palette)
    }
}
pub use floating_menu::{FLOATING_MENU_INSPECTOR_SPEC, FloatingMenuInspectorAdapter};

mod overlay_window {
    use std::sync::Arc;

    use xgpui_luma::controls::overlay_window::OverlayWindowMode;
    use xgpui_luma::theme::ControlSize;
    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::input::overlay_window_palette_rows;
    use super::super::super::metrics::overlay_window_layout_section;
    use super::super::super::specs::{CHOICE_SIZES, DEFAULT_INTERACTION_STATES, OVERLAY_WINDOW_MODE_VARIANTS};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub static OVERLAY_WINDOW_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Overlay Window",
        id_prefix: "overlay-window-theme-inspector",
        parts: &[],
        variants: &OVERLAY_WINDOW_MODE_VARIANTS,
        states: &DEFAULT_INTERACTION_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "modeless",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct OverlayWindowInspectorAdapter;

    impl OverlayWindowInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for OverlayWindowInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(overlay_window_layout_section(
                    look,
                    "overlay-window-theme-inspector-box-model",
                    selection.variant_id,
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
    ) -> Vec<super::super::super::schema::InspectColorRow> {
        let palette = ShadcnInspect::new(look).inspect_overlay_window_color_palette(
            overlay_window_size(selection.size_id),
            overlay_window_mode(selection.variant_id),
        );
        overlay_window_palette_rows(&palette)
    }

    fn overlay_window_size(size_id: &str) -> ControlSize {
        match size_id {
            "sm" => ControlSize::Sm,
            "lg" => ControlSize::Lg,
            _ => ControlSize::Md,
        }
    }

    fn overlay_window_mode(variant_id: &str) -> OverlayWindowMode {
        match variant_id {
            "modal" => OverlayWindowMode::Modal,
            _ => OverlayWindowMode::Modeless,
        }
    }
}
pub use overlay_window::{OVERLAY_WINDOW_INSPECTOR_SPEC, OverlayWindowInspectorAdapter};

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
