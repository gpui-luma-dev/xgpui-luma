//! Radix ownership, state precedence, and shared geometry contracts.
use super::*;
use xgpui_luma::controls::{button_family::ButtonFamilyRole, pager::PagerStyle};
use xgpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale, ThemeMode};

#[test]
fn group_rows_reuse_radix_button_colors_across_modes_palettes_sizes_and_states() {
    let look = Look::built_in();
    let theme = control_group_theme(&look);
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        for accent in [Accent::Blue, Accent::Lime] {
            look.set_palettes(accent, Gray::Auto);
            for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                let scale = StandardBoxScale::compute(size, &look.metrics(), 1.0);
                for bits in 0..16 {
                    let state = InteractionState {
                        hovered: bits & 1 != 0,
                        pressed: bits & 2 != 0,
                        focused: bits & 4 != 0,
                        disabled: bits & 8 != 0,
                        ..Default::default()
                    };
                    for selected in [false, true] {
                        let item = theme.default_item_palette(selected, state, size, &scale);
                        let button = button_family_theme(&look, ButtonVariant::Ghost).resolve(
                            ButtonFamilyRole::Toggle { selected },
                            size,
                            state,
                        );
                        assert_eq!(item.background, button.background);
                        assert_eq!(item.foreground, button.foreground);
                    }
                }
            }
        }
    }
}

#[test]
fn navigation_colors_and_disabled_precedence_follow_the_bound_look() {
    let look = Look::built_in();
    let accordion = accordion_theme(&look);
    let sidebar = sidebar_theme(&look);
    let pager = pager_theme(&look);
    let stepper = stepper_theme(&look);
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        for accent in [Accent::Blue, Accent::Lime] {
            look.set_palettes(accent, Gray::Auto);
            for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                for bits in 0..16 {
                    let state = InteractionState {
                        hovered: bits & 1 != 0,
                        pressed: bits & 2 != 0,
                        focused: bits & 4 != 0,
                        disabled: bits & 8 != 0,
                        ..Default::default()
                    };
                    let trigger = accordion.resolve_trigger(state, size);
                    let item = sidebar.resolve_item(true, state, size);
                    if state.disabled {
                        assert_eq!(trigger.foreground, look.resolve_role(SemanticRole::MutedForeground).hsla());
                        assert_eq!(item.foreground, trigger.foreground);
                        assert_eq!(item.background, None);
                        assert_eq!(item.focus_border, None);
                        assert_eq!(trigger.background, None);
                    } else {
                        assert_eq!(trigger.foreground, look.resolve_role(SemanticRole::Foreground).hsla());
                        assert_eq!(item.foreground, Tone::Accent.step(&look, 11));
                        assert_eq!(item.focus_border.is_some(), state.focused);
                    }
                }
                let steps = stepper.resolve(true, size);
                assert_eq!(steps.complete_bg, Tone::Accent.step(&look, 9));
                assert_eq!(steps.complete_fg, Tone::Accent.contrast(&look));
                assert_eq!(steps.in_progress_fg, Tone::Accent.step(&look, 11));
                let disabled = stepper.resolve(false, size);
                assert_ne!(disabled.complete_bg, steps.complete_bg);
                assert!(steps.step_badge_size > 0.0);
            }
            for style in [PagerStyle::Minimal, PagerStyle::MinimalEdge, PagerStyle::Numeric] {
                let resolved = pager.resolve(true, style);
                assert_eq!(resolved.panel_background, look.resolve_role(SemanticRole::Surface).hsla());
                assert_eq!(resolved.selected_foreground, Tone::Accent.step(&look, 11));
                assert_eq!(
                    pager.resolve(false, style).body_text,
                    look.resolve_role(SemanticRole::MutedForeground).hsla()
                );
            }
        }
    }
}

#[test]
fn live_geometry_and_palette_edits_remain_local_to_forks() {
    let base = Look::built_in();
    let fork = base.fork();
    let group = control_group_theme(&fork);
    let accordion = accordion_theme(&fork);
    let pager = pager_theme(&fork);
    let steps = stepper_theme(&fork);
    let sidebar = sidebar_theme(&fork);
    let before = sidebar.resolve_item(true, Default::default(), ControlSize::Md);
    let mut sheet = fork.common_stylesheet();
    sheet.control_group.geometry.gap = Some(17.0);
    sheet.accordion.geometry.trigger_height = Some(52.0);
    sheet.pager.geometry.button_min_width = Some(47.0);
    sheet.stepper.sizes.md.step_badge_size = Some(43.0);
    sheet.sidebar.geometry.engine_item_height = Some(49.0);
    fork.set_common_stylesheet(sheet);
    fork.set_mode(ThemeMode::Dark);
    fork.set_palettes(Accent::Red, Gray::Slate);
    assert_eq!(group.resolve_list(true).gap, 17.0);
    assert_eq!(accordion.resolve_scale(ControlSize::Md, 1.0).trigger_height, 52.0);
    assert_eq!(pager.resolve(true, PagerStyle::Numeric).button_min_width, 47.0);
    assert_eq!(steps.resolve(true, ControlSize::Md).step_badge_size, 43.0);
    assert_eq!(sidebar.resolve_item(true, Default::default(), ControlSize::Md).height, 49.0);
    assert_ne!(sidebar.resolve_item(true, Default::default(), ControlSize::Md).foreground, before.foreground);
    let unchanged = sidebar_theme(&base).resolve_item(true, Default::default(), ControlSize::Md);
    assert_eq!(unchanged.foreground, before.foreground);
    assert_eq!(unchanged.height, before.height);
    let drag = accordion.resolve_drag(ControlSize::Md);
    assert_eq!(drag.valid_marker, fork.resolve_role(SemanticRole::Focus).hsla());
    assert_eq!(drag.invalid_marker, fork.resolve_role(SemanticRole::Destructive).hsla());
}
