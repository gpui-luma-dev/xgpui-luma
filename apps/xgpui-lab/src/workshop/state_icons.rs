//! Interaction-state icons (issue #301), from the Lucide crate: the same states Radix Studio's style
//! guide marks (default, hover, focused, pressed, disabled), plus checked and invalid.

use gpui::{AnyElement, Hsla, IntoElement, Styled, px, svg};
use lucide_svg_static::Icon;

/// Drawn at the pane's body text size.
const STATE_ICON_SIZE: f32 = 14.;

/// The icon for an `.lss` state name, or for `None`: no state (Radix Studio's "Default").
pub(super) fn state_icon_for(state: Option<&str>) -> Option<Icon> {
    Some(match state {
        None => Icon::House,
        Some("hover") => Icon::MousePointer,
        Some("focused") => Icon::SquareDashed,
        Some("pressed") => Icon::ArrowDown,
        Some("disabled") => Icon::Ban,
        Some("checked") => Icon::Check,
        Some("invalid") => Icon::TriangleAlert,
        Some(_) => return None,
    })
}

/// The state's icon in `color`, or nothing for a state with no icon.
pub(super) fn state_icon(state: Option<&str>, color: Hsla) -> Option<AnyElement> {
    state_icon_for(state).map(|icon| {
        svg()
            .flex_none()
            .size(px(STATE_ICON_SIZE))
            .path(icon.asset_path())
            .text_color(color)
            .into_any_element()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_states_have_icons_and_others_do_not() {
        let states = [
            None,
            Some("hover"),
            Some("focused"),
            Some("pressed"),
            Some("disabled"),
            Some("checked"),
            Some("invalid"),
        ];
        for state in states {
            assert!(state_icon_for(state).is_some(), "{state:?} has an icon");
        }
        assert_eq!(state_icon_for(Some("high_contrast")), None, "states without an icon draw none");
    }
}
