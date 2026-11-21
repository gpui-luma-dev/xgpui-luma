//! Inspector specs for every Shadcn control styled with `.lss` (issue #285), all served by the generic
//! [`LssInspectorAdapter`]: rows are the control's `.lss` properties per part, with the rule and tokens that
//! set them. The part selector filters by `.lss` part; variants and sizes are the look's `.lss` axes (an id
//! that is no axis value, e.g. `primary` or `md`, selects the base); the value selector turns the control's
//! value state on or off.

use super::lss::{LSS_INTERACTION_STATES, LssCompositeAdapter, LssInspectorAdapter, lss_part};
use super::super::schema::{
    ControlInspectorSpec, InspectorPart, InspectorSize, InspectorValueMode, InspectorVariant, SharedInspectorResolver,
};

const fn variant(id: &'static str, label: &'static str) -> InspectorVariant {
    InspectorVariant { id, label }
}

static SIZES: [InspectorSize; 3] = [
    InspectorSize { id: "sm", label: "sm" },
    InspectorSize { id: "md", label: "md" },
    InspectorSize { id: "lg", label: "lg" },
];

static SELECTED_VALUES: [InspectorValueMode; 2] = [
    InspectorValueMode { id: "selected", label: "selected" },
    InspectorValueMode { id: "unselected", label: "unselected" },
];

/// A spec over the `.lss` interaction states. `sized` adds the look's sizes; `selectable` adds the
/// `selected` value selector.
const fn spec(
    control_label: &'static str,
    id_prefix: &'static str,
    parts: &'static [InspectorPart],
    variants: &'static [InspectorVariant],
    sized: bool,
    selectable: bool,
) -> ControlInspectorSpec {
    ControlInspectorSpec {
        control_label,
        id_prefix,
        parts,
        variants,
        states: &LSS_INTERACTION_STATES,
        sizes: if sized { &SIZES } else { &[] },
        value_modes: if selectable { &SELECTED_VALUES } else { &[] },
        default_part_id: if parts.is_empty() { "" } else { "all" },
        default_variant_id: if variants.is_empty() { "" } else { variants[0].id },
        default_size_id: if sized { "md" } else { "" },
        default_value_id: if selectable { "selected" } else { "" },
    }
}

/// `XInspectorAdapter::shared()`: the generic adapter for one `.lss` control. The value state is
/// `selected` (selector ids `selected` / `unselected`).
macro_rules! lss_adapter {
    ($(#[$doc:meta])* $name:ident, $control:literal) => {
        $(#[$doc])*
        pub struct $name;

        impl $name {
            pub fn shared() -> SharedInspectorResolver {
                LssInspectorAdapter::new($control, "selected").shared()
            }
        }
    };
}

// Buttons and choice.

static BUTTON_VARIANTS: [InspectorVariant; 5] = [
    variant("primary", "Primary"),
    variant("secondary", "Secondary"),
    variant("outline", "Outline"),
    variant("ghost", "Ghost"),
    variant("content-only", "Content Only"),
];
pub static BUTTON_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Button", "button-theme-inspector", &[], &BUTTON_VARIANTS, true, false);
lss_adapter!(ButtonInspectorAdapter, "button");

static TOGGLE_VARIANTS: [InspectorVariant; 4] = [
    variant("primary", "Primary"),
    variant("secondary", "Secondary"),
    variant("ghost", "Ghost"),
    variant("content-only", "Content Only"),
];
pub static TOGGLE_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Toggle", "toggle-theme-inspector", &[], &TOGGLE_VARIANTS, true, true);
lss_adapter!(ToggleInspectorAdapter, "toggle");

static TOOLBAR_PARTS: [InspectorPart; 3] =
    [lss_part("all", "All"), lss_part("root", "Shell"), lss_part("separator", "Separator")];
static TOOLBAR_VARIANTS: [InspectorVariant; 2] = [variant("default", "Default"), variant("ghost", "Ghost")];
pub static TOOLBAR_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Toolbar", "toolbar-theme-inspector", &TOOLBAR_PARTS, &TOOLBAR_VARIANTS, false, false);
lss_adapter!(ToolbarInspectorAdapter, "toolbar");

// Text and selection.

static TEXTFIELD_VARIANTS: [InspectorVariant; 4] = [
    variant("default", "Default"),
    variant("input", "Input"),
    variant("primary", "Primary"),
    variant("surface", "Surface"),
];
pub static TEXTFIELD_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("TextField", "textfield-theme-inspector", &[], &TEXTFIELD_VARIANTS, false, false);
lss_adapter!(TextFieldInspectorAdapter, "textfield");

pub static TEXTAREA_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("TextArea", "textarea-theme-inspector", &[], &TEXTFIELD_VARIANTS, false, false);
lss_adapter!(TextAreaInspectorAdapter, "textarea");

pub static AUTOCOMPLETE_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Autocomplete", "autocomplete-theme-inspector", &[], &[], false, false);
lss_adapter!(AutocompleteInspectorAdapter, "autocomplete");

static TRIGGER_VARIANTS: [InspectorVariant; 3] =
    [variant("default", "Default"), variant("outline", "Outline"), variant("ghost", "Ghost")];
pub static SELECTOR_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Selector", "selector-theme-inspector", &[], &TRIGGER_VARIANTS, true, false);
lss_adapter!(SelectorInspectorAdapter, "selector");

static FIELD_MENU_PARTS: [InspectorPart; 2] = [lss_part("field", "Field"), lss_part("menu", "Menu")];

/// A text field over a menu: the `input` text field and the floating menu.
fn field_menu_adapter() -> SharedInspectorResolver {
    LssCompositeAdapter::new(vec![
        ("field", LssInspectorAdapter::new("textfield", "selected").fixed_variant("input")),
        ("menu", LssInspectorAdapter::new("floating_menu", "selected")),
    ])
    .shared()
}

pub static COMBOBOX_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("ComboBox", "combobox-theme-inspector", &FIELD_MENU_PARTS, &[], false, false);
pub fn combobox_inspector_adapter() -> SharedInspectorResolver {
    field_menu_adapter()
}

pub static SEARCH_SELECTOR_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("SearchSelector", "search-selector-theme-inspector", &FIELD_MENU_PARTS, &[], false, false);
pub fn search_selector_inspector_adapter() -> SharedInspectorResolver {
    field_menu_adapter()
}

// Range.

static SLIDER_PARTS: [InspectorPart; 5] = [
    lss_part("all", "All"),
    lss_part("root", "Slider"),
    lss_part("track", "Track"),
    lss_part("fill", "Fill"),
    lss_part("thumb", "Thumb"),
];
static SLIDER_VARIANTS: [InspectorVariant; 2] = [variant("default", "Default"), variant("secondary", "Secondary")];
pub static SLIDER_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Slider", "slider-theme-inspector", &SLIDER_PARTS, &SLIDER_VARIANTS, true, false);
lss_adapter!(SliderInspectorAdapter, "slider");

static PROGRESS_PARTS: [InspectorPart; 4] = [
    lss_part("all", "All"),
    lss_part("root", "Progress"),
    lss_part("track", "Track"),
    lss_part("fill", "Fill"),
];
pub static PROGRESS_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Progress", "progress-theme-inspector", &PROGRESS_PARTS, &[], true, false);
lss_adapter!(ProgressInspectorAdapter, "progress");

// Navigation.

static TABS_PARTS: [InspectorPart; 4] =
    [lss_part("all", "All"), lss_part("root", "Tabs"), lss_part("list", "List"), lss_part("item", "Tab")];
pub static TABS_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Tabs Navigation", "tabs-navigation-theme-inspector", &TABS_PARTS, &[], true, true);
lss_adapter!(TabsInspectorAdapter, "tabs");

pub static ACCORDION_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Accordion", "accordion-theme-inspector", &[], &[], false, false);
lss_adapter!(AccordionInspectorAdapter, "accordion");

pub static STEPPER_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Stepper", "stepper-theme-inspector", &[], &[], true, false);
lss_adapter!(StepperInspectorAdapter, "stepper");

static PAGER_VARIANTS: [InspectorVariant; 2] = [variant("default", "Default"), variant("minimal", "Minimal")];
pub static PAGER_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Pager", "pager-theme-inspector", &[], &PAGER_VARIANTS, false, false);
lss_adapter!(PagerInspectorAdapter, "pager");

static SIDEBAR_PARTS: [InspectorPart; 4] = [
    lss_part("all", "All"),
    lss_part("root", "Sidebar"),
    lss_part("section", "Section"),
    lss_part("item", "Item"),
];
pub static SIDEBAR_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Sidebar", "sidebar-theme-inspector", &SIDEBAR_PARTS, &[], false, true);
lss_adapter!(SidebarInspectorAdapter, "sidebar");

// Collections.

static TABLE_PARTS: [InspectorPart; 4] = [
    lss_part("all", "All"),
    lss_part("root", "Frame"),
    lss_part("header", "Header"),
    lss_part("row", "Row"),
];
pub static TABLE_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Table", "table-theme-inspector", &TABLE_PARTS, &[], false, true);
lss_adapter!(TableInspectorAdapter, "table");

pub static TREE_VIEW_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("TreeView", "tree-view-theme-inspector", &[], &[], false, true);
lss_adapter!(TreeViewInspectorAdapter, "tree_view");

static LISTBOX_PARTS: [InspectorPart; 3] =
    [lss_part("all", "All"), lss_part("root", "Surface"), lss_part("row", "Row")];
pub static LISTBOX_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("ListBox composition", "listbox-theme-inspector", &LISTBOX_PARTS, &[], false, true);
lss_adapter!(ListBoxInspectorAdapter, "listbox");

// Layout.

static SCROLLBAR_VARIANTS: [InspectorVariant; 2] = [variant("ghost", "Ghost"), variant("soft", "Soft")];
pub static SCROLLBAR_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Scrollbar", "scrollbar-theme-inspector", &[], &SCROLLBAR_VARIANTS, true, false);
lss_adapter!(ScrollbarInspectorAdapter, "scrollbar");

pub static SPLIT_VIEW_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Split View", "split-view-theme-inspector", &[], &[], false, false);
lss_adapter!(SplitViewInspectorAdapter, "split_view");

pub static RESIZABLE_PANELS_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Resizable Panels", "resizable-panels-theme-inspector", &[], &[], false, false);
lss_adapter!(ResizablePanelsInspectorAdapter, "resizable_panels");

// Display.

static BADGE_VARIANTS: [InspectorVariant; 4] = [
    variant("default", "Default"),
    variant("secondary", "Secondary"),
    variant("outline", "Outline"),
    variant("ghost", "Ghost"),
];
pub static BADGE_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Badge", "badge-theme-inspector", &[], &BADGE_VARIANTS, true, false);
lss_adapter!(BadgeInspectorAdapter, "badge");

// Overlays.

static SELECTION_PANEL_PARTS: [InspectorPart; 4] = [
    lss_part("all", "All"),
    lss_part("root", "Panel"),
    lss_part("item", "Item"),
    lss_part("separator", "Separator"),
];
pub static SELECTION_PANEL_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Selection Panel", "selection-panel-theme-inspector", &SELECTION_PANEL_PARTS, &[], true, false);
lss_adapter!(SelectionPanelInspectorAdapter, "selection_panel");

pub static POPUP_MENU_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Popup Menu", "popup-menu-theme-inspector", &[], &TRIGGER_VARIANTS, false, false);
lss_adapter!(PopupMenuInspectorAdapter, "popup_menu");

pub static CONTEXT_MENU_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Context Menu", "context-menu-theme-inspector", &[], &[], false, false);
lss_adapter!(ContextMenuInspectorAdapter, "context_menu");

static FLOATING_MENU_PARTS: [InspectorPart; 4] = [
    lss_part("all", "All"),
    lss_part("root", "Menu"),
    lss_part("item", "Item"),
    lss_part("separator", "Separator"),
];
pub static FLOATING_MENU_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Floating Menu", "floating-menu-theme-inspector", &FLOATING_MENU_PARTS, &[], false, false);
lss_adapter!(FloatingMenuInspectorAdapter, "floating_menu");

static OVERLAY_WINDOW_VARIANTS: [InspectorVariant; 2] = [variant("modeless", "Modeless"), variant("modal", "Modal")];
pub static OVERLAY_WINDOW_INSPECTOR_SPEC: ControlInspectorSpec =
    spec("Overlay Window", "overlay-window-theme-inspector", &[], &OVERLAY_WINDOW_VARIANTS, true, false);
lss_adapter!(OverlayWindowInspectorAdapter, "overlay_window");

#[cfg(test)]
mod tests {
    use xgpui_luma_look_shadcn::ShadcnLook;

    use super::*;
    use super::super::super::schema::{InspectorCategoryContent, InspectorSelection};

    fn selection<'a>(part: &'a str, variant: &'a str, value: &'a str) -> InspectorSelection<'a> {
        InspectorSelection {
            part_id: part,
            variant_id: variant,
            state_id: "default",
            size_id: "md",
            value_id: value,
            scale_factor: 1.0,
        }
    }

    fn colors(adapter: &SharedInspectorResolver, selection: InspectorSelection<'_>) -> Vec<String> {
        let look = ShadcnLook::from_css_str(xgpui_luma_look_shadcn::FALLBACK_CSS).expect("catalog");
        match adapter.resolve_category(&look, selection, "color") {
            InspectorCategoryContent::Colors(rows) => rows.iter().map(|r| r.label.to_string()).collect(),
            _ => panic!("colors"),
        }
    }

    #[test]
    fn a_composite_picks_its_control_by_part() {
        let combobox = combobox_inspector_adapter();
        let field = colors(&combobox, selection("field", "", ""));
        let menu = colors(&combobox, selection("menu", "", ""));
        assert!(field.iter().any(|l| l == "placeholder"), "{field:?}");
        assert!(menu.iter().any(|l| l.starts_with("item ")), "{menu:?}");
    }

    #[test]
    fn the_part_selector_filters_rows_to_one_lss_part() {
        let table = TableInspectorAdapter::shared();
        let all = colors(&table, selection("all", "", "unselected"));
        assert!(all.iter().any(|l| l.starts_with("header ")) && all.iter().any(|l| l.starts_with("row ")));
        let rows = colors(&table, selection("row", "", "unselected"));
        assert!(!rows.is_empty() && rows.iter().all(|l| l.starts_with("row ")), "{rows:?}");
        let frame = colors(&table, selection("root", "", "unselected"));
        assert!(frame.iter().all(|l| !l.starts_with("row ") && !l.starts_with("header ")), "{frame:?}");
    }

    #[test]
    fn every_lss_control_has_rows() {
        for (name, adapter, spec) in [
            ("button", ButtonInspectorAdapter::shared(), &BUTTON_INSPECTOR_SPEC),
            ("toggle", ToggleInspectorAdapter::shared(), &TOGGLE_INSPECTOR_SPEC),
            ("toolbar", ToolbarInspectorAdapter::shared(), &TOOLBAR_INSPECTOR_SPEC),
            ("textfield", TextFieldInspectorAdapter::shared(), &TEXTFIELD_INSPECTOR_SPEC),
            ("textarea", TextAreaInspectorAdapter::shared(), &TEXTAREA_INSPECTOR_SPEC),
            ("autocomplete", AutocompleteInspectorAdapter::shared(), &AUTOCOMPLETE_INSPECTOR_SPEC),
            ("selector", SelectorInspectorAdapter::shared(), &SELECTOR_INSPECTOR_SPEC),
            ("slider", SliderInspectorAdapter::shared(), &SLIDER_INSPECTOR_SPEC),
            ("progress", ProgressInspectorAdapter::shared(), &PROGRESS_INSPECTOR_SPEC),
            ("tabs", TabsInspectorAdapter::shared(), &TABS_INSPECTOR_SPEC),
            ("accordion", AccordionInspectorAdapter::shared(), &ACCORDION_INSPECTOR_SPEC),
            ("stepper", StepperInspectorAdapter::shared(), &STEPPER_INSPECTOR_SPEC),
            ("pager", PagerInspectorAdapter::shared(), &PAGER_INSPECTOR_SPEC),
            ("sidebar", SidebarInspectorAdapter::shared(), &SIDEBAR_INSPECTOR_SPEC),
            ("table", TableInspectorAdapter::shared(), &TABLE_INSPECTOR_SPEC),
            ("tree_view", TreeViewInspectorAdapter::shared(), &TREE_VIEW_INSPECTOR_SPEC),
            ("listbox", ListBoxInspectorAdapter::shared(), &LISTBOX_INSPECTOR_SPEC),
            ("scrollbar", ScrollbarInspectorAdapter::shared(), &SCROLLBAR_INSPECTOR_SPEC),
            ("split_view", SplitViewInspectorAdapter::shared(), &SPLIT_VIEW_INSPECTOR_SPEC),
            ("resizable_panels", ResizablePanelsInspectorAdapter::shared(), &RESIZABLE_PANELS_INSPECTOR_SPEC),
            ("popup_menu", PopupMenuInspectorAdapter::shared(), &POPUP_MENU_INSPECTOR_SPEC),
            ("context_menu", ContextMenuInspectorAdapter::shared(), &CONTEXT_MENU_INSPECTOR_SPEC),
            ("floating_menu", FloatingMenuInspectorAdapter::shared(), &FLOATING_MENU_INSPECTOR_SPEC),
            ("overlay_window", OverlayWindowInspectorAdapter::shared(), &OVERLAY_WINDOW_INSPECTOR_SPEC),
            ("badge", BadgeInspectorAdapter::shared(), &BADGE_INSPECTOR_SPEC),
            ("selection_panel", SelectionPanelInspectorAdapter::shared(), &SELECTION_PANEL_INSPECTOR_SPEC),
            ("combobox", combobox_inspector_adapter(), &COMBOBOX_INSPECTOR_SPEC),
            ("search_selector", search_selector_inspector_adapter(), &SEARCH_SELECTOR_INSPECTOR_SPEC),
        ] {
            let rows =
                colors(&adapter, selection(spec.default_part_id, spec.default_variant_id, spec.default_value_id));
            assert!(!rows.is_empty(), "{name} has no color rows");
        }
    }
}
