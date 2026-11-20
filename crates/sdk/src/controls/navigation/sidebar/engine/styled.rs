//! Sidebar rows and section labels read from `.lss` (issue #285): `sidebar.lss`. Looks keep the shell
//! layout, typography roles, section font features and drag preview, and delegate the rows to
//! [`StyledSidebar`].

use gpui::Hsla;
use xgpui_luma_style::{PartId, PartView, Value};

use super::theme::{SidebarItemLook, SidebarSectionLook, SidebarTheme};
use crate::style::StyleBinding;
use crate::theme::{InteractionState, LumaTextStyle};

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn painted(part: PartView<'_>, name: &str) -> Option<Hsla> {
    Some(color(part, name)).filter(|color| color.a > 0.0)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

fn typography(part: PartView<'_>) -> LumaTextStyle {
    LumaTextStyle {
        size: part.font_size().unwrap_or(12.5),
        line_height: part.line_height().unwrap_or(18.0),
        weight: gpui::FontWeight(number(part, "font_weight").max(100.0)),
    }
}

/// The sidebar's `.lss` values for one binding (the `sidebar` control).
#[derive(Clone)]
pub struct StyledSidebar {
    binding: StyleBinding,
}

impl StyledSidebar {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("sidebar") }
    }

    /// A section label.
    pub fn section(&self) -> SidebarSectionLook {
        let resolved = self.binding.resolve(&[], 1.0);
        let section = resolved.part(PartId::new("section"));
        SidebarSectionLook {
            label_color: color(section, "foreground"),
            typography: typography(section),
            height: number(section, "height"),
        }
    }

    /// The rows' text color at rest.
    pub fn foreground(&self) -> Hsla {
        color(self.binding.resolve(&[], 1.0).part(PartId::new("item")), "foreground")
    }

    /// A row, selected (the current page) or not, in `state`.
    pub fn item(&self, selected: bool, state: InteractionState) -> SidebarItemLook {
        let states = [
            ("selected", selected),
            ("hover", state.hovered),
            ("pressed", state.pressed),
            ("focused", state.focused),
            ("disabled", state.disabled),
            ("invalid", state.invalid),
        ];
        let resolved = self.binding.resolve(&states, 1.0);
        let (root, item) = (resolved.root(), resolved.part(PartId::new("item")));
        let foreground = color(item, "foreground");
        SidebarItemLook {
            background: painted(item, "background"),
            focus_border: painted(item, "focus_border"),
            foreground,
            icon_color: foreground,
            typography: typography(item),
            radius: number(item, "radius"),
            height: number(item, "height"),
            padding_x: number(item, "padding_x"),
            gap: number(item, "gap"),
            icon_size: number(root, "item_icon_size"),
        }
    }

    /// A branch row, which is never the current page.
    pub fn branch(&self, state: InteractionState) -> SidebarItemLook {
        self.item(false, state)
    }
}

/// The SDK's sidebar theme: `sidebar.lss` for the rows, with the SDK's layout and drag preview.
pub struct StyledSidebarTheme(StyledSidebar);

impl StyledSidebarTheme {
    pub fn sdk() -> Self {
        Self(StyledSidebar::sdk())
    }
}

impl SidebarTheme for StyledSidebarTheme {
    fn foreground(&self) -> Hsla {
        self.0.foreground()
    }

    fn resolve_section(&self) -> SidebarSectionLook {
        self.0.section()
    }

    fn resolve_branch(&self, state: InteractionState) -> SidebarItemLook {
        self.0.branch(state)
    }

    fn resolve_item(&self, selected: bool, state: InteractionState) -> SidebarItemLook {
        self.0.item(selected, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_sidebar_matches_the_token_sidebar() {
        let close = |a: Option<Hsla>, b: Option<Hsla>| match (a, b) {
            (Some(a), Some(b)) => {
                let (a, b) = (a.to_rgb(), b.to_rgb());
                [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
            }
            (None, None) => true,
            _ => false,
        };
        let native = super::super::theme::DefaultSidebarTheme::default();
        let lss = StyledSidebarTheme::sdk();
        let (got, want) = (lss.resolve_section(), native.resolve_section());
        assert!(close(Some(got.label_color), Some(want.label_color)));
        assert_eq!((got.height, got.typography.size), (want.height, want.typography.size));
        assert!(close(Some(lss.foreground()), Some(native.foreground())));
        for selected in [true, false] {
            for bits in 0..16u8 {
                let state = InteractionState {
                    hovered: bits & 1 != 0,
                    pressed: bits & 2 != 0,
                    focused: bits & 4 != 0,
                    disabled: bits & 8 != 0,
                    ..Default::default()
                };
                for (got, want, kind) in [
                    (lss.resolve_item(selected, state), native.resolve_item(selected, state), "item"),
                    (lss.resolve_branch(state), native.resolve_branch(state), "branch"),
                ] {
                    assert!(close(got.background, want.background), "{kind} {selected} {state:?} background");
                    assert!(close(got.focus_border, want.focus_border), "{kind} {selected} {state:?} focus");
                    assert!(close(Some(got.foreground), Some(want.foreground)), "{kind} {selected} {state:?} fg");
                    assert_eq!(
                        (got.radius, got.height, got.padding_x, got.gap, got.icon_size, got.typography.size),
                        (want.radius, want.height, want.padding_x, want.gap, want.icon_size, want.typography.size)
                    );
                }
            }
        }
    }
}
