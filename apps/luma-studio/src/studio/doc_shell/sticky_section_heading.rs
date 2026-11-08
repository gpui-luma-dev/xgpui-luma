use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{AnyElement, Bounds, Hsla, ParentElement, Pixels, div, prelude::*, px};
use xgpui_luma_look_shadcn::LumaTypographyExt;

pub(crate) const SECTION_HEADING_SHELL_PAD_TOP: f32 = 2.0;
pub(crate) const SECTION_HEADING_SHELL_PAD_BOTTOM: f32 = 6.0;
pub(crate) const SECTION_HEADING_CONTENT_GAP: f32 = 10.0;

const SECTION_HEADING_INNER_GAP: f32 = 2.0;
const SECTION_HEADING_DIVIDER_MARGIN: f32 = 6.0;
const STICKY_FADE_RANGE: f32 = 18.0;

thread_local! {
    static ACTIVE_STICKY_HEADING_TRACKER: RefCell<Option<Rc<RefCell<StickySectionHeadingTracker>>>> =
        const { RefCell::new(None) };
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SectionHeading {
    pub title: &'static str,
    pub description: &'static str,
    pub title_color: Hsla,
    pub muted_text: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Debug)]
struct SectionHeadingAnchor {
    order: usize,
    top: f32,
    height: f32,
    heading: SectionHeading,
}

#[derive(Clone, Debug)]
pub(crate) struct StickySectionSnapshot {
    pub heading: SectionHeading,
    pub opacity: f32,
}

#[derive(Default)]
pub(crate) struct StickySectionHeadingTracker {
    content_origin_y: f32,
    anchors: HashMap<&'static str, SectionHeadingAnchor>,
}

impl StickySectionHeadingTracker {
    pub fn reset(&mut self) {
        self.content_origin_y = 0.0;
        self.anchors.clear();
    }

    pub fn set_content_origin_y(&mut self, origin_y: f32) -> bool {
        if (self.content_origin_y - origin_y).abs() <= 0.5 {
            return false;
        }

        self.content_origin_y = origin_y;
        true
    }

    pub fn register(
        &mut self,
        title: &'static str,
        order: usize,
        bounds: Bounds<Pixels>,
        heading: SectionHeading,
    ) -> bool {
        let top = bounds.origin.y.as_f32() - self.content_origin_y;
        let height = bounds.size.height.as_f32();
        let next = SectionHeadingAnchor { order, top, height, heading };

        match self.anchors.get(title) {
            Some(current)
                if (current.top - next.top).abs() <= 0.5
                    && (current.height - next.height).abs() <= 0.5
                    && current.heading == next.heading =>
            {
                return false;
            }
            _ => {}
        }

        self.anchors.insert(title, next);
        true
    }

    pub fn snapshot(&self, scroll_y: f32) -> Option<StickySectionSnapshot> {
        if self.anchors.is_empty() {
            return None;
        }

        let anchors = self.sorted_anchors();

        let active_idx = anchors.iter().rposition(|anchor| anchor.top <= scroll_y + 0.5)?;
        let active = &anchors[active_idx];

        if scroll_y <= active.top + 0.5 {
            return None;
        }

        let sticky_height = active.height + SECTION_HEADING_SHELL_PAD_TOP + 2.0;

        if let Some(next) = anchors.get(active_idx + 1) {
            let next_in_viewport = next.top - scroll_y;
            if next_in_viewport <= sticky_height {
                return None;
            }

            let opacity = if next_in_viewport < sticky_height + STICKY_FADE_RANGE {
                ((next_in_viewport - sticky_height) / STICKY_FADE_RANGE).clamp(0.0, 1.0)
            } else {
                1.0
            };

            return Some(StickySectionSnapshot { heading: active.heading.clone(), opacity });
        }

        Some(StickySectionSnapshot { heading: active.heading.clone(), opacity: 1.0 })
    }

    pub fn active_title(&self, scroll_y: f32) -> Option<&'static str> {
        let anchors = self.sorted_anchors();
        let first = anchors.first()?;
        let active = anchors
            .iter()
            .rposition(|anchor| anchor.top <= scroll_y + 0.5)
            .and_then(|index| anchors.get(index))
            .unwrap_or(first);

        Some(active.heading.title)
    }

    pub fn anchor_top(&self, title: &str) -> Option<f32> {
        self.anchors.get(title).map(|anchor| anchor.top)
    }

    fn sorted_anchors(&self) -> Vec<SectionHeadingAnchor> {
        let mut anchors: Vec<_> = self.anchors.values().cloned().collect();
        anchors.sort_by_key(|anchor| anchor.order);
        anchors
    }
}

pub(crate) fn with_sticky_heading_tracker<T>(
    tracker: Rc<RefCell<StickySectionHeadingTracker>>,
    f: impl FnOnce() -> T,
) -> T {
    ACTIVE_STICKY_HEADING_TRACKER.with(|slot| {
        *slot.borrow_mut() = Some(tracker);
        let result = f();
        *slot.borrow_mut() = None;
        result
    })
}

fn active_sticky_heading_tracker() -> Option<Rc<RefCell<StickySectionHeadingTracker>>> {
    ACTIVE_STICKY_HEADING_TRACKER.with(|slot| slot.borrow().clone())
}

pub(crate) fn render_section_heading(heading: &SectionHeading) -> AnyElement {
    render_section_heading_with_options(heading, true)
}

pub(crate) fn render_section_heading_with_options(heading: &SectionHeading, truncate_description: bool) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(SECTION_HEADING_INNER_GAP))
        .child(div().text_h3().text_color(heading.title_color).child(heading.title))
        .child({
            let mut description = div()
                .w_full()
                .min_w(px(0.0))
                .typography_xs()
                .text_color(heading.muted_text)
                .child(heading.description);
            if truncate_description {
                description = description.truncate();
            }
            description
        })
        .child(div().w_full().h(px(1.0)).mt(px(SECTION_HEADING_DIVIDER_MARGIN)).bg(heading.border))
        .into_any_element()
}

pub(crate) fn render_section_heading_anchor(
    title: &'static str,
    description: &'static str,
    title_color: Hsla,
    muted_text: Hsla,
    border: Hsla,
) -> AnyElement {
    render_section_heading_anchor_with_order(
        title,
        description,
        style_guide_section_order_for_title(title),
        title_color,
        muted_text,
        border,
    )
}

pub(crate) fn render_section_heading_anchor_with_order(
    title: &'static str,
    description: &'static str,
    order: usize,
    title_color: Hsla,
    muted_text: Hsla,
    border: Hsla,
) -> AnyElement {
    render_section_heading_anchor_with_order_options(title, description, order, title_color, muted_text, border, true)
}

pub(crate) fn render_section_heading_anchor_with_order_options(
    title: &'static str,
    description: &'static str,
    order: usize,
    title_color: Hsla,
    muted_text: Hsla,
    border: Hsla,
    truncate_description: bool,
) -> AnyElement {
    let heading = SectionHeading { title, description, title_color, muted_text, border };
    let tracker = active_sticky_heading_tracker();
    let title_for_register = title;

    let header = render_section_heading_with_options(&heading, truncate_description);

    let Some(tracker) = tracker else {
        return header;
    };

    div()
        .on_children_prepainted(move |child_bounds, window, cx| {
            let Some(bounds) = child_bounds.first() else {
                return;
            };

            let changed = tracker.borrow_mut().register(title_for_register, order, *bounds, heading.clone());
            if changed {
                cx.notify(window.current_view());
            }
        })
        .id(format!("doc-section-heading-{title}"))
        .child(header)
        .into_any_element()
}

pub(crate) fn render_sticky_section_heading_lane(
    snapshot: Option<StickySectionSnapshot>,
    background: Hsla,
    max_width: f32,
) -> AnyElement {
    let Some(snapshot) = snapshot else {
        return div().into_any_element();
    };

    div()
        .id("doc-sticky-section-heading-lane")
        .absolute()
        .top(px(0.0))
        .left(px(0.0))
        .right(px(0.0))
        .flex()
        .justify_start()
        .opacity(snapshot.opacity)
        .bg(background)
        .child(
            div()
                .w_full()
                .max_w(px(max_width))
                .pt(px(SECTION_HEADING_SHELL_PAD_TOP))
                .pb(px(2.0))
                .child(render_section_heading(&snapshot.heading)),
        )
        .into_any_element()
}

fn style_guide_section_order_for_title(title: &str) -> usize {
    match title {
        "Accordion" => 0,
        "Buttons" => 1,
        "Checkbox" => 2,
        "Feedback" => 3,
        "Icon Button" => 4,
        "Table" => 5,
        "Listbox" => 6,
        "Menus" => 7,
        "Pager" => 8,
        "Radio" => 9,
        "Scrollbar" => 10,
        "Selectors" => 11,
        "Sidebar" => 12,
        "Slider" => 13,
        "Switch" => 14,
        "Tabs" => 15,
        "Text Area" => 16,
        "Text Field" => 17,
        "Toggles" => 18,
        "Toolbar" => 19,
        "Tree View" => 20,
        "Typography" => 21,
        "Shadow Tokens" => 22,
        _ => usize::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_sticks_after_header_scrolls_past_top() {
        let mut tracker = StickySectionHeadingTracker { content_origin_y: 0.0, ..Default::default() };
        tracker.register(
            "Typography",
            0,
            gpui::Bounds { origin: gpui::point(px(0.0), px(100.0)), size: gpui::size(px(800.0), px(72.0)) },
            SectionHeading {
                title: "Typography",
                description: "desc",
                title_color: gpui::black(),
                muted_text: gpui::black(),
                border: gpui::black(),
            },
        );
        tracker.register(
            "Buttons",
            1,
            gpui::Bounds { origin: gpui::point(px(0.0), px(900.0)), size: gpui::size(px(800.0), px(72.0)) },
            SectionHeading {
                title: "Buttons",
                description: "desc",
                title_color: gpui::black(),
                muted_text: gpui::black(),
                border: gpui::black(),
            },
        );

        assert!(tracker.snapshot(80.0).is_none());
        let sticky = tracker.snapshot(120.0).expect("typography should stick");
        assert_eq!(sticky.heading.title, "Typography");
        assert_eq!(sticky.opacity, 1.0);
    }

    #[test]
    fn snapshot_hides_when_next_header_enters_sticky_zone() {
        let mut tracker = StickySectionHeadingTracker::default();
        tracker.register(
            "Typography",
            0,
            gpui::Bounds { origin: gpui::point(px(0.0), px(100.0)), size: gpui::size(px(800.0), px(72.0)) },
            SectionHeading {
                title: "Typography",
                description: "desc",
                title_color: gpui::black(),
                muted_text: gpui::black(),
                border: gpui::black(),
            },
        );
        tracker.register(
            "Buttons",
            1,
            gpui::Bounds { origin: gpui::point(px(0.0), px(900.0)), size: gpui::size(px(800.0), px(72.0)) },
            SectionHeading {
                title: "Buttons",
                description: "desc",
                title_color: gpui::black(),
                muted_text: gpui::black(),
                border: gpui::black(),
            },
        );

        assert!(tracker.snapshot(860.0).is_none());
    }

    #[test]
    fn snapshot_fades_as_next_header_approaches() {
        let mut tracker = StickySectionHeadingTracker::default();
        tracker.register(
            "Typography",
            0,
            gpui::Bounds { origin: gpui::point(px(0.0), px(100.0)), size: gpui::size(px(800.0), px(72.0)) },
            SectionHeading {
                title: "Typography",
                description: "desc",
                title_color: gpui::black(),
                muted_text: gpui::black(),
                border: gpui::black(),
            },
        );
        tracker.register(
            "Buttons",
            1,
            gpui::Bounds { origin: gpui::point(px(0.0), px(900.0)), size: gpui::size(px(800.0), px(72.0)) },
            SectionHeading {
                title: "Buttons",
                description: "desc",
                title_color: gpui::black(),
                muted_text: gpui::black(),
                border: gpui::black(),
            },
        );

        let sticky = tracker.snapshot(810.0).expect("typography should fade");
        assert!(sticky.opacity > 0.0 && sticky.opacity < 1.0);
    }

    #[test]
    fn active_title_tracks_registered_section_without_sticky_visibility_gaps() {
        let mut tracker = StickySectionHeadingTracker::default();
        tracker.register(
            "Accordion",
            style_guide_section_order_for_title("Accordion"),
            gpui::Bounds { origin: gpui::point(px(0.0), px(100.0)), size: gpui::size(px(800.0), px(72.0)) },
            SectionHeading {
                title: "Accordion",
                description: "desc",
                title_color: gpui::black(),
                muted_text: gpui::black(),
                border: gpui::black(),
            },
        );
        tracker.register(
            "Table",
            style_guide_section_order_for_title("Table"),
            gpui::Bounds { origin: gpui::point(px(0.0), px(900.0)), size: gpui::size(px(800.0), px(72.0)) },
            SectionHeading {
                title: "Table",
                description: "desc",
                title_color: gpui::black(),
                muted_text: gpui::black(),
                border: gpui::black(),
            },
        );

        assert_eq!(tracker.active_title(0.0), Some("Accordion"));
        assert_eq!(tracker.active_title(860.0), Some("Accordion"));
        assert_eq!(tracker.active_title(920.0), Some("Table"));
        assert_eq!(tracker.anchor_top("Table"), Some(900.0));
    }
}
