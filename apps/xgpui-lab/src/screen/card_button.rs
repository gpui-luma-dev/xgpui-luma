//! Selectable card: a Luma button drawn as a card. It keeps the button's input,
//! keyboard focus and accessibility. Selected, hover, press and focus fills come
//! from the look's interaction colors.
use std::sync::Arc;

use gpui::{App, Context, Entity, InteractiveElement, IntoElement, Pixels, SharedString, Styled, prelude::FluentBuilder};
use xgpui_luma::{
    controls::button::{Button, DefaultButtonTemplate},
    prelude::HasPresenter,
};
use xgpui_luma_look_radix::{self as radix, Look};

use super::card_surface;
use crate::theme::ColorVizLookExt;

/// How a selectable card is sized in its row.
#[derive(Clone, Copy)]
pub(crate) enum CardWidth {
    /// Fills its host; `LabScreen::card` sizes the host like a `LabCard`.
    Flexible,
    /// A fixed width that may shrink to fit (Italy's photo cards).
    Fixed(Pixels),
}

/// Spawn a selectable card. `content` builds the card's inside on every render, usually
/// `LabCard::…​.contents(look)`; `selected` reports whether this card is the current one.
/// The button id doubles as its debug selector.
pub(crate) fn card_button<M: 'static, E: IntoElement + 'static>(
    look: &Arc<Look>,
    id: impl Into<SharedString>,
    width: CardWidth,
    selected: impl Fn() -> bool + Send + Sync + 'static,
    content: impl Fn(&mut App) -> E + Send + Sync + 'static,
    cx: &mut Context<M>,
    on_click: impl Fn(&mut M, &mut Context<M>) + 'static,
) -> Entity<Button> {
    let id: SharedString = id.into();
    let geometry = radix::button_look_for::<()>(
        look,
        radix::ButtonVariant::Ghost,
        radix::Paint::gray(),
        radix::ButtonSize::One,
        Default::default(),
    );
    let button = Button::new(id.clone())
        .with_look(move |model| {
            let mut resolved = geometry(model);
            // Selection and keyboard focus use the card fill, without an outer ring.
            resolved.border = None;
            resolved
        })
        .content(move |_, cx| content(cx))
        .on_click(cx, on_click)
        .spawn(cx);
    let paint_look = look.clone();
    let selector = id.to_string();
    let template = DefaultButtonTemplate::new(radix::button_family_theme_with(
        look,
        radix::ButtonVariant::Ghost,
        radix::Paint::gray(),
    ))
    .with_modifier(move |row, model| {
        let colors = paint_look.interaction_colors();
        let selector = selector.clone();
        let row = card_surface(row, &paint_look);
        let row = match width {
            CardWidth::Fixed(width) => row.w(width).max_w_full().min_w_0(),
            CardWidth::Flexible => row.w_full().justify_start(),
        };
        row.debug_selector(move || selector)
            .h_auto()
            .when_some(colors.background(selected(), model.state), |row, color| row.bg(color))
    });
    button.update(cx, |button, cx| button.set_template(Arc::new(template), cx));
    button
}
