mod behavior;
mod control;
mod model;
mod styled;
pub use styled::{StyledAutocompleteTheme};
pub use styled::status_typography;
mod template;
mod text_selection;
pub(crate) mod theme;

pub use behavior::SelectionItem;
pub use control::{Autocomplete, AutocompleteControl, AutocompleteEvent};
pub use model::{AutocompleteBuilder, AutocompleteModel, new};
pub use template::{
    AutocompleteItemsRenderModel, AutocompleteItemsTemplate, AutocompleteItemsTemplateHandlers,
    AutocompleteItemsTemplateModifier, AutocompleteRenderModel, AutocompleteTemplate, AutocompleteTemplateHandlers,
    AutocompleteTemplateModifier, DefaultAutocompleteItemsTemplate, DefaultAutocompleteTemplate,
    default_autocomplete_items_template, default_autocomplete_template,
};
pub use theme::{AutocompleteTheme, default_autocomplete_theme};

mod callback;
pub use callback::AutocompleteEventExt;

pub use model::AutocompleteMotion;
