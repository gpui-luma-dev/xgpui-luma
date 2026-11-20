mod control;
mod model;
mod template;
pub(crate) mod theme;

pub use control::{SplitView, SplitViewEvent};
pub use model::{
    PaneRender, SplitViewBuilder, SplitViewModel, SplitViewRenderModel, SplitViewSeparatorVisibility, render_pane,
};
pub use template::{
    SplitViewTemplate, SplitViewTemplateModifier, SplitViewTemplateHandlers, ThemedSplitViewTemplate,
    default_split_view_template,
};
pub use theme::{DefaultSplitViewTheme, SplitViewLook, SplitViewTheme, default_split_view_theme};

mod callback;
pub use callback::SplitViewEventExt;

pub use model::SplitViewMotion;
pub use super::styled::StyledSplitViewTheme;
