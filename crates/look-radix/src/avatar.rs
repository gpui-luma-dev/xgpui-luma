//! Non-interactive, look-owned Radix Avatar: image with a short-text or icon fallback.
//! Geometry follows the [Radix Avatar reference](https://www.radix-ui.com/themes/docs/components/avatar).

use std::sync::Arc;
use gpui::{AnyElement, Div, FontWeight, Hsla, ImageSource, IntoElement, SharedString, div, img, prelude::*, px};
use xgpui_luma::controls::button::ControlIcon;
use crate::{Look, Radius, Tone};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarVariant {
    Solid,
    #[default]
    Soft,
}
impl AvatarVariant {
    pub const ALL: [Self; 2] = [Self::Solid, Self::Soft];
    pub fn label(self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::Soft => "Soft",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    One,
    Two,
    #[default]
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}
impl AvatarSize {
    pub const ALL: [Self; 9] = [
        Self::One,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
    ];
    pub fn as_str(self) -> &'static str {
        ["1", "2", "3", "4", "5", "6", "7", "8", "9"][self as usize]
    }
    pub fn label(self) -> &'static str {
        ["Size 1", "Size 2", "Size 3", "Size 4", "Size 5", "Size 6", "Size 7", "Size 8", "Size 9"][self as usize]
    }
    pub fn diameter(self) -> f32 {
        crate::look::embedded_common_stylesheet()
            .avatar
            .resolve_geometry(self.as_str(), Default::default())
            .diameter
    }
    fn font_size(self, two_letters: bool) -> f32 {
        let geometry =
            crate::look::embedded_common_stylesheet().avatar.resolve_geometry(self.as_str(), Default::default());
        if two_letters {
            geometry.two_letter_font_size
        } else {
            geometry.font_size
        }
    }
    pub fn corner_radius(self, radius: Radius) -> f32 {
        if radius == Radius::Full {
            self.diameter() / 2.0
        } else {
            [4.0, 4.0, 6.0, 6.0, 8.0, 12.0, 12.0, 16.0, 16.0][self as usize] * radius.factor()
        }
    }
}

#[derive(Clone)]
enum Fallback {
    Text(SharedString),
    Icon(ControlIcon),
}

/// Resolved Avatar geometry, typography, and colors in logical pixels.
#[derive(Clone, Debug)]
pub struct AvatarStyle {
    pub diameter: f32,
    pub radius: f32,
    pub icon_size: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: FontWeight,
    pub font_family: SharedString,
    pub background: Hsla,
    pub foreground: Hsla,
}
type StyleOverride = Arc<dyn Fn(&mut AvatarStyle) + Send + Sync>;

/// A display element, not a button. Rebuild it with the owning view when the look changes.
#[derive(Clone)]
pub struct Avatar {
    look: Look,
    image: Option<ImageSource>,
    fallback: Fallback,
    variant: AvatarVariant,
    tone: Tone,
    high_contrast: bool,
    size: AvatarSize,
    radius: Radius,
    style_override: Option<StyleOverride>,
}
impl Avatar {
    /// Short text is uppercased and limited to two Unicode characters.
    pub fn new(look: &Look, text: impl Into<SharedString>) -> Self {
        Self {
            look: look.clone(),
            image: None,
            fallback: Fallback::Text(short_text(text.into())),
            variant: AvatarVariant::Soft,
            tone: Tone::Accent,
            high_contrast: false,
            size: AvatarSize::Three,
            radius: Radius::Medium,
            style_override: None,
        }
    }
    /// Image is cropped to cover the avatar. Text/icon remains its loading/error fallback.
    pub fn image(mut self, image: impl Into<ImageSource>) -> Self {
        self.image = Some(image.into());
        self
    }
    pub fn icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.fallback = Fallback::Icon(icon.into());
        self
    }
    pub fn variant(mut self, variant: AvatarVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    pub fn high_contrast(mut self, high_contrast: bool) -> Self {
        self.high_contrast = high_contrast;
        self
    }
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }
    pub fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }

    /// Adjust resolved appearance locally, preserving palette updates for untouched colors.
    /// Later calls replace the previous override.
    /// ```
    /// use xgpui_luma_look_radix::{Avatar, Look};
    /// let avatar = Avatar::new(&Look::built_in(), "BG").style_override(|style| {
    ///     style.icon_size = 20.0;
    ///     style.font_size = 15.0;
    ///     style.line_height = 18.0;
    /// });
    /// ```
    pub fn style_override(mut self, customize: impl Fn(&mut AvatarStyle) + Send + Sync + 'static) -> Self {
        self.style_override = Some(Arc::new(customize));
        self
    }

    /// Resolve current palette and size defaults, then apply the local override.
    pub fn resolve_style(&self) -> AvatarStyle {
        let (background, foreground) = self.colors();
        let stylesheet = self.look.common_stylesheet();
        let diameter = self.size.diameter();
        let geometry = stylesheet.avatar.resolve_geometry(
            self.size.as_str(),
            xgpui_luma::theme::stylesheet::AvatarGeometry {
                diameter,
                icon_size: diameter * 0.6,
                font_size: self.size.font_size(false),
                two_letter_font_size: self.size.font_size(true),
            },
        );
        let icon_size = if stylesheet.avatar.authored_geometry(self.size.as_str()).icon_size.is_none() {
            geometry.diameter * 0.6
        } else {
            geometry.icon_size
        };
        let font_size = if matches!(&self.fallback, Fallback::Text(text) if text.chars().count() > 1) {
            geometry.two_letter_font_size
        } else {
            geometry.font_size
        };
        let mut style = AvatarStyle {
            diameter: geometry.diameter,
            radius: if self.radius == Radius::Full {
                geometry.diameter / 2.0
            } else {
                self.size.corner_radius(self.radius)
            },
            icon_size,
            font_size,
            line_height: font_size,
            font_weight: FontWeight::MEDIUM,
            font_family: crate::typography::font_family(&self.look),
            background,
            foreground,
        };
        if let Some(customize) = &self.style_override {
            customize(&mut style);
        }
        style
    }

    fn colors(&self) -> (Hsla, Hsla) {
        let step = |n| self.tone.step(&self.look, n);
        match (self.variant, self.high_contrast) {
            (AvatarVariant::Solid, true) => (step(12), step(1)),
            (AvatarVariant::Solid, false) => (step(9), self.tone.contrast(&self.look)),
            (AvatarVariant::Soft, high) => (step(3), step(if high { 12 } else { 11 })),
        }
    }
    fn fallback_element(&self, style: &AvatarStyle) -> AnyElement {
        let foreground = style.foreground;
        let content = match &self.fallback {
            Fallback::Text(text) => div().child(text.clone()).into_any_element(),
            Fallback::Icon(ControlIcon::SvgPath(path)) => {
                gpui::svg().path(path.clone()).size(px(style.icon_size)).text_color(foreground).into_any_element()
            }
            Fallback::Icon(ControlIcon::Lucide(icon)) => {
                xgpui_luma::infra::icon::lucide_icon(*icon, foreground, style.icon_size)
            }
        };
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(style.background)
            .rounded(px(style.radius))
            .text_color(foreground)
            .font_family(style.font_family.clone())
            .font_weight(style.font_weight)
            .text_size(px(style.font_size))
            .line_height(px(style.line_height))
            .child(content)
            .into_any_element()
    }
}
fn short_text(text: SharedString) -> SharedString {
    text.trim().chars().flat_map(char::to_uppercase).take(2).collect::<String>().into()
}
impl IntoElement for Avatar {
    type Element = Div;
    fn into_element(mut self) -> Div {
        let style = self.resolve_style();
        let content = if let Some(source) = self.image.take() {
            let loading = self.clone();
            let failure = self.clone();
            let loading_style = style.clone();
            let failure_style = style.clone();
            img(source)
                .size_full()
                .rounded(px(style.radius))
                .object_fit(gpui::ObjectFit::Cover)
                .with_loading(move || loading.fallback_element(&loading_style))
                .with_fallback(move || failure.fallback_element(&failure_style))
                .into_any_element()
        } else {
            self.fallback_element(&style)
        };
        div()
            .size(px(style.diameter))
            .flex_none()
            .rounded(px(style.radius))
            .overflow_hidden()
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn icon_fallback_tracks_resolved_diameter_but_preserves_authored_sizes() {
        let look = Look::built_in();
        let avatar = Avatar::new(&look, "A");
        let mut stylesheet = look.common_stylesheet();
        stylesheet.avatar.sizes.get_mut("3").unwrap().diameter = Some(75.0);
        look.set_common_stylesheet(stylesheet.clone());
        assert_eq!(avatar.resolve_style().icon_size, 75.0 * 0.6);

        stylesheet.avatar.geometry.icon_size = Some(17.0);
        look.set_common_stylesheet(stylesheet.clone());
        assert_eq!(avatar.resolve_style().icon_size, 17.0);

        stylesheet.avatar.sizes.get_mut("3").unwrap().icon_size = Some(0.0);
        look.set_common_stylesheet(stylesheet);
        assert_eq!(avatar.resolve_style().icon_size, 0.0);
    }

    #[test]
    fn local_override_preserves_live_palette_and_does_not_change_siblings() {
        let look = Look::built_in();
        let custom = Avatar::new(&look, "BG").style_override(|style| {
            style.icon_size = 20.0;
            style.font_weight = FontWeight::BOLD;
            style.diameter = 48.0;
        });
        let normal = Avatar::new(&look, "BG");
        let before = custom.resolve_style().background;
        look.set_mode(xgpui_luma::theme::ThemeMode::Dark);
        let style = custom.resolve_style();
        assert_ne!(style.background, before);
        assert_eq!(style.background, normal.resolve_style().background);
        assert_eq!(style.icon_size, 20.0);
        assert_eq!(style.diameter, 48.0);
        assert_eq!(style.font_weight, FontWeight::BOLD);
        assert_eq!(normal.resolve_style().diameter, 40.0);
        assert_eq!(normal.resolve_style().font_weight, FontWeight::MEDIUM);
    }

    #[test]
    fn sizes_and_radius_match_radix() {
        for (size, diameter) in
            AvatarSize::ALL.into_iter().zip([24.0, 32.0, 40.0, 48.0, 64.0, 80.0, 96.0, 128.0, 160.0])
        {
            assert_eq!(size.diameter(), diameter);
            assert_eq!(size.corner_radius(Radius::None), 0.0);
            assert_eq!(size.corner_radius(Radius::Full), diameter / 2.0);
            assert!(size.corner_radius(Radius::Small) < size.corner_radius(Radius::Medium));
            assert!(size.corner_radius(Radius::Medium) < size.corner_radius(Radius::Large));
        }
        assert_eq!(short_text("bg-long".into()).as_ref(), "BG");
        assert_eq!(short_text("éø".into()).as_ref(), "ÉØ");
    }
    #[test]
    fn colors_follow_mode_tone_and_contrast() {
        let look = Look::built_in();
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for tone in [Tone::Accent, Tone::Gray] {
                for variant in AvatarVariant::ALL {
                    let normal = Avatar::new(&look, "V").tone(tone).variant(variant).colors();
                    let high = Avatar::new(&look, "BG").tone(tone).variant(variant).high_contrast(true).colors();
                    assert_eq!(normal.0, tone.step(&look, if variant == AvatarVariant::Solid { 9 } else { 3 }));
                    assert_eq!(high.1, tone.step(&look, if variant == AvatarVariant::Solid { 1 } else { 12 }));
                }
            }
        }
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_avatar_stylesheet() -> xgpui_luma::theme::stylesheet::AvatarStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    AvatarStylesheet {
        sizes: [
            ("1", 24.0, 14.0, 12.0),
            ("2", 32.0, 16.0, 14.0),
            ("3", 40.0, 18.0, 16.0),
            ("4", 48.0, 20.0, 18.0),
            ("5", 64.0, 24.0, 24.0),
            ("6", 80.0, 28.0, 28.0),
            ("7", 96.0, 28.0, 28.0),
            ("8", 128.0, 35.0, 35.0),
            ("9", 160.0, 60.0, 60.0),
        ]
        .into_iter()
        .map(|(key, diameter, font, two)| {
            (
                key.into(),
                AvatarGeometryStylesheet {
                    diameter: Some(diameter),
                    font_size: Some(font),
                    two_letter_font_size: Some(two),
                    ..Default::default()
                },
            )
        })
        .collect(),
        ..Default::default()
    }
}
