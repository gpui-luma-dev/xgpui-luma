//! Tabs read from `.lss` (issue #285): `tabs.lss`. Looks keep their own drag preview, baseline and content
//! padding, and delegate the list and tabs to [`StyledTabs`].

use gpui::{Hsla, SharedString};
use xgpui_luma_style::{PartId, PartView, Value};

use super::theme::{TabsItemLook, TabsListLook, TabsTheme};
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

/// The tabs' `.lss` values for one binding (the `tabs` control).
#[derive(Clone)]
pub struct StyledTabs {
    binding: StyleBinding,
}

impl StyledTabs {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("tabs") }
    }

    /// The tab list, enabled or not.
    pub fn list(&self, enabled: bool) -> TabsListLook {
        let resolved = self.binding.resolve(&[("disabled", !enabled)], 1.0);
        let list = resolved.part(PartId::new("list"));
        TabsListLook {
            border_width: number(list, "border_width"),
            background: painted(list, "background"),
            border: painted(list, "border"),
            radius: number(list, "radius"),
            padding: number(list, "padding"),
            gap: number(list, "gap"),
        }
    }

    /// A tab, active or not, in `state`.
    pub fn item(&self, active: bool, state: InteractionState) -> TabsItemLook {
        let states = [
            ("selected", active),
            ("hover", state.hovered),
            ("pressed", state.pressed),
            ("focused", state.focused),
            ("disabled", state.disabled),
        ];
        let resolved = self.binding.resolve(&states, 1.0);
        let (root, item) = (resolved.root(), resolved.part(PartId::new("item")));
        TabsItemLook {
            accessory_size: number(root, "accessory_size"),
            accessory_gap: number(root, "accessory_gap"),
            label_color: color(item, "foreground"),
            indicator: if active { painted(root, "indicator") } else { None },
            background: painted(item, "background"),
            label_typography: LumaTextStyle {
                size: item.font_size().unwrap_or(14.0),
                line_height: item.line_height().unwrap_or(20.0),
                weight: gpui::FontWeight(number(item, "font_weight").max(100.0)),
            },
            radius: number(item, "radius"),
            padding_x: number(item, "padding_x"),
            height: number(item, "height"),
            indicator_height: number(root, "indicator_height"),
        }
    }

    /// The tabs' font family.
    pub fn font_family(&self) -> SharedString {
        let resolved = self.binding.resolve(&[], 1.0);
        resolved.part(PartId::new("item")).font_family().unwrap_or("").to_string().into()
    }
}

/// The SDK's tabs theme: `tabs.lss`, with the SDK's drag preview.
pub struct StyledTabsTheme(StyledTabs);

impl StyledTabsTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self(StyledTabs::new(binding))
    }

    pub fn sdk() -> Self {
        Self(StyledTabs::sdk())
    }
}

impl TabsTheme for StyledTabsTheme {
    fn resolve_list(&self, enabled: bool) -> TabsListLook {
        self.0.list(enabled)
    }

    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsItemLook {
        self.0.item(active, state)
    }

    fn font_family(&self) -> SharedString {
        self.0.font_family()
    }
}
