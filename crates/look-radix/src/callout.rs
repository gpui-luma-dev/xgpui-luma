//! Soft informational callout built from the current Radix accent scale.

use gpui::{App, Div, Hsla, SharedString, Window, div, prelude::*, px};
use xgpui_luma::infra::locale::{LayoutDirection, LocaleContext, LogicalStyled};
use xgpui_luma::controls::button::ControlIcon;

use crate::{Look, ScaleFamily};

/// Paint a callout using accent 3 for its surface and accent 11 for text and icon.
/// Applications supply content and assets; spacing and colors belong to the look.
/// The icon sits at the logical start, read from the locale when painted.
pub fn callout(look: &Look, text: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> Div {
    let geometry = resolve_geometry(look);
    div()
        .flex()
        .w_full()
        .rounded_lg()
        .p(px(geometry.padding))
        .bg(look.resolve_step(ScaleFamily::Color, 3).hsla())
        .child(CalloutRow {
            foreground: look.resolve_step(ScaleFamily::Color, 11).hsla(),
            icon: icon.into(),
            text: text.into(),
            geometry,
        })
}

#[derive(IntoElement)]
struct CalloutRow {
    foreground: Hsla,
    icon: ControlIcon,
    text: SharedString,
    geometry: xgpui_luma::theme::stylesheet::CalloutGeometry,
}

impl RenderOnce for CalloutRow {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let direction = LocaleContext::current(cx).direction;
        div()
            .flex()
            .flex_row_logical(direction)
            .items_center()
            .gap(px(self.geometry.gap))
            .w_full()
            .child(
                div()
                    .debug_selector(|| "radix-callout-icon".into())
                    .flex_none()
                    .child(self.icon.element(self.foreground, self.geometry.icon_size)),
            )
            .child(
                div()
                    .debug_selector(|| "radix-callout-text".into())
                    .when(direction == LayoutDirection::Rtl, |text| text.text_right())
                    .text_size(px(self.geometry.font_size))
                    .text_color(self.foreground)
                    .child(self.text),
            )
    }
}

pub(crate) fn resolve_geometry(look: &Look) -> xgpui_luma::theme::stylesheet::CalloutGeometry {
    let metrics = look.metrics();
    let typography = xgpui_luma::theme::ThemeTokens::default().typography.text.label;
    look.common_stylesheet().callout.resolve_geometry(
        "",
        xgpui_luma::theme::stylesheet::CalloutGeometry {
            padding: metrics.spacing.s4,
            gap: metrics.control.md.gap,
            icon_size: metrics.control.md.icon_size,
            font_size: typography.size,
        },
    )
}

/// Native common geometry owned by this control.
pub(crate) fn native_callout_stylesheet() -> xgpui_luma::theme::stylesheet::CalloutStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    CalloutStylesheet {
        geometry: CalloutGeometryStylesheet {
            padding: Some(12.0),
            gap: Some(12.0),
            icon_size: Some(15.0),
            font_size: Some(14.0),
        },
        ..Default::default()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod locale_tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext};
    use xgpui_luma::infra::locale::set_locale_context;

    struct Harness;
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(400.))
                .child(callout(&Look::built_in(), "Note", ControlIcon::SvgPath("icons/info.svg".into())))
        }
    }

    /// #266: the icon leads the text at the logical start in both directions.
    #[test]
    fn callout_icon_sits_at_the_logical_start() {
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, _| Harness);
        for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
            cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
            cx.run_until_parked();
            let icon = cx.debug_bounds("radix-callout-icon").unwrap();
            let text = cx.debug_bounds("radix-callout-text").unwrap();
            assert_eq!(icon.left() < text.left(), direction == LayoutDirection::Ltr, "{direction:?}");
        }
    }
}
