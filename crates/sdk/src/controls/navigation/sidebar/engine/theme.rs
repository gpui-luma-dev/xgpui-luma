use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct SidebarSectionLook {
    pub label_color: Hsla,
    pub typography: LumaTextStyle,
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SidebarItemLook {
    pub background: Option<Hsla>,
    pub focus_border: Option<Hsla>,
    pub foreground: Hsla,
    pub icon_color: Hsla,
    pub typography: LumaTextStyle,
    pub radius: f32,
    pub height: f32,
    pub padding_x: f32,
    pub gap: f32,
    pub icon_size: f32,
}

pub trait SidebarTheme: Send + Sync {
    /// Shared title/subtitle typography and section OpenType treatment.
    fn typography(&self) -> crate::theme::LumaTypography {
        crate::theme::LumaTypography::default()
    }
    fn section_font_features(&self) -> gpui::FontFeatures {
        crate::theme::stylesheet::CommonStylesheet::sdk_default().font_features.resolve("sidebar_section")
    }

    /// Shared shell and row decoration used by the stock template.
    fn geometry(&self) -> crate::theme::stylesheet::SidebarGeometry {
        crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .sidebar
            .resolve_geometry(Default::default())
    }
    /// Appearance of native drag previews and insertion feedback.
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        let mut drag = crate::infra::drag_drop::DragDropLook::from_tokens(&ThemeTokens::default());
        let item = self.resolve_item(true, InteractionState { focused: true, ..Default::default() });
        drag.background = item.background.unwrap_or(drag.background);
        drag.foreground = item.foreground;
        drag.valid_marker = item.focus_border.unwrap_or(item.foreground);
        drag.invalid_marker =
            self.resolve_item(false, InteractionState { invalid: true, ..Default::default() }).foreground;
        drag.radius = item.radius;
        drag.typography = item.typography;
        drag
    }
    fn foreground(&self) -> Hsla;
    fn resolve_section(&self) -> SidebarSectionLook;
    fn resolve_branch(&self, state: InteractionState) -> SidebarItemLook;
    fn resolve_item(&self, selected: bool, state: InteractionState) -> SidebarItemLook;

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSidebarTheme {
    tokens: ThemeTokens,
}

pub fn default_sidebar_theme() -> Arc<dyn SidebarTheme> {
    static THEME: OnceLock<Arc<dyn SidebarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSidebarTheme::default())).clone()
}

impl DefaultSidebarTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }

    fn base_item(&self, state: InteractionState) -> SidebarItemLook {
        let palette = self.tokens.stylesheet.recipes.palette("sidebar", &self.tokens.palette);
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size_metrics = metrics.control;
        let foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.navigation.foreground
        };

        SidebarItemLook {
            background: None,
            focus_border: (state.focused && !state.disabled).then_some(palette.focus.ring),
            foreground,
            icon_color: foreground,
            typography: typography.text.label,
            radius: metrics.radius(),
            height: self.geometry().engine_item_height,
            padding_x: self.geometry().engine_item_padding_x,
            gap: size_metrics.gap,
            icon_size: self.geometry().item_icon_size,
        }
    }
}

impl SidebarTheme for DefaultSidebarTheme {
    fn typography(&self) -> crate::theme::LumaTypography {
        self.tokens.typography.clone()
    }
    fn section_font_features(&self) -> gpui::FontFeatures {
        self.tokens.stylesheet.font_features.resolve("sidebar_section")
    }

    fn geometry(&self) -> crate::theme::stylesheet::SidebarGeometry {
        self.tokens.stylesheet.sidebar.resolve_geometry(Default::default())
    }
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        crate::infra::drag_drop::DragDropLook::from_tokens(&self.tokens)
    }
    fn foreground(&self) -> Hsla {
        self.tokens.stylesheet.recipes.palette("sidebar", &self.tokens.palette).navigation.foreground
    }

    fn resolve_section(&self) -> SidebarSectionLook {
        SidebarSectionLook {
            label_color: self
                .tokens
                .stylesheet
                .recipes
                .palette("sidebar", &self.tokens.palette)
                .navigation
                .muted_foreground,
            typography: self.tokens.typography.text.caption,
            height: self.geometry().engine_section_height,
        }
    }

    fn resolve_branch(&self, state: InteractionState) -> SidebarItemLook {
        let mut look = self.base_item(state);
        let palette = self.tokens.stylesheet.recipes.palette("sidebar", &self.tokens.palette);

        look.background = match state.layer() {
            InteractionLayer::Disabled | InteractionLayer::Default => None,
            InteractionLayer::Hovered => Some(palette.navigation.hover_background),
            InteractionLayer::Pressed => Some(palette.state.pressed.background),
        };

        look
    }

    fn resolve_item(&self, selected: bool, state: InteractionState) -> SidebarItemLook {
        let mut look = self.base_item(state);
        let palette = self.tokens.stylesheet.recipes.palette("sidebar", &self.tokens.palette);

        look.background = match (selected, state.layer()) {
            (_, InteractionLayer::Disabled) => None,
            (true, InteractionLayer::Pressed) => Some(palette.state.pressed.background),
            (true, InteractionLayer::Hovered) => Some(palette.navigation.hover_background),
            (true, InteractionLayer::Default) => Some(palette.navigation.selected_background),
            (false, InteractionLayer::Pressed) => Some(palette.state.pressed.background),
            (false, InteractionLayer::Hovered) => Some(palette.navigation.hover_background),
            (false, InteractionLayer::Default) => None,
        };

        if selected && !state.disabled {
            look.foreground = palette.navigation.selected_foreground;
            look.icon_color = palette.navigation.selected_foreground;
        }

        look
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}
