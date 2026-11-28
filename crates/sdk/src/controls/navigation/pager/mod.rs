mod styled;

pub use styled::{PagerButtonSource, StyledPagerTheme, resolve_pager};
mod control;
mod model;
mod template;
pub(crate) mod theme;

pub use control::{PagerControl, PagerEvent};
pub use model::{
    PagerBuilder, PagerIcons, PagerInfoSlot, PagerModel, PagerPageIndicatorFormatter, PagerPageItem, PagerRenderModel,
    PagerStyle, PagerTemplateParameters, new,
};
pub use template::{PagerTemplate, PagerTemplateHandlers, ThemedPagerTemplate, default_pager_template, numeric_page_items};
pub(crate) use template::{render_info_slot, render_nav_group, render_page_indicator, render_page_size_select};

pub use theme::{PagerTheme, default_pager_theme};

use gpui::Entity;

pub type Pager = Entity<PagerControl>;

mod callback;
pub use callback::PagerEventExt;
