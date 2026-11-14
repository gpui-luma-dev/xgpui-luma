use anyhow::Result;
use gpui::{AssetSource, SharedString};
use lucide_svg_static::asset_bytes as lucide_asset_bytes;
use std::borrow::Cow;

use super::react_icons::{Icon, asset_bytes as react_icon_asset_bytes};

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let avatar: Option<&'static [u8]> = match path.trim_start_matches('/') {
            // Two Lucide icons newer than the workspace's pinned icon catalog.
            "assets/lucide/square-padding-horizontal.svg" => {
                Some(include_bytes!("lucide/square-padding-horizontal.svg"))
            }
            "assets/lucide/square-padding-vertical.svg" => Some(include_bytes!("lucide/square-padding-vertical.svg")),
            "assets/avatars/portrait.jpg" => Some(include_bytes!("avatars/portrait.jpg")),
            "assets/avatars/people.svg" => Some(include_bytes!("avatars/people.svg")),
            _ => None,
        };
        if let Some(bytes) = avatar {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        Ok(react_icon_asset_bytes(path).or_else(|| lucide_asset_bytes(path)).map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let path = path.trim_start_matches('/');
        if path == "assets/react-icons" || path == "assets/react-icons/" {
            return Ok(Icon::all().iter().map(|icon| SharedString::from(icon.asset_path())).collect());
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_and_split_button_icons_are_available_from_the_asset_source() {
        for path in [
            "assets/react-icons/share-2.svg",
            "assets/react-icons/copy.svg",
            "assets/react-icons/figma-logo.svg",
            lucide_svg_static::Icon::Save.asset_path(),
            lucide_svg_static::Icon::TextAlignStart.asset_path(),
            lucide_svg_static::Icon::TextAlignEnd.asset_path(),
            "assets/lucide/square-padding-horizontal.svg",
            "assets/lucide/square-padding-vertical.svg",
        ] {
            let bytes = Assets.load(path).expect("asset loading succeeds").expect("menu icon exists");
            assert!(bytes.starts_with(b"<svg"), "invalid SVG asset: {path}");
        }
    }
}
