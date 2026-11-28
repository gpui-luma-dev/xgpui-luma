//! Selector property mappings — outline/ghost command-button trigger + accent item panel.

#[cfg(test)]
mod tests {
    use gpui::Hsla;
    use std::sync::Arc;
    use xgpui_luma::controls::selector::{SelectorTriggerStyle, SelectorVisualState, resolve_selector_trigger};
    use xgpui_luma::style::PartRead;
    use xgpui_luma::style::lss::Resolved;
    use xgpui_luma::theme::{InteractionState, ThemeMode};

    use crate::ShadcnLook;

    fn close(a: Hsla, b: Hsla) -> bool {
        let (a, b) = (a.to_rgb(), b.to_rgb());
        [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
    }

    fn look() -> ShadcnLook {
        let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).unwrap();
        look.set_mode(ThemeMode::Light);
        look
    }

    fn css(look: &ShadcnLook, name: &str) -> Hsla {
        look.mode_tokens().catalog.color(name).unwrap()
    }

    /// The trigger's resolved style: what the SDK template draws from (#433).
    fn trigger(look: &ShadcnLook, style: SelectorTriggerStyle, state: SelectorVisualState) -> Arc<Resolved> {
        resolve_selector_trigger(&look.selector_theme().style(), style, state, 1.0)
    }

    fn with(interaction: InteractionState) -> SelectorVisualState {
        SelectorVisualState { interaction, ..Default::default() }
    }

    #[test]
    fn outline_is_the_outline_button_with_its_shadow_and_a_muted_chevron() {
        let look = look();
        let outline = trigger(&look, SelectorTriggerStyle::Outline, Default::default());
        let outline = outline.root();
        assert!(close(outline.color("background"), css(&look, "background")));
        assert!(close(outline.color("border"), css(&look, "border")));
        assert!(close(outline.color("icon"), css(&look, "muted-foreground")));
        assert!(outline.box_shadow_opt().as_ref().is_some_and(|layers| !layers.is_empty()));
        assert_eq!(
            outline.number("padding_x"),
            look.mode_tokens().control_sizes.md.padding_x,
            "the md control padding"
        );
    }

    /// shadcn fades a disabled trigger over its resting colors, chevron included (#430).
    #[test]
    fn disabled_fades_over_the_resting_trigger_and_chevron() {
        let look = look();
        let style = SelectorTriggerStyle::Outline;
        let rest = trigger(&look, style, Default::default());
        let rest = rest.root();
        let disabled = trigger(&look, style, with(InteractionState { disabled: true, ..Default::default() }));
        let disabled = disabled.root();
        assert!(disabled.opacity() < 1.0, "{}", disabled.opacity());
        assert!(close(disabled.color("icon"), rest.color("icon")));
        assert!(close(disabled.color("foreground"), rest.color("foreground")));
        assert_eq!(rest.opacity(), 1.0);
    }

    #[test]
    fn ghost_has_no_face_edge_or_shadow() {
        let look = look();
        let ghost = trigger(&look, SelectorTriggerStyle::Ghost, Default::default());
        let ghost = ghost.root();
        assert_eq!(ghost.color("background").a, 0.0);
        assert!(ghost.color_opt("border").is_none_or(|border| border.a == 0.0));
        assert!(ghost.box_shadow_opt().is_none());
    }

    #[test]
    fn open_looks_hovered_but_disabled_still_wins() {
        let look = look();
        let style = SelectorTriggerStyle::Outline;
        let open = trigger(&look, style, SelectorVisualState { open: true, ..Default::default() });
        let open = open.root();
        let hover = trigger(&look, style, with(InteractionState { hovered: true, ..Default::default() }));
        let hover = hover.root();
        assert_eq!(open.color("background"), hover.color("background"));
        let disabled_state = InteractionState { disabled: true, ..Default::default() };
        let disabled_open = trigger(
            &look,
            style,
            SelectorVisualState { open: true, interaction: disabled_state, ..Default::default() },
        );
        let disabled_open = disabled_open.root();
        let disabled = trigger(&look, style, with(disabled_state));
        let disabled = disabled.root();
        assert_eq!(disabled_open.color("background"), disabled.color("background"));
        assert_ne!(disabled.color("background"), hover.color("background"));
    }

    #[test]
    fn invalid_marks_the_edge_and_ring_destructive() {
        let look = look();
        let invalid =
            trigger(&look, SelectorTriggerStyle::Outline, SelectorVisualState { invalid: true, ..Default::default() });
        let invalid = invalid.root();
        assert!(close(invalid.color("border"), css(&look, "destructive")));
        assert!(close(invalid.color("focus_border"), css(&look, "destructive")));
    }

    #[test]
    fn sizes_scale_the_label_font() {
        let look = look();
        let size = |s| {
            trigger(&look.with_size(s), SelectorTriggerStyle::Outline, Default::default())
                .root()
                .typography(14.0, 20.0)
                .size
        };
        assert_eq!(
            (size(crate::ShadcnSize::Sm), size(crate::ShadcnSize::Md), size(crate::ShadcnSize::Lg)),
            (12.0, 14.0, 16.0)
        );
    }
}
