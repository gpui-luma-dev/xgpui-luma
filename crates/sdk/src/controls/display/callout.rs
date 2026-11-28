//! The callout read from `.lss` (issue #285): `callout.lss`. Looks render the element from [`CalloutLook`].

use crate::style::PartRead;
use gpui::Hsla;

use crate::style::StyleBinding;

/// A callout's colors, frame and text size.
#[deprecated(note = "the callout draws from `StyledCallout::resolve` (#433)")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalloutLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub radius: f32,
    pub padding: f32,
    pub gap: f32,
    pub icon_size: f32,
    pub font_size: f32,
}

/// The callout's `.lss` values for one binding (the `callout` control).
#[derive(Clone)]
pub struct StyledCallout {
    binding: StyleBinding,
}

impl StyledCallout {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyleBinding::sdk("callout"))
    }

    /// The callout's style for the binding's attributes and states (#433): the callout draws from its parts.
    pub fn resolve(&self) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        self.binding.resolve(&[], 1.0)
    }

    #[deprecated(note = "the callout draws from `resolve()` (#433)")]
    #[allow(deprecated)]
    pub fn look(&self) -> CalloutLook {
        let resolved = self.resolve();
        let root = resolved.root();
        CalloutLook {
            background: root.color("background"),
            foreground: root.color("foreground"),
            radius: root.number("radius"),
            padding: root.number("padding_x"),
            gap: root.number("gap"),
            icon_size: root.number("icon_size"),
            font_size: root.font_size().unwrap_or(14.0),
        }
    }
}

/// A callout drawn from `.lss`: a message on a tinted strip, with an optional leading icon aligned to its first
/// line. Looks keep their own callouts (with tones and variants); this is the SDK's.
pub struct Callout {
    text: gpui::SharedString,
    icon: Option<crate::controls::button::ControlIcon>,
    styled: StyledCallout,
}

impl Callout {
    /// A callout showing `text`, from the SDK's `.lss` alone.
    pub fn new(text: impl Into<gpui::SharedString>) -> Self {
        Self { text: text.into(), icon: None, styled: StyledCallout::sdk() }
    }

    /// Lead the message with `icon`.
    pub fn icon(mut self, icon: impl Into<crate::controls::button::ControlIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Draw from `styled` (a look's binding or variant) instead of the SDK alone.
    pub fn styled(mut self, styled: StyledCallout) -> Self {
        self.styled = styled;
        self
    }
}

impl gpui::IntoElement for Callout {
    type Element = gpui::Div;

    fn into_element(self) -> gpui::Div {
        use gpui::{div, prelude::*, px};
        let look = self.styled.resolve();
        let icon = self.icon.map(|icon| match icon {
            crate::controls::button::ControlIcon::Lucide(icon) => {
                crate::infra::icon::lucide_icon(icon, look.root().color("foreground"), look.root().number("icon_size"))
            }
            crate::controls::button::ControlIcon::SvgPath(path) => gpui::svg()
                .path(path)
                .size(px(look.root().number("icon_size")))
                .text_color(look.root().color("foreground"))
                .into_any_element(),
        });
        div()
            .flex()
            .items_start()
            .gap(px(look.root().number("gap")))
            .p(px(look.root().number("padding_x")))
            .rounded(px(look.root().number("radius")))
            .bg(look.root().color("background"))
            .text_color(look.root().color("foreground"))
            .text_size(px(look.root().typography(14.0, 20.0).size))
            .children(icon.map(|icon| div().flex_none().child(icon)))
            .child(div().debug_selector(|| "callout-text".into()).min_w(px(0.)).child(self.text))
    }
}

#[cfg(all(test, feature = "test-support"))]
mod element_tests {
    use gpui::{Context, Render, TestAppContext, Window, div, prelude::*, px};

    use super::*;

    struct Harness {
        icon: bool,
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let callout = Callout::new("Shipping delays may apply.");
            let callout = if self.icon {
                callout.icon(lucide_svg_static::Icon::Info)
            } else {
                callout
            };
            div().w(px(400.)).child(div().debug_selector(|| "callout".into()).child(callout))
        }
    }

    /// The SDK callout paints its tinted strip, and an icon takes its size and gap ahead of the text (#397).
    #[test]
    fn paints_its_strip_and_leads_with_the_icon() {
        let look = StyledCallout::sdk().resolve();
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, _| Harness { icon: false });
        cx.run_until_parked();
        let painted = cx.update(|window, _| {
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background == gpui::Background::from(look.root().color("background")))
        });
        assert!(painted, "the strip is painted");
        let plain = cx.debug_bounds("callout").expect("the callout is drawn");
        view.update(cx, |view, cx| {
            view.icon = true;
            cx.notify();
        });
        cx.run_until_parked();
        let with_icon = cx.debug_bounds("callout").expect("the callout is drawn");
        let text = cx.debug_bounds("callout-text").expect("the text is drawn");
        assert_eq!(with_icon.size.width, plain.size.width, "the icon goes inside the strip");
        assert!(
            text.left()
                >= with_icon.left()
                    + px(look.root().number("padding_x") + look.root().number("icon_size") + look.root().number("gap")),
            "the text starts after the icon: {text:?} in {with_icon:?}"
        );
    }
}
