//! Local checks for the conditions behind gpui-luma-dev/xgpui#332: plain GPUI
//! text stays on flat lines, and apps use `typeset()` or the paragraph API for
//! wrapped right-to-left text.
//!
//! A failure here means that decision needs another look, not necessarily that
//! something broke:
//! - `plain_text_wraps_rtl_in_painting_order_*`: plain GPUI text (or upstream)
//!   changed how right-to-left lines wrap.
//! - `typeset_covers_styled_text_api`: `StyledText`, `TextStyle` or
//!   `HighlightStyle` changed, so `typeset()` may have fallen behind.
//! - `bidi_page_wrapped_section_*`: the bidi lab's section 5 no longer shows the
//!   difference it documents.
use super::{examples as bidi, ParagraphApiPage};
use crate::parley_lab::{paragraph, samples};
use gpui::{
    FontRun, ParagraphPosition, PlatformTextSystem, StyledText, TestAppContext, TestDispatcher, TextAffinity,
    VisualTestContext, div, point, prelude::*, px, size,
};
use std::{
    sync::{Arc, Mutex},
};
use xgpui_parley::ParleyTextSystem;

const FAMILY: &str = "Noto Sans Hebrew";

fn parley() -> Arc<ParleyTextSystem> {
    let mut fonts = parley::FontContext::new();
    for data in std::iter::once(samples::PLEX).chain(paragraph::FONTS) {
        fonts.collection.register_fonts(parley::fontique::Blob::new(Arc::new(data.to_vec())), None);
    }
    Arc::new(ParleyTextSystem::new(fonts, "IBM Plex Sans").unwrap())
}

#[cfg(target_os = "macos")]
fn native() -> Arc<dyn PlatformTextSystem> {
    let backend: Arc<dyn PlatformTextSystem> = Arc::new(gpui_apple::AppleTextSystem::new());
    backend
        .add_fonts(std::iter::once(samples::PLEX).chain(paragraph::FONTS).map(std::borrow::Cow::Borrowed).collect())
        .unwrap();
    backend
}

/// Plain GPUI text breaks right-to-left lines on the visual (painted) glyph
/// order, so its wrap boundaries run backwards through the text and the
/// sentence's first words land on the last line. The paragraph API, which
/// `typeset()` uses, breaks in logical order.
fn wrap_order_checks(backend: Arc<dyn PlatformTextSystem>) {
    let text = bidi::WRAPPED;
    let mut cx = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend);
    let window = cx.add_empty_window();
    let styled = StyledText::new(text);
    let layout = styled.layout().clone();
    let element = Mutex::new(Some(styled));
    window.draw(point(px(0.), px(0.)), size(px(600.), px(600.)), |_, _| {
        div()
            .w(bidi::WRAP_WIDTH)
            .font_family(FAMILY)
            .text_size(px(15.))
            .child(element.lock().unwrap().take().unwrap())
    });

    // Plain text: byte offsets of the soft-wrap boundaries, in line order.
    // (`TextLayout::wrapped_text` slices with these and panics on them.)
    let lines = layout.line_layouts();
    assert_eq!(lines.len(), 1, "one hard line");
    let line = &lines[0];
    let boundaries: Vec<usize> = line
        .wrap_boundaries
        .iter()
        .map(|b| line.unwrapped_layout.runs[b.run_ix].glyphs[b.glyph_ix].index)
        .collect();
    assert!(boundaries.len() >= 2, "the sample must wrap to at least three lines: {boundaries:?}");
    assert!(
        boundaries.windows(2).all(|pair| pair[0] > pair[1]),
        "plain GPUI text now wraps right-to-left text in another order \
         (boundaries {boundaries:?}); revisit gpui-luma-dev/xgpui#332"
    );

    // Paragraph API: logical order, first character at the top right.
    window.update(|window, _| {
        let system = window.text_system();
        let runs = [FontRun { len: text.len(), font_id: system.resolve_font(&gpui::font(FAMILY)) }];
        let paragraph = system.layout_paragraph(text.into(), px(15.), &runs, bidi::WRAP_WIDTH).unwrap();
        let caret = |byte_index| {
            paragraph
                .caret_bounds(ParagraphPosition { byte_index, affinity: TextAffinity::Downstream })
                .unwrap()
                .origin
        };
        let first = caret(0);
        assert_eq!(first.y, px(0.), "paragraph starts on the first line");
        assert!(first.x > bidi::WRAP_WIDTH / 2., "and at the right edge: {first:?}");
        let words: Vec<_> = std::iter::once(0)
            .chain(text.match_indices(' ').map(|(ix, _)| ix + 1))
            .map(|ix| caret(ix).y)
            .collect();
        assert!(words.windows(2).all(|pair| pair[0] <= pair[1]), "paragraph lines follow logical order: {words:?}");
        assert!(words.last().unwrap() > &px(0.), "paragraph wraps");
    });
}

#[test]
fn plain_text_wraps_rtl_in_painting_order_parley() {
    wrap_order_checks(parley());
}

#[cfg(target_os = "macos")]
#[test]
fn plain_text_wraps_rtl_in_painting_order_native() {
    wrap_order_checks(native());
}

/// Compile against published GPUI rather than reading a sibling checkout.
/// `typeset` deliberately has no StyledText layout or font-family-override
/// accessors; use paragraph geometry and per-run fonts for those gaps.
#[test]
fn typeset_covers_styled_text_api() {
    // Exhaustive patterns make new style fields a compile failure, preserving
    // upstream's API-drift guard without paths into the dependency's sources.
    let gpui::TextStyle {
        color: _,
        font_family: _,
        font_features: _,
        font_variations: _,
        font_fallbacks: _,
        font_size: _,
        line_height: _,
        font_weight: _,
        font_style: _,
        background_color: _,
        underline: _,
        strikethrough: _,
        white_space: _,
        text_overflow: _,
        text_align: _,
        line_clamp: _,
    } = gpui::TextStyle::default();
    let style = gpui::TextStyle::default();
    let highlight = gpui::HighlightStyle {
        color: Some(gpui::rgb(0xff0000).into()),
        font_weight: Some(gpui::FontWeight::BOLD),
        font_style: Some(gpui::FontStyle::Italic),
        background_color: Some(gpui::rgb(0x00ff00).into()),
        underline: Some(gpui::UnderlineStyle { thickness: px(1.), color: None, wavy: false }),
        strikethrough: Some(gpui::StrikethroughStyle { thickness: px(1.), color: None }),
        fade_out: Some(0.5),
    };
    let run = gpui::TextRun {
        len: bidi::WRAPPED.len(),
        font: gpui::font(FAMILY),
        color: style.color,
        background_color: highlight.background_color,
        underline: highlight.underline,
        strikethrough: highlight.strikethrough,
    };
    let value = xgpui_parley::typeset(bidi::WRAPPED)
        .with_runs(vec![run.clone()])
        .with_highlights([(0..bidi::WRAPPED.len(), highlight)]);
    assert_eq!(value.text().as_ref(), bidi::WRAPPED);
    let plain = StyledText::new(bidi::WRAPPED)
        .with_default_highlights(&style, [])
        .with_font_family_overrides([(0..bidi::WRAPPED.len(), FAMILY.into())]);
    let _ = plain.layout();
    let _ = StyledText::new(bidi::WRAPPED).with_highlights([]).with_runs(vec![run]);
}

/// Draws the bidi lab and returns section 5's frame sizes.
fn wrapped_section(
    backend: Arc<dyn PlatformTextSystem>,
    parley: Option<Arc<ParleyTextSystem>>,
) -> impl Fn(&str) -> gpui::Size<gpui::Pixels> {
    let mut cx = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend);
    cx.update(|cx| xgpui_luma::init(cx).unwrap());
    if let Some(backend) = parley {
        cx.update(|cx| super::install_typeset(backend, cx));
    }
    let window =
        cx.open_window(size(px(1100.), px(3000.)), |_, cx| ParagraphApiPage::new(crate::theme::default_look(), cx));
    let lab = window.root(&mut cx).unwrap();
    let mut visual = VisualTestContext::from_window(window.into(), &cx);
    visual.draw(point(px(0.), px(0.)), size(px(1100.), px(3000.)), |_, _| div().size_full().child(lab.clone()));
    let sizes: Vec<_> = ["rtl-wrap-plain", "rtl-wrap-typeset", "rtl-clamp-plain", "rtl-clamp-typeset"]
        .into_iter()
        .map(|selector| {
            let bounds = visual.debug_bounds(selector).unwrap_or_else(|| panic!("{selector} not drawn"));
            (selector, bounds.size)
        })
        .collect();
    move |selector| sizes.iter().find(|(name, _)| *name == selector).unwrap().1
}

#[test]
fn bidi_page_wrapped_section_parley() {
    let parley = parley();
    let size = wrapped_section(parley.clone(), Some(parley));
    for kind in ["plain", "typeset"] {
        let wrap = size(&format!("rtl-wrap-{kind}"));
        let clamp = size(&format!("rtl-clamp-{kind}"));
        assert_eq!(wrap.width, bidi::WRAP_WIDTH, "{kind}");
        assert!(clamp.height < wrap.height, "{kind}: clamp is shorter");
    }
    // Both wrap the same words into the same number of lines here; only the
    // line order differs, which the wrap-order test checks.
    assert!(size("rtl-wrap-typeset").height > size("rtl-clamp-typeset").height);
}

#[cfg(target_os = "macos")]
#[test]
fn bidi_page_wrapped_section_native() {
    // Without a Parley engine, typeset() renders as plain GPUI text.
    let size = wrapped_section(native(), None);
    for id in ["wrap", "clamp"] {
        assert_eq!(size(&format!("rtl-{id}-plain")), size(&format!("rtl-{id}-typeset")), "{id}");
    }
}
