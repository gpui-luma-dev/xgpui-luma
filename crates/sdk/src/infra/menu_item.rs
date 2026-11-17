use gpui::SharedString;
use lucide_svg_static::Icon as LucideIcon;

#[derive(Clone, Debug)]
pub enum MenuItemIcon {
    Lucide(LucideIcon),
    SvgPath(SharedString),
    /// SVG supplied by the application's registered asset source.
    AssetPath(SharedString),
    /// Opt-in directional artwork, mirrored only in RTL UI.
    Mirrored(Box<MenuItemIcon>),
}

impl MenuItemIcon {
    /// Mirror this directional icon in RTL; ordinary artwork is unchanged by default.
    pub fn mirrored_in_rtl(self) -> Self {
        if matches!(self, Self::Mirrored(_)) {
            self
        } else {
            Self::Mirrored(Box::new(self))
        }
    }
    pub(crate) fn mirrors_in_rtl(&self) -> bool {
        matches!(self, Self::Mirrored(_))
    }

    /// Use an SVG from the application's registered asset source.
    pub fn asset(path: impl Into<SharedString>) -> Self {
        Self::AssetPath(path.into())
    }

    pub fn asset_path(&self) -> Option<&SharedString> {
        match self {
            Self::AssetPath(path) => Some(path),
            Self::Mirrored(icon) => icon.asset_path(),
            _ => None,
        }
    }

    pub fn lucide(&self) -> Option<LucideIcon> {
        match self {
            Self::Lucide(icon) => Some(*icon),
            Self::Mirrored(icon) => icon.lucide(),
            Self::SvgPath(_) | Self::AssetPath(_) => None,
        }
    }

    pub fn svg_path(&self) -> Option<&SharedString> {
        match self {
            Self::Lucide(_) | Self::AssetPath(_) => None,
            Self::SvgPath(path) => Some(path),
            Self::Mirrored(icon) => icon.svg_path(),
        }
    }
}

impl MenuItemIcon {
    pub(crate) fn render(
        &self,
        color: gpui::Hsla,
        size: f32,
        direction: crate::infra::locale::LayoutDirection,
    ) -> gpui::AnyElement {
        use gpui::{prelude::*, px, svg};
        let icon = if let Some(icon) = self.lucide() {
            svg().path(icon.asset_path())
        } else if let Some(path) = self.asset_path() {
            svg().path(path.clone())
        } else if let Some(path) = self.svg_path() {
            svg().external_path(path.clone())
        } else {
            return gpui::Empty.into_any_element();
        };
        icon.size(px(size))
            .text_color(color)
            .when(self.mirrors_in_rtl() && direction == crate::infra::locale::LayoutDirection::Rtl, |icon| {
                icon.with_transformation(gpui::Transformation::scale(gpui::size(-1.0, 1.0)))
            })
            .into_any_element()
    }
}

impl From<LucideIcon> for MenuItemIcon {
    fn from(icon: LucideIcon) -> Self {
        Self::Lucide(icon)
    }
}

impl From<&str> for MenuItemIcon {
    fn from(icon: &str) -> Self {
        Self::SvgPath(icon.to_string().into())
    }
}

impl From<String> for MenuItemIcon {
    fn from(icon: String) -> Self {
        Self::from(icon.as_str())
    }
}

impl From<SharedString> for MenuItemIcon {
    fn from(icon: SharedString) -> Self {
        Self::SvgPath(icon)
    }
}

#[derive(Clone, Debug)]
pub struct MenuItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<MenuItemIcon>,
    pub(crate) submenu_items: Vec<MenuItem>,
    pub(crate) enabled: bool,
    separator: bool,
}

impl MenuItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self { label: id.clone(), id, icon: None, submenu_items: Vec::new(), enabled: true, separator: false }
    }

    /// A noninteractive divider styled by the active floating-menu look.
    pub fn separator(id: impl Into<SharedString>) -> Self {
        Self { separator: true, ..Self::new(id) }
    }

    /// Whether this entry is a divider rather than an action or submenu.
    pub fn is_separator(&self) -> bool {
        self.separator
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<MenuItemIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn submenu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.submenu_items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn icon_ref(&self) -> Option<&MenuItemIcon> {
        self.icon.as_ref()
    }

    pub fn submenu_items(&self) -> &[MenuItem] {
        &self.submenu_items
    }

    /// Whether the entry can be selected or navigated to. Separators always return false.
    pub fn is_enabled(&self) -> bool {
        self.enabled && !self.separator
    }
}
