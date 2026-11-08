//! App-local task-list content; completion styling is not a checkbox theme rule.

use gpui::{Context, Hsla, IntoElement, StyledText, HighlightStyle, StrikethroughStyle, div, prelude::*, px};
use xgpui_luma::controls::checkbox::Checkbox;
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_radix::{self as radix, Look, ScaleFamily};

#[derive(Clone)]
pub struct TaskSamples {
    tasks: Vec<Checkbox>,
}

impl TaskSamples {
    pub fn spawn<T: 'static>(look: &Look, cx: &mut Context<T>) -> Self {
        let tasks = [
            ("Respond to comment #384 from Travis", Some("#384"), false),
            ("Invite Acme Co. team to Slack", None, true),
            ("Create a report requested by Danilo", Some("requested"), false),
            ("Close Q2 finances", None, true),
            ("Review invoice #3456", None, true),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (text, accent, checked))| {
            let label_look = look.clone();
            radix::Checkbox::new(format!("preview-task-{index}"))
                .look(look)
                .surface()
                .checked(checked)
                .content(move |model, _| {
                    let completed = model.data.checked;
                    let body = label_look.resolve_step(ScaleFamily::Gray, if completed { 11 } else { 12 }).hsla();
                    let highlights = if completed {
                        vec![(
                            0..text.len(),
                            HighlightStyle {
                                strikethrough: Some(StrikethroughStyle { thickness: px(1.0), color: Some(body) }),
                                ..Default::default()
                            },
                        )]
                    } else {
                        accent
                            .and_then(|phrase| {
                                text.find(phrase).map(|start| {
                                    (
                                        start..start + phrase.len(),
                                        HighlightStyle {
                                            color: Some(label_look.resolve_step(ScaleFamily::Color, 11).hsla()),
                                            ..Default::default()
                                        },
                                    )
                                })
                            })
                            .into_iter()
                            .collect()
                    };
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::NORMAL)
                        .text_color(body)
                        .child(StyledText::new(text).with_highlights(highlights))
                })
                .spawn(cx)
        })
        .collect();
        Self { tasks }
    }

    pub fn notify<T: 'static>(&self, cx: &mut Context<T>) {
        for task in &self.tasks {
            task.update(cx, |_, cx| cx.notify());
        }
    }

    pub fn render(self, surface: Hsla) -> impl IntoElement {
        div()
            .w_full()
            .rounded(px(12.0))
            .bg(surface)
            .p(px(16.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .children(self.tasks)
    }
}
