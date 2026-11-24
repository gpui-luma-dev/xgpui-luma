//! Wide-middle triptych: leading | middle | trailing when wide enough;
//! otherwise leading|trailing on the first row and a full-width middle below
//! (Radix Colors–style “wide middle” reorder).
//!
//! In the three-column row, the middle is **capped** (`middle_max_width`) and does
//! not grow with leftover space — sides absorb flex. When stacked, middle is `w_full`.

use gpui::{AnyElement, App, Div, Entity, IntoElement, div, px, prelude::*};

use crate::infra::ElementExt;

/// Layout host that tracks whether the middle column has been promoted to its own row.
pub struct WideMiddle {
    stacked: bool,
    gap: f32,
    side_min_width: f32,
    side_preferred_width: f32,
    /// Reserved / minimum width of the middle in the three-column row.
    middle_min_width: f32,
    /// Hard cap for the middle in the three-column row (does not flex-grow past this).
    middle_max_width: f32,
}

impl WideMiddle {
    pub fn new() -> Self {
        let geometry = crate::theme::stylesheet::SharedGeometry::sdk();
        Self {
            stacked: false,
            gap: geometry.wide_middle_gap,
            side_min_width: geometry.wide_side_min_width,
            side_preferred_width: geometry.wide_side_preferred_width,
            middle_min_width: geometry.wide_middle_min_width,
            middle_max_width: geometry.wide_middle_max_width,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    pub fn side_min_width(mut self, width: f32) -> Self {
        self.side_min_width = width.max(1.0);
        self.side_preferred_width = self.side_preferred_width.max(self.side_min_width);
        self
    }

    pub fn side_preferred_width(mut self, width: f32) -> Self {
        self.side_preferred_width = width.max(self.side_min_width);
        self
    }

    pub fn middle_min_width(mut self, width: f32) -> Self {
        self.middle_min_width = width.max(1.0);
        self.middle_max_width = self.middle_max_width.max(self.middle_min_width);
        self
    }

    /// Maximum width of the middle column while in the three-column row.
    pub fn middle_max_width(mut self, width: f32) -> Self {
        self.middle_max_width = width.max(self.middle_min_width);
        self
    }

    pub fn is_stacked(&self) -> bool {
        self.stacked
    }

    /// Minimum width that still fits leading + middle + trailing with gaps.
    pub fn row_threshold(&self) -> f32 {
        self.side_min_width * 2.0 + self.middle_min_width + self.gap * 2.0
    }

    /// Returns `true` when the stacked mode changed.
    pub fn update_from_width(&mut self, width: f32) -> bool {
        let stacked = width + 0.5 < self.row_threshold();
        if stacked != self.stacked {
            self.stacked = stacked;
            true
        } else {
            false
        }
    }
}

impl Default for WideMiddle {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawn a configured [`WideMiddle`] host.
pub fn spawn_wide_middle(config: WideMiddle, cx: &mut App) -> Entity<WideMiddle> {
    cx.new(|_| config)
}

/// Measure-aware triptych builder. Call from a parent `Render` with `&App` so the
/// current stacked mode can be read; `on_prepaint` flips mode and refreshes.
pub struct WideMiddleLayout {
    host: Entity<WideMiddle>,
    leading: AnyElement,
    middle: AnyElement,
    trailing: AnyElement,
}

impl WideMiddleLayout {
    pub fn new(
        host: Entity<WideMiddle>,
        leading: impl IntoElement,
        middle: impl IntoElement,
        trailing: impl IntoElement,
    ) -> Self {
        Self {
            host,
            leading: leading.into_any_element(),
            middle: middle.into_any_element(),
            trailing: trailing.into_any_element(),
        }
    }

    pub fn build(self, cx: &App) -> Div {
        let WideMiddleLayout { host, leading, middle, trailing } = self;
        let (gap, side_min, side_pref, middle_min, middle_max, stacked) = {
            let state = host.read(cx);
            (
                state.gap,
                state.side_min_width,
                state.side_preferred_width,
                state.middle_min_width,
                state.middle_max_width,
                state.stacked,
            )
        };

        let leading = side_column(side_min, side_pref, leading);
        let trailing = side_column(side_min, side_pref, trailing);
        let middle = middle_column(middle_min, middle_max, stacked, middle);

        let host_for_paint = host.clone();
        let content = if stacked {
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(gap))
                .child(div().w_full().flex().gap(px(gap)).items_start().child(leading).child(trailing))
                .child(middle)
        } else {
            div().w_full().flex().gap(px(gap)).items_start().child(leading).child(middle).child(trailing)
        };

        div()
            .w_full()
            .min_w(px(0.0))
            .on_prepaint(move |bounds, window, cx| {
                let width = bounds.size.width.as_f32();
                host_for_paint.update(cx, |this, _cx| {
                    if this.update_from_width(width) {
                        window.refresh();
                    }
                });
            })
            .child(content)
    }
}

fn side_column(min_w: f32, preferred_w: f32, child: impl IntoElement) -> Div {
    div().flex_1().min_w(px(min_w)).flex_basis(px(preferred_w)).child(div().w_full().child(child))
}

fn middle_column(min_w: f32, max_w: f32, stacked: bool, child: impl IntoElement) -> Div {
    let inner = div().w_full().child(child);
    if stacked {
        // Own row: expand; interior content may still center a fixed panel.
        div().w_full().flex_none().min_w(px(min_w)).child(inner)
    } else {
        // Three-column row: hard-capped; sides take leftover flex.
        div().flex_none().w(px(max_w)).min_w(px(min_w)).max_w(px(max_w)).child(inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacks_below_combined_min_widths() {
        let mut layout = WideMiddle::new().gap(24.0).side_min_width(240.0).middle_min_width(435.0);
        assert!(!layout.update_from_width(1200.0));
        assert!(!layout.is_stacked());
        assert!(layout.update_from_width(800.0));
        assert!(layout.is_stacked());
        assert!(!layout.update_from_width(800.0));
        assert!(layout.update_from_width(1000.0));
        assert!(!layout.is_stacked());
    }

    #[test]
    fn threshold_includes_gaps() {
        let layout = WideMiddle::new().gap(24.0).side_min_width(240.0).middle_min_width(435.0);
        assert_eq!(layout.row_threshold(), 240.0 * 2.0 + 435.0 + 48.0);
    }

    #[test]
    fn default_middle_max_is_435() {
        let layout = WideMiddle::new();
        assert_eq!(layout.middle_max_width, 435.0);
        assert_eq!(layout.middle_min_width, 435.0);
    }
}
