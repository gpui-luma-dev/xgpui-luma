//! Shared popup placement: one place that decides where a popup opens next to
//! its anchor, flips it when it does not fit, and reports the space it may use
//! (#433).
//!
//! Placement is logical. A popup opens on a [`PopupSide`] of the anchor and is
//! aligned along that side by a [`PopupAlign`]; `Start` and `End` are resolved
//! through a [`LayoutDirection`], so the same request mirrors in right-to-left
//! layouts. The result feeds GPUI's `anchored()` element, which still clamps the
//! popup to the window margin.

use gpui::{Anchor, Bounds, Pixels, Point, Size, point, px};

/// The direction inline content flows in. `Start`/`End` placements follow it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum LayoutDirection {
    #[default]
    LeftToRight,
    RightToLeft,
}

/// The side of the anchor a popup opens on.
///
/// `End` and `Start` are logical: in a left-to-right layout `End` is the right
/// side, in a right-to-left layout it is the left side.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum PopupSide {
    #[default]
    Below,
    Above,
    End,
    Start,
}

impl PopupSide {
    /// The side across the anchor from this one.
    pub fn opposite(self) -> Self {
        match self {
            Self::Below => Self::Above,
            Self::Above => Self::Below,
            Self::End => Self::Start,
            Self::Start => Self::End,
        }
    }

    fn is_vertical(self) -> bool {
        matches!(self, Self::Below | Self::Above)
    }
}

/// How a popup lines up with the anchor along the side it opens on.
///
/// Below or above the anchor, `Start`/`End` are the logical inline edges. Beside
/// the anchor, `Start` lines up the tops and `End` the bottoms.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum PopupAlign {
    #[default]
    Start,
    Center,
    End,
}

/// What a popup needs placed: its anchor, preferred side and alignment, its
/// size, and the window it must fit in.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct PopupPlacementRequest {
    /// The trigger's bounds, or a zero-size bounds at a pointer position.
    pub anchor: Bounds<Pixels>,
    pub side: PopupSide,
    pub align: PopupAlign,
    /// Open on the opposite side when the preferred one is too small.
    pub flip: bool,
    /// Space between the anchor and the popup.
    pub gap: Pixels,
    /// The popup's measured or estimated size; without it the popup never flips.
    pub content_size: Option<Size<Pixels>>,
    pub viewport: Size<Pixels>,
    pub window_margin: Pixels,
    /// The smallest main-axis size the popup is given, even when less is free.
    pub min_main_size: Pixels,
    pub direction: LayoutDirection,
}

impl PopupPlacementRequest {
    pub fn new(anchor: Bounds<Pixels>, side: PopupSide, viewport: Size<Pixels>) -> Self {
        Self {
            anchor,
            side,
            align: PopupAlign::Start,
            flip: false,
            gap: px(0.0),
            content_size: None,
            viewport,
            window_margin: px(0.0),
            min_main_size: px(0.0),
            direction: LayoutDirection::LeftToRight,
        }
    }

    pub fn align(mut self, align: PopupAlign) -> Self {
        self.align = align;
        self
    }

    pub fn flip(mut self, flip: bool) -> Self {
        self.flip = flip;
        self
    }

    pub fn gap(mut self, gap: Pixels) -> Self {
        self.gap = gap;
        self
    }

    pub fn content_size(mut self, size: Option<Size<Pixels>>) -> Self {
        self.content_size = size;
        self
    }

    pub fn window_margin(mut self, margin: Pixels) -> Self {
        self.window_margin = margin;
        self
    }

    pub fn min_main_size(mut self, size: Pixels) -> Self {
        self.min_main_size = size;
        self
    }

    pub fn direction(mut self, direction: LayoutDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Free space between the anchor (plus gap) and the window margin on `side`.
    pub fn available(&self, side: PopupSide) -> Pixels {
        let Self { anchor, gap, viewport, window_margin, .. } = *self;
        let space = match self.physical(side) {
            Physical::Below => viewport.height - window_margin - (anchor.bottom() + gap),
            Physical::Above => anchor.top() - gap - window_margin,
            Physical::Right => viewport.width - window_margin - (anchor.right() + gap),
            Physical::Left => anchor.left() - gap - window_margin,
        };
        space.max(px(0.0))
    }

    /// Choose the side and compute where the popup goes.
    pub fn resolve(&self) -> ResolvedPopupPlacement {
        let side = self.resolve_side();
        let vertical = side.is_vertical();
        let available = self.available(side).max(self.min_main_size);
        let cross_space = if vertical {
            self.viewport.width
        } else {
            self.viewport.height
        };
        let cross_space = (cross_space - self.window_margin * 2.0).max(px(0.0));
        let max_size = if vertical {
            Size { width: cross_space, height: available }
        } else {
            Size { width: available, height: cross_space }
        };
        let scrolling = self.content_size.is_some_and(|size| {
            if vertical {
                size.height > available
            } else {
                size.width > available
            }
        });

        let (anchor, position, offset) = self.anchor_point(side);
        ResolvedPopupPlacement { side, anchor, position, offset, max_size, scrolling }
    }

    /// Center the popup over the anchor instead of beside it, limited to the
    /// window within its margins. Side, alignment, gap and flip are ignored.
    pub fn resolve_centered(&self) -> ResolvedPopupPlacement {
        let size = self.content_size.unwrap_or_default();
        let max_size = Size {
            width: (self.viewport.width - self.window_margin * 2.0).max(px(0.0)),
            height: (self.viewport.height - self.window_margin * 2.0).max(px(0.0)).max(self.min_main_size),
        };
        let height = size.height.min(max_size.height);
        ResolvedPopupPlacement {
            side: self.side,
            anchor: Anchor::TopLeft,
            position: self.anchor.center(),
            offset: point(-(size.width * 0.5), -(height * 0.5)),
            max_size,
            scrolling: size.height > max_size.height,
        }
    }

    /// The preferred side if it fits, else the opposite one if that fits, else
    /// whichever has more room.
    fn resolve_side(&self) -> PopupSide {
        let preferred = self.side;
        let Some(size) = self.content_size.filter(|_| self.flip) else {
            return preferred;
        };
        let needed = if preferred.is_vertical() {
            size.height
        } else {
            size.width
        };
        let opposite = preferred.opposite();
        let (here, there) = (self.available(preferred), self.available(opposite));
        if here >= needed {
            preferred
        } else if there >= needed || there > here {
            opposite
        } else {
            preferred
        }
    }

    fn physical(&self, side: PopupSide) -> Physical {
        let rtl = self.direction == LayoutDirection::RightToLeft;
        match side {
            PopupSide::Below => Physical::Below,
            PopupSide::Above => Physical::Above,
            PopupSide::End if rtl => Physical::Left,
            PopupSide::End => Physical::Right,
            PopupSide::Start if rtl => Physical::Right,
            PopupSide::Start => Physical::Left,
        }
    }

    /// The popup corner, the point on the anchor it attaches to, and the offset.
    /// Centered popups use a left corner and a half-size offset, which is what
    /// the overlay presence animation scales from.
    fn anchor_point(&self, side: PopupSide) -> (Anchor, Point<Pixels>, Point<Pixels>) {
        let Self { anchor: a, gap, .. } = *self;
        let size = self.content_size.unwrap_or_default();
        let rtl = self.direction == LayoutDirection::RightToLeft;
        match self.physical(side) {
            Physical::Below | Physical::Above => {
                let below = self.physical(side) == Physical::Below;
                let (y, dy) = if below { (a.bottom(), gap) } else { (a.top(), -gap) };
                // Logical start is the left edge, or the right edge right-to-left.
                let start_on_left = !rtl;
                let (on_left, x, dx) = match self.align {
                    PopupAlign::Center => (true, a.center().x, -(size.width * 0.5)),
                    PopupAlign::Start if start_on_left => (true, a.left(), px(0.0)),
                    PopupAlign::End if !start_on_left => (true, a.left(), px(0.0)),
                    PopupAlign::Start | PopupAlign::End => (false, a.right(), px(0.0)),
                };
                let corner = match (below, on_left) {
                    (true, true) => Anchor::TopLeft,
                    (true, false) => Anchor::TopRight,
                    (false, true) => Anchor::BottomLeft,
                    (false, false) => Anchor::BottomRight,
                };
                (corner, point(x, y), point(dx, dy))
            }
            physical => {
                let right = physical == Physical::Right;
                let (x, dx) = if right { (a.right(), gap) } else { (a.left(), -gap) };
                let (top, y, dy) = match self.align {
                    PopupAlign::Start => (true, a.top(), px(0.0)),
                    PopupAlign::Center => (true, a.center().y, -(size.height * 0.5)),
                    PopupAlign::End => (false, a.bottom(), px(0.0)),
                };
                let corner = match (top, right) {
                    (true, true) => Anchor::TopLeft,
                    (true, false) => Anchor::TopRight,
                    (false, true) => Anchor::BottomLeft,
                    (false, false) => Anchor::BottomRight,
                };
                (corner, point(x, y), point(dx, dy))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Physical {
    Below,
    Above,
    Right,
    Left,
}

/// Where a popup goes, for GPUI's `anchored()`: attach `anchor` (a corner of the
/// popup) to `position`, then move it by `offset`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ResolvedPopupPlacement {
    /// The side chosen after flipping.
    pub side: PopupSide,
    pub anchor: Anchor,
    pub position: Point<Pixels>,
    pub offset: Point<Pixels>,
    /// The space the popup may use: the free space on the chosen side (at least
    /// the request's minimum), and the window size within its margins across.
    pub max_size: Size<Pixels>,
    /// The content is larger than `max_size` on the main axis and should scroll.
    pub scrolling: bool,
}

#[cfg(test)]
mod tests {
    use gpui::size;

    use super::*;

    fn trigger(x: f32, y: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), size(px(40.0), px(20.0)))
    }

    fn request(anchor: Bounds<Pixels>, side: PopupSide) -> PopupPlacementRequest {
        PopupPlacementRequest::new(anchor, side, size(px(500.0), px(500.0)))
            .gap(px(4.0))
            .window_margin(px(8.0))
    }

    #[test]
    fn below_start_attaches_top_left_to_anchor_bottom_left() {
        let resolved = request(trigger(20.0, 10.0), PopupSide::Below).resolve();
        assert_eq!(resolved.side, PopupSide::Below);
        assert_eq!(resolved.anchor, Anchor::TopLeft);
        assert_eq!(resolved.position, point(px(20.0), px(30.0)));
        assert_eq!(resolved.offset, point(px(0.0), px(4.0)));
        // 500 - 8 margin - (30 bottom + 4 gap).
        assert_eq!(resolved.max_size.height, px(458.0));
        assert_eq!(resolved.max_size.width, px(484.0));
    }

    #[test]
    fn center_alignment_offsets_by_half_the_content_width() {
        let resolved = request(trigger(20.0, 10.0), PopupSide::Below)
            .align(PopupAlign::Center)
            .content_size(Some(size(px(100.0), px(50.0))))
            .resolve();
        assert_eq!(resolved.anchor, Anchor::TopLeft);
        assert_eq!(resolved.position, point(px(40.0), px(30.0)));
        assert_eq!(resolved.offset, point(px(-50.0), px(4.0)));
    }

    #[test]
    fn flips_above_when_only_above_fits() {
        let resolved = request(trigger(20.0, 450.0), PopupSide::Below)
            .flip(true)
            .content_size(Some(size(px(100.0), px(80.0))))
            .resolve();
        assert_eq!(resolved.side, PopupSide::Above);
        assert_eq!(resolved.anchor, Anchor::BottomLeft);
        assert_eq!(resolved.position, point(px(20.0), px(450.0)));
        assert_eq!(resolved.offset, point(px(0.0), px(-4.0)));
        assert!(!resolved.scrolling);
    }

    #[test]
    fn does_not_flip_without_flip_or_content_size() {
        let low = trigger(20.0, 450.0);
        let fixed = request(low, PopupSide::Below).content_size(Some(size(px(100.0), px(80.0)))).resolve();
        assert_eq!(fixed.side, PopupSide::Below);
        let unmeasured = request(low, PopupSide::Below).flip(true).resolve();
        assert_eq!(unmeasured.side, PopupSide::Below);
    }

    #[test]
    fn neither_side_fits_picks_the_larger_and_scrolls() {
        // 202 below, 254 above; the popup needs 300.
        let anchor = trigger(20.0, 266.0);
        let resolved = request(anchor, PopupSide::Below)
            .flip(true)
            .content_size(Some(size(px(100.0), px(300.0))))
            .resolve();
        assert_eq!(resolved.side, PopupSide::Above);
        assert_eq!(resolved.max_size.height, px(254.0));
        assert!(resolved.scrolling);

        let resolved = request(trigger(20.0, 100.0), PopupSide::Above)
            .flip(true)
            .content_size(Some(size(px(100.0), px(600.0))))
            .resolve();
        assert_eq!(resolved.side, PopupSide::Below);
        assert!(resolved.scrolling);
    }

    #[test]
    fn min_main_size_floors_the_available_space() {
        let resolved = request(trigger(20.0, 470.0), PopupSide::Below).min_main_size(px(40.0)).resolve();
        assert_eq!(resolved.max_size.height, px(40.0));
    }

    #[test]
    fn centered_popup_is_limited_to_the_window() {
        let anchor = trigger(200.0, 240.0);
        let fits = request(anchor, PopupSide::Below).content_size(Some(size(px(100.0), px(60.0)))).resolve_centered();
        assert_eq!(fits.anchor, Anchor::TopLeft);
        assert_eq!(fits.position, anchor.center());
        assert_eq!(fits.offset, point(px(-50.0), px(-30.0)));
        assert!(!fits.scrolling);

        let tall = request(anchor, PopupSide::Below).content_size(Some(size(px(100.0), px(900.0)))).resolve_centered();
        assert_eq!(tall.max_size.height, px(484.0));
        assert_eq!(tall.offset.y, px(-242.0));
        assert!(tall.scrolling);
    }

    #[test]
    fn end_side_opens_right_bottom_aligned_left_to_right() {
        let resolved = request(trigger(20.0, 10.0), PopupSide::End).align(PopupAlign::End).resolve();
        assert_eq!(resolved.anchor, Anchor::BottomLeft);
        assert_eq!(resolved.position, point(px(60.0), px(30.0)));
        assert_eq!(resolved.offset, point(px(4.0), px(0.0)));
        assert_eq!(resolved.max_size.height, px(484.0));
    }

    #[test]
    fn submenu_flips_to_the_start_side_near_the_window_edge() {
        let resolved = request(trigger(400.0, 10.0), PopupSide::End)
            .flip(true)
            .content_size(Some(size(px(120.0), px(200.0))))
            .resolve();
        assert_eq!(resolved.side, PopupSide::Start);
        assert_eq!(resolved.anchor, Anchor::TopRight);
        assert_eq!(resolved.position, point(px(400.0), px(10.0)));
        assert_eq!(resolved.offset, point(px(-4.0), px(0.0)));
    }

    #[test]
    fn right_to_left_mirrors_start_and_end() {
        let rtl = |side, align| {
            request(trigger(200.0, 10.0), side).align(align).direction(LayoutDirection::RightToLeft).resolve()
        };

        let below = rtl(PopupSide::Below, PopupAlign::Start);
        assert_eq!(below.anchor, Anchor::TopRight);
        assert_eq!(below.position, point(px(240.0), px(30.0)));

        let below_end = rtl(PopupSide::Below, PopupAlign::End);
        assert_eq!(below_end.anchor, Anchor::TopLeft);
        assert_eq!(below_end.position, point(px(200.0), px(30.0)));

        let end = rtl(PopupSide::End, PopupAlign::Start);
        assert_eq!(end.anchor, Anchor::TopRight);
        assert_eq!(end.position, point(px(200.0), px(10.0)));
        assert_eq!(end.offset, point(px(-4.0), px(0.0)));
        // 200 left - 4 gap - 8 margin.
        assert_eq!(end.max_size.width, px(188.0));
    }
}
