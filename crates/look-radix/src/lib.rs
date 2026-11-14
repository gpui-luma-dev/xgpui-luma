//! Radix look for XGPUI-Luma: palette scales, control styling, and SDK adapters.
//!
//! [`Look`] owns named or custom color scales, light/dark mode, metrics, and shared
//! styling settings. Interactive builders produce SDK controls, retaining their
//! input, focus, event, and accessibility behavior. Display elements render directly.
//!
//! # Available controls
//!
//! - Display: [`Avatar`], [`Badge`], [`Card`], and [`callout`].
//! - Actions and choices: [`Button`], [`Checkbox`], [`CheckboxGroup`], [`Radio`], [`RadioGroup`], [`Switch`], [`Toggle`].
//! - Input: [`TextField`], [`TextArea`], [`Slider`].
//! - Progress: [`Progress`] (Surface / Soft bars, sizes 1–3).
//! - Navigation and collections: [`Tabs`], [`Toolbar`], [`PopupMenu`], [`ContextMenu`], [`TreeView`], [`ListBoxBuilder`].
//! - Selection factories: [`autocomplete_builder`], [`combobox_builder`], [`search_selector_builder`], [`selection_panel_builder`].
//! - Groups/navigation: [`control_group_builder`], [`icon_group_builder`], [`accordion_builder`], [`pager_builder`], [`stepper_builder`], [`sidebar_builder`].
//! - Tables: [`table_builder`], [`paging_table_builder`], [`table_theme`], [`table_template`].
//! - Resizing: [`dock_splitter_builder`], [`split_view_builder`].
//! - Panels: [`resizable_panels_builder`], [`slide_panel_state`], [`render_slide_panel_overlay`], [`render_slide_panel_inset`].
//! - Split actions: [`split_button_builder`], [`split_button_template`], [`split_button_theme`].
//! - Shared chrome: [`floating_menu_look`], [`selector_items_panel_look`], [`scrollbar_template`].
//! - Popup content: [`HoverCard`] (interactive hover details), with [`hover_card_look`].
//! - Composition: [`segmented_radio_template`] and [`LookControlExt::overlay_window`].
//!
//! # Start with a control builder
//!
//! Choose the control's supported variant, size, tone, and radius options first.
//! Bind `.look(&look)` explicitly when using a fork. Without an explicit look,
//! interactive builders use the ambient GPUI global [`Look`], then built-in defaults.
//!
//! ```no_run
//! use gpui::{Context, Entity};
//! use xgpui_luma::infra::presenter::HasPresenter;
//! use xgpui_luma_look_radix::{Button, ButtonSize, Look};
//!
//! fn save_button<M: 'static>(look: &Look, cx: &mut Context<M>)
//!     -> Entity<xgpui_luma::controls::button::Button>
//! {
//!     Button::new("save").look(look).solid().size(ButtonSize::Two)
//!         .label("Save").spawn(cx)
//! }
//! ```
//!
//! [`Avatar`], [`Badge`], and [`Card`] take a look in `new` and implement
//! [`gpui::IntoElement`]; rebuild them in the owning view's render method.
//!
//! # Choose the customization scope
//!
//! | Need | Extension point |
//! | --- | --- |
//! | Standard control appearance | Builder options such as `variant`, `size`, `tone`, and `radius`, where supported |
//! | Application-specific label/icon composition | Presenter via [`xgpui_luma::infra::presenter::HasPresenter::content`] |
//! | Local button/tab layout | [`Button::with_template_modifier`] or [`Tabs::with_template_modifier`] |
//! | Local display styling | [`Avatar::style_override`] and [`Card::style_override`] |
//! | Local toolbar geometry | [`Toolbar::style`], [`Toolbar::command_size`], [`Toolbar::icon_size`] |
//! | Shared tab or Classic button styling | [`Look::set_tabs_style`] or [`Look::set_classic_params`] |
//! | Custom SDK integration or forced-state previews | Exported `*_theme`, `*_template`, and resolver functions; [`LookControlExt`] |
//!
//! Keep profile content, selected-label emphasis, and other application-specific
//! presentation in the application. Local overrides do not mutate the shared look.
//! Avatar/Card overrides receive resolved styles: change only the fields needed
//! so the remaining colors still follow mode and palette changes.
//!
//! ```
//! use xgpui_luma_look_radix::{Card, CardVariant, Look};
//! let look = Look::built_in();
//! let card = Card::new(&look).variant(CardVariant::Classic)
//!     .style_override(|style| style.padding = 20.0);
//! assert_eq!(card.resolve_style().padding, 20.0);
//! ```
//!
//! # Palettes, ownership, and redraws
//!
//! [`Accent`] and [`Gray`] select named palettes; [`Tone`] chooses accent or neutral
//! control colors. [`ScaleFamily`] and [`SemanticRole`] provide direct resolution.
//! [`generate_colors`] retains source seeds and generated colors, with precomputed
//! sRGB previews for the current GPUI backend. [`Look::set_custom_colors`] updates only the active mode.
//!
//! Cloning a look shares its state. [`Look::fork`] copies it independently for a
//! preview or editor. Shared settings affect controls bound to that same look;
//! control-local overrides remain local.
//!
//! Setters update state and revision, **not GPUI notifications**. Notify affected
//! control entities and their owning view after changes. See [`Look`]'s compiling
//! redraw example. A revision is a cache key, not a redraw subscription.
//!
//! # Source organization
//!
//! Palette state and resolution live in `look`, `palette`, `scale`, `semantic`, and
//! `custom_colors`. Each interactive control's `*_builder` module adapts public
//! options to SDK construction; its companion module resolves styling/templates.
//! Display controls keep their element and resolved style together. All public
//! entry points are re-exported here; private modules are implementation details.

mod accordion;
mod control_group;
mod navigation;
mod pager;
mod sidebar;
mod table;
mod splitters;
mod split_button;
mod panels;
mod avatar;
mod badge;
mod button;
mod button_builder;
mod button_layout;
mod callout;
mod card;
mod checkbox;
mod checkbox_builder;
mod checkbox_group;
mod colors;
mod custom_colors;
mod context_menu;
mod context_menu_builder;
mod ext;
mod look;
mod hover_card;
mod listbox;
mod overlay_window;
mod palette;
mod popup_menu;
mod popup_menu_builder;
mod progress;
mod progress_builder;
mod radio;
mod radio_builder;
mod radio_group;
mod scale;
mod segmented;
mod selector;
mod selector_list;
mod selection;
mod scrollbar;
mod semantic;
mod slider;
mod slider_builder;
mod switch;
mod switch_builder;
mod tabs;
mod tabs_builder;
mod textarea;
mod textarea_builder;
mod textfield;
mod textfield_builder;
mod toggle;
mod toggle_builder;
mod tone;
mod tooltip;
mod toolbar;
mod toolbar_builder;
mod tree_view;
mod typography;

// Palette state, scales, and shared styling.
pub use look::{PageBackground, Look};
pub use hover_card::{HoverCard, HoverCardBuilder, HoverCardEvent, HoverCardEventExt, hover_card_look};
pub use listbox::{ListBoxBuilder, listbox_row, listbox_template};
pub use palette::{PaletteSlot, Accent, Gray, ThemePalettes, scale_pair};
pub use scale::{CUSTOM_PALETTE, ColorScale, ModeScales, SCALE_LEN, ScaleFamily, ScalePair, ScaleStep};
pub use colors::{BLACK_ALPHA_STEPS, DARK_FAMILIES, LIGHT_FAMILIES, COLORS_VERSION, WHITE_ALPHA_STEPS};
pub use colors::{
    RawColorScale, ColorValueKind, families as color_families, parse_color, parse_source_color, family_source_steps,
};
pub use custom_colors::{CustomColors, GeneratedColors, generate_colors};
pub use semantic::{SemanticMapping, SemanticRole};
pub use tone::Tone;
pub use button_layout::{ButtonSize, Radius, button_box_for, metric_tokens, resolve_button_radius};

// Controls and their styling adapters (alphabetical).
pub use avatar::{Avatar, AvatarSize, AvatarStyle, AvatarVariant};
pub use badge::{Badge, BadgeSize, BadgeVariant};
pub use button::{
    ClassicButtonParams, Paint, ButtonVariant, button_family_theme, button_family_theme_with, button_look_for,
    button_template, classic_button_template, classic_button_template_with,
};
pub use button_builder::Button;
pub use callout::callout;
pub use context_menu::{ContextMenuVariant, context_menu_template, context_menu_theme};
pub use context_menu_builder::ContextMenu;
pub use card::{Card, CardSize, CardStyle, CardVariant};
pub use checkbox::{
    CheckboxSize, CheckboxVariant, checkbox_scale_for, checkbox_template, checkbox_template_for, checkbox_theme,
    checkbox_theme_for, checkbox_theme_with, resolve_checkbox_radius,
};
pub use checkbox_builder::Checkbox;
pub use checkbox_group::{CheckboxGroup, checkbox_group_template};
pub use popup_menu::{PopupMenuVariant, floating_menu_look, floating_menu_look_for, popup_menu_template, popup_menu_theme};
pub use popup_menu_builder::PopupMenu;
pub use progress::{
    ProgressSize, ProgressVariant, progress_template, progress_theme, progress_theme_with, resolve_progress_radius,
};
pub use progress_builder::Progress;
pub use radio::{
    RadioSize, RadioVariant, radio_scale_for, radio_template, radio_template_for, radio_theme, radio_theme_for,
    radio_theme_with,
};
pub use radio_builder::Radio;
pub use radio_group::{RadioGroup, radio_group_template};
pub use slider::{
    SliderSize, SliderVariant, slider_geometry, slider_template, slider_template_for, slider_theme, slider_theme_for,
    slider_theme_with,
};
pub use slider_builder::Slider;
pub use switch::{
    SwitchSize, SwitchVariant, resolve_switch_radius, switch_scale_for, switch_template, switch_template_for,
    switch_theme, switch_theme_for, switch_theme_with, switch_geometry,
};
pub use switch_builder::Switch;
pub use tabs::{
    TabsBaselineStyle, TabsMetrics, TabsStyle, TabsSize, TabsVariant, tabs_template, tabs_template_for, tabs_theme,
    tabs_theme_for, tabs_geometry,
};
pub use tabs_builder::Tabs;
pub use textarea::{TextAreaSize, textarea_template, textarea_theme, textarea_theme_with};
pub use textarea_builder::TextArea;
pub use textfield::{TextFieldSize, TextFieldVariant, textfield_template, textfield_theme, textfield_theme_with};
pub use textfield_builder::TextField;
pub use toggle::{page_toggle_template, toggle_template, toggle_template_for};
pub use toggle_builder::Toggle;
pub use toolbar::{ToolbarStyle, toolbar_template, toolbar_template_with, toolbar_theme, toolbar_theme_with};
pub use toolbar_builder::Toolbar;
pub use tree_view::{TreeView, tree_view_frame, tree_view_template, tree_view_theme};

// SDK composition helpers.
pub use ext::LookControlExt;
pub use overlay_window::overlay_window_theme;
pub use segmented::segmented_radio_template;
pub use selector::{Selector, selector_template, selector_theme};
pub use selector_list::{selector_items_panel_look, selector_items_panel_look_for};
pub use selection::{
    autocomplete_builder, autocomplete_theme, combobox_builder, search_selector_builder, selection_panel_builder,
    selection_panel_look,
};
pub use scrollbar::{scrollbar_template, scrollbar_theme};

pub use tooltip::tooltip_theme;

pub use panels::{
    resizable_panels_builder, resizable_panels_template, resizable_panels_theme, slide_panel_state, slide_panel_look,
    render_slide_panel_overlay, render_slide_panel_surface, render_slide_panel_inset,
};
pub use split_button::{split_button_builder, split_button_template, split_button_theme};
pub use splitters::{
    dock_splitter_builder, dock_splitter_template, dock_splitter_theme, split_view_builder, split_view_template,
    split_view_theme,
};
pub use table::{table_builder, table_template, table_theme, paging_table_builder};

// Group and navigation composition retains the full SDK builder APIs.
pub use accordion::{accordion_builder, accordion_template, accordion_theme};
pub use control_group::{
    control_group_builder, control_group_template, control_group_theme, control_group_item_template,
    icon_group_builder, icon_group_template, icon_group_item_template,
};
pub use navigation::{stepper_builder, stepper_template, stepper_theme};
pub use pager::{pager_builder, pager_template, pager_theme};
pub use sidebar::{sidebar_builder, sidebar_template, sidebar_theme};

#[cfg(test)]
mod stylesheet_tests;

#[cfg(all(test, feature = "test-support"))]
mod interaction_tests;

#[cfg(all(test, feature = "test-support"))]
mod event_callback_tests;

#[cfg(all(test, feature = "test-support"))]
xgpui_luma::test_look_motion_contract!(crate::Look::built_in(), crate);

#[cfg(all(test, feature = "test-support"))]
xgpui_luma::test_stylesheet_motion_contract!(
    |source: &str| {
        let look = crate::Look::built_in();
        look.set_common_stylesheet(xgpui_luma::theme::stylesheet::CommonStylesheet::parse(source).unwrap());
        look
    },
    crate
);

mod button_recipes;

#[cfg(test)]
mod navigation_tests;

#[cfg(all(test, feature = "test-support"))]
mod focus_tests;
