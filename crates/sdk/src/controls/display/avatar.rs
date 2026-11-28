//! The avatar read from `.lss` (issue #285): `avatar.lss`. Looks render the element from [`AvatarLook`].

use crate::style::PartRead;
use gpui::{Hsla, SharedString};

use crate::style::StyleBinding;

/// An avatar's colors, size and fallback text.
#[deprecated(note = "the avatar draws from `StyledAvatar::resolve` (#433)")]
#[derive(Clone, Debug, PartialEq)]
pub struct AvatarLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub diameter: f32,
    pub radius: f32,
    pub icon_size: f32,
    pub font_size: f32,
    pub two_letter_font_size: f32,
    pub line_height: f32,
    pub font_weight: gpui::FontWeight,
    pub font_family: SharedString,
}

/// The avatar's `.lss` values for one binding (the `avatar` control, with the look's variant and size).
#[derive(Clone)]
pub struct StyledAvatar {
    binding: StyleBinding,
}

impl StyledAvatar {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyleBinding::sdk("avatar"))
    }

    /// The avatar's style for the binding's attributes and states (#433): the avatar draws from its root.
    pub fn resolve(&self) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        self.binding.resolve(&[], 1.0)
    }

    #[deprecated(note = "the avatar draws from `resolve()` (#433)")]
    #[allow(deprecated)]
    pub fn look(&self) -> AvatarLook {
        let resolved = self.binding.resolve(&[], 1.0);
        let root = resolved.root();
        AvatarLook {
            background: root.color("background"),
            foreground: root.color("foreground"),
            diameter: root.number("height"),
            radius: root.number("radius"),
            icon_size: root.number("icon_size"),
            font_size: root.font_size().unwrap_or(16.0),
            two_letter_font_size: root.number("two_letter_font_size"),
            line_height: root.line_height().unwrap_or(16.0),
            font_weight: root.font_weight(),
            font_family: root.font_family_name(),
        }
    }
}

/// What an avatar shows when it has no image, or while its image loads or if it fails.
#[derive(Clone)]
enum AvatarFallback {
    /// Up to two letters, upper-cased.
    Text(SharedString),
    Icon(crate::controls::button::ControlIcon),
}

/// Up to the first two characters of `text`, upper-cased (`"jg"` → `"JG"`).
fn avatar_initials(text: &str) -> SharedString {
    text.trim().chars().flat_map(char::to_uppercase).take(2).collect::<String>().into()
}

/// An avatar drawn from `.lss`: an image cropped to the circle, or its initials or an icon. Looks keep their
/// own avatars (with tones and variants); this is the SDK's.
#[derive(Clone)]
pub struct Avatar {
    image: Option<gpui::ImageSource>,
    fallback: AvatarFallback,
    styled: StyledAvatar,
}

impl Avatar {
    /// An avatar showing `text`'s first two letters, from the SDK's `.lss` alone.
    pub fn new(text: impl AsRef<str>) -> Self {
        Self {
            image: None,
            fallback: AvatarFallback::Text(avatar_initials(text.as_ref())),
            styled: StyledAvatar::sdk(),
        }
    }

    /// Show `image`, cropped to cover the avatar; the initials or icon show while it loads or if it fails.
    pub fn image(mut self, image: impl Into<gpui::ImageSource>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Fall back to `icon` instead of initials.
    pub fn icon(mut self, icon: impl Into<crate::controls::button::ControlIcon>) -> Self {
        self.fallback = AvatarFallback::Icon(icon.into());
        self
    }

    /// Draw from `styled` (a look's binding, variant or size) instead of the SDK alone.
    pub fn styled(mut self, styled: StyledAvatar) -> Self {
        self.styled = styled;
        self
    }

    /// The fallback's font size: the two-letter size for two letters.
    fn font_size(&self, look: &xgpui_luma_style::Resolved) -> f32 {
        match &self.fallback {
            AvatarFallback::Text(text) if text.chars().count() > 1 => look.root().number("two_letter_font_size"),
            _ => look.root().typography(16.0, 16.0).size,
        }
    }

    /// The initials or icon on the avatar's fill.
    fn fallback_element(&self, look: &xgpui_luma_style::Resolved) -> gpui::AnyElement {
        use gpui::{div, prelude::*, px};
        let content = match &self.fallback {
            AvatarFallback::Text(text) => div().child(text.clone()).into_any_element(),
            AvatarFallback::Icon(crate::controls::button::ControlIcon::Lucide(icon)) => {
                crate::infra::icon::lucide_icon(*icon, look.root().color("foreground"), look.root().number("icon_size"))
            }
            AvatarFallback::Icon(crate::controls::button::ControlIcon::SvgPath(path)) => gpui::svg()
                .path(path.clone())
                .size(px(look.root().number("icon_size")))
                .text_color(look.root().color("foreground"))
                .into_any_element(),
        };
        let font_size = self.font_size(look);
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            // Rounded here, not only on the outer box: GPUI clips to a rectangle, so the fill must be round itself.
            .rounded(px(look.root().number("radius").min(look.root().number("height") / 2.)))
            .bg(look.root().color("background"))
            .text_color(look.root().color("foreground"))
            .when(!look.root().font_family_name().is_empty(), |avatar| {
                avatar.font_family(look.root().font_family_name())
            })
            .font_weight(look.root().font_weight())
            .text_size(px(font_size))
            .line_height(px(font_size))
            .child(content)
            .into_any_element()
    }
}

impl gpui::IntoElement for Avatar {
    type Element = gpui::Div;

    fn into_element(mut self) -> gpui::Div {
        use gpui::{div, prelude::*, px};
        let look = self.styled.resolve();
        let content = match self.image.take() {
            Some(source) => {
                let (loading, failure) = (self.clone(), self.clone());
                let (loading_look, failure_look) = (look.clone(), look.clone());
                gpui::img(source)
                    .size_full()
                    .rounded(px(look.root().number("radius").min(look.root().number("height") / 2.)))
                    .object_fit(gpui::ObjectFit::Cover)
                    .with_loading(move || loading.fallback_element(&loading_look))
                    .with_fallback(move || failure.fallback_element(&failure_look))
                    .into_any_element()
            }
            None => self.fallback_element(&look),
        };
        div()
            .size(px(look.root().number("height")))
            .flex_none()
            .rounded(px(look.root().number("radius").min(look.root().number("height") / 2.)))
            .overflow_hidden()
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_avatar_is_a_filled_circle() {
        let look = StyledAvatar::sdk().resolve();
        assert!(
            look.root().number("height") > 0.0 && look.root().number("radius") >= look.root().number("height") / 2.0
        );
        assert!(look.root().color("background").a > 0.0 && look.root().typography(16.0, 16.0).size > 0.0);
        // An icon fallback reads as the same size as the letters, not larger (#397).
        assert!(
            look.root().number("icon_size") > look.root().typography(16.0, 16.0).size
                && look.root().number("icon_size") <= look.root().typography(16.0, 16.0).size * 1.5,
            "{look:?}"
        );
    }
}

#[cfg(all(test, feature = "test-support"))]
mod element_tests {
    use gpui::{Context, Render, TestAppContext, Window, div, prelude::*};

    use super::*;

    struct Harness(Avatar);

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .items_start()
                .child(div().flex_none().debug_selector(|| "avatar".into()).child(self.0.clone()))
        }
    }

    /// Initials are the first two letters, upper-cased; two letters use the two-letter font size (#397).
    #[test]
    fn initials_and_their_font_size() {
        assert_eq!(avatar_initials("  jg  "), "JG");
        assert_eq!(avatar_initials("quinn"), "QU");
        assert_eq!(avatar_initials("q"), "Q");
        let look = StyledAvatar::sdk().resolve();
        assert_eq!(Avatar::new("q").font_size(&look), look.root().typography(16.0, 16.0).size);
        assert_eq!(Avatar::new("jg").font_size(&look), look.root().number("two_letter_font_size"));
    }

    /// The SDK avatar draws a circle the `.lss` diameter wide, filled with its background (#397).
    #[test]
    fn draws_a_filled_circle_of_its_diameter() {
        let look = StyledAvatar::sdk().resolve();
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, _| Harness(Avatar::new("JG")));
        cx.run_until_parked();
        let bounds = cx.debug_bounds("avatar").expect("the avatar is drawn");
        assert_eq!(
            (bounds.size.width, bounds.size.height),
            (gpui::px(look.root().number("height")), gpui::px(look.root().number("height")))
        );
        let radius = look.root().number("radius").min(look.root().number("height") / 2.);
        let round_fill = cx.update(|window, _| {
            let scale = window.scale_factor();
            window.painted_quads().iter().any(|quad| {
                quad.background == gpui::Background::from(look.root().color("background"))
                    && (quad.corner_radii.top_left.0 - radius * scale).abs() < 0.5
                    && (quad.corner_radii.bottom_right.0 - radius * scale).abs() < 0.5
            })
        });
        assert!(round_fill, "its fill is painted round, not clipped to a square");
    }
}
