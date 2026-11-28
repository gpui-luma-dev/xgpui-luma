mod control;
mod model;
mod template;
pub(crate) mod theme;

pub use control::{DockSplitter, DockSplitterDrag, DockSplitterEvent};
pub use model::{
    DockSplitterBuilder, DockSplitterModel, DockSplitterRenderModel, DockSplitterResizeHandler, SplitterOrientation,
};
pub use template::{
    DockSplitterTemplate, DockSplitterTemplateHandlers, ThemedDockSplitterTemplate, default_dock_splitter_template,
};
pub use theme::{DockSplitterTheme, default_dock_splitter_theme};

mod callback;
pub use callback::DockSplitterEventExt;
pub use super::styled::{StyledDockSplitterTheme, resolve_dock_splitter};
