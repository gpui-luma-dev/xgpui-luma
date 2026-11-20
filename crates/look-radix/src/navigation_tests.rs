//! Radix ownership, state precedence, and shared geometry contracts.
use super::*;
use xgpui_luma::controls::{button_family::ButtonFamilyRole, pager::PagerStyle};
use xgpui_luma::theme::{InteractionState, StandardBoxScale, ThemeMode};

#[test]
fn group_rows_reuse_radix_button_colors_across_modes_palettes_sizes_and_states() {
    let look = Look::built_in();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        for accent in [Accent::Blue, Accent::Lime] {
            look.set_palettes(accent, Gray::Auto);
            for size in [RadixSize::One, RadixSize::Two, RadixSize::Three] {
                let scale = StandardBoxScale::compute(&look.metrics_for(size), 1.0);
                let theme = control_group_theme(&look.with_size(size));
                for bits in 0..16 {
                    let state = InteractionState {
                        hovered: bits & 1 != 0,
                        pressed: bits & 2 != 0,
                        focused: bits & 4 != 0,
                        disabled: bits & 8 != 0,
                        ..Default::default()
                    };
                    for selected in [false, true] {
                        let item = theme.default_item_palette(selected, state, &scale);
                        let button = crate::button_template_for::<()>(
                            &look.with_size(size),
                            ButtonVariant::Ghost,
                            crate::Paint::accent(),
                        )
                        .look(ButtonFamilyRole::Toggle { selected }, state, 1.0);
                        crate::control_group::assert_close(item.background, button.background);
                        crate::control_group::assert_close(item.foreground, button.foreground);
                    }
                }
            }
        }
    }
}

#[test]
fn navigation_colors_and_disabled_precedence_follow_the_bound_look() {
    let look = Look::built_in();
    let pager = pager_theme(&look);
    let stepper = stepper_theme(&look);
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        for accent in [Accent::Blue, Accent::Lime] {
            look.set_palettes(accent, Gray::Auto);
            for size in [RadixSize::One, RadixSize::Two, RadixSize::Three] {
                let accordion = accordion_theme(&look.with_size(size));
                let sidebar = sidebar_theme(&look.with_size(size));
                for bits in 0..16 {
                    let state = InteractionState {
                        hovered: bits & 1 != 0,
                        pressed: bits & 2 != 0,
                        focused: bits & 4 != 0,
                        disabled: bits & 8 != 0,
                        ..Default::default()
                    };
                    let trigger = accordion.resolve_trigger(state);
                    let item = sidebar.resolve_item(true, state);
                    if state.disabled {
                        crate::control_group::assert_close(
                            trigger.foreground,
                            look.resolve_role(SemanticRole::MutedForeground).hsla(),
                        );
                        crate::control_group::assert_close(item.foreground, trigger.foreground);
                        assert_eq!(item.background, None);
                        assert_eq!(item.focus_border, None);
                        assert_eq!(trigger.background, None);
                    } else {
                        crate::control_group::assert_close(
                            trigger.foreground,
                            look.resolve_role(SemanticRole::Foreground).hsla(),
                        );
                        crate::control_group::assert_close(item.foreground, Tone::Accent.step(&look, 11));
                        assert_eq!(item.focus_border.is_some(), state.focused);
                    }
                }
                let steps = stepper.resolve(true);
                crate::control_group::assert_close(steps.complete_bg, Tone::Accent.step(&look, 9));
                crate::control_group::assert_close(steps.complete_fg, Tone::Accent.contrast(&look));
                crate::control_group::assert_close(steps.in_progress_fg, Tone::Accent.step(&look, 11));
                let disabled = stepper.resolve(false);
                assert_ne!(disabled.complete_bg, steps.complete_bg);
                assert!(steps.step_badge_size > 0.0);
            }
            for style in [PagerStyle::Minimal, PagerStyle::MinimalEdge, PagerStyle::Numeric] {
                let resolved = pager.resolve(true, style);
                crate::control_group::assert_close(
                    resolved.panel_background,
                    look.resolve_role(SemanticRole::Surface).hsla(),
                );
                crate::control_group::assert_close(resolved.selected_foreground, Tone::Accent.step(&look, 11));
                crate::control_group::assert_close(
                    pager.resolve(false, style).body_text,
                    look.resolve_role(SemanticRole::MutedForeground).hsla(),
                );
            }
        }
    }
}

#[test]
fn live_palette_edits_remain_local_to_forks() {
    let base = Look::built_in();
    let fork = base.fork();
    let sidebar = sidebar_theme(&fork);
    let before = sidebar.resolve_item(true, Default::default());
    fork.set_mode(ThemeMode::Dark);
    fork.set_palettes(Accent::Red, Gray::Slate);
    assert_ne!(sidebar.resolve_item(true, Default::default()).foreground, before.foreground);
    let unchanged = sidebar_theme(&base).resolve_item(true, Default::default());
    assert_eq!(unchanged.foreground, before.foreground);
    assert_eq!(unchanged.height, before.height);
    let drag = accordion_theme(&fork).resolve_drag();
    crate::control_group::assert_close(drag.valid_marker, fork.resolve_role(SemanticRole::Focus).hsla());
    crate::control_group::assert_close(drag.invalid_marker, fork.resolve_role(SemanticRole::Destructive).hsla());
}
