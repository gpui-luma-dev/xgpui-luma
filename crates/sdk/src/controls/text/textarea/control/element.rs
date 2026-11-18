use std::sync::Arc;

use gpui::{
    App, Bounds, ElementInputHandler, GlobalElementId, IntoElement, LayoutId, PaintQuad, Pixels, ShapedLine, Style,
    TextAlign, TextRun, Window, fill, font, point, px, relative, size,
};

use super::layout::{TextAreaCachedLine, TextAreaDecorationKey, TextAreaLayoutCache, TextAreaLayoutKey, TextAreaPaintLine};
use super::TextArea;
use crate::theme::StandardBoxScale;

#[cfg(all(test, feature = "test-support"))]
thread_local! {
    pub(super) static SHAPING_COUNTS: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
}

pub(super) struct TextAreaElement {
    pub(super) input: gpui::Entity<TextArea>,
}

pub(super) struct TextAreaPrepaintState {
    lines: Arc<[TextAreaCachedLine]>,
    key: TextAreaLayoutKey,
    paint_lines: Vec<TextAreaPaintLine>,
    selection_quads: Vec<PaintQuad>,
    caret_quad: Option<PaintQuad>,
    placeholder_line: Option<ShapedLine>,
    line_height: Pixels,
    vertical_scroll: Pixels,
}
impl IntoElement for TextAreaElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl gpui::Element for TextAreaElement {
    type RequestLayoutState = ();
    type PrepaintState = TextAreaPrepaintState;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let (theme, state, enabled, rows, look_override) = {
            let input = self.input.read(cx);
            (
                input.model.theme.clone(),
                input.state,
                input.model.enabled,
                input.model.rows.max(1),
                input.model.look_override.clone(),
            )
        };
        let scale_factor = window.scale_factor();
        let scale = StandardBoxScale::compute(&theme.metrics(), scale_factor);
        let mut look = theme.resolve_look(state, enabled, &scale);
        if let Some(override_fn) = &look_override {
            look = override_fn(look);
        }
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = px(look.typography.line_height * rows as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let (theme, state, enabled, look_override) = {
            let input = self.input.read(cx);
            (input.model.theme.clone(), input.state, input.model.enabled, input.model.look_override.clone())
        };
        let scale_factor = window.scale_factor();
        let scale = StandardBoxScale::compute(&theme.metrics(), scale_factor);
        let input = self.input.read(cx);
        let mut look = theme.resolve_look(state, enabled, &scale);
        if let Some(override_fn) = &look_override {
            look = override_fn(look);
        }
        let line_height = px(look.typography.line_height);
        let font_size = px(look.typography.size);
        let mut selection_quads = Vec::new();
        let mut caret_quad = None;
        let selection = input.state.selection_range();
        let show_placeholder = input.model.value.is_empty() && !input.state.focused;
        let mut text_font = font(look.font_family.clone());
        text_font.weight = look.typography.weight;
        let run_for = |len: usize, color| TextRun {
            len,
            font: text_font.clone(),
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let placeholder_line = if show_placeholder {
            let run = run_for(input.model.placeholder.len(), look.placeholder);
            Some(window.text_system().shape_line(input.model.placeholder.clone(), font_size, &[run], None))
        } else {
            None
        };

        let wrap_width = bounds.size.width.max(px(1.0));
        let key = TextAreaLayoutKey {
            text: input.model.value.clone(),
            wrap_width,
            font: text_font.clone(),
            font_size,
            line_height,
            foreground: look.foreground,
            scale_factor_bits: scale_factor.to_bits(),
        };
        let lines = if let Some(cache) = input.layout_cache.as_ref().filter(|cache| cache.key.as_ref() == Some(&key)) {
            Arc::clone(&cache.lines)
        } else {
            let mut lines = Vec::new();
            let logical_lines = TextArea::logical_lines(input.model.value.as_ref());

            for (hard_start, _hard_end, text) in logical_lines.into_iter() {
                let full_run = run_for(text.len(), look.foreground);
                #[cfg(all(test, feature = "test-support"))]
                SHAPING_COUNTS.with(|counts| {
                    let (geometry, decoration) = counts.get();
                    counts.set((geometry + 1, decoration));
                });
                let full_shaped = window.text_system().shape_line(text.clone().into(), font_size, &[full_run], None);
                if text.is_empty() {
                    lines.push(TextAreaCachedLine {
                        start: hard_start,
                        end: hard_start,
                        text: String::new(),
                        line: full_shaped,
                    });
                    continue;
                }

                let byte_offsets = TextArea::char_byte_offsets(&text);
                let char_count = byte_offsets.len() - 1;
                // A wrap must not divide a shaped glyph cluster (ligatures/combining sequences).
                let mut glyph_boundaries: Vec<usize> = full_shaped
                    .runs
                    .iter()
                    .flat_map(|run| run.glyphs.iter().map(|glyph| glyph.index))
                    .chain([0, text.len()])
                    .collect();
                glyph_boundaries.sort_unstable();
                glyph_boundaries.dedup();
                let mut shaped_cursor = full_shaped.cursor();
                let mut local_start = 0usize;
                while local_start < char_count {
                    let start_x = full_shaped.x_for_index(byte_offsets[local_start]);
                    let mut fit_end = local_start + 1;
                    for (probe, _) in byte_offsets.iter().enumerate().take(char_count + 1).skip(local_start + 1) {
                        let probe_x = full_shaped.x_for_index(byte_offsets[probe]);
                        if probe_x - start_x <= wrap_width {
                            fit_end = probe;
                        } else {
                            break;
                        }
                    }

                    let mut local_end = fit_end;
                    if fit_end < char_count {
                        let fitting_text = &text[byte_offsets[local_start]..byte_offsets[fit_end]];
                        if let Some(distance) = fitting_text.chars().rev().position(char::is_whitespace) {
                            local_end = fit_end - distance;
                        }
                        local_end = local_end.max(local_start + 1);
                    }

                    let byte_start = byte_offsets[local_start];
                    let byte_end = byte_offsets[local_end];
                    if let Err(boundary) = glyph_boundaries.binary_search(&byte_end) {
                        let previous = glyph_boundaries[boundary.saturating_sub(1)];
                        let end = if previous > byte_start {
                            previous
                        } else {
                            glyph_boundaries[boundary]
                        };
                        // Glyph indices and UTF-8 character offsets share valid boundaries.
                        if let Ok(end) = byte_offsets.binary_search(&end) {
                            local_end = end;
                        }
                    }
                    let segment_text = text[byte_offsets[local_start]..byte_offsets[local_end]].to_owned();
                    // Preserve the original glyphs rather than shaping each segment again.
                    let shaped = shaped_cursor.take_until(byte_offsets[local_end]);

                    lines.push(TextAreaCachedLine {
                        start: hard_start + local_start,
                        end: hard_start + local_end,
                        text: segment_text,
                        line: shaped,
                    });

                    local_start = local_end;
                }
            }

            Arc::from(lines)
        };

        let cursor = input.state.cursor.min(lines.last().map_or(0, |line| line.end));
        let content_height = line_height * lines.len() as f32;
        let max_vertical_scroll = (content_height - bounds.size.height).max(px(0.0));
        let vertical_scroll = input.vertical_scroll.max(px(0.0)).min(max_vertical_scroll);

        let mut paint_lines = Vec::new();
        let previous_paint_lines = input
            .layout_cache
            .as_ref()
            .filter(|cache| Arc::ptr_eq(&cache.lines, &lines))
            .map(|cache| cache.paint_lines.as_slice())
            .unwrap_or_default();
        let visible = TextArea::visible_line_range(lines.len(), line_height, vertical_scroll, bounds.size.height);
        for (line_ix, line) in lines.iter().enumerate().take(visible.end).skip(visible.start) {
            let top = bounds.top() + line_height * line_ix as f32 - vertical_scroll;
            let bottom = top + line_height;

            if bottom >= bounds.top() && top <= bounds.bottom() {
                let start = line.start;
                let end = line.end;
                let local_range = |range: (usize, usize)| {
                    let start = range.0.max(line.start).min(line.end) - line.start;
                    let end = range.1.max(line.start).min(line.end) - line.start;
                    (start < end).then_some((start, end))
                };
                let decoration_key = TextAreaDecorationKey {
                    composition_underline_width: look.helper_geometry.composition_underline_width,
                    line_index: line_ix,
                    selection: selection.and_then(local_range),
                    marked_range: input
                        .marked_range
                        .as_ref()
                        .and_then(|range| local_range((range.start, range.end)))
                        .map(|(start, end)| start..end),
                    selection_foreground: look.selection_foreground,
                };
                let painted = if decoration_key.selection.is_none() && decoration_key.marked_range.is_none() {
                    line.line.clone()
                } else if let Some(cached) = previous_paint_lines.iter().find(|cached| cached.key == decoration_key) {
                    cached.line.clone()
                } else {
                    let runs = crate::controls::text::runs::colored_runs(
                        &line.text,
                        0,
                        decoration_key.selection,
                        decoration_key
                            .marked_range
                            .as_ref()
                            .map(|range| (range, decoration_key.composition_underline_width)),
                        text_font.clone(),
                        look.foreground,
                        look.selection_foreground,
                    );
                    #[cfg(all(test, feature = "test-support"))]
                    SHAPING_COUNTS.with(|counts| {
                        let (geometry, decoration) = counts.get();
                        counts.set((geometry, decoration + 1));
                    });
                    let mut painted = window.text_system().shape_line(line.text.clone().into(), font_size, &runs, None);
                    // Keep the exact glyph geometry used by wrapping and hit testing. Shaping
                    // a substring can otherwise change contextual forms or ligatures at wraps.
                    *painted = (*line.line).clone();
                    painted
                };
                paint_lines.push(TextAreaPaintLine { key: decoration_key, line: painted });

                if let Some((selection_start, selection_end)) = selection {
                    let local_start = selection_start.max(start).min(end).saturating_sub(start);
                    let local_end = selection_end.max(start).min(end).saturating_sub(start);
                    if local_start < local_end || (selection_start <= end && selection_end > end && end == start) {
                        let x1 = line.line.x_for_index(TextArea::char_to_byte_offset(&line.text, local_start));
                        let x2 = line.line.x_for_index(TextArea::char_to_byte_offset(&line.text, local_end));
                        selection_quads.push(fill(
                            Bounds::from_corners(
                                point(bounds.left() + x1, top),
                                point(bounds.left() + x2.max(x1), bottom),
                            ),
                            look.selection_background,
                        ));
                    }
                }

                if selection.is_none() && cursor >= start && cursor <= end {
                    let local_cursor = cursor.saturating_sub(start);
                    let x = line.line.x_for_index(TextArea::char_to_byte_offset(&line.text, local_cursor));
                    caret_quad = Some(fill(
                        Bounds::new(point(bounds.left() + x, top), size(px(look.caret_width), line_height)),
                        look.caret,
                    ));
                }
            }
        }

        TextAreaPrepaintState {
            key,
            lines,
            paint_lines,
            selection_quads,
            caret_quad,
            placeholder_line,
            line_height,
            vertical_scroll,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(&focus_handle, ElementInputHandler::new(bounds, self.input.clone()), cx);

        for selection in prepaint.selection_quads.drain(..) {
            window.paint_quad(selection);
        }

        if let Some(line) = prepaint.placeholder_line.take() {
            line.paint(bounds.origin, prepaint.line_height, TextAlign::Left, None, window, cx).ok();
        } else {
            for line in &prepaint.paint_lines {
                let line_ix = line.key.line_index;
                let origin = point(
                    bounds.left(),
                    bounds.top() + prepaint.line_height * line_ix as f32 - prepaint.vertical_scroll,
                );
                if origin.y + prepaint.line_height >= bounds.top() && origin.y <= bounds.bottom() {
                    line.line.paint(origin, prepaint.line_height, TextAlign::Left, None, window, cx).ok();
                }
            }
        }

        let focused = focus_handle.is_focused(window);
        if focused && let Some(caret) = prepaint.caret_quad.take() {
            window.paint_quad(caret);
        }

        self.input.update(cx, |input, cx| {
            input.vertical_scroll = prepaint.vertical_scroll;
            input.layout_cache = Some(TextAreaLayoutCache {
                key: Some(prepaint.key.clone()),
                text_viewport: bounds,
                line_height: prepaint.line_height,
                lines: prepaint.lines.clone(),
                paint_lines: std::mem::take(&mut prepaint.paint_lines),
            });
            input.clamp_vertical_scroll_to_cache();
            input.sync_scrollbar(cx);
        });
    }
}
