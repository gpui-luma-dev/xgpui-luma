//! Derive tests with a checkbox-shaped style and stand-in tokens.

use std::collections::HashMap;

use gpui::{Hsla, hsla};

use crate::State::*;
use crate::*;

mod tok {
    use crate::{Color, FontFamily, Px, Scalar, Token};

    pub const CONTROL_HEIGHT: Token<Px> = Token::new("metrics.control.control_height");
    pub const GAP: Token<Px> = Token::new("metrics.control.gap");
    pub const INPUT_BACKGROUND: Token<Color> = Token::new("palette.form.input.background");
    pub const INPUT_BORDER: Token<Color> = Token::new("palette.form.input.border");
    pub const HOVER: Token<Color> = Token::new("palette.state.hover.background");
    pub const PRESSED: Token<Color> = Token::new("palette.state.pressed.background");
    pub const SELECTED: Token<Color> = Token::new("palette.state.selected.background");
    pub const SELECTED_FOREGROUND: Token<Color> = Token::new("palette.state.selected.foreground");
    pub const DISABLED: Token<Color> = Token::new("palette.state.disabled.background");
    pub const DISABLED_FOREGROUND: Token<Color> = Token::new("palette.state.disabled.foreground");
    pub const FOREGROUND: Token<Color> = Token::new("palette.app.foreground");
    pub const FOCUS_RING: Token<Color> = Token::new("palette.focus.ring");
    pub const SANS: Token<FontFamily> = Token::new("typography.font.sans");
    pub const BODY_SIZE: Token<Px> = Token::new("typography.text.body.size");
    pub const LABEL_SIZE: Token<Px> = Token::new("typography.text.label.size");
    pub const LABEL_LINE_HEIGHT: Token<Scalar> = Token::new("typography.text.label.line_height");
}

/// A distinct color per token path, so tests can tell tokens apart.
fn color_for(path: &str) -> Hsla {
    let hash = path.bytes().fold(7u32, |hash, byte| hash.wrapping_mul(31).wrapping_add(byte as u32));
    hsla((hash % 360) as f32 / 360.0, 0.5, 0.5, 1.0)
}

struct Tokens(HashMap<&'static str, Dyn>);

impl Tokens {
    fn new() -> Self {
        let mut map: HashMap<&'static str, Dyn> = HashMap::new();
        for token in [
            tok::INPUT_BACKGROUND,
            tok::INPUT_BORDER,
            tok::HOVER,
            tok::PRESSED,
            tok::SELECTED,
            tok::SELECTED_FOREGROUND,
            tok::DISABLED,
            tok::DISABLED_FOREGROUND,
            tok::FOREGROUND,
            tok::FOCUS_RING,
        ] {
            map.insert(token.path, Dyn::Color(color_for(token.path)));
        }
        map.insert(tok::CONTROL_HEIGHT.path, Dyn::Number(32.0));
        map.insert(tok::GAP.path, Dyn::Number(8.0));
        map.insert(tok::SANS.path, Dyn::Text("Inter".into()));
        map.insert(tok::BODY_SIZE.path, Dyn::Number(14.0));
        map.insert(tok::LABEL_SIZE.path, Dyn::Number(14.0));
        map.insert(tok::LABEL_LINE_HEIGHT.path, Dyn::Number(20.0));
        Self(map)
    }
}

impl TokenSource for Tokens {
    fn token(&self, path: &str) -> Option<Dyn> {
        self.0.get(path).cloned()
    }
}

/// Checkbox-shaped test style.
#[derive(Style, Clone, Debug)]
#[style(path = "test.checkbox", snap)]
pub struct CheckStyle {
    /// Whole-control opacity when disabled.
    #[style(range = 0.0..=1.0)]
    pub disabled_opacity: V<Opacity>,
    /// Space between indicator and label.
    pub gap: V<Px>,
    /// Indicator box size.
    #[style(range = 8.0..=48.0, label = "Box size")]
    pub indicator_size: V<Px>,
    pub glyph_inset: V<Px>,
    pub glyph_size: V<Px>,
    /// Always twice the gap; derived in Rust.
    pub double_gap: Computed<Px>,
    pub indicator: IndicatorPart,
    pub glyph: GlyphPart,
    pub label: LabelPart,
}

impl CheckStyle {
    fn double_gap(&self, cx: &ResolveCx<'_>, at: StateSet) -> f32 {
        self.gap.eval(cx, self, at, 0).0 * 2.0
    }
}

#[derive(Style, Clone, Debug)]
pub struct IndicatorPart {
    pub background: States<V<Color>>,
    pub border: States<V<Color>>,
    pub shadow: States<V<Option<Shadow>>>,
}

#[derive(Style, Clone, Debug)]
pub struct GlyphPart {
    pub foreground: States<V<Color>>,
}

#[derive(Style, Clone, Debug)]
pub struct LabelPart {
    pub foreground: States<V<Color>>,
    pub font_family: V<FontFamily>,
    pub font_size: V<Px>,
    pub line_height: V<Px>,
}

impl Default for CheckStyle {
    fn default() -> Self {
        Self {
            disabled_opacity: opacity(0.56).into(),
            gap: tok::GAP.into(),
            indicator_size: tok::CONTROL_HEIGHT * 0.5,
            glyph_inset: px(3.0).into(),
            glyph_size: (Self::INDICATOR_SIZE - Self::GLYPH_INSET * 2.0).max(px(0.0)),
            double_gap: Computed::default(),
            indicator: IndicatorPart {
                background: States::new(tok::INPUT_BACKGROUND)
                    .on(Hovered, tok::HOVER)
                    .on(Pressed, tok::PRESSED)
                    .on(Checked, tok::SELECTED)
                    .on(Checked + Hovered, tok::SELECTED)
                    .on(Disabled, tok::DISABLED),
                border: States::new(tok::INPUT_BORDER)
                    .on(Checked, IndicatorPart::BACKGROUND)
                    .on(Disabled, tok::INPUT_BORDER),
                shadow: States::new(none::<Shadow>()),
            },
            glyph: GlyphPart {
                foreground: States::new(tok::FOREGROUND)
                    .on(Checked, tok::SELECTED_FOREGROUND)
                    .on(Disabled, tok::DISABLED_FOREGROUND),
            },
            label: LabelPart {
                foreground: States::new(tok::FOREGROUND).on(Disabled, tok::DISABLED_FOREGROUND),
                font_family: tok::SANS.into(),
                font_size: tok::BODY_SIZE.into(),
                line_height: LabelPart::FONT_SIZE * tok::LABEL_LINE_HEIGHT / tok::LABEL_SIZE,
            },
        }
    }
}

fn every_combination() -> impl Iterator<Item = StateSet> {
    (0u16..1 << State::ALL.len()).map(|bits| {
        State::ALL
            .iter()
            .enumerate()
            .filter(|(index, _)| bits & (1 << index) != 0)
            .fold(StateSet::NONE, |set, (_, state)| set.with(*state))
    })
}

/// Today's `DefaultCheckboxTheme::resolve` paint logic, using the test tokens.
fn current_paint(active: StateSet) -> (Hsla, Hsla, Hsla, Hsla) {
    let c = |token: Token<Color>| color_for(token.path);
    let checked = active.contains(Checked);
    let disabled = active.contains(Disabled);
    let background = if disabled {
        c(tok::DISABLED)
    } else if active.contains(Pressed) {
        c(tok::PRESSED)
    } else if checked {
        c(tok::SELECTED)
    } else if active.contains(Hovered) {
        c(tok::HOVER)
    } else {
        c(tok::INPUT_BACKGROUND)
    };
    let border = if checked && !disabled {
        background
    } else {
        c(tok::INPUT_BORDER)
    };
    let glyph = if disabled {
        c(tok::DISABLED_FOREGROUND)
    } else if checked {
        c(tok::SELECTED_FOREGROUND)
    } else {
        c(tok::FOREGROUND)
    };
    let label = if disabled {
        c(tok::DISABLED_FOREGROUND)
    } else {
        c(tok::FOREGROUND)
    };
    (background, border, glyph, label)
}

#[test]
fn paint_matches_the_current_checkbox_for_every_state_combination() {
    let tokens = Tokens::new();
    let style = CheckStyle::default();
    for active in every_combination() {
        let cx = ResolveCx::new(&tokens, 1.0);
        let resolved = style.resolve(&cx, active);
        assert!(cx.errors().is_empty(), "{active}: {:?}", cx.errors());
        let (background, border, glyph, label) = current_paint(active);
        assert_eq!(resolved.indicator.background, background, "background, states: {active}");
        assert_eq!(resolved.indicator.border, border, "border, states: {active}");
        assert_eq!(resolved.glyph.foreground, glyph, "glyph, states: {active}");
        assert_eq!(resolved.label.foreground, label, "label, states: {active}");
        assert_eq!(resolved.indicator.shadow, None);
    }
}

#[test]
fn geometry_follows_tokens_and_field_references() {
    let tokens = Tokens::new();
    let cx = ResolveCx::new(&tokens, 1.0);
    let resolved = CheckStyle::default().resolve(&cx, StateSet::NONE);
    assert!(cx.errors().is_empty(), "{:?}", cx.errors());
    assert_eq!(resolved.indicator_size, 16.0);
    assert_eq!(resolved.glyph_size, 10.0);
    assert_eq!(resolved.gap, 8.0);
    assert_eq!(resolved.double_gap, 16.0);
    assert_eq!(resolved.disabled_opacity, 0.56);
    assert_eq!(resolved.label.font_family.as_ref(), "Inter");
    assert_eq!(resolved.label.line_height, 20.0);
}

#[test]
fn snap_rounds_lengths_to_device_pixels() {
    let mut tokens = Tokens::new();
    tokens.0.insert(tok::CONTROL_HEIGHT.path, Dyn::Number(33.0));
    let cx = ResolveCx::new(&tokens, 2.0);
    let resolved = CheckStyle::default().resolve(&cx, StateSet::NONE);
    assert_eq!(resolved.indicator_size, 16.5);
    let cx = ResolveCx::new(&tokens, 1.0);
    let resolved = CheckStyle::default().resolve(&cx, StateSet::NONE);
    assert_eq!(resolved.indicator_size, 17.0);
}

#[test]
fn token_edits_cascade_through_expressions() {
    let mut tokens = Tokens::new();
    tokens.0.insert(tok::CONTROL_HEIGHT.path, Dyn::Number(40.0));
    let cx = ResolveCx::new(&tokens, 1.0);
    let resolved = CheckStyle::default().resolve(&cx, StateSet::NONE);
    assert_eq!(resolved.indicator_size, 20.0);
    assert_eq!(resolved.glyph_size, 14.0);
}

#[test]
fn a_variant_is_a_clone_with_a_few_fields_changed() {
    const ACCENT: Token<Color> = Token::new("palette.accent");
    let mut tokens = Tokens::new();
    tokens.0.insert(ACCENT.path, Dyn::Color(color_for(ACCENT.path)));

    let mut style = CheckStyle::default();
    style.indicator_size = px(20.0).into();
    style.indicator.background = States::new(tok::INPUT_BACKGROUND).on(Checked, ACCENT);

    let cx = ResolveCx::new(&tokens, 1.0);
    let checked = style.resolve(&cx, Checked.into());
    assert_eq!(checked.indicator_size, 20.0);
    assert_eq!(checked.glyph_size, 14.0, "references see the edited field");
    assert_eq!(checked.indicator.background, color_for(ACCENT.path));
    assert_eq!(checked.indicator.border, color_for(ACCENT.path), "border follows background");

    let hovered = style.resolve(&cx, Hovered.into());
    assert_eq!(hovered.indicator.background, color_for(tok::INPUT_BACKGROUND.path), "table replaced");
    assert!(cx.errors().is_empty(), "{:?}", cx.errors());
}

#[test]
fn problems_are_recorded_not_panicked() {
    let tokens = Tokens(HashMap::new());
    let cx = ResolveCx::new(&tokens, 1.0);
    let resolved = CheckStyle::default().resolve(&cx, StateSet::NONE);
    assert_eq!(resolved.indicator_size, 0.0);
    assert!(cx.errors().iter().any(|error| error.contains("metrics.control.control_height")));
}

#[test]
fn reference_cycles_are_reported() {
    let mut style = CheckStyle::default();
    style.gap = CheckStyle::GLYPH_INSET.into();
    style.glyph_inset = CheckStyle::GAP.into();
    let tokens = Tokens::new();
    let cx = ResolveCx::new(&tokens, 1.0);
    let _ = style.resolve(&cx, StateSet::NONE);
    assert!(cx.errors().iter().any(|error| error.contains("cycle")));
}

#[test]
fn lerp_blends_colors_between_off_and_on() {
    let tokens = Tokens::new();
    let cx = ResolveCx::new(&tokens, 1.0);
    let style = CheckStyle::default();
    let off = style.resolve(&cx, StateSet::NONE);
    let on = style.resolve(&cx, Checked.into());
    assert_eq!(Lerp::lerp(&off, &on, 0.0), off);
    assert_eq!(Lerp::lerp(&off, &on, 1.0), on);
}

#[test]
fn descriptors_describe_fields_and_parts() {
    let shape = <CheckStyle as Style>::SHAPE;
    assert_eq!(shape.name, "test.checkbox");
    assert_eq!(shape.type_name, "CheckStyle");
    assert_eq!(shape.doc, "Checkbox-shaped test style.");

    let size = shape.find("indicator_size").unwrap();
    assert_eq!(size.label, "Box size");
    assert_eq!(size.doc, "Indicator box size.");
    let FieldContent::Value { kind, range, stateful, computed } = &size.content else {
        panic!("value field")
    };
    assert_eq!(kind.value, ValueKind::Px);
    assert_eq!(*range, Some((8.0, 48.0)));
    assert!(!stateful && !computed);

    let shadow = shape.find("indicator.shadow").unwrap();
    let FieldContent::Value { kind, stateful, .. } = &shadow.content else {
        panic!("value field")
    };
    assert_eq!(*kind, FieldKind { value: ValueKind::Shadow, optional: true });
    assert!(*stateful);

    let FieldContent::Value { computed, .. } = &shape.find("double_gap").unwrap().content else {
        panic!()
    };
    assert!(*computed);
    assert_eq!(shape.find("disabled_opacity").unwrap().label, "Disabled opacity");

    let leaves: Vec<String> = shape.leaves().into_iter().map(|(path, _)| path).collect();
    assert!(leaves.contains(&"label.line_height".to_owned()));
    assert!(leaves.contains(&"glyph.foreground".to_owned()));
}

#[test]
fn top_level_styles_are_registered() {
    assert!(registered().any(|shape| shape.name == "test.checkbox"));
    assert!(!registered().any(|shape| shape.type_name == "IndicatorPart"), "parts are not registered");
}

#[test]
fn values_describe_their_links() {
    let style = CheckStyle::default();
    assert_eq!(style.indicator_size.describe(), "(metrics.control.control_height * 0.5)");
    assert!(style.indicator_size.is_linked());
    assert_eq!(style.glyph_size.describe(), "max((self.indicator_size - (self.glyph_inset * 2)), 0)");
    assert!(!style.glyph_inset.is_linked());
}

#[test]
fn get_set_and_describe_by_path() {
    let tokens = Tokens::new();
    let cx = ResolveCx::new(&tokens, 1.0);
    let mut style = CheckStyle::default();

    assert_eq!(style.get("indicator_size", &cx, StateSet::NONE), Some(Dyn::Number(16.0)));
    assert_eq!(
        style.get("indicator.background", &cx, Checked.into()),
        Some(Dyn::Color(color_for(tok::SELECTED.path)))
    );
    assert_eq!(style.get("indicator", &cx, StateSet::NONE), None, "parts have no value");
    assert_eq!(style.get("double_gap", &cx, StateSet::NONE), None, "computed fields are not readable by path");
    assert_eq!(
        style.describe("indicator_size", StateSet::NONE).as_deref(),
        Some("(metrics.control.control_height * 0.5)")
    );
    assert_eq!(style.describe("indicator.border", Checked.into()).as_deref(), Some("self.background"));

    assert!(style.set("glyph_inset", StateSet::NONE, Dyn::Number(5.0)));
    assert_eq!(style.get("glyph_size", &cx, StateSet::NONE), Some(Dyn::Number(6.0)), "references follow the edit");
    assert!(style.set("indicator.background", Hovered.into(), Dyn::Color(color_for("x"))));
    assert_eq!(style.resolve(&cx, Hovered.into()).indicator.background, color_for("x"));
    assert_eq!(style.resolve(&cx, StateSet::NONE).indicator.background, color_for(tok::INPUT_BACKGROUND.path));

    assert!(!style.set("glyph_inset", StateSet::NONE, Dyn::Color(color_for("x"))), "wrong type");
    assert!(!style.set("nope", StateSet::NONE, Dyn::Number(1.0)));
    assert!(!style.set("double_gap", StateSet::NONE, Dyn::Number(1.0)));
    assert!(cx.errors().is_empty(), "{:?}", cx.errors());
}

#[test]
fn states_used_collects_rule_states_across_parts() {
    let used = CheckStyle::default().states_used();
    assert_eq!(used, StateSet::NONE.with(Hovered).with(Pressed).with(Checked).with(Disabled));
}

#[test]
fn snapped_expressions_round_only_where_written() {
    let mut tokens = Tokens::new();
    tokens.0.insert(tok::GAP.path, Dyn::Number(7.3));
    let cx = ResolveCx::new(&tokens, 2.0);
    let mut style = CheckStyle::default();
    style.gap = tok::GAP.snapped();
    assert_eq!(style.describe("gap", StateSet::NONE).as_deref(), Some("snap(metrics.control.gap)"));
    assert_eq!(style.get("gap", &cx, StateSet::NONE), Some(Dyn::Number(7.5)));
    let half: V<Px> = (tok::GAP * 0.5).snapped();
    assert_eq!(half.describe(), "snap((metrics.control.gap * 0.5))");
}
