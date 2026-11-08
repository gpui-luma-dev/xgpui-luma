//! A text area's hard lines laid out as separate paragraphs, stacked vertically
//! behind one [`PlatformParagraphLayout`].
//!
//! Parley gives a paragraph one base direction, set by its first strong
//! character. A text area that is one paragraph would make every line follow
//! its first line; as in native editors, each hard line here gets its own
//! direction and alignment. Positions stay byte offsets into the whole text,
//! whose lines are joined by one separator character.
use std::{ops::Range, sync::Arc};

use anyhow::{Result, ensure};
use gpui::{
    Bounds, ParagraphPosition, Pixels, PlatformParagraphLayout, Point, ShapedRun, Size, TextAffinity, VisualDirection,
    point, px, size,
};

struct Line {
    /// Byte range in the whole text, without its separator.
    range: Range<usize>,
    top: Pixels,
    layout: Arc<dyn PlatformParagraphLayout>,
}

pub(super) struct StackedLayout {
    source: String,
    font_size: Pixels,
    size: Size<Pixels>,
    runs: Vec<ShapedRun>,
    lines: Vec<Line>,
}

fn at(byte_index: usize) -> ParagraphPosition {
    ParagraphPosition { byte_index, affinity: TextAffinity::Downstream }
}

impl StackedLayout {
    /// `layouts` holds one paragraph per `separator`-separated line of `source`, in order.
    pub(super) fn new(
        source: String,
        separator: char,
        font_size: Pixels,
        layouts: Vec<Arc<dyn PlatformParagraphLayout>>,
    ) -> Result<Self> {
        let mut lines = Vec::with_capacity(layouts.len());
        let mut runs = Vec::new();
        let (mut start, mut top, mut width) = (0, px(0.0), px(0.0));
        let mut texts = source.split(separator);
        for layout in layouts {
            let text = texts.next().unwrap_or_default();
            ensure!(layout.source() == text, "paragraph does not match its line");
            let range = start..start + text.len();
            for run in layout.runs() {
                let mut run = run.clone();
                for glyph in &mut run.glyphs {
                    glyph.position.y += top;
                    glyph.index += range.start;
                }
                runs.push(run);
            }
            width = width.max(layout.size().width);
            start = range.end + separator.len_utf8();
            lines.push(Line { range, top, layout: layout.clone() });
            top += layout.size().height;
        }
        ensure!(texts.next().is_none() && !lines.is_empty(), "one paragraph per line is required");
        Ok(Self { source, font_size, size: size(width, top), runs, lines })
    }

    /// The line holding a byte offset; an offset inside a separator belongs to the line before it.
    fn line_of(&self, byte_index: usize) -> usize {
        self.lines.partition_point(|line| line.range.start <= byte_index).saturating_sub(1)
    }

    fn local(&self, index: usize, position: ParagraphPosition) -> ParagraphPosition {
        let range = &self.lines[index].range;
        ParagraphPosition {
            byte_index: position.byte_index.clamp(range.start, range.end) - range.start,
            affinity: position.affinity,
        }
    }

    fn global(&self, index: usize, position: ParagraphPosition) -> ParagraphPosition {
        ParagraphPosition { byte_index: self.lines[index].range.start + position.byte_index, ..position }
    }

    fn shift(&self, index: usize, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        Bounds::new(bounds.origin + point(px(0.0), self.lines[index].top), bounds.size)
    }

    /// Whether a line's base direction is right-to-left: start alignment puts its
    /// logical start on the right half.
    pub(super) fn is_rtl(&self, index: usize) -> bool {
        let layout = &self.lines[index].layout;
        layout.caret_bounds(at(0)).is_ok_and(|caret| caret.left() > layout.size().width / 2.0)
    }

    fn end_of(&self, index: usize) -> ParagraphPosition {
        ParagraphPosition { byte_index: self.lines[index].range.end, affinity: TextAffinity::Upstream }
    }
}

impl PlatformParagraphLayout for StackedLayout {
    fn source(&self) -> &str {
        &self.source
    }

    fn font_size(&self) -> Pixels {
        self.font_size
    }

    fn size(&self) -> Size<Pixels> {
        self.size
    }

    fn runs(&self) -> &[ShapedRun] {
        &self.runs
    }

    fn hit_test(&self, point: Point<Pixels>) -> Result<ParagraphPosition> {
        let index = self.lines.partition_point(|line| line.top <= point.y).saturating_sub(1);
        let line = &self.lines[index];
        let height = line.layout.size().height;
        let y = (point.y - line.top).clamp(px(0.0), (height - px(1.0)).max(px(0.0)));
        let hit = line.layout.hit_test(gpui::point(point.x, y))?;
        Ok(self.global(index, hit))
    }

    fn caret_bounds(&self, position: ParagraphPosition) -> Result<Bounds<Pixels>> {
        let index = self.line_of(position.byte_index);
        let caret = self.lines[index].layout.caret_bounds(self.local(index, position))?;
        Ok(self.shift(index, caret))
    }

    /// Within a line, the paragraph's visual movement; past a line's visual edge,
    /// on to the neighboring line in that line's reading direction.
    fn move_visual(&self, position: ParagraphPosition, direction: VisualDirection) -> Result<ParagraphPosition> {
        let index = self.line_of(position.byte_index);
        let local = self.local(index, position);
        if let Ok(next) = self.lines[index].layout.move_visual(local, direction)
            && next.byte_index != local.byte_index
        {
            return Ok(self.global(index, next));
        }
        let forward = matches!(direction, VisualDirection::Right) != self.is_rtl(index);
        Ok(match (forward, index) {
            (true, index) if index + 1 < self.lines.len() => at(self.lines[index + 1].range.start),
            (false, index) if index > 0 => self.end_of(index - 1),
            _ => position,
        })
    }

    /// Each line's own selection rectangles, plus a mark at the end of a line
    /// whose line break is selected.
    fn selection_bounds(&self, anchor: ParagraphPosition, focus: ParagraphPosition) -> Result<Vec<Bounds<Pixels>>> {
        let (start, end) = if anchor.byte_index <= focus.byte_index {
            (anchor, focus)
        } else {
            (focus, anchor)
        };
        let mut rects = Vec::new();
        for index in self.line_of(start.byte_index)..=self.line_of(end.byte_index) {
            let range = &self.lines[index].range;
            let from = if start.byte_index >= range.start {
                self.local(index, start)
            } else {
                at(0)
            };
            let to = if end.byte_index <= range.end {
                self.local(index, end)
            } else {
                self.local(index, self.end_of(index))
            };
            if from.byte_index < to.byte_index {
                for rect in self.lines[index].layout.selection_bounds(from, to)? {
                    rects.push(self.shift(index, rect));
                }
            }
            if end.byte_index > range.end && index + 1 < self.lines.len() {
                let caret = self.caret_bounds(self.end_of(index))?;
                let width = self.font_size * 0.4;
                let left = if self.is_rtl(index) {
                    caret.left() - width
                } else {
                    caret.left()
                };
                rects.push(Bounds::new(point(left, caret.top()), size(width, caret.size.height)));
            }
        }
        Ok(rects)
    }
}
