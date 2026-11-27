//! The selector trigger read from `.lss` (issue #285): `selector.lss`, which extends the button. The
//! popup panel is supplied separately (it is drawn with the floating menu's values).

use std::sync::Arc;

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value, box_shadows};

use super::model::SelectorTriggerStyle;
use super::theme::{SelectorLook, SelectorPalette, SelectorTheme, SelectorVisualState};
use crate::controls::selector_list::SelectorItemsPanelLook;
use crate::style::{StyleBinding, ThemeSource};
use crate::theme::{FocusMetricTokens, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale};

/// Supplies the selector's popup panel look.
pub type ItemsPanelSource = Arc<dyn Fn() -> SelectorItemsPanelLook + Send + Sync>;

fn color(root: PartView<'_>, name: &str) -> Option<Hsla> {
    root.value(name).and_then(Value::as_color).map(Hsla::from)
}

fn number(root: PartView<'_>, name: &str) -> f32 {
    root.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// A selector theme whose trigger is read from `.lss`.
#[derive(Clone)]
pub struct StyledSelectorTheme {
    binding: StyleBinding,
    items_panel: ItemsPanelSource,
}

impl StyledSelectorTheme {
    /// The `selector` control from `source`, with its popup panel from `items_panel`.
    pub fn new(source: ThemeSource, items_panel: ItemsPanelSource) -> Self {
        Self { binding: StyleBinding::new("selector", source), items_panel }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self {
            binding: StyleBinding::sdk("selector"),
            items_panel: Arc::new(|| {
                crate::controls::selector_list::default_selector_items_panel_look(&Default::default())
            }),
        }
    }

    /// Change the binding (a look's size).
    pub fn with(self, f: impl FnOnce(StyleBinding) -> StyleBinding) -> Self {
        Self { binding: f(self.binding), ..self }
    }

    /// The trigger in `state`; `open` sets the `open` state (the trigger styles of `trigger_button.lss`).
    fn resolve_root(
        &self,
        style: SelectorTriggerStyle,
        state: InteractionState,
        open: bool,
    ) -> Arc<xgpui_luma_style::Resolved> {
        let variant = match style {
            SelectorTriggerStyle::Outline => "outline",
            SelectorTriggerStyle::Ghost => "ghost",
        };
        let states = [
            ("hover", state.hovered),
            ("pressed", state.pressed),
            ("focused", state.focused),
            ("disabled", state.disabled),
            ("invalid", state.invalid),
            ("open", open),
        ];
        self.binding.resolve_for(&[("variant", variant)], &states, 1.0)
    }
}

impl SelectorTheme for StyledSelectorTheme {
    fn resolve(
        &self,
        style: SelectorTriggerStyle,
        state: InteractionState,
        without_elevation: bool,
    ) -> SelectorPalette {
        let look = self.resolve_look(style, state, &StandardBoxScale::compute(&self.metrics(), 1.0), without_elevation);
        SelectorPalette {
            trigger_focus_border: look.trigger_focus_border,
            trigger_background: look.trigger_background,
            trigger_foreground: look.trigger_foreground,
            trigger_icon: look.trigger_icon,
            trigger_border: look.trigger_border,
            trigger_shadow: look.trigger_shadow,
            trigger_typography: look.trigger_typography,
            items_panel: look.items_panel,
        }
    }

    /// The focus geometry and border width, for templates that read them from the theme's metrics.
    fn metrics(&self) -> MetricTokens {
        let resolved = self.resolve_root(SelectorTriggerStyle::Outline, InteractionState::default(), false);
        let root = resolved.root();
        MetricTokens {
            border_width: number(root, "border_width"),
            focus: FocusMetricTokens {
                gap: number(root, "focus_gap"),
                width: number(root, "focus_width"),
                offset: number(root, "focus_offset"),
            },
            ..MetricTokens::default()
        }
    }

    fn resolve_look(
        &self,
        style: SelectorTriggerStyle,
        state: InteractionState,
        _scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        self.trigger_look(style, state, false, without_elevation)
    }

    /// The trigger as drawn: open looks hovered and sets the `open` state.
    fn resolve_visual_look(
        &self,
        style: SelectorTriggerStyle,
        visual: SelectorVisualState,
        _scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        self.trigger_look(style, visual.resolved_interaction(), visual.open, without_elevation)
    }
}

impl StyledSelectorTheme {
    fn trigger_look(
        &self,
        style: SelectorTriggerStyle,
        state: InteractionState,
        open: bool,
        without_elevation: bool,
    ) -> SelectorLook {
        let resolved = self.resolve_root(style, state, open);
        let root = resolved.root();
        let transparent = gpui::transparent_black();
        SelectorLook {
            trigger_background: color(root, "background").unwrap_or(transparent),
            trigger_foreground: color(root, "foreground").unwrap_or(transparent),
            trigger_icon: color(root, "icon").unwrap_or(transparent),
            trigger_border: color(root, "border"),
            trigger_shadow: if without_elevation {
                None
            } else {
                root.shadow().filter(|layers| !layers.is_empty()).map(|layers| box_shadows(&layers))
            },
            trigger_typography: LumaTextStyle {
                size: root.font_size().unwrap_or(14.0),
                line_height: root.line_height().unwrap_or(20.0),
                weight: gpui::FontWeight(number(root, "font_weight").max(100.0)),
            },
            trigger_radius: root.radius().unwrap_or(0.0),
            trigger_padding_x: number(root, "padding_x"),
            trigger_padding_y: number(root, "padding_y"),
            trigger_gap: root.gap().unwrap_or(0.0),
            trigger_height: root.height().unwrap_or(0.0),
            trigger_icon_size: number(root, "icon_size"),
            trigger_focus_border: color(root, "focus_border").unwrap_or(transparent),
            trigger_opacity: crate::style::part_opacity(root),
            menu_offset_y: number(root, "menu_offset_y"),
            items_panel: (self.items_panel)(),
        }
    }
}
