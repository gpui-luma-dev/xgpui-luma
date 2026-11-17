//! App localization: generated resources, shell locale selection, and backend demonstrations.
use std::sync::Arc;
use anyhow::{Result, anyhow};
use gpui::{App, Context, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma_l10n::{
    LayoutDirection, Locale, Localizer, Message,
    formatting::{DateStyle, ListKind, ByteUnits, DurationStyle},
};
use xgpui_luma_look_radix::Look;
use crate::{
    lab_controls::Choice,
    screen::{
        Hero, LabScreen, TextRole,
        localized::{self, TextKind},
    },
};

#[allow(clippy::too_many_arguments)] // Generated constructors mirror the resource argument schema.
pub(crate) mod messages {
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
    for (tag, lab, sdk) in [
        (
            "en-XA",
            include_str!(concat!(env!("OUT_DIR"), "/en-XA-lab.ftl")),
            include_str!(concat!(env!("OUT_DIR"), "/en-XA-sdk.ftl")),
        ),
        (
            "ar-XB",
            include_str!(concat!(env!("OUT_DIR"), "/ar-XB-lab.ftl")),
            include_str!(concat!(env!("OUT_DIR"), "/ar-XB-sdk.ftl")),
        ),
    ] {
        localizer.add_resource(&tag.parse()?, lab)?;
        localizer.add_resource(&tag.parse()?, sdk)?;
    }
    Ok(localizer)
}

fn tr(cx: &App, message: Message<'_>) -> SharedString {
    xgpui_luma_l10n::L10nExt::tr_message(cx, message)
}

pub(crate) fn choices(cx: &App) -> Vec<(String, String)> {
    let mut choices: Vec<_> = [
        ("en", "English"),
        ("de", "Deutsch"),
        ("ja", "日本語"),
        ("ar", "العربية"),
        ("ar-u-nu-arab", "العربية · ٠١٢"),
        ("en-XA", "Pseudo · expanded"),
        ("ar-XB", "Pseudo · RTL"),
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

pub(crate) fn selected(cx: &App, choices: &[(String, String)]) -> usize {
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
            (tr(cx, messages::msg_l10n_time()), locale.fmt_time(14, 35, 9)?.into()),
            (tr(cx, messages::msg_l10n_percent()), locale.fmt_percent(0.125, 1)?.into()),
            (tr(cx, messages::msg_l10n_bytes()), locale.fmt_bytes(1_572_864, ByteUnits::Binary, 1)?.into()),
            (
                tr(cx, messages::msg_l10n_duration()),
                locale.fmt_duration(std::time::Duration::from_secs(3661), DurationStyle::Long)?.into(),
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
                xgpui_luma_l10n::install(localizer("en".parse()?)?, cx)?;
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
        let comparison_text = sample.saved.clone();
        let comparison = div()
            .flex()
            .flex_wrap()
            .gap_4()
            .when(sample.rtl, |row| row.flex_row_reverse())
            .child(
                div()
                    .debug_selector(|| "l10n-native-column".into())
                    .flex_1()
                    .min_w(px(260.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(label(TextRole::Section, tr(cx, messages::msg_l10n_native_label()), TextKind::Ui))
                    .child(
                        TextRole::Detail
                            .apply(&self.look, div())
                            .when(sample.rtl, |text| text.text_right())
                            .child(comparison_text.clone()),
                    ),
            )
            .child(
                div()
                    .debug_selector(|| "l10n-parley-column".into())
                    .flex_1()
                    .min_w(px(260.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(label(TextRole::Section, tr(cx, messages::msg_l10n_parley_label()), TextKind::Ui))
                    .child(label(TextRole::Detail, comparison_text, TextKind::Ui))
                    .when(!crate::parley_lab::paragraph_api::typeset_active(cx), |column| {
                        column.child(label(
                            TextRole::Caption,
                            tr(cx, messages::msg_l10n_parley_inactive()),
                            TextKind::Ui,
                        ))
                    }),
            );
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
            .section(comparison)
            .control(control)
            .note(tr(cx, messages::msg_l10n_native_note()))
            .notice(self.startup_error.clone())
            .into_any_element()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests;

mod copy;
pub(crate) use copy::{ui, data, label, resolve, number, row};

pub(crate) fn choose_locale(index: usize, cx: &mut App) {
    let Some((tag, _)) = choices(cx).get(index).cloned() else {
        return;
    };
    match tag.parse() {
        Ok(locale) if cx.try_global::<Localizer>().is_some() => {
            xgpui_luma_l10n::set_locale(locale, cx);
        }
        Ok(locale) => {
            match localizer(locale).and_then(|localizer| xgpui_luma_l10n::install(localizer, cx).map_err(Into::into)) {
                Ok(()) => {}
                Err(error) => log::error!("Lab locale: {error}"),
            }
        }
        Err(error) => log::error!("Lab locale: {error}"),
    }
}

mod controls;
pub(crate) use controls::{
    localize_selector, localize_popup, localize_context, localize_field, localize_tabs, localize_sidebar, name_button,
};

pub(crate) fn tooltip(source: impl Into<SharedString>) -> xgpui_luma::controls::tooltip::Tooltip {
    let source = source.into();
    xgpui_luma::controls::tooltip::Tooltip::new(source.clone()).text_provider(move |cx| resolve(&source, cx))
}
