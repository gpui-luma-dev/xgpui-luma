//! Radix tabs, drawn from `.lss` (`style/tabs.lss`) with the look-wide `TabsStyle` tuning over them.
#![allow(deprecated)] // Still reads a deprecated Look until its family has a style-override replacement (#433).

use std::sync::Arc;

use gpui::{Hsla, SharedString};
use xgpui_luma::controls::tabs::{TabsBaseline, TabsTemplate, TabsTheme, ThemedTabsTemplate};
use xgpui_luma::style::lss::{Overrides, PartId, Prop};
#[cfg(test)]
use xgpui_luma::theme::InteractionState;

use crate::look::Look;
use crate::typography::font_family;

/// Radix `size` prop on Tabs. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsSize {
    One,
    #[default]
    Two,
}

impl TabsSize {
    pub const ALL: [Self; 2] = [Self::One, Self::Two];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::One => "Size 1",
            Self::Two => "Size 2",
        }
    }

    pub fn control_size(self) -> crate::RadixSize {
        match self {
            Self::One => crate::RadixSize::One,
            Self::Two => crate::RadixSize::Two,
        }
    }
}

/// Radix Themes `variant` on Tabs. Default is [`Self::Line`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsVariant {
    /// Underline indicator with a full-width baseline, no list box.
    #[default]
    Line,
    /// Padded gray list with a selected chip. No underline.
    Surface,
}

/// Geometry for one Radix tab size, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsMetrics {
    pub height: f32,
    /// Total horizontal trigger padding (outer + inner padding in Radix CSS).
    pub padding_x: f32,
    pub font_size: f32,
    pub line_height: f32,
}

/// Full-control baseline, independent of the active-tab indicator. The defaults are the look's `tabs.lss`
/// (`baseline_*`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsBaselineStyle {
    pub height: f32,
    pub inset_x: f32,
    pub bottom: f32,
    /// An exact color; otherwise the `.lss` `baseline_color` in the current mode (gray 5).
    pub color: Option<Hsla>,
}

impl Default for TabsBaselineStyle {
    /// The built-in look's baseline.
    fn default() -> Self {
        Self::from_look(&Look::built_in())
    }
}

impl TabsBaselineStyle {
    /// `look`'s line tabs baseline from its `.lss`, with the color left to the current mode.
    pub(crate) fn from_look(look: &Look) -> Self {
        let root = styled_tabs_root(look, TabsVariant::Line, crate::RadixSize::Two);
        Self {
            height: root.number("baseline_height"),
            inset_x: root.number("baseline_inset_x"),
            bottom: root.number("baseline_bottom"),
            color: None,
        }
    }
}

/// Look-wide tab tokens. The baseline and selected indicator can be tuned separately.
///
/// ```
/// let look = xgpui_luma_look_radix::Look::built_in();
/// let mut style = look.tabs_style();
/// if let Some(baseline) = &mut style.baseline {
///     baseline.height = 2.0;
///     baseline.inset_x = 4.0;
/// }
/// look.set_tabs_style(style);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsStyle {
    /// `None` hides only the full-list baseline.
    pub baseline: Option<TabsBaselineStyle>,
    pub indicator_height: f32,
    /// Zero spans the whole tab, independently of label padding.
    pub indicator_inset: f32,
    pub gap: f32,
    pub small: TabsMetrics,
    pub medium: TabsMetrics,
}

impl Default for TabsStyle {
    /// The built-in look's line tabs.
    fn default() -> Self {
        Self::from_look(&Look::built_in())
    }
}

impl TabsStyle {
    /// `look`'s line tabs from its `.lss`: the list gap, indicator height and baseline at size 2, and the
    /// size 1 and size 2 tab metrics.
    pub(crate) fn from_look(look: &Look) -> Self {
        use xgpui_luma::controls::tabs::{resolve_tab, resolve_tabs_list};
        use xgpui_luma::style::PartRead;
        let active = |size| resolve_tab(&tabs_binding(look, TabsVariant::Line, size), true, Default::default());
        let (list, tab) = (
            resolve_tabs_list(&tabs_binding(look, TabsVariant::Line, crate::RadixSize::Two), true),
            active(crate::RadixSize::Two),
        );
        let metrics = |size| {
            let resolved = active(size);
            let item = resolved.part(PartId::new("item"));
            let typography = item.typography(14.0, 20.0);
            TabsMetrics {
                height: item.number("height"),
                padding_x: item.number("padding_x"),
                font_size: typography.size,
                line_height: typography.line_height,
            }
        };
        Self {
            baseline: Some(TabsBaselineStyle::from_look(look)),
            indicator_height: tab.root().number("indicator_height"),
            indicator_inset: 0.0,
            gap: list.part(PartId::new("list")).number("gap"),
            small: metrics(crate::RadixSize::One),
            medium: metrics(crate::RadixSize::Two),
        }
    }
}

impl TabsStyle {
    fn metrics(self, size: crate::RadixSize) -> TabsMetrics {
        match size {
            crate::RadixSize::One => self.small,
            crate::RadixSize::Two | crate::RadixSize::Three => self.medium,
        }
    }
}

/// The Radix tabs: list and tabs from `.lss` (`style/tabs.lss`), with a look-wide [`TabsStyle`] set through
/// `Look::set_tabs_style` applied over them; the baseline, indicator inset, content padding and drag preview
/// from the look.
struct TabsThemeAdapter {
    look: Look,
    variant: TabsVariant,
    styled: xgpui_luma::controls::tabs::StyledTabs,
}

impl TabsTheme for TabsThemeAdapter {
    fn drag_style(&self) -> Arc<xgpui_luma::style::lss::Resolved> {
        crate::navigation::drag_style(&self.look, self.look.size())
    }
    /// The `.lss` tabs with the look-wide [`TabsStyle`] tuning set over them (#433).
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        let binding = self.styled.binding();
        let Some(style) = self.look.tabs_style_override() else {
            return binding;
        };
        let tab = style.metrics(self.look.size());
        let (root, list, item) = (PartId::ROOT, PartId::new("list"), PartId::new("item"));
        let mut overrides = Overrides::new()
            .set(item, Prop::<f32>::new("height"), tab.height)
            .set(item, Prop::<f32>::new("padding_x"), tab.padding_x)
            .set(item, Prop::<f32>::new("font_size"), tab.font_size)
            .set(item, Prop::<f32>::new("line_height"), tab.line_height);
        if self.variant == TabsVariant::Line {
            overrides = overrides.set(list, Prop::<f32>::new("gap"), style.gap).set(
                root,
                Prop::<f32>::new("indicator_height"),
                style.indicator_height,
            );
        }
        binding.with_overrides(&overrides)
    }

    fn baseline(&self, _enabled: bool) -> Option<TabsBaseline> {
        if self.variant != TabsVariant::Line {
            return None;
        }
        let lss = styled_tabs_root(&self.look, self.variant, self.look.size());
        self.look.tabs_style().baseline.map(|baseline| TabsBaseline {
            color: baseline.color.unwrap_or_else(|| lss.color("baseline_color")),
            height: baseline.height,
            inset_x: baseline.inset_x,
            bottom: baseline.bottom,
        })
    }

    fn indicator_inset(&self) -> Option<f32> {
        Some(self.look.tabs_style().indicator_inset.max(0.0))
    }

    fn content_padding(&self) -> Option<(f32, f32)> {
        let lss = styled_tabs_root(&self.look, self.variant, self.look.size());
        (self.variant == TabsVariant::Line).then(|| (lss.number("content_padding_x"), lss.number("content_padding_y")))
    }

    fn font_family(&self) -> SharedString {
        font_family(&self.look)
    }
}

pub fn tabs_theme(look: &Look) -> Arc<dyn TabsTheme> {
    tabs_theme_for(look, TabsVariant::Line)
}

pub fn tabs_theme_for(look: &Look, variant: TabsVariant) -> Arc<dyn TabsTheme> {
    Arc::new(TabsThemeAdapter { look: look.clone(), variant, styled: styled_tabs(look, variant, look.size()) })
}

/// The tabs control's own `.lss` values (`baseline_*`, `content_padding_*`) for `variant` at `size`.
struct TabsRoot(std::sync::Arc<xgpui_luma::style::lss::Resolved>);

impl TabsRoot {
    fn number(&self, name: &str) -> f32 {
        self.0.root().value(name).and_then(xgpui_luma::style::lss::Value::as_number).unwrap_or(0.0)
    }

    fn color(&self, name: &str) -> Hsla {
        self.0
            .root()
            .value(name)
            .and_then(xgpui_luma::style::lss::Value::as_color)
            .map(Hsla::from)
            .unwrap_or_else(gpui::transparent_black)
    }
}

fn styled_tabs_root(look: &Look, variant: TabsVariant, size: crate::RadixSize) -> TabsRoot {
    TabsRoot(tabs_binding(look, variant, size).resolve(&[], 1.0))
}

/// The `.lss` tabs (`style/tabs.lss`) for `variant` at `size`, before any [`TabsStyle`] override.
fn styled_tabs(look: &Look, variant: TabsVariant, size: crate::RadixSize) -> xgpui_luma::controls::tabs::StyledTabs {
    xgpui_luma::controls::tabs::StyledTabs::new(tabs_binding(look, variant, size))
}

fn tabs_binding(look: &Look, variant: TabsVariant, size: crate::RadixSize) -> xgpui_luma::style::StyleBinding {
    let binding = xgpui_luma::style::StyleBinding::new("tabs", crate::textfield::live_source(look));
    let binding = if variant == TabsVariant::Surface {
        binding.variant("surface")
    } else {
        binding
    };
    if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    }
}

pub fn tabs_template(look: &Look) -> Arc<dyn TabsTemplate> {
    tabs_template_for(look, TabsVariant::Line)
}

pub fn tabs_template_for(look: &Look, variant: TabsVariant) -> Arc<dyn TabsTemplate> {
    Arc::new(ThemedTabsTemplate::new(tabs_theme_for(look, variant)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::ScaleFamily;
    use crate::semantic::SemanticRole;
    use crate::RadixSize;

    #[test]
    fn embedded_geometry_preserves_all_modes_sizes_and_variants() {
        let look = Look::built_in();
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for size in [RadixSize::One, RadixSize::Two, RadixSize::Three] {
                for variant in [TabsVariant::Line, TabsVariant::Surface] {
                    let theme = tabs_theme_for(&look.with_size(size), variant);
                    let surface = variant == TabsVariant::Surface;
                    for enabled in [true, false] {
                        let list = theme.resolve_list(enabled);
                        assert_eq!(list.gap, if surface { 2.0 } else { 0.0 });
                        assert_eq!(list.padding, if surface { 4.0 } else { 0.0 });
                    }
                    for state in [
                        InteractionState::default(),
                        InteractionState { hovered: true, ..Default::default() },
                        InteractionState { pressed: true, ..Default::default() },
                        InteractionState { focused: true, ..Default::default() },
                        InteractionState { disabled: true, ..Default::default() },
                    ] {
                        for active in [false, true] {
                            let item = theme.resolve_item(active, state);
                            assert_eq!(item.indicator_height, if surface { 0.0 } else { 2.0 });
                            assert_eq!(item.height, if size == RadixSize::One { 32.0 } else { 40.0 });
                            assert_eq!(item.padding_x, if size == RadixSize::One { 8.0 } else { 16.0 });
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn tabs_style_tuning_updates_live_themes() {
        let look = Look::built_in();
        let line = tabs_theme(&look);
        let surface = tabs_theme_for(&look, TabsVariant::Surface);
        // The look-wide TabsStyle tuning applies over the `.lss` tabs.
        let mut tuning = look.tabs_style();
        tuning.gap = 11.0;
        tuning.indicator_height = 7.0;
        look.set_tabs_style(tuning);
        assert_eq!(line.resolve_list(true).gap, 11.0);
        assert_eq!(line.resolve_item(true, Default::default()).indicator_height, 7.0);
        assert_eq!(surface.resolve_list(true).gap, 2.0);
        assert_eq!(surface.resolve_item(true, Default::default()).indicator_height, 0.0);
    }

    #[test]
    fn line_hover_and_press_use_gray_ghost_palette() {
        let look = Look::built_in();
        let tabs = tabs_theme(&look);
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for state in [
                InteractionState { hovered: true, ..Default::default() },
                InteractionState { hovered: true, pressed: true, ..Default::default() },
            ] {
                for active in [false, true] {
                    let item = tabs.resolve_item(active, state);
                    // The gray ghost fill: gray 3 hovered, gray 4 pressed.
                    let step = if state.pressed { 4 } else { 3 };
                    crate::control_group::assert_close(
                        item.background.expect("a hover or press fill"),
                        look.resolve_step(ScaleFamily::Gray, step).hsla(),
                    );
                    let resting = tabs.resolve_item(active, InteractionState::default());
                    assert_eq!(item.indicator, resting.indicator);
                    assert_eq!(item.padding_x, resting.padding_x);
                    assert_eq!(item.height, resting.height);
                    assert!(resting.background.is_none());
                    assert!(
                        tabs.resolve_item(active, InteractionState { disabled: true, ..state }).background.is_none()
                    );
                }
            }
        }
        assert_eq!(tabs_theme(&look.with_size(RadixSize::One)).content_padding(), Some((4.0, 2.0)));
        assert_eq!(tabs.content_padding(), Some((8.0, 4.0)));
    }

    #[test]
    fn baseline_can_change_without_changing_tab_geometry_or_indicator() {
        let look = Look::built_in();
        let theme = tabs_theme(&look);
        let original = theme.resolve_item(true, InteractionState::default());
        let mut style = look.tabs_style();
        // An explicit color is exact in every mode; without one the `.lss` baseline color follows the mode.
        let color = look.resolve_step(ScaleFamily::Gray, 8).hsla();
        style.baseline = Some(TabsBaselineStyle { height: 3.0, inset_x: 12.0, bottom: 4.0, color: Some(color) });
        look.set_tabs_style(style);
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            let baseline = theme.baseline(true).unwrap();
            assert_eq!((baseline.height, baseline.inset_x, baseline.bottom), (3.0, 12.0, 4.0));
            assert_eq!(baseline.color, color);
            let active = theme.resolve_item(true, InteractionState::default());
            assert_eq!(active.height, original.height);
            assert_eq!(active.padding_x, original.padding_x);
            assert_eq!(active.indicator_height, original.indicator_height);
            assert_eq!(theme.indicator_inset(), Some(0.0));
        }
        style.baseline = None;
        look.set_tabs_style(style);
        assert!(theme.baseline(true).is_none());
        assert!(theme.resolve_item(true, InteractionState::default()).indicator.is_some());
    }

    #[test]
    fn fork_copies_tab_style_and_keeps_edits_independent() {
        let look = Look::built_in();
        let mut style = look.tabs_style();
        style.gap = 4.0;
        style.small.height = 36.0;
        look.set_tabs_style(style);
        let fork = look.fork();
        assert_eq!(fork.tabs_style(), style);
        style.indicator_inset = 8.0;
        fork.set_tabs_style(style);
        assert_eq!(look.tabs_style().indicator_inset, 0.0);
        assert_eq!(tabs_theme(&fork.with_size(RadixSize::One)).indicator_inset(), Some(8.0));
    }

    #[test]
    fn line_tabs_use_underline_indicator() {
        let look = Look::built_in();
        let theme = tabs_theme(&look);
        let list = theme.resolve_list(true);
        let active = theme.resolve_item(true, InteractionState::default());
        let inactive = theme.resolve_item(false, InteractionState::default());

        assert!(list.background.is_none());
        assert_eq!(list.padding, 0.0);
        assert_eq!(list.radius, 0.0);
        assert!(active.indicator.is_some());
        assert!(active.background.is_none());
        assert!(inactive.indicator.is_none());
        assert!(active.radius > 0.0);
        assert_eq!(active.height, 40.0);
        assert_eq!(active.padding_x, 16.0);
        assert_eq!(list.gap, 0.0);
        assert_eq!(theme.baseline(true).unwrap().height, 1.0);
    }

    #[test]
    fn surface_tabs_use_list_fill_and_selected_chip() {
        let look = Look::built_in();
        let theme = tabs_theme_for(&look, TabsVariant::Surface);
        let list = theme.resolve_list(true);
        let active = theme.resolve_item(true, InteractionState::default());
        let inactive = theme.resolve_item(false, InteractionState::default());

        assert!(list.background.is_some());
        assert!(list.padding > 0.0);
        assert!(list.radius > 0.0);
        assert!(active.indicator.is_none());
        assert!(inactive.indicator.is_none());
        crate::control_group::assert_close(
            active.background.unwrap(),
            look.resolve_role(SemanticRole::Background).hsla(),
        );
        assert!(inactive.background.is_none());
        assert!(active.radius > 0.0);
        assert!(theme.baseline(true).is_none());
    }
}

#[cfg(test)]
mod drag_tests {
    use xgpui_luma::style::PartRead;
    use super::*;
    #[test]
    fn drag_surfaces_follow_semantic_roles_in_both_modes() {
        let look = Look::built_in();
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            let drag = tabs_theme(&look).drag_style();
            let drag = drag.root();
            crate::control_group::assert_close(
                drag.color("background"),
                look.resolve_role(crate::SemanticRole::Surface).hsla(),
            );
            crate::control_group::assert_close(
                drag.color("foreground"),
                look.resolve_role(crate::SemanticRole::Foreground).hsla(),
            );
            crate::control_group::assert_close(
                drag.color("valid_marker"),
                look.resolve_role(crate::SemanticRole::Focus).hsla(),
            );
            crate::control_group::assert_close(
                drag.color("invalid_marker"),
                look.resolve_role(crate::SemanticRole::Destructive).hsla(),
            );
        }
    }
}
