//! Studio navigation uses SDK tabs with a look-backed selected chip.
use std::sync::Arc;
use xgpui_luma::controls::tabs::{TabsTemplate, TabsTheme, ThemedTabsTemplate};
use xgpui_luma::style::lss::{Overrides, PartId, Prop};
use xgpui_luma::style::{StyleBinding, lss_color};
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_radix::Look;
use crate::theme::ColorVizLookExt;

struct StudioTabsTheme {
    look: Arc<Look>,
    segmented: bool,
}

impl TabsTheme for StudioTabsTheme {
    /// The Radix tabs with an unboxed list (#433).
    fn style(&self) -> StyleBinding {
        let list = PartId::new("list");
        let clear = lss_color(gpui::transparent_black());
        xgpui_luma_look_radix::tabs_theme(&self.look).style().with_overrides(
            &Overrides::new()
                .set(list, Prop::new("background"), clear)
                .set(list, Prop::new("border"), clear)
                .set(list, Prop::<f32>::new("padding"), if self.segmented { 3.0 } else { 0.0 })
                .set(list, Prop::<f32>::new("radius"), 5.0)
                .set(list, Prop::<f32>::new("gap"), if self.segmented { 2.0 } else { 8.0 }),
        )
    }

    /// A look-backed chip with no indicator: its fill and label follow the look's interaction colors.
    fn item_style(&self, active: bool, state: InteractionState) -> StyleBinding {
        let chrome = self.look.chrome();
        let (root, item) = (PartId::ROOT, PartId::new("item"));
        let label = if !state.disabled && active {
            chrome.body_text
        } else {
            chrome.muted_text
        };
        let background =
            self.look.interaction_colors().background(active, state).unwrap_or_else(gpui::transparent_black);
        let mut overrides = Overrides::new()
            .set(root, Prop::new("indicator"), lss_color(gpui::transparent_black()))
            .set(root, Prop::<f32>::new("indicator_height"), 0.0)
            .set(item, Prop::new("background"), lss_color(background))
            .set(item, Prop::new("foreground"), lss_color(label))
            .set(item, Prop::<f32>::new("radius"), 4.0)
            .set(item, Prop::<f32>::new("padding_x"), 14.0)
            .set(item, Prop::<f32>::new("height"), if self.segmented { 26.0 } else { 32.0 });
        if !self.segmented {
            overrides = overrides.set(item, Prop::<f32>::new("font_size"), 14.0).set(
                item,
                Prop::<f32>::new("line_height"),
                20.0,
            );
        }
        self.style().with_overrides(&overrides)
    }

    fn font_family(&self) -> gpui::SharedString {
        "System UI".to_string().into()
    }
}

pub(crate) fn template(look: Arc<Look>, segmented: bool) -> Arc<dyn TabsTemplate> {
    Arc::new(ThemedTabsTemplate::new(Arc::new(StudioTabsTheme { look, segmented })))
}
