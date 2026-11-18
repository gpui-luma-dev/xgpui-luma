#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ControlCategory {
    Command,
    Choice,
    Inputs,
    Color,
    Selection,
    Collection,
    NavigationPanels,
    OverlaysDialogs,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlDocEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    #[allow(dead_code)]
    pub category: ControlCategory,
    pub snippet: &'static str,
    pub section_order: usize,
}

pub const CONTROL_CATALOG: &[ControlDocEntry] = &[
    ControlDocEntry {
        id: "button",
        title: "Button",
        description: "Command buttons commit actions in forms, dialogs, and toolbars. The look exposes primary, secondary, outline, and ghost builders that map to Shadcn emphasis tiers—use primary for the main call to action and ghost for low-priority utilities. Each button is a GPUI entity: use on_click for application actions and on_focus_changed, on_hover_changed, or on_enabled_changed for state transitions. The Button retains callbacks automatically.",
        category: ControlCategory::Command,
        snippet: "shadcn::Button::new(\"save\").primary().label(\"Save\")\n    .on_click(cx, |app, cx| {\n        app.save_requests += 1;\n        cx.notify();\n    })\n    .spawn(cx);",
        section_order: 100,
    },
    ControlDocEntry {
        id: "split-button",
        title: "Split Button",
        description: "A primary action with an adjacent disclosure menu and one tab stop. Enter or Space runs the primary action; Up or Down opens alternate actions. Each face has independent hover and pressed feedback. Disable only the primary action with primary_enabled(false) while keeping menu actions available.",
        category: ControlCategory::Selection,
        snippet: "let save = shadcn::SplitButton::new(\"save\").look(look.as_ref()).primary()\n    .label(\"Save\")\n    .items([MenuItem::new(\"save-as\").label(\"Save as\")])\n    .on_action_click(cx, |app: &mut Self, cx| {\n        app.save_requests += 1;\n        cx.notify();\n    })\n    .on_select(cx, |app, item_id, _, cx| {\n        app.last_action = item_id.to_string();\n        cx.notify();\n    })\n    .spawn(cx);",
        section_order: 235,
    },
    ControlDocEntry {
        id: "toolbar",
        title: "Toolbar",
        description: "Horizontal control_group hosting buttons, toggles, selectors, menus, and text fields. ShadcnToolbarItemExt factories compose editing toolbars with roving or sequential keyboard focus.",
        category: ControlCategory::Command,
        snippet: "shadcn::Toolbar::new(\"editor\").look(look.as_ref())\n    .item(look.toolbar_toggle(\"bold\", LucideIcon::Bold, cx))\n    .item(look.toolbar_button(\"link\", LucideIcon::Link, cx))\n    .spawn(cx);",
        section_order: 102,
    },
    ControlDocEntry {
        id: "custom-button",
        title: "Custom Button",
        description: "Advanced Button architecture — custom templates with layered modifiers, free-form content presenters, typed reactive data, and factory icon/text layouts beyond Shadcn presets.",
        category: ControlCategory::Command,
        snippet: "Button::new(\"custom\")\n    .content(|model, _| { /* layout */ })\n    .template(custom_template())\n    .spawn(cx);",
        section_order: 105,
    },
    ControlDocEntry {
        id: "shadow-button",
        title: "Shadow Button",
        description: "Gallery prototype for a CSS-style box shadow wrapped around the primary button template. Tune offset, blur, spread, and opacity live; preview default, hover, pressed, focus, and disabled states.",
        category: ControlCategory::Command,
        snippet: "Button::new(\"fab\")\n    .label(\"Floating Action\")\n    .template(prototype_shadow_button_template(look, controls, color))\n    .spawn(cx);",
        section_order: 106,
    },
    ControlDocEntry {
        id: "checkbox",
        title: "Checkbox",
        description: "Boolean choice controls for multi-select forms and settings. Primary and secondary builders map to Shadcn emphasis tiers; add a content closure for labeled checkboxes or omit it for indicator-only rows. Use on_value_changed to assign application state from the checked value; use set_data for programmatic sync.",
        category: ControlCategory::Choice,
        snippet: "let terms = shadcn::Checkbox::new(\"terms\").look(look.as_ref()).primary()\n    .with_data(false).label(\"Accept terms\")\n    .on_value_changed(cx, |app: &mut Self, checked, cx| {\n        app.accepted_terms = checked;\n        cx.notify();\n    })\n    .spawn(cx);",
        section_order: 110,
    },
    ControlDocEntry {
        id: "radio-button",
        title: "Radio Button",
        description: "Single-select choice within a mutually exclusive set. Standalone radio buttons emit RadioButtonEvent::Change when selected; coordinate sibling deselection in the parent or use a radio group for managed value binding. Primary and secondary builders support indicator-only or labeled content layouts.",
        category: ControlCategory::Choice,
        snippet: "let option_a = shadcn::Radio::new(\"option-a\").look(look.as_ref()).primary()\n    .with_data(false).label(\"Option A\")\n    .on_value_changed(cx, |app: &mut Self, selected, cx| {\n        app.option_a_selected = selected;\n        cx.notify();\n    })\n    .spawn(cx);",
        section_order: 120,
    },
    ControlDocEntry {
        id: "switch",
        title: "Switch",
        description: "On/off toggles for settings and feature flags. Switches use track-and-thumb presentation with primary and secondary emphasis tiers; optional content closures add inline labels. Use on_value_changed for user toggles and set_data for programmatic updates.",
        category: ControlCategory::Choice,
        snippet: "let notifications = shadcn::Switch::new(\"notifications\").look(look.as_ref()).primary()\n    .with_data(true)\n    .on_value_changed(cx, |app: &mut Self, on, cx| {\n        app.notifications_enabled = on;\n        cx.notify();\n    })\n    .spawn(cx);",
        section_order: 130,
    },
    ControlDocEntry {
        id: "radio-group",
        title: "Radio Group",
        description: "Single-selection groups with vertical, horizontal, custom indent, and card-style item layouts. Built on control_group with RadioGroupEvent::Change for value binding.",
        category: ControlCategory::Choice,
        snippet: "shadcn::RadioGroup::new(\"density\").look(look.as_ref())\n    .items(items)\n    .selected(\"comfortable\")\n    .spawn(cx);",
        section_order: 121,
    },
    ControlDocEntry {
        id: "toggle",
        title: "Toggle",
        description: "Boolean selection controls using button-family toggle chrome. Primary and secondary builders support text labels or round icon layouts.",
        category: ControlCategory::Choice,
        snippet: "shadcn::Toggle::new(\"bold\").look(look.as_ref()).secondary()\n    .with_data(false)\n    .content(|_, _| div().child(\"Bold\").into_any_element())\n    .spawn(cx);",
        section_order: 131,
    },
    ControlDocEntry {
        id: "toggle-group",
        title: "Toggle Group",
        description: "Icon toggle button clusters for toolbar-style single or multi selection. Uses control_group with toggle_button_item_template and custom pill shell layouts.",
        category: ControlCategory::Choice,
        snippet: "shadcn::IconGroup::new(\"placement\").look(look.as_ref())\n    .items(items)\n    .selected(\"bottom\")\n    .spawn(cx);",
        section_order: 132,
    },
    ControlDocEntry {
        id: "pager",
        title: "Pager",
        description: "Standalone pagination control with Minimal, MinimalEdge, and Numeric styles. Derives page count from total items and page size; emits PageChanged and PageSizeChanged.",
        category: ControlCategory::Choice,
        snippet: "shadcn::Pager::new(\"results\").look(look.as_ref())\n    .style(PagerStyle::Numeric)\n    .page_count(12)\n    .current_page(0)\n    .spawn(cx);",
        section_order: 133,
    },
    ControlDocEntry {
        id: "tree-view",
        title: "Tree View",
        description: "Virtualized hierarchical list for file explorers and nested navigation. Supports expand/collapse, single selection, and keyboard traversal.",
        category: ControlCategory::Collection,
        snippet: "shadcn::TreeView::new(\"files\").look(look.as_ref())\n    .items(tree_nodes)\n    .selection_mode(TreeViewSelectionMode::Single)\n    .spawn(cx);",
        section_order: 134,
    },
    ControlDocEntry {
        id: "badge",
        title: "Badge",
        description: "Look-specific inline status element with variant colors, shared metric scaling, and optional start/end Lucide icons. Compose at render time via look.badge — no entity lifecycle.",
        category: ControlCategory::Command,
        snippet: "look.badge(\"Verified\")\n    .variant(BadgeVariant::Default)\n    .start_icon(LucideIcon::BadgeCheck);",
        section_order: 135,
    },
    ControlDocEntry {
        id: "progress",
        title: "Progress",
        description: "Read-only progress indicator with circular ring and linear bar templates. Configure direction, optional thumb, and update value programmatically via set_value.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::Progress::linear(\"upload\").look(look.as_ref())\n    .range(0..100)\n    .value(41)\n    .direction(ProgressDirection::LeftToRight)\n    .show_thumb(true)\n    .spawn(cx);",
        section_order: 136,
    },
    ControlDocEntry {
        id: "stepper",
        title: "Stepper",
        description: "Discrete multi-step process indicator with complete, in-progress, and incomplete step badges plus connector tracks. Configure step count, current step, optional labels, and layout direction.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::Stepper::new(\"checkout\", 4).look(look.as_ref())\n    .current_step(1)\n    .labels(vec![\"Account\", \"Shipping\", \"Payment\", \"Review\"])\n    .direction(ProgressDirection::LeftToRight)\n    .spawn(cx);",
        section_order: 137,
    },
    ControlDocEntry {
        id: "accordion",
        title: "Accordion",
        description: "Collapsible sections with single or multiple expansion modes. Triggers support icons, disabled items, and custom content closures for interactive panels.",
        category: ControlCategory::Choice,
        snippet: "shadcn::Accordion::new(\"settings\").look(look.as_ref())\n    .single()\n    .item(AccordionItem::new(\"general\", trigger, content))\n    .spawn(cx);",
        section_order: 138,
    },
    ControlDocEntry {
        id: "listbox",
        title: "ListBox",
        description: "Keyed collection state with host-composed vertical rows and horizontal cards. Independent multi-select examples, keyboard navigation, and scoped composition inspectors.",
        category: ControlCategory::Collection,
        snippet: "// Create once and retain as self.list.\nlet list = ListBoxControl::new(\n    ListBoxState::try_new(items, |item| item.id, SelectionMode::Multiple)?,\n    Self::handle_input, // Apply input with self.list.apply(input, cx); forward update.events.\n    |key| (\"items\", *key).into(),\n    |item| item.label.clone(),\n    cx,\n).require_focus_for_scroll(true);\n\n// Render with your own item template.\nlook.render_listbox(\n    &mut self.list,\n    \"items\",\n    ListBoxFlow::Vertical { visible_items: 5, item_height: 36.0, gap: 4.0 },\n    (16.0, 8.0),\n    |model, cx| render_item_card(model, cx),\n    window,\n    cx,\n)",
        section_order: 305,
    },
    ControlDocEntry {
        id: "scrolling-table",
        title: "Scrolling Table",
        description: "10,000-task grid with fixed-height virtualized rows, resizable columns, range selection, drag reordering, and a jump to the last row. Resizes with the window.",
        category: ControlCategory::Collection,
        snippet: "let table = shadcn::Table::new(\"tasks\")\n    .look(&look)\n    .items(rows)\n    .extended()\n    .fill_height()\n    .visible_row_height(36.0_f32)\n    .grid_view(columns)\n    .spawn(cx);\ntable.update(cx, |table, cx| {\n    table.set_row_key(|row| row.id.clone(), cx).expect(\"unique IDs\");\n    table.set_row_reordering(true, cx).expect(\"keyed table\");\n});",
        section_order: 140,
    },
    ControlDocEntry {
        id: "paging-table",
        title: "Paging Table",
        description: "Grid table with selected-group drag reordering and an embedded pager toolbar. Page size and pager style stay wired to Table page commands for prev/next navigation.",
        category: ControlCategory::Collection,
        snippet: "let table = paging_table! {\n    table_theme = look.table_theme();\n    id = \"tasks\";\n    items = rows;\n    page_size = 10;\n    selection = TableSelectionMode::Extended;\n    pager = shadcn::Pager::new(\"pager\").look(look.as_ref()).style(PagerStyle::MinimalEdge).into_sdk_builder(cx);\n    grid_view = { /* columns */ };\n}.spawn(cx);\nlet list = table.read(cx).list().clone();\nlist.update(cx, |table, cx| {\n    table.set_row_key(|row| row.id.clone(), cx).expect(\"unique IDs\");\n    table.set_row_reordering(true, cx).expect(\"keyed table\");\n});",
        section_order: 140,
    },
    ControlDocEntry {
        id: "textfield",
        title: "Text Field",
        description: "Single-line text entry for labels, search, and form values. Supports placeholder text, full-width layout, compact density, and look overrides for monospace or token-style fields. Focus, selection, and clipboard behavior live in the SDK editing engine; the parent owns the string value and updates the field programmatically when model data changes.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::TextField::new(\"email\").look(look.as_ref())\n    .placeholder(\"Email\")\n    .full_width(true)\n    .spawn(cx);",
        section_order: 200,
    },
    ControlDocEntry {
        id: "textarea",
        title: "Text Area",
        description: "Multiline text input with hard-line editing, selection, escape-clear policy, optional validation, and fixed row height. Gallery-aligned demo includes option toggles and variant state previews.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::TextArea::new(\"notes\").look(look.as_ref())\n    .placeholder(\"Write a message…\")\n    .rows(6)\n    .full_width(true)\n    .spawn(cx);",
        section_order: 201,
    },
    ControlDocEntry {
        id: "slider",
        title: "Slider",
        description: "Unified linear and angular slider engine — fill tracks, vertical and reversed orientation, blocked intervals, multi-stop thumbs, dials, and size variants. Subscribe to SliderEvent::Change for live preview and SliderEvent::Release to commit.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::Slider::new(\"volume\").look(look.as_ref())\n    .range(0..100)\n    .step(1)\n    .value(50)\n    .spawn(cx);",
        section_order: 202,
    },
    ControlDocEntry {
        id: "scrollbar",
        title: "Scrollbar",
        description: "Standalone scroll thumb and track control for custom scroll surfaces. Supports horizontal and vertical orientation, step/page step, and thumb fraction sizing. Pair with a clipped content viewport for classic scroll UI.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::Scrollbar::new(\"list-scroll\").look(look.as_ref())\n    .vertical()\n    .range(0..500)\n    .step(20)\n    .value(0)\n    .spawn(cx);",
        section_order: 203,
    },
    ControlDocEntry {
        id: "context-menu",
        title: "Context Menu",
        description: "Right-click targets that open anchored floating menus. Compose MenuItem trees with icons and nested submenus; subscribe to ContextMenuEvent::Select for the chosen action. Shadcn look binds themed target chrome and floating menu tokens.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "shadcn::ContextMenu::new(\"ctx\").look(look.as_ref())\n    .label(\"Right-click me\")\n    .items(menu_items)\n    .spawn(cx);",
        section_order: 220,
    },
    ControlDocEntry {
        id: "floating-menu",
        title: "Floating Menu",
        description: "Template primitive for menu item lists — shared by popup menus, context menus, and navigation submenus. render_floating_menu accepts item hover/click handler slots; FloatingMenuState drives keyboard navigation and submenu paths.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "render_floating_menu(&id, &items, open_submenu, active_path, look, hovers, clicks);",
        section_order: 221,
    },
    ControlDocEntry {
        id: "popup-menu",
        title: "Popup Menu",
        description: "Labeled triggers that open anchored popup menus with smart or fixed placement. Supports ghost triggers, submenu nesting, and disabled items. Subscribe to PopupMenuEvent::Select for activation.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "shadcn::PopupMenu::new(\"menu\").look(look.as_ref())\n    .label(\"Actions\")\n    .items(menu_items)\n    .placement(PopupMenuPlacement::Smart)\n    .spawn(cx);",
        section_order: 222,
    },
    ControlDocEntry {
        id: "autocomplete-textfield",
        title: "Autocomplete TextBox",
        description: "Single-line text field with typeahead popup filtering. Emits Change while typing and Select or Complete when an item is committed. Ideal for token fields, address lookup, and constrained free text.",
        category: ControlCategory::Selection,
        snippet: "shadcn::Autocomplete::new(\"city\", items).look(look.as_ref())\n    .placeholder(\"Start typing…\")\n    .full_width(true)\n    .spawn(cx);",
        section_order: 230,
    },
    ControlDocEntry {
        id: "combobox",
        title: "ComboBox",
        description: "Editable text field with anchored item popup — strict or permissive typing policies, down-arrow and clear affordances, and second-tier list/panel templates for custom row chrome.",
        category: ControlCategory::Selection,
        snippet: "shadcn::ComboBox::new(\"state\", items).look(look.as_ref())\n    .typing_policy(TypingPolicy::Strict)\n    .show_down_arrow(true)\n    .spawn(cx);",
        section_order: 231,
    },
    ControlDocEntry {
        id: "search-selector",
        title: "SearchSelector",
        description: "Read-only trigger with a popup search field and scrollable item list. Separates trigger label from in-popup filtering — useful when the committed value should stay compact while search is expansive.",
        category: ControlCategory::Selection,
        snippet: "shadcn::SearchSelector::new(\"state\", items).look(look.as_ref())\n    .placeholder(\"Choose…\")\n    .search_placeholder(\"Search\")\n    .spawn(cx);",
        section_order: 232,
    },
    ControlDocEntry {
        id: "popup-selector",
        title: "Selector",
        description: "Labeled triggers with anchored selector popups — placement strategies include smart flip, below/above start, and overlay-on-trigger. Custom item templates support swatches, icons, and two-line rows.",
        category: ControlCategory::Selection,
        snippet: "shadcn::Selector::new(\"status\").look(look.as_ref())\n    .label(\"Select status\")\n    .items(selector_items)\n    .placement(SelectorPlacement::Smart)\n    .spawn(cx);",
        section_order: 233,
    },
    ControlDocEntry {
        id: "selection-panel",
        title: "Selection Panel",
        description: "Scrollable selectable item panels with keyboard navigation, row hover/active states, and template hooks for custom item and panel chrome. Emits ActivateRow, ActiveIndexChanged, and HoverChanged for list-driven workflows.",
        category: ControlCategory::Selection,
        snippet: "shadcn::SelectionPanel::new(\"actions\").look(look.as_ref())\n    .items(items)\n    .scrolling(true)\n    .spawn(cx);",
        section_order: 234,
    },
    ControlDocEntry {
        id: "color-slider",
        title: "Color Slider",
        description: "Spectrum sliders built on the unified Slider engine with color-specific delegates for hue, saturation, alpha, channel, and gradient interpolation. ColorSliderBuilder returns Slider entities—subscribe to SliderEvent::Change for live preview and SliderEvent::Release to commit values. Linked saturation and alpha tracks refresh when the base color changes.",
        category: ControlCategory::Color,
        snippet: "let hue_slider = ColorSliderBuilder::hue(\"accent-hue\", 210.0)\n    .size(ShadcnSize::Sm).thumb_medium().spawn(cx)\n    .on_release(cx, |app: &mut Self, _, value, cx| {\n        app.hue = value;\n        cx.notify();\n    });",
        section_order: 210,
    },
    ControlDocEntry {
        id: "color-field",
        title: "Color Field",
        description: "2D color selection surfaces for saturation/value, hue/value, triangle, and wheel domains. Supports vector rendering and prewarmed raster image fields.",
        category: ControlCategory::Color,
        snippet: "cx.new(|_| {\n    ColorFieldState::saturation_value_rect(\"sv\", hsv, thumb_size)\n        .vector()\n        .rounded(px(12.0))\n});",
        section_order: 211,
    },
    ControlDocEntry {
        id: "color-ring",
        title: "Color Ring",
        description: "Circular hue, saturation, and lightness sliders built on the unified slider engine with ring-specific delegates. Vector and raster rendering paths available.",
        category: ControlCategory::Color,
        snippet: "ColorRingBuilder::hue(\"ring\", h, s, l)\n    .size(Size::Medium)\n    .allow_inner_target(true)\n    .spawn(cx);",
        section_order: 212,
    },
    ControlDocEntry {
        id: "color-arc",
        title: "Color Arc",
        description: "Partial-ring color sliders with configurable sweep angles and track thickness. Hue, saturation, and lightness delegates share arc geometry helpers.",
        category: ControlCategory::Color,
        snippet: "ColorArcBuilder::hue(\"arc\", h, s, l)\n    .size(Size::Medium)\n    .start_degrees(-45.0)\n    .sweep_degrees(270.0)\n    .spawn(cx);",
        section_order: 213,
    },
    ControlDocEntry {
        id: "color-multi-mixer",
        title: "Multi Mixer",
        description: "Per-color-space channel mixers — HueAlpha, RGBA, HSLA, HSVA, Lab variants, and OKLCH. Each card binds ColorSliderBuilder channel delegates to a ColorSpecification.",
        category: ControlCategory::Color,
        snippet: "ColorSpaceMixerState::new(\"hsva\", None, true, look, Hsv::from_hsla(color), cx);",
        section_order: 219,
    },
    ControlDocEntry {
        id: "color-slider-revealed",
        title: "Color Slider Revealed",
        description: "Styling survey for the color-slider primitive — scale, corner radius, thumb shapes, edge-to-edge layout, interpolation modes, and delegate families.",
        category: ControlCategory::Color,
        snippet: "ColorSliderBuilder::hue(\"hue\", 180.0)\n    .size(ShadcnSize::Lg)\n    .thumb_square()\n    .edge_to_edge()\n    .spawn(cx);",
        section_order: 221,
    },
    ControlDocEntry {
        id: "modal-overlay",
        title: "Modal Overlay",
        description: "Modal overlay windows block the workspace for confirm/cancel or short-form tasks. They trap focus, dim the background, and restore focus to the opener on dismiss. Compose header, body, and footer in the content closure; register persistent child controls with theme_children so token changes refresh without app-level notify plumbing.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "look\n    .overlay_window(\"confirm-archive\")\n    .mode(OverlayWindowMode::Modal)\n    .content(|_, _, _| { /* body */ })\n    .theme_children([cancel.clone(), confirm.clone()])\n    .spawn(cx);",
        section_order: 300,
    },
    ControlDocEntry {
        id: "modeless-overlay",
        title: "Modeless Overlay",
        description: "Non-blocking overlay surfaces leave the workspace interactive. Use modeless overlays for notes, palettes, and inspectors that should stay open while the user keeps working in the background.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "look\n    .overlay_window(\"notes\")\n    .mode(OverlayWindowMode::Modeless)\n    .position(OverlayWindowPosition::TopRight)\n    .content(|_, _, _| { /* body */ })\n    .spawn(cx);",
        section_order: 301,
    },
    ControlDocEntry {
        id: "overlay-positioning",
        title: "Overlay Positioning",
        description: "Reposition the same modeless overlay at center, corner, or absolute coordinates. Call set_position before open to anchor the surface where the workflow needs it.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "overlay.set_position(OverlayWindowPosition::TopRight, cx);\noverlay.open_from(Some(opener_focus), cx);",
        section_order: 302,
    },
    ControlDocEntry {
        id: "draggable-overlay",
        title: "Draggable Overlay",
        description: "Modeless overlays with header drag repositioning. Enable draggable on the builder or entity, then drag the top strip to move tool palettes and floating panels.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "look\n    .overlay_window(\"palette\")\n    .mode(OverlayWindowMode::Modeless)\n    .absolute_position(240.0, 180.0)\n    .draggable(true)\n    .spawn(cx);",
        section_order: 303,
    },
    ControlDocEntry {
        id: "dock-panel",
        title: "Dock Panel",
        description: "Ordered docking layout macro — left, top, right, bottom bands plus a fill child. Pair dock boundaries with DockSplitter entities for draggable resize handles.",
        category: ControlCategory::NavigationPanels,
        snippet: "dock_panel! {\n    left: sidebar,\n    left: left_splitter,\n    fill: editor_content,\n}",
        section_order: 310,
    },
    ControlDocEntry {
        id: "resizable-panels",
        title: "Resizable Panels",
        description: "Weighted panel splits with draggable handles, min/max constraints, hover visibility, and collapse gestures. Horizontal or vertical orientation via builder or resizable_panels! macro.",
        category: ControlCategory::NavigationPanels,
        snippet: "look.resizable_panels(\"workspace\")\n    .orientation(Horizontal)\n    .panels([sidebar_spec, content_spec])\n    .spawn(cx);",
        section_order: 311,
    },
    ControlDocEntry {
        id: "sidebar",
        title: "Sidebar",
        description: "Bare sidebar navigation with header, grouped menus, footer, and expanded/icon presentation. Wrap in Frame for container styling; the host owns width and visibility.",
        category: ControlCategory::NavigationPanels,
        snippet: r#"fn spawn_sidebar_control(
    look: &Arc<ShadcnLook>,
    cx: &mut Context<SidebarControlExposition>,
) -> Entity<SidebarControl> {
    let mut pinned_menu = shadcn::Sidebar::menu("pinned_menu");

    for leaf in PINNED_PROPERTIES {
        pinned_menu = pinned_menu.item(property_leaf_menu_item(look, leaf));
    }

    let mut properties_menu = shadcn::Sidebar::menu("properties_menu");

    for group in PROPERTY_GROUPS {
        let mut sub = shadcn::Sidebar::menu_sub();

        for leaf in group.leaves {
            sub = sub.item(property_leaf_menu_item(look, leaf));
        }

        properties_menu = properties_menu.item(
            shadcn::Sidebar::menu_item(group.id, group.label)
                .icon(group.icon)
                .expanded(group.expanded)
                .sub(sub),
        );
    }

    shadcn::Sidebar::new("controls-doc-sidebar-control").look(look.as_ref())
        .auto_hide_scrollbar(true)
        .auto_hide_scrollbar_activate(ScrollbarAutoHideActivate::Move)
        .sidebar(
            shadcn::Sidebar::panel("workbench_sidebar")
                .header(
                    shadcn::Sidebar::header()
                        .title("Properties")
                        .subtitle("Rectangle / Prominent card"),
                )
                .content(
                    shadcn::Sidebar::content()
                        .group(shadcn::Sidebar::group().label("Pinned").menu(pinned_menu))
                        .group(
                            shadcn::Sidebar::group()
                                .label("Properties")
                                .menu(properties_menu),
                        ),
                )
                ,
        )
        .overlay_scrollbar(true)
        .spawn(cx)
}"#,
        section_order: 312,
    },
    ControlDocEntry {
        id: "tabs-navigation",
        title: "Tabs Navigation",
        description: "Horizontal tab strip with intrinsic or uniform width modes, keyboard roving focus, and template overrides. Emits Activate and Change for tab selection binding.",
        category: ControlCategory::NavigationPanels,
        snippet: "shadcn::Tabs::new(\"project\").look(look.as_ref())\n    .items(tabs)\n    .active(\"overview\")\n    .spawn(cx);",
        section_order: 313,
    },
    ControlDocEntry {
        id: "slide-panel",
        title: "Slide Panel",
        description: "Gallery drawer prototype — deferred window overlay with edge-anchored panels, open/close animation, focus trap and restore, Escape and backdrop dismissal, and draggable resize handles.",
        category: ControlCategory::NavigationPanels,
        snippet: "let mut state = SlidePanelState::new(SlidePanelTopAnchor::BelowTopBar, SlidePanelSizeConfig::new(360.0, 280.0, 720.0));\nstate.open(SlidePanelEdge::Right, opener_focus);\nrender_slide_panel_overlay(slide_panel_background(&look), &slide_panel_panels_look(&look), &state, viewport, content, handlers, resize_handlers);",
        section_order: 314,
    },
];

pub fn catalog_entry(id: &str) -> Option<&'static ControlDocEntry> {
    CONTROL_CATALOG.iter().find(|entry| entry.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::studio::components::catalog::{catalog_group, controls_exposition_id, groups_for_column};

    #[test]
    fn collection_menu_and_documentation_agree() {
        let expected = ["listbox", "paging-table", "scrolling-table", "tree-view"];
        let group = catalog_group("collection").unwrap();
        assert_eq!(group.label, "COLLECTION");
        assert_eq!(group.entries.iter().map(|entry| entry.id).collect::<Vec<_>>(), expected);

        let mut documented: Vec<_> = CONTROL_CATALOG
            .iter()
            .filter(|entry| entry.category == ControlCategory::Collection)
            .map(|entry| entry.id)
            .collect();
        documented.sort_unstable();
        assert_eq!(documented, expected);

        // Check the actual dropdown columns, including duplicate or missing groups.
        let groups: Vec<_> = (0..4).flat_map(groups_for_column).collect();
        assert_eq!(groups.iter().filter(|group| group.id == "collection").count(), 1);
        for id in expected {
            let entries: Vec<_> =
                groups.iter().flat_map(|group| group.entries.iter()).filter(|entry| entry.id == id).collect();
            assert_eq!(entries.len(), 1, "dropdown entry {id}");
            // Stable exposition IDs also preserve the picker's active-entry match.
            assert_eq!(controls_exposition_id(id), Some(id));
            assert_eq!(catalog_entry(id).unwrap().title, entries[0].label);
        }
    }
}
