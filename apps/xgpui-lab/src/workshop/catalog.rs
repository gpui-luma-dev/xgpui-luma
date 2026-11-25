//! Catalog copied from Luma Studio’s Controls popup. IDs, membership, and entry order match Studio,
//! except `LAB_ONLY` and `STUDIO_ONLY` (checked by the catalog test); folder labels use Explorer casing in the Lab.

/// Entries the Lab has and Studio doesn’t: SDK controls that get their own design surface page.
#[cfg(test)]
pub const LAB_ONLY: &[&str] = &["icon-button", "dialog"];

/// Studio entries the Lab folds into one page: the four Dialog entries differ only in behavior, so the Lab
/// has one Dialog page.
#[cfg(test)]
pub const STUDIO_ONLY: &[&str] = &["dialog-modal", "dialog-modeless", "dialog-positioning", "dialog-draggable"];

#[derive(Clone, Copy, Debug)]
pub struct ComponentCatalogEntry {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct ComponentCatalogGroup {
    pub id: &'static str,
    pub label: &'static str,
    pub entries: &'static [ComponentCatalogEntry],
}

const COMMAND_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "button", label: "Button" },
    ComponentCatalogEntry { id: "custom-button", label: "Custom Button" },
    ComponentCatalogEntry { id: "icon-button", label: "Icon Button" },
    ComponentCatalogEntry { id: "toolbar", label: "Toolbar" },
];

const CHOICE_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "accordion", label: "Accordion" },
    ComponentCatalogEntry { id: "checkbox", label: "Checkbox" },
    ComponentCatalogEntry { id: "pager", label: "Pager" },
    ComponentCatalogEntry { id: "radio-button", label: "Radio Button" },
    ComponentCatalogEntry { id: "radio-group", label: "Radio Group" },
    ComponentCatalogEntry { id: "switch", label: "Switch" },
    ComponentCatalogEntry { id: "toggle", label: "Toggle" },
    ComponentCatalogEntry { id: "toggle-group", label: "Toggle Group" },
];

const COLLECTION_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "listbox", label: "ListBox" },
    ComponentCatalogEntry { id: "paging-table", label: "Paging Table" },
    ComponentCatalogEntry { id: "scrolling-table", label: "Scrolling Table" },
    ComponentCatalogEntry { id: "tree-view", label: "Tree View" },
];

const INPUT_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "scrollbar", label: "Scrollbar" },
    ComponentCatalogEntry { id: "slider", label: "Slider" },
    ComponentCatalogEntry { id: "textarea", label: "Text Area" },
    ComponentCatalogEntry { id: "textfield", label: "Text Field" },
];

const COLOR_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "color-arc", label: "Color Arc" },
    ComponentCatalogEntry { id: "color-field", label: "Color Field" },
    ComponentCatalogEntry { id: "color-ring", label: "Color Ring" },
    ComponentCatalogEntry { id: "color-slider", label: "Color Slider" },
    ComponentCatalogEntry { id: "color-slider-revealed", label: "Color Slider Revealed" },
    ComponentCatalogEntry { id: "color-multi-mixer", label: "Multi Mixer" },
];

const MENU_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "context-menu", label: "Context Menu" },
    ComponentCatalogEntry { id: "floating-menu", label: "Floating Menu" },
    ComponentCatalogEntry { id: "popup-menu", label: "Popup Menu" },
];

const LAYOUT_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "dock-panel", label: "DockPanel" },
    ComponentCatalogEntry { id: "resizable-panels", label: "Resizable Panels" },
    ComponentCatalogEntry { id: "split-view-inset", label: "Split View: Inset" },
    ComponentCatalogEntry { id: "split-view-unified", label: "Split View: Unified" },
    ComponentCatalogEntry { id: "slide-panel", label: "Slide Panel" },
];

const NAVIGATION_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "sidebar", label: "Sidebar" },
    ComponentCatalogEntry { id: "tabs-navigation", label: "Tabs Navigation" },
];

const FEEDBACK_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "badge", label: "Badge" },
    ComponentCatalogEntry { id: "progress", label: "Progress" },
    ComponentCatalogEntry { id: "stepper", label: "Stepper" },
    ComponentCatalogEntry { id: "dialog", label: "Dialog" },
];

const SELECTION_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "autocomplete-textfield", label: "Autocomplete TextBox" },
    ComponentCatalogEntry { id: "combobox", label: "ComboBox" },
    ComponentCatalogEntry { id: "search-selector", label: "SearchSelector" },
    ComponentCatalogEntry { id: "popup-selector", label: "Selector" },
    ComponentCatalogEntry { id: "selection-panel", label: "Selection Panel" },
    ComponentCatalogEntry { id: "split-button", label: "Split Button" },
];

const PROTOTYPES_ENTRIES: &[ComponentCatalogEntry] =
    &[ComponentCatalogEntry { id: "shadow-button", label: "Shadow Button" }];

/// Control catalog for the Controls tab (grouped SDK families and exposition ids).
pub const COMPONENT_CATALOG: &[ComponentCatalogGroup] = &[
    ComponentCatalogGroup { id: "command", label: "Command", entries: COMMAND_ENTRIES },
    ComponentCatalogGroup { id: "choice", label: "Choice", entries: CHOICE_ENTRIES },
    ComponentCatalogGroup { id: "collection", label: "Collection", entries: COLLECTION_ENTRIES },
    ComponentCatalogGroup { id: "input", label: "Input", entries: INPUT_ENTRIES },
    ComponentCatalogGroup { id: "color", label: "Color", entries: COLOR_ENTRIES },
    ComponentCatalogGroup { id: "menu", label: "Menu", entries: MENU_ENTRIES },
    ComponentCatalogGroup { id: "layout", label: "Layout", entries: LAYOUT_ENTRIES },
    ComponentCatalogGroup { id: "navigation", label: "Navigation", entries: NAVIGATION_ENTRIES },
    ComponentCatalogGroup { id: "feedback", label: "Feedback", entries: FEEDBACK_ENTRIES },
    ComponentCatalogGroup { id: "selection", label: "Selection", entries: SELECTION_ENTRIES },
    ComponentCatalogGroup { id: "prototypes", label: "Prototypes", entries: PROTOTYPES_ENTRIES },
];
