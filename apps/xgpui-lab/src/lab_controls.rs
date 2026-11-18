//! Labeled, directly selectable lab settings. Numeric IDs stay independent of display labels.
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
use gpui::{prelude::*, Context, Entity, SharedString, div};
use xgpui_luma::controls::control_group::ControlGroupItemLike;
use xgpui_luma::controls::selector::{Selector, SelectorItem};
use xgpui_luma_look_radix::{Look, Selector as RadixSelector};

#[derive(Clone)]
pub(crate) struct Choice {
    pub selector: Entity<Selector>,
    label: SharedString,
    labels: Rc<RefCell<Vec<String>>>,
    display: Arc<Mutex<Vec<crate::l10n::UiCopy>>>,
}

pub(crate) fn labels(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn items(labels: &[String]) -> Vec<SelectorItem> {
    labels
        .iter()
        .enumerate()
        .map(|(index, label)| SelectorItem::new(index.to_string()).label(label.clone()))
        .collect()
}

impl Choice {
    pub fn new<M: 'static>(
        id: impl Into<SharedString>,
        label: &str,
        look: &Look,
        options: Vec<String>,
        selected: usize,
        cx: &mut Context<M>,
        on_change: impl Fn(&mut M, usize, &mut Context<M>) + 'static,
    ) -> Self {
        let labels = Rc::new(RefCell::new(options));
        let callback_labels = labels.clone();
        let display: Arc<Mutex<Vec<crate::l10n::UiCopy>>> =
            Arc::new(Mutex::new(labels.borrow().iter().cloned().map(Into::into).collect()));
        let render_labels = display.clone();
        let selector = crate::l10n::localize_selector_with(
            RadixSelector::new(id)
                .look(look)
                .enabled(!labels.borrow().is_empty())
                .items(items(&labels.borrow()))
                .selected_id(selected.to_string())
                .on_value_changed(cx, move |owner, id, _, cx| {
                    let count = callback_labels.borrow().len();
                    if let Ok(index) = id.parse::<usize>()
                        && index < count
                    {
                        on_change(owner, index, cx);
                    }
                })
                .spawn(cx),
            Arc::new(move |item, cx| {
                item.id()
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| render_labels.lock().ok().and_then(|labels| labels.get(index).cloned()))
                    .map_or_else(|| crate::l10n::resolve(item.label_text(), cx), |label| label.resolve(cx))
            }),
            cx,
        );
        Self { selector, label: label.to_owned().into(), labels, display }
    }

    /// The label, for layouts that place it apart from the selector (e.g. `screen::form_group`).
    pub fn label(&self) -> SharedString {
        self.label.clone()
    }

    pub fn sync_indexed(&self, options: Vec<String>, kind: &'static str, selected: usize, cx: &mut gpui::App) {
        self.sync_messages(
            options
                .into_iter()
                .enumerate()
                .map(|(i, source)| {
                    let number = crate::l10n::UiCopy::numeric((i + 1).to_string());
                    (
                        source,
                        crate::l10n::UiCopy::compose(
                            |a| crate::l10n::messages::msg_lab_indexed(a[0].clone(), a[1].clone()),
                            vec![crate::l10n::UiCopy::raw(kind), number],
                        ),
                    )
                })
                .collect(),
            selected,
            cx,
        );
    }

    pub fn sync(&self, options: Vec<String>, selected: usize, cx: &mut gpui::App) {
        self.sync_messages(options.into_iter().map(|source| (source.clone(), source.into())).collect(), selected, cx);
    }

    /// Source labels remain event data; retained display messages resolve at render.
    pub fn sync_messages(&self, options: Vec<(String, crate::l10n::UiCopy)>, selected: usize, cx: &mut gpui::App) {
        let (options, display): (Vec<_>, Vec<_>) = options.into_iter().unzip();
        // Display messages can change while source labels stay the same
        // (e.g. a count inside a label); the selector must repaint then too.
        let relabeled = self.display.lock().is_ok_and(|mut labels| {
            let relabeled = labels.len() != display.len()
                || labels.iter().zip(&display).any(|(old, new)| old.resolve(cx) != new.resolve(cx));
            *labels = display;
            relabeled
        });
        let changed = *self.labels.borrow() != options;
        if changed {
            *self.labels.borrow_mut() = options;
        }
        let id = (selected < self.labels.borrow().len()).then(|| selected.to_string());
        if changed || relabeled || self.selector.read(cx).selected_id().map(|id| id.as_ref()) != id.as_deref() {
            self.selector.update(cx, |selector, cx| {
                if relabeled {
                    cx.notify();
                }
                if changed {
                    selector.set_items(items(&self.labels.borrow()), cx);
                    selector.set_enabled(!self.labels.borrow().is_empty(), cx);
                }
                if let Some(id) = id {
                    selector.set_selected_id(id, cx);
                }
            });
        }
    }

    #[cfg(test)]
    pub fn select(&self, index: usize, cx: &mut gpui::App) {
        let label = self.labels.borrow()[index].clone();
        self.selector.update(cx, |selector, cx| {
            selector.set_selected_id(index.to_string(), cx);
            cx.emit(xgpui_luma::controls::selector::SelectorEvent::Change {
                item_id: index.to_string().into(),
                label: label.into(),
            });
        });
    }
}

impl IntoElement for Choice {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        crate::l10n::row(div().flex().items_center().gap_2().child(crate::l10n::ui(self.label)).child(self.selector))
            .into_any_element()
    }
}

/// A labeled toggle group: every option visible as one segment, one selected.
/// Use it for short option lists (two to five); `Choice` is the dropdown for longer ones.
#[derive(Clone)]
pub(crate) struct Segmented {
    pub group: xgpui_luma::controls::radio_group::RadioGroup<xgpui_luma::controls::radio_group::RadioGroupItem>,
    label: SharedString,
}

impl Segmented {
    pub fn new<M: 'static>(
        id: impl Into<SharedString>,
        label: &str,
        look: &Look,
        options: &[&str],
        selected: usize,
        cx: &mut Context<M>,
        on_change: impl Fn(&mut M, usize, &mut Context<M>) + 'static,
    ) -> Self {
        use xgpui_luma::controls::radio_group::{self, RadioGroupItem};
        let count = options.len();
        let accessible_label = SharedString::from(label.to_owned());
        let base = xgpui_luma_look_radix::segmented_radio_template(look);
        let template: xgpui_luma::controls::radio_group::RadioGroupTemplate<RadioGroupItem> =
            std::sync::Arc::new(move |model, handlers, window, cx| {
                base(model, handlers, window, cx).aria_label(crate::l10n::resolve(&accessible_label, cx))
            });
        let group = radio_group::horizontal(id)
            .single_required()
            .selection_follows_active(true)
            .items(
                options
                    .iter()
                    .enumerate()
                    .map(|(index, label)| RadioGroupItem::new(index.to_string()).label(*label)),
            )
            .selected(selected.to_string())
            .template(template)
            .item_template(std::sync::Arc::new(|item, _, _| {
                crate::l10n::ui(item.item.label().clone()).into_any_element()
            }))
            .on_value_changed(cx, move |owner, value, cx| {
                if let Some(index) = value.as_ref().and_then(|id| id.parse::<usize>().ok())
                    && index < count
                {
                    on_change(owner, index, cx);
                }
            })
            .spawn(cx);
        Self { group, label: label.to_owned().into() }
    }

    pub fn set_enabled(&self, enabled: bool, cx: &mut gpui::App) {
        self.group.update(cx, |group, cx| group.set_enabled(enabled, cx));
    }

    #[cfg(all(test, feature = "test-support"))]
    pub fn select(&self, index: usize, cx: &mut gpui::App) {
        self.group.update(cx, |group, cx| {
            group.set_selected(index.to_string(), cx);
            cx.emit(xgpui_luma::controls::radio_group::RadioGroupEvent::Change {
                value: Some(index.to_string().into()),
            });
        });
    }
}

impl IntoElement for Segmented {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        crate::l10n::row(div().flex().items_center().gap_2().child(crate::l10n::ui(self.label)).child(self.group))
            .into_any_element()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{TestAppContext, Window};

    struct Owner {
        choice: Option<Choice>,
    }
    impl Render for Owner {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().children(self.choice.as_ref().map(|choice| choice.selector.clone()))
        }
    }

    /// #264: a display message can change while the source labels and the
    /// selection stay the same; the selector must still repaint.
    #[test]
    fn choice_repaints_when_only_display_messages_change() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (owner, cx) = app.add_window_view(|_, cx| {
            let choice = Choice::new(
                "choice-relabel",
                "Choice",
                &crate::theme::default_look(),
                labels(&["a"]),
                0,
                cx,
                |_: &mut Owner, _, _| {},
            );
            Owner { choice: Some(choice) }
        });
        let choice = owner.read_with(cx, |owner, _| owner.choice.clone().unwrap());
        let repaints = Rc::new(RefCell::new(0));
        let count = repaints.clone();
        let _subscription = cx.update(|_, cx| cx.observe(&choice.selector, move |_, _| *count.borrow_mut() += 1));
        let relabel = |text: &'static str, cx: &mut gpui::App| {
            choice.sync_messages(vec![("a".to_owned(), crate::l10n::UiCopy::raw(text))], 0, cx)
        };
        cx.update(|_, cx| relabel("first", cx));
        let before = *repaints.borrow();
        cx.update(|_, cx| relabel("first", cx));
        assert_eq!(*repaints.borrow(), before, "unchanged labels must not repaint");
        cx.update(|_, cx| relabel("second", cx));
        assert!(*repaints.borrow() > before, "a changed display message must repaint");
    }
}
