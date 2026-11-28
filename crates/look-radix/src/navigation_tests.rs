//! Radix ownership, state precedence, and shared geometry contracts.
// These contracts read the compatibility Looks, which build from the resolved styles (#433).
use super::*;
use xgpui_luma::controls::pager::resolve_pager;
use xgpui_luma::controls::sidebar::resolve_sidebar_row;
use xgpui_luma::controls::stepper::resolve_stepper;
use xgpui_luma::style::lss::PartId;
use xgpui_luma::style::PartRead;
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
                        .resolve_face(ButtonFamilyRole::Toggle { selected }, state, 1.0);
                        crate::control_group::assert_close(item.background, button.root().color("background"));
                        crate::control_group::assert_close(item.foreground, button.root().color("foreground"));
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
                    let item_style = resolve_sidebar_row(&sidebar.style(), true, state);
                    let item = item_style.part(PartId::new("item"));
                    if state.disabled {
                        crate::control_group::assert_close(
                            trigger.foreground,
                            look.resolve_role(SemanticRole::MutedForeground).hsla(),
                        );
                        crate::control_group::assert_close(item.color("foreground"), trigger.foreground);
                        assert_eq!(item.painted("background"), None);
                        assert_eq!(item.painted("focus_border"), None);
                        assert_eq!(trigger.background, None);
                    } else {
                        crate::control_group::assert_close(
                            trigger.foreground,
                            look.resolve_role(SemanticRole::Foreground).hsla(),
                        );
                        crate::control_group::assert_close(item.color("foreground"), Tone::Accent.step(&look, 11));
                        assert_eq!(item.painted("focus_border").is_some(), state.focused);
                    }
                }
                let steps_style = resolve_stepper(&stepper.style(), true);
                let steps = steps_style.root();
                crate::control_group::assert_close(steps.color("complete_background"), Tone::Accent.step(&look, 9));
                crate::control_group::assert_close(steps.color("complete_foreground"), Tone::Accent.contrast(&look));
                crate::control_group::assert_close(steps.color("current_foreground"), Tone::Accent.step(&look, 11));
                let disabled_style = resolve_stepper(&stepper.style(), false);
                assert_ne!(disabled_style.root().color("complete_background"), steps.color("complete_background"));
                assert!(steps.number("step_badge_size") > 0.0);
            }
            for style in [PagerStyle::Minimal, PagerStyle::MinimalEdge, PagerStyle::Numeric] {
                let resolved = resolve_pager(&pager.style(), true, style);
                let resolved = resolved.root();
                crate::control_group::assert_close(
                    resolved.color("background"),
                    look.resolve_role(SemanticRole::Surface).hsla(),
                );
                crate::control_group::assert_close(resolved.color("selected_foreground"), Tone::Accent.step(&look, 11));
                crate::control_group::assert_close(
                    resolve_pager(&pager.style(), false, style).root().color("foreground"),
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
    let item = |theme: &std::sync::Arc<dyn xgpui_luma::controls::sidebar::SidebarTheme>| {
        resolve_sidebar_row(&theme.style(), true, Default::default())
    };
    let before = item(&sidebar);
    let (before_foreground, before_height) = (
        before.part(PartId::new("item")).color("foreground"),
        before.part(PartId::new("item")).number("height"),
    );
    fork.set_mode(ThemeMode::Dark);
    fork.set_palettes(Accent::Red, Gray::Slate);
    assert_ne!(item(&sidebar).part(PartId::new("item")).color("foreground"), before_foreground);
    let unchanged = item(&sidebar_theme(&base));
    assert_eq!(unchanged.part(PartId::new("item")).color("foreground"), before_foreground);
    assert_eq!(unchanged.part(PartId::new("item")).number("height"), before_height);
    let drag = accordion_theme(&fork).drag_style();
    crate::control_group::assert_close(
        drag.root().color("valid_marker"),
        fork.resolve_role(SemanticRole::Focus).hsla(),
    );
    crate::control_group::assert_close(
        drag.root().color("invalid_marker"),
        fork.resolve_role(SemanticRole::Destructive).hsla(),
    );
}
