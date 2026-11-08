//! Forced SDK split-face templates at intrinsic control widths.
use std::sync::Arc;
use gpui::{AnyElement, App, Entity, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::{
    popup_menu::{PopupMenuPlacement, PopupMenuTriggerMetrics, PopupMenuTriggerStyle},
    split_button::*,
    tabs::Tabs,
};
use xgpui_luma::infra::{icon::DisclosureIcons, menu_item::MenuItem, state::ControlFocusState};
use xgpui_luma::theme::{ControlSize, InteractionState};
use xgpui_luma_look_radix::{self as radix, Accent, Gray, Look, PopupMenuVariant, SemanticRole, Tone};
use super::{matrix_grid::*, states};

#[derive(Clone, Copy)]
struct Style {
    label: &'static str,
    menu: PopupMenuVariant,
    trigger: PopupMenuTriggerStyle,
}
const STYLES: [Style; 4] = [
    Style { label: "Solid", menu: PopupMenuVariant::Solid, trigger: PopupMenuTriggerStyle::Primary },
    Style { label: "Soft", menu: PopupMenuVariant::Soft, trigger: PopupMenuTriggerStyle::Secondary },
    Style { label: "Outline", menu: PopupMenuVariant::Solid, trigger: PopupMenuTriggerStyle::Outline },
    Style { label: "Ghost", menu: PopupMenuVariant::Solid, trigger: PopupMenuTriggerStyle::Ghost },
];
pub(super) fn tabbed(look: &Arc<Look>, tabs: Entity<Tabs>, window: &mut Window, cx: &mut App) -> AnyElement {
    let active = tabs.read(cx).active_id().cloned().unwrap_or_else(|| "template-preview".into());
    let fg = look.resolve_role(SemanticRole::Foreground).hsla();
    let muted = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let border = look.resolve_role(SemanticRole::Border).hsla();
    let states = states::samples();
    let count = match active.as_ref() {
        "colors" => 2,
        "all-sizes" => 3,
        _ => states.len(),
    };
    let mut grid = fixed_grid(equal_data_columns(112.0, 144.0, count), 12.0, 24.0).child(empty_corner(), 0, 0);
    match active.as_ref() {
        "colors" => {
            for (col, label) in ["Solid", "Soft"].into_iter().enumerate() {
                grid = grid.child(column_header(label, muted), 0, col + 1);
            }
            for (row, accent) in Accent::ALL.into_iter().enumerate() {
                let fork = look.fork();
                fork.set_palettes(accent, Gray::Auto);
                grid = grid.child(row_label(super::palettes::title_case(accent.as_str()), fg), row + 1, 0);
                for (col, style) in STYLES[..2].iter().enumerate() {
                    grid = grid.child(
                        sample(
                            &fork,
                            Cell {
                                style: *style,
                                size: ControlSize::Md,
                                state: Default::default(),
                                face: Face::Primary,
                                id: format!("color-{row}-{col}"),
                            },
                            window,
                            cx,
                        ),
                        row + 1,
                        col + 1,
                    );
                }
            }
        }
        "all-sizes" => {
            for (col, label) in ["Size 1", "Size 2", "Size 3"].into_iter().enumerate() {
                grid = grid.child(column_header(label, muted), 0, col + 1);
            }
            for (row, style) in STYLES.iter().enumerate() {
                grid = grid.child(row_label(style.label, fg), row + 1, 0);
                for (col, size) in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg].into_iter().enumerate() {
                    grid = grid.child(
                        sample(
                            look,
                            Cell {
                                style: *style,
                                size,
                                state: Default::default(),
                                face: Face::Primary,
                                id: format!("size-{row}-{col}"),
                            },
                            window,
                            cx,
                        ),
                        row + 1,
                        col + 1,
                    );
                }
            }
        }
        _ => {
            for (col, state) in states.iter().enumerate() {
                grid = grid.child(states::header_cell(state, muted), 0, col + 1);
            }
            for (row, label) in ["Solid", "Soft", "Outline", "Ghost", "Menu face", "Icon"].into_iter().enumerate() {
                grid = grid.child(row_label(label, fg), row + 1, 0);
                for (col, state) in states.iter().enumerate() {
                    grid = grid.child(
                        sample(
                            look,
                            Cell {
                                style: STYLES[if row < 4 { row } else { 0 }],
                                size: ControlSize::Md,
                                state: state.state,
                                face: match row {
                                    4 => Face::Menu,
                                    5 => Face::Icon,
                                    _ => Face::Primary,
                                },
                                id: format!("state-{row}-{col}"),
                            },
                            window,
                            cx,
                        ),
                        row + 1,
                        col + 1,
                    );
                }
            }
        }
    }
    preview_tabbed(tabs, border, grid.into_any_element())
}
#[derive(Clone, Copy, PartialEq)]
enum Face {
    Primary,
    Menu,
    Icon,
}

struct Cell {
    style: Style,
    size: ControlSize,
    state: InteractionState,
    face: Face,
    id: String,
}
fn sample(look: &Look, cell: Cell, window: &mut Window, cx: &mut App) -> AnyElement {
    let id = SharedString::from(format!("guide-split-button-{}", cell.id));
    let label = SharedString::from("Save");
    let items = [
        MenuItem::new("save-as").label("Save as"),
        MenuItem::new("export").label("Export"),
        MenuItem::new("unavailable").enabled(false),
    ];
    let icons = DisclosureIcons::menu_from_stylesheet(&look.common_stylesheet());
    let primary = if cell.face == Face::Menu {
        InteractionState { disabled: cell.state.disabled, focused: cell.state.focused, ..Default::default() }
    } else {
        cell.state
    };
    let disclosure = if cell.face == Face::Menu {
        cell.state
    } else {
        InteractionState { disabled: cell.state.disabled, focused: cell.state.focused, ..Default::default() }
    };
    let content = if cell.face == Face::Icon {
        // GPUI SVGs need an explicit foreground; the SDK icon presenter supplies none.
        let resolved = radix::split_button_theme(look, cell.style.menu, Tone::Accent).resolve_look(
            cell.style.trigger,
            PopupMenuTriggerMetrics { size: cell.size, menu_size: cell.size, ..Default::default() },
            primary,
            window.scale_factor(),
            cx,
        );
        let selector = format!("{id}-icon");
        Arc::new(move |_: &SplitButtonTriggerModel, _: &mut App| {
            div()
                .size(px(resolved.trigger_icon_size))
                .flex_none()
                .debug_selector({
                    let selector = selector.clone();
                    move || selector.clone()
                })
                .child(xgpui_luma::infra::icon::lucide_icon(
                    lucide_svg_static::Icon::Save,
                    resolved.trigger_foreground,
                    resolved.trigger_icon_size,
                ))
                .into_any_element()
        }) as _
    } else {
        Arc::new(|model: &SplitButtonTriggerModel, _: &mut App| div().child(model.label.clone()).into_any_element())
            as _
    };
    let model = SplitButtonRenderModel {
        id: &id,
        label: &label,
        content,
        items: &items,
        open: false,
        disclosure_progress: 0.0,
        presence: xgpui_luma::OverlayPresence::new(false, false),
        submenu_transition: None,
        trigger_bounds: None,
        placement: PopupMenuPlacement::BelowStart,
        trigger_style: cell.style.trigger,
        trigger_size: cell.size,
        menu_size: cell.size,
        icon_only: false,
        icon: None,
        end_icon: None,
        disclosure_icons: &icons,
        full_width: false,
        without_elevation: false,
        split: true,
        split_state: SplitButtonState { primary, disclosure },
        trigger_radius_override: None,
        open_submenu: None,
        active_path: None,
        highlight: None,
        enabled: !cell.state.disabled,
        focus: ControlFocusState { focused: cell.state.focused, focus_visible: cell.state.focused },
        state: cell.state,
    };
    let selector = format!("{id}-control");
    centered(
        div().flex_none().flex().child(
            radix::split_button_template(look, cell.style.menu, Tone::Accent)
                .render(&model, super::menus::preview_handlers(items.len()), window, cx)
                .debug_selector(move || selector.clone()),
        ),
    )
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Render, TestAppContext, px};
    use xgpui_luma::theme::ThemeMode;
    struct Harness {
        look: Arc<Look>,
        tabs: Entity<Tabs>,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            tabbed(&self.look, self.tabs.clone(), window, cx)
        }
    }
    #[test]
    fn split_previews_fit_and_keep_independent_faces_in_both_modes() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(Look::built_in());
            let tabs = super::super::state::preview_navigation("split-button", &look, cx);
            Harness { look, tabs }
        });
        cx.simulate_resize(gpui::size(px(1040.0), px(1000.0)));
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for tab in ["colors", "all-sizes", "template-preview"] {
                cx.update(|_, app| {
                    view.update(app, |view, cx| {
                        view.look.set_mode(mode);
                        view.tabs.update(cx, |tabs, cx| tabs.set_active(tab, cx));
                        cx.notify();
                    })
                });
                cx.run_until_parked();
            }
            for selector in [
                "guide-split-button-state-0-0-control",
                "guide-split-button-state-0-4-control",
                "guide-split-button-state-5-1-control",
                "guide-split-button-state-5-4-control",
            ] {
                let bounds = cx.debug_bounds(selector).unwrap();
                assert!(bounds.size.width < px(144.0));
                assert!(bounds.left() >= px(0.0));
                assert!(bounds.right() <= px(1040.0));
                assert!(bounds.bottom() <= px(1000.0));
            }
            let icon = cx.debug_bounds("guide-split-button-state-5-1-icon").unwrap();
            assert!(icon.size.width > px(0.0) && icon.size.height > px(0.0));
            let primary = cx.debug_bounds("guide-split-button-state-5-1/primary").unwrap();
            let menu = cx.debug_bounds("guide-split-button-state-5-1/disclosure").unwrap();
            assert!(primary.right() <= menu.left());
            assert_eq!(primary.top(), menu.top());
            assert_eq!(primary.size.height, menu.size.height);
        }
    }
}
