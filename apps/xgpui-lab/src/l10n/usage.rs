//! Test-only observation of rendered messages; catalog membership is not reachability.
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};
use gpui::{App, Global};
use xgpui_luma_l10n::Message;

#[derive(Default, serde::Serialize)]
pub(crate) struct Snapshot {
    pub ids: BTreeSet<String>,
    pub unresolved: BTreeSet<String>,
}
#[derive(Clone, Default)]
pub(crate) struct CopyUsage(pub Arc<Mutex<Snapshot>>);
impl Global for CopyUsage {}
pub(crate) fn message(message: &Message<'_>, cx: &App) {
    if let Some(usage) = cx.try_global::<CopyUsage>() {
        let id = message
            .attribute()
            .map_or_else(|| message.id().to_owned(), |attribute| format!("{}.{attribute}", message.id()));
        if let Ok(mut snapshot) = usage.0.lock() {
            snapshot.ids.insert(id);
        }
    }
}
pub(crate) fn unknown(source: &str, cx: &App) {
    if source.contains('⟦') || source.trim().is_empty() {
        return;
    }
    if let Some(usage) = cx.try_global::<CopyUsage>() {
        if let Ok(mut snapshot) = usage.0.lock() {
            snapshot.unresolved.insert(source.to_owned());
        }
    }
}
