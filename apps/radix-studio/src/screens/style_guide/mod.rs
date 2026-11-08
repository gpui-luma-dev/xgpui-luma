//! Style Guide tab screen.

mod state;
pub use state::State;

mod avatars;
mod badges;
mod buttons;
mod cards;
mod collections;
mod checkboxes;
mod context_menus;
mod high_contrast;
mod matrix_grid;
mod menus;
mod selection_previews;
mod navigation_previews;
mod palettes;
mod panel_previews;
mod preview_handlers;
mod progress;
mod radios;
mod sliders;
mod split_button_previews;
mod states;
mod switches;
mod table;
mod table_previews;
mod tabs;
mod textareas;
mod textfields;
mod tooltips;

use std::sync::Arc;

use gpui::{AnyElement, App, Entity, IntoElement, Window, div, prelude::*, px};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::controls::tree_view::TreeView;
use xgpui_luma::vstack;
use xgpui_luma_look_radix::{Look, ScaleFamily, SemanticRole};

use super::section::section;

pub use tabs::TabsExamples;

#[derive(Clone)]
pub struct PreviewTabs {
    pub avatars: Entity<Tabs>,
    pub badges: Entity<Tabs>,
    pub examples: TabsExamples,
    pub buttons: Entity<Tabs>,
    pub checkboxes: Entity<Tabs>,
    pub radios: Entity<Tabs>,
    pub switches: Entity<Tabs>,
    pub textfields: Entity<Tabs>,
    pub textareas: Entity<Tabs>,
    pub sliders: Entity<Tabs>,
    pub progress: Entity<Tabs>,
    pub autocomplete: Entity<Tabs>,
    pub combobox: Entity<Tabs>,
    pub search_selector: Entity<Tabs>,
    pub selection_panel: Entity<Tabs>,
    pub scrollbar: Entity<Tabs>,
    pub table: Entity<Tabs>,
    pub split_button: Entity<Tabs>,
    navigation: navigation_previews::Previews,
    panels: panel_previews::Previews,
    pub(crate) tooltips: tooltips::TooltipExamples,
    pub(crate) collections: collections::Previews,
}

pub fn page(
    look: &Arc<Look>,
    tabs: PreviewTabs,
    tree: TreeView<()>,
    disabled_tree: TreeView<()>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let fg = look.resolve_role(SemanticRole::Foreground).hsla();
    let muted = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let border = look.resolve_role(SemanticRole::Border).hsla();
    let accent = palettes::title_case(look.palette_label(ScaleFamily::Color));
    let gray = palettes::title_case(look.palette_label(ScaleFamily::Gray));

    vstack! {
        gap=28;
        section(
            "Palette",
            format!("Every matrix below paints from {accent} (accent) and {gray} (gray)."),
            fg,
            muted,
            border,
            palettes::indicator(look, muted),
        ),
        section(
            "Accordion",
            "Collapsed and expanded content, trigger states, and all control sizes.",
            fg, muted, border,
            tabs.navigation.render(navigation_previews::Kind::Accordion, look, window, cx),
        ),
        section(
            "Autocomplete",
            "Text-field states, completion status, and an expanded suggestions template in the focused sample.",
            fg, muted, border,
            selection_previews::tabbed(selection_previews::Kind::Autocomplete, look, tabs.autocomplete, [fg, muted, border], window, cx),
        ),
        section(
            "Avatar",
            "Image, icon, or initials; Solid and Soft, with nine sizes and five radius options.",
            fg, muted, border,
            avatars::tabbed(look, tabs.avatars, muted, border, cx),
        ),
        section(
            "Badge",
            "Solid, Soft, Surface, and Outline; accent and gray, each paired with high contrast.",
            fg, muted, border,
            badges::tabbed(look, tabs.badges, muted, border, cx),
        ),
        section(
            "Button",
            format!("Radix variants on {accent}, across interaction states."),
            fg,
            muted,
            border,
            buttons::tabbed(look, tabs.buttons, fg, muted, border, window, cx),
        ),
        section(
            "Button · High contrast",
            format!("Gray tone ({gray}) with Radix highContrast."),
            fg,
            muted,
            border,
            high_contrast::strip(look, muted, window, cx),
        ),
        section(
            "Card",
            "Surface, Classic, and Ghost containers in sizes 1–3.",
            fg, muted, border,
            cards::matrix(look, muted),
        ),
        section(
            "Checkbox",
            format!(
                "Radix variants on {accent}, across interaction states. Each cell pairs unchecked and checked."
            ),
            fg,
            muted,
            border,
            checkboxes::tabbed(look, tabs.checkboxes, fg, muted, border, window, cx),
        ),
        tabs.collections.checkboxes.clone(),
        section(
            "ComboBox",
            "Text-field and disclosure chrome, with an expanded options template in the focused sample.",
            fg, muted, border,
            selection_previews::tabbed(selection_previews::Kind::ComboBox, look, tabs.combobox, [fg, muted, border], window, cx),
        ),
        section(
            "Context Menu",
            "Template preview · Solid and Soft, in accent and gray.",
            fg, muted, border,
            context_menus::preview(look, fg),
        ),
        section(
            "Dock Splitter",
            "SDK resize separators in both orientations, with hover, focus, drag, and disabled chrome.",
            fg, muted, border,
            tabs.navigation.render(navigation_previews::Kind::DockSplitter, look, window, cx),
        ),
        section(
            "Icon Group",
            "Single and multiple selection in the shared icon-toolbar shell.",
            fg, muted, border,
            tabs.navigation.render(navigation_previews::Kind::IconGroup, look, window, cx),
        ),
        tabs.collections.lists.clone(),
        section(
            "Menu",
            format!(
                "Popup menu triggers on {accent}, with a {gray}-locked pair for the neutral slot."
            ),
            fg,
            muted,
            border,
            menus::matrix(look, &menus::RADIX_VARIANTS, fg, muted, window, cx),
        ),
        section(
            "Menu Popup Pane",
            "Template preview · Solid and Soft, in accent and gray, with a selected item and separator.",
            fg, muted, border,
            menus::pane_preview(look, fg),
        ),
        section(
            "Pager",
            "Minimal, edge, and numeric layouts at the first, middle, and last pages, plus disabled states.",
            fg, muted, border,
            tabs.navigation.render(navigation_previews::Kind::Pager, look, window, cx),
        ),
        section(
            "Progress",
            "Surface and Soft; sizes 1–3, high contrast, and all five radius options.",
            fg, muted, border,
            progress::tabbed(look, tabs.progress, fg, muted, border, window, cx),
        ),
        section(
            "Radio",
            format!(
                "Radix variants on {accent}, across interaction states. Each cell pairs unselected and selected."
            ),
            fg,
            muted,
            border,
            radios::tabbed(look, tabs.radios, fg, muted, border, window, cx),
        ),
        tabs.collections.radios.clone(),
        section(
            "Resizable Panels",
            "SDK pane surfaces and resize handles in both orientations, across states, palettes, and handle sizes.",
            fg, muted, border,
            tabs.panels.render(panel_previews::Kind::Resizable, look, window, cx),
        ),
        section(
            "Scrollbar",
            "Ghost and Soft tracks in both orientations; palette comparisons use neutral scales.",
            fg, muted, border,
            selection_previews::tabbed(selection_previews::Kind::Scrollbar, look, tabs.scrollbar, [fg, muted, border], window, cx),
        ),
        section(
            "Search Selector",
            "Outline and Ghost triggers, with an expanded searchable panel in the focused sample.",
            fg, muted, border,
            selection_previews::tabbed(selection_previews::Kind::SearchSelector, look, tabs.search_selector, [fg, muted, border], window, cx),
        ),
        section(
            "Selection Panel",
            "Selected, active, pressed, and disabled option rows rendered through the SDK panel template.",
            fg, muted, border,
            selection_previews::tabbed(selection_previews::Kind::SelectionPanel, look, tabs.selection_panel, [fg, muted, border], window, cx),
        ),
        section(
            "Sidebar",
            "Expanded navigation and icon rails, with selected, focused, and disabled rows.",
            fg, muted, border,
            tabs.navigation.render(navigation_previews::Kind::Sidebar, look, window, cx),
        ),
        section(
            "Slide Panel",
            "SDK drawer surfaces on all four edges, shown locally with resting, hover, and resize chrome.",
            fg, muted, border,
            tabs.panels.render(panel_previews::Kind::Slide, look, window, cx),
        ),
        section(
            "Slider",
            format!("Radix variants on {accent}, across interaction states."),
            fg,
            muted,
            border,
            sliders::tabbed(look, tabs.sliders, fg, muted, border, window, cx),
        ),
        section(
            "Split Button",
            "SDK primary and menu faces, including independent feedback and icon content.",
            fg, muted, border,
            split_button_previews::tabbed(look, tabs.split_button, window, cx),
        ),
        section(
            "Split View",
            "Expanded and collapsed SDK panes with themed separators and geometry comparisons.",
            fg, muted, border,
            tabs.navigation.render(navigation_previews::Kind::SplitView, look, window, cx),
        ),
        section(
            "Stepper",
            "Complete, current, and upcoming steps in horizontal and vertical layouts.",
            fg, muted, border,
            tabs.navigation.render(navigation_previews::Kind::Stepper, look, window, cx),
        ),
        section(
            "Switch",
            format!(
                "Radix variants on {accent}, across interaction states. Each cell pairs off and on."
            ),
            fg,
            muted,
            border,
            switches::tabbed(look, tabs.switches, fg, muted, border, window, cx),
        ),
        section(
            "Table",
            "SDK headers and rows across interaction states, palettes, and sizes; selected and disabled rows included.",
            fg, muted, border,
            table_previews::tabbed(look, tabs.table, window, cx),
        ),
        section(
            "Tabs",
            "Underline tabs across colors and sizes. Select tabs or use arrow keys; the Colors sample tab is disabled.",
            fg,
            muted,
            border,
            tabs.examples.render(look, muted, border, cx),
        ),
        section(
            "Text Area",
            format!("Radix variants on {accent}, across interaction states including invalid."),
            fg,
            muted,
            border,
            textareas::tabbed(look, tabs.textareas, fg, muted, border, window, cx),
        ),
        section(
            "Text Field",
            format!("Radix variants on {accent}, across interaction states including invalid."),
            fg,
            muted,
            border,
            textfields::tabbed(look, tabs.textfields, fg, muted, border, window, cx),
        ),
        section(
            "Tooltips",
            "Standard Radix help, shortcut hints and wrapping. Static presentation previews.",
            fg, muted, border,
            tabs.tooltips.render(look),
        ),
        section(
            "Tree View",
            "Select rows and expand Grid. Use arrow keys to navigate. Selection uses accent 3/12; hover outlines use accent 8; pressed uses accent 5.",
            fg,
            muted,
            border,
            div().flex().flex_wrap().gap(px(24.0)).children([
                vstack! {
                    gap=8;
                    div().text_sm().text_color(muted).child("Interactive"),
                    super::shared::tree_view::panel(tree, look),
                }.w(px(320.0)).into_any_element(),
                vstack! {
                    gap=8;
                    div().text_sm().text_color(muted).child("Disabled"),
                    super::shared::tree_view::panel(disabled_tree, look),
                }.w(px(320.0)).into_any_element(),
            ]).into_any_element(),
        ),
    }
    .w_full()
    .into_any_element()
}
