//! The Shadcn drag preview, drawn from `.lss` (`style/drag_preview.lss`) at the look's size.

use std::sync::Arc;

use xgpui_luma::infra::drag_drop::resolve_drag_preview;
use xgpui_luma::style::lss::Resolved;

use crate::look::ShadcnLook;

pub(crate) fn drag_style(theme: &ShadcnLook) -> Arc<Resolved> {
    let size = theme.size();
    let style = theme.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "drag_preview",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || style.style_theme())),
    );
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    resolve_drag_preview(&binding)
}
