//! First localized Lab page and app startup resources. Shell-wide extraction and
//! SDK mirroring follow the lab-first text/direction prototype (#220).
use std::sync::Arc;
use anyhow::{Result, anyhow};
use gpui::{App, Context, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma_l10n::{
    LayoutDirection, Locale, Localizer, Message,
    formatting::{DateStyle, ListKind},
};
use xgpui_luma_look_radix::Look;
use crate::{
    lab_controls::Choice,
    screen::{
        Hero, LabScreen, TextRole,
        localized::{self, TextKind},
    },
};

mod messages {
    include!(concat!(env!("OUT_DIR"), "/lab_messages.rs"));
}

const RESOURCES: &[(&str, &str)] = &[
    ("en", include_str!("../locales/en/lab.ftl")),
    ("de", include_str!("../locales/de/lab.ftl")),
    ("ja", include_str!("../locales/ja/lab.ftl")),
    ("ar", include_str!("../locales/ar/lab.ftl")),
];

pub(crate) fn localizer(locale: Locale) -> Result<Localizer> {
    let mut localizer = Localizer::new(locale);
    for (tag, source) in RESOURCES {
        localizer.add_resource(&tag.parse()?, source)?;
    }
    Ok(localizer)
}

fn tr(cx: &App, message: Message<'_>) -> SharedString {
    xgpui_luma_l10n::L10nExt::tr_message(cx, message)
}

fn choices(cx: &App) -> Vec<(String, String)> {
    let mut choices: Vec<_> = [
        ("en", "English"),
        ("de", "Deutsch"),
        ("ja", "日本語"),
        ("ar", "العربية"),
        ("ar-u-nu-arab", "العربية · ٠١٢"),
    ]
    .into_iter()
    .map(|(tag, label)| (tag.to_owned(), format!("{label} ({tag})")))
    .collect();
    if let Some(locale) = cx.try_global::<Localizer>().map(Localizer::locale) {
        let tag = locale.to_string();
        if !choices.iter().any(|(id, _)| id == &tag) {
            choices.push((tag.clone(), tag));
        }
    }
    choices
}

fn selected(cx: &App, choices: &[(String, String)]) -> usize {
    let tag = cx.try_global::<Localizer>().map(|localizer| localizer.locale().to_string());
    choices.iter().position(|(id, _)| Some(id) == tag.as_ref()).unwrap_or(0)
}

struct Samples {
    greeting: SharedString,
    saved: SharedString,
    items: SharedString,
    rows: Vec<(SharedString, SharedString)>,
    rtl: bool,
}

fn samples(cx: &App) -> Result<Samples> {
    let localizer = cx.try_global::<Localizer>().ok_or_else(|| anyhow!("Lab localizer is not installed"))?;
    let locale = localizer.locale();
    let rtl = locale.direction() == LayoutDirection::Rtl;
    let number = locale.fmt_number(&3.into())?;
    Ok(Samples {
        greeting: tr(cx, messages::msg_l10n_greeting()),
        saved: tr(cx, messages::msg_l10n_saved("שלום.txt")),
        items: tr(cx, messages::msg_l10n_items(3, number)),
        rows: vec![
            (
                tr(cx, messages::msg_l10n_direction()),
                tr(
                    cx,
                    if rtl {
                        messages::msg_l10n_rtl()
                    } else {
                        messages::msg_l10n_ltr()
                    },
                ),
            ),
            (tr(cx, messages::msg_l10n_number()), locale.fmt_number(&"1234567.50".parse()?)?.into()),
            (tr(cx, messages::msg_l10n_date()), locale.fmt_date(2026, 11, 16, DateStyle::Medium)?.into()),
            (
                tr(cx, messages::msg_l10n_list()),
                locale.fmt_list(&["Alpha", "日本語", "العربية"], ListKind::And)?.into(),
            ),
        ],
        rtl,
    })
}

pub(crate) struct Page {
    look: Arc<Look>,
    locale: Choice,
    startup_error: Option<String>,
}

impl Page {
    pub(crate) fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        // Direct headless page construction uses English; normal startup installs
        // CLI/OS preferences before opening any views.
        let startup_error = if cx.try_global::<Localizer>().is_none() {
            let result = (|| -> Result<()> {
                cx.set_global(localizer("en".parse()?)?);
                Ok(())
            })();
            result.err().map(|error| error.to_string())
        } else {
            None
        };
        let options = choices(cx);
        let locale = Choice::new(
            "l10n-locale",
            "",
            &look,
            options.iter().map(|(_, label)| label.clone()).collect(),
            selected(cx, &options),
            cx,
            |_: &mut Self, index, cx| {
                if let Some((tag, _)) = choices(cx).get(index) {
                    match tag.parse() {
                        Ok(locale) => {
                            xgpui_luma_l10n::set_locale(locale, cx);
                        }
                        Err(error) => log::error!("Lab locale: {error}"),
                    }
                }
                cx.notify();
            },
        );
        Self { look, locale, startup_error }
    }
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let options = choices(cx);
        self.locale
            .sync(options.iter().map(|(_, label)| label.clone()).collect(), selected(cx, &options), cx);
        let screen = LabScreen::new("l10n", self.look.clone())
            .localized()
            .crumb("xgpui-parley", tr(cx, messages::msg_l10n_title()).as_ref())
            .title(tr(cx, messages::msg_l10n_title()))
            .lede(tr(cx, messages::msg_l10n_description()));
        let sample = match samples(cx) {
            Ok(sample) => sample,
            Err(error) => return screen.notice(Some(error.to_string())).into_any_element(),
        };
        let label = |role, content: SharedString, kind| localized::text(&self.look, role, content, kind, cx);
        let preview = div()
            .flex()
            .flex_col()
            .gap_4()
            .child(label(TextRole::Section, tr(cx, messages::msg_l10n_preview()), TextKind::Ui))
            .child(label(TextRole::Name, sample.greeting, TextKind::Ui))
            .child(label(TextRole::Detail, sample.saved, TextKind::Ui))
            .child(label(TextRole::Detail, sample.items, TextKind::Ui))
            .children(sample.rows.into_iter().enumerate().map(|(index, (key, value))| {
                div()
                    .debug_selector(move || format!("l10n-row-{index}"))
                    .flex()
                    .w_full()
                    .gap_4()
                    .when(sample.rtl, |row| row.flex_row_reverse())
                    .child(
                        label(TextRole::Label, key, TextKind::Ui)
                            .w(px(180.))
                            .flex_shrink_0()
                            .debug_selector(move || format!("l10n-key-{index}")),
                    )
                    .child(
                        label(TextRole::Value, value, TextKind::Data)
                            .flex_1()
                            .debug_selector(move || format!("l10n-value-{index}")),
                    )
            }))
            .child(label(TextRole::Label, tr(cx, messages::msg_l10n_data()), TextKind::Ui))
            .child(label(TextRole::Detail, "שלום · مرحبا · example.txt".into(), TextKind::Data));
        let control = div()
            .flex()
            .items_center()
            .gap_2()
            .when(sample.rtl, |row| row.flex_row_reverse())
            .child(label(TextRole::Field, tr(cx, messages::msg_l10n_locale()), TextKind::Ui))
            .child(self.locale.selector.clone());
        screen
            .hero(Hero::full(preview))
            .control(control)
            .note(tr(cx, messages::msg_l10n_native_note()))
            .notice(self.startup_error.clone())
            .into_any_element()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests;
