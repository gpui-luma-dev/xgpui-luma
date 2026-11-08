use gpui::Subscription;
use xgpui_luma::controls::resizable_panels::ResizablePanelsEventExt;
use xgpui_luma::controls::switch::SwitchEventExt;
use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FocusHandle, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::{ButtonContentContext, ControlIcon, ControlPresenter, HasPresenter};
use xgpui_luma::controls::icon_button::IconButton;
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::resizable_panels::{PanelHideMode, ResizablePanelsEvent};
use xgpui_luma::controls::switch::{Switch, SwitchData};
use xgpui_luma::shell::TitleBar;
use xgpui_luma::color::ColorValue;
use xgpui_luma::theme::{ControlSize, InteractionState, LumaThemeSyncExt, ThemeMode};
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextRole, sync_color_control_theme};
use xgpui_luma_look_shadcn as shadcn;
use lucide_svg_static::Icon as LucideIcon;

use crate::theme::{LumaStudioLaunchOptions, LumaStudioThemeChoice};

use super::content_tabs::{BoardSnapshot, ContentPaneHost};
use super::controls::LumaStudioAppHandle;
use super::controls::workbench_layout::{CONTENT_PANEL_INDEX, LEFT_SIDEBAR_PANEL_INDEX, WorkbenchLayout, WorkbenchSidebar};
use super::demo_controls::DemoControls;
use super::hs_mixer::{
    ThemePaletteHsOverride, clamp_palette_temperature_amount, clamp_palette_vividness_amount,
    derive_palette_hs_color_overrides,
};
use super::overrides::{
    StudioOverrides, ThemePaletteHslOverride, ThemeShadowOverride, clamp_palette_hue_deg,
    clamp_palette_lightness_multiplier, clamp_palette_saturation_multiplier, clamp_shadow_blur, clamp_shadow_offset_x,
    clamp_shadow_offset_y, clamp_shadow_opacity, clamp_shadow_spread, default_shadow_override,
    format_font_family_stack,
};
use super::theme_sidebar::{ThemeSidebar, palette_tokens};

pub struct LumaStudioApp {
    focus_scope: FocusHandle,
    pub(super) look: Arc<ShadcnLook>,
    pub(super) demos: DemoControls,
    pub(super) overrides: StudioOverrides,
    active_theme_id: String,
    theme_sidebar: Entity<ThemeSidebar>,
    content_pane: Entity<ContentPaneHost>,
    inspector_open: bool,
    workbench: WorkbenchLayout,
    sidebar_toggle: IconButton,
    reset_theme_button: IconButton,
    mode_toggle: Switch,
    sidebar_hidden: bool,
    /// Suppresses sidebar `Change` handlers while programmatically syncing sidebar values.
    syncing_sidebar_tokens: bool,
    _subscriptions: Vec<Subscription>,
}

impl LumaStudioApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>, launch_options: LumaStudioLaunchOptions) -> Self {
        let app = cx.entity();
        let focus_scope = cx.focus_handle();
        let active_theme_id = launch_options.theme_choice.id();
        let mode = launch_options.initial_mode;
        let look = Self::load_theme(&active_theme_id);
        look.set_mode(mode);
        sync_color_control_theme(look.as_ref());
        let demos = DemoControls::spawn(cx, look.clone(), shadcn::ShadcnSize::Md);

        let overrides = StudioOverrides::default();
        cx.set_global(LumaStudioAppHandle { entity: app.clone() });
        let theme_sidebar =
            cx.new(|cx| ThemeSidebar::new(app.clone(), look.clone(), active_theme_id.clone(), &overrides, cx));

        let board_snapshot = BoardSnapshot { demos: demos.clone(), look: look.clone(), overrides: overrides.clone() };
        let content_pane = cx.new(|cx| ContentPaneHost::new(app.clone(), board_snapshot, cx));
        let left_sidebar_entity = theme_sidebar.clone();
        let content_pane_entity = content_pane.clone();
        let workbench = WorkbenchLayout::new(
            "luma-studio",
            look.clone(),
            WorkbenchSidebar::new(move || left_sidebar_entity.clone().into_any_element())
                .width(px(360.0))
                .min(px(360.0))
                .max(px(460.0)),
            move || content_pane_entity.clone().into_any_element(),
            cx,
        );

        let sidebar_toggle = shadcn::Button::icon_button("luma-studio-sidebar-toggle", LucideIcon::PanelLeft)
            .look(look.as_ref())
            .content_only()
            .size(shadcn::ShadcnSize::Sm)
            .spawn(cx);
        let reset_theme_button = shadcn::Button::icon_button("luma-studio-reset-theme", LucideIcon::RefreshCcw)
            .look(look.as_ref())
            .content_only()
            .size(shadcn::ShadcnSize::Sm)
            .spawn(cx);
        let mut mode_toggle = shadcn::Switch::new("mode-toggle")
            .look(look.as_ref())
            .content_only()
            .with_data(matches!(mode, ThemeMode::Dark))
            .size(shadcn::ShadcnSize::Sm)
            .fixed_geometry(px(32.0), px(17.0), px(14.0));
        mode_toggle.set_presenter(Arc::new(|_, _| div().into_any_element()));
        let mode_toggle = mode_toggle.spawn(cx);
        for (button, icon) in [(&sidebar_toggle, LucideIcon::PanelLeft), (&reset_theme_button, LucideIcon::RefreshCcw)]
        {
            button.update(cx, |button, cx| {
                button.set_presenter(titlebar_icon_presenter(ControlIcon::Lucide(icon), app_bar_icon_color(&look)), cx);
            });
        }
        mode_toggle.update(cx, |switch, cx| {
            let icon_color = if mode == ThemeMode::Dark {
                gpui::hsla(0.0, 0.0, 1.0, 1.0)
            } else {
                gpui::hsla(0.0, 0.0, 0.0, 1.0)
            };
            switch.set_switch_thumb_content(mode_switch_thumb_content(icon_color), cx);
        });

        let mut subscriptions = Vec::new();
        ThemeSidebar::wire_subscriptions(&theme_sidebar, cx, &mut subscriptions);
        demos.subscribe(cx, &mut subscriptions);
        workbench.panels().clone().on_event(cx, |this, event: &ResizablePanelsEvent, cx| {
            this.handle_workbench_event(event, cx);
        });
        sidebar_toggle.clone().on_click(cx, |this, cx| {
            this.toggle_sidebar(cx);
        });
        reset_theme_button.clone().on_click(cx, |this, cx| {
            this.reload_active_theme(cx);
        });
        mode_toggle.clone().on_value_changed(cx, |this, on, cx| {
            this.set_mode(on, cx);
        });
        Self {
            focus_scope,
            look,
            demos,
            overrides,
            active_theme_id,
            theme_sidebar,
            content_pane,
            inspector_open: true,
            workbench,
            sidebar_toggle,
            reset_theme_button,
            mode_toggle,
            sidebar_hidden: false,
            syncing_sidebar_tokens: false,
            _subscriptions: subscriptions,
        }
    }

    pub(crate) fn set_inspector_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.inspector_open == open {
            return;
        }
        self.inspector_open = open;
        cx.notify();
    }

    pub(crate) fn inspector_open(&self) -> bool {
        self.inspector_open
    }

    fn sync_sidebar(&mut self, sync_tokens: bool, cx: &mut Context<Self>) {
        let theme = self.look.clone();
        let overrides = self.overrides.clone();
        let active_theme_id = self.active_theme_id.clone();

        self.syncing_sidebar_tokens = sync_tokens;
        self.theme_sidebar.update(cx, |sidebar, cx| {
            if sync_tokens {
                sidebar.apply_theme_snapshot(theme, &overrides, cx);
            } else {
                sidebar.sync_global_overrides(&overrides, cx);
            }
            sidebar.sync_theme_selector(active_theme_id, cx);
        });
        self.syncing_sidebar_tokens = false;
    }

    fn load_theme(theme_id: &str) -> Arc<ShadcnLook> {
        LumaStudioThemeChoice::from_id(theme_id).shadcn_look()
    }

    fn apply_theme_overrides(&mut self, cx: &mut Context<Self>) {
        let base = Self::load_theme(&self.active_theme_id);
        base.set_mode(self.look.mode());
        let (light_palette_color_overrides, dark_palette_color_overrides) =
            self.derived_palette_color_overrides(base.as_ref());
        self.look.replace_theme(base.as_ref());
        if let Err(err) =
            self.look.apply_mode_color_overrides(&light_palette_color_overrides, &dark_palette_color_overrides)
        {
            tracing::warn!("failed to apply studio color overrides: {err:?}");
        }
        if let Err(err) = self.overrides.token_overrides().and_then(|tokens| self.look.apply_token_overrides(&tokens)) {
            tracing::warn!("failed to apply studio token overrides: {err:?}");
        }
        sync_color_control_theme(self.look.as_ref());
        self.sync_titlebar_controls(cx);
        cx.bump_luma_theme_revision();
        self.sync_split_themes(cx);
        self.refresh_content_pane(cx);
    }

    fn derived_palette_color_overrides(
        &self,
        base: &ShadcnLook,
    ) -> (HashMap<String, ColorValue>, HashMap<String, ColorValue>) {
        let derive_for_mode = |mode: ThemeMode| {
            let palette_hsl = self.overrides.palette_hsl(mode).clone();
            let palette_hs = self.overrides.palette_hs(mode).clone();
            let mode_tokens = match mode {
                ThemeMode::Light => base.light_tokens(),
                ThemeMode::Dark => base.dark_tokens(),
            };
            let primary = self
                .overrides
                .global_color_override("--primary")
                .or_else(|| mode_tokens.catalog.source_color("primary").ok())
                .unwrap_or(ColorValue::srgb(0.5, 0.5, 0.5, 1.0));
            let hs_generated = derive_palette_hs_color_overrides(mode, primary, &palette_hs).unwrap_or_else(|err| {
                tracing::warn!("failed to derive studio palette: {err:?}");
                HashMap::new()
            });

            palette_tokens()
                .into_iter()
                .filter_map(|token| {
                    let css_name = token_css_name(token);
                    let source = self
                        .overrides
                        .global_color_override(&css_name)
                        .or_else(|| hs_generated.get(&css_name).copied())
                        .or_else(|| mode_tokens.catalog.source_color(token).ok())?;
                    match palette_hsl.apply(source) {
                        Ok(color) => Some((css_name, color)),
                        Err(err) => {
                            tracing::warn!("failed to adjust studio color {token}: {err:?}");
                            None
                        }
                    }
                })
                .collect::<HashMap<_, _>>()
        };

        (derive_for_mode(ThemeMode::Light), derive_for_mode(ThemeMode::Dark))
    }

    fn board_snapshot(&self) -> BoardSnapshot {
        BoardSnapshot { demos: self.demos.clone(), look: self.look.clone(), overrides: self.overrides.clone() }
    }

    fn refresh_content_pane(&self, cx: &mut Context<Self>) {
        let board = self.board_snapshot();
        self.content_pane.update(cx, |pane, cx| {
            pane.sync_board_snapshot(board, cx);
            pane.notify_tabs(cx);
        });
    }

    fn sync_split_themes(&self, cx: &mut Context<Self>) {
        let theme = self.look.resizable_panels_theme();
        self.workbench.sync_theme(&theme, cx);
    }

    pub fn change_theme(&mut self, theme_id: &str, cx: &mut Context<Self>) {
        if self.active_theme_id == theme_id {
            return;
        }
        self.active_theme_id = theme_id.to_string();
        self.overrides.clear_metric_overrides();
        self.overrides.clear_shadow_override();
        self.apply_theme_overrides(cx);
        self.sync_sidebar(true, cx);
        cx.notify();
    }

    pub fn set_global_color(&mut self, token: &str, color: ColorValue, cx: &mut Context<Self>) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let css_name = token_css_name(token);
        if self.overrides.global_color_override(&css_name) == Some(color) {
            return;
        }

        self.overrides.set_global_color(css_name, color);
        self.apply_theme_overrides(cx);
        self.syncing_sidebar_tokens = true;
        let overrides = self.overrides.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| sidebar.sync_global_overrides(&overrides, cx));
        self.syncing_sidebar_tokens = false;
        cx.notify();
    }

    pub fn set_palette_hue_deg(&mut self, hue_deg: f32, cx: &mut Context<Self>) {
        self.update_palette_hsl_override(|palette_hsl| palette_hsl.hue_deg = clamp_palette_hue_deg(hue_deg), cx);
    }

    pub fn set_palette_saturation_multiplier(&mut self, multiplier: f32, cx: &mut Context<Self>) {
        self.update_palette_hsl_override(
            |palette_hsl| palette_hsl.saturation_multiplier = clamp_palette_saturation_multiplier(multiplier),
            cx,
        );
    }

    pub fn set_palette_lightness_multiplier(&mut self, multiplier: f32, cx: &mut Context<Self>) {
        self.update_palette_hsl_override(
            |palette_hsl| palette_hsl.lightness_multiplier = clamp_palette_lightness_multiplier(multiplier),
            cx,
        );
    }

    pub fn set_palette_vividness_amount(&mut self, amount: f32, cx: &mut Context<Self>) {
        self.update_palette_hs_override(
            |palette_hs| {
                palette_hs.active = true;
                palette_hs.vividness_amount = clamp_palette_vividness_amount(amount);
            },
            cx,
        );
    }

    pub fn set_palette_temperature_amount(&mut self, amount: f32, cx: &mut Context<Self>) {
        self.update_palette_hs_override(
            |palette_hs| {
                palette_hs.active = true;
                palette_hs.temperature_amount = clamp_palette_temperature_amount(amount);
            },
            cx,
        );
    }

    pub fn set_radius_rem(&mut self, rem: f32, cx: &mut Context<Self>) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let rem = crate::studio::overrides::clamp_radius_rem(rem);
        if self.overrides.radius_rem() == Some(rem) {
            return;
        }

        self.overrides.set_radius_rem(rem);
        self.apply_theme_overrides(cx);
        self.syncing_sidebar_tokens = true;
        let overrides = self.overrides.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| sidebar.sync_global_overrides(&overrides, cx));
        self.syncing_sidebar_tokens = false;
        cx.notify();
    }

    pub fn set_spacing_rem(&mut self, rem: f32, cx: &mut Context<Self>) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let rem = crate::studio::overrides::clamp_spacing_rem(rem);
        if self.overrides.spacing_rem() == Some(rem) {
            return;
        }

        self.overrides.set_spacing_rem(rem);
        self.apply_theme_overrides(cx);
        self.syncing_sidebar_tokens = true;
        let overrides = self.overrides.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| sidebar.sync_global_overrides(&overrides, cx));
        self.syncing_sidebar_tokens = false;
        cx.notify();
    }

    pub fn set_font_sans(&mut self, family: &str, cx: &mut Context<Self>) {
        self.set_font_stack_override("font-sans", family, "sans-serif", cx);
    }

    pub fn set_font_serif(&mut self, family: &str, cx: &mut Context<Self>) {
        self.set_font_stack_override("font-serif", family, "serif", cx);
    }

    pub fn set_font_mono(&mut self, family: &str, cx: &mut Context<Self>) {
        self.set_font_stack_override("font-mono", family, "monospace", cx);
    }

    fn set_font_stack_override(&mut self, token: &str, family: &str, generic: &str, cx: &mut Context<Self>) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let stack = format_font_family_stack(family, generic);
        if self.overrides.font_stack(token) == Some(stack.as_str()) {
            return;
        }

        match token {
            "font-sans" => self.overrides.set_font_sans(stack),
            "font-serif" => self.overrides.set_font_serif(stack),
            "font-mono" => self.overrides.set_font_mono(stack),
            _ => return,
        }

        self.apply_theme_overrides(cx);
        self.syncing_sidebar_tokens = true;
        let look = self.look.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| sidebar.sync_typography_selectors(look, cx));
        self.syncing_sidebar_tokens = false;
        cx.notify();
    }

    pub fn reload_active_theme(&mut self, cx: &mut Context<Self>) {
        self.overrides.clear_all_overrides();
        self.apply_theme_overrides(cx);
        self.sync_sidebar(true, cx);
        cx.notify();
    }

    pub fn set_shadow_color(&mut self, color: ColorValue, cx: &mut Context<Self>) {
        self.update_shadow_override(|shadow| shadow.color = color, cx);
    }

    pub fn set_shadow_opacity(&mut self, opacity: f32, cx: &mut Context<Self>) {
        self.update_shadow_override(|shadow| shadow.set_opacity(clamp_shadow_opacity(opacity)), cx);
    }

    pub fn set_shadow_blur(&mut self, blur_px: f32, cx: &mut Context<Self>) {
        self.update_shadow_override(|shadow| shadow.blur_px = clamp_shadow_blur(blur_px), cx);
    }

    pub fn set_shadow_spread(&mut self, spread_px: f32, cx: &mut Context<Self>) {
        self.update_shadow_override(|shadow| shadow.spread_px = clamp_shadow_spread(spread_px), cx);
    }

    pub fn set_shadow_offset_x(&mut self, offset_x_px: f32, cx: &mut Context<Self>) {
        self.update_shadow_override(|shadow| shadow.offset_x_px = clamp_shadow_offset_x(offset_x_px), cx);
    }

    pub fn set_shadow_offset_y(&mut self, offset_y_px: f32, cx: &mut Context<Self>) {
        self.update_shadow_override(|shadow| shadow.offset_y_px = clamp_shadow_offset_y(offset_y_px), cx);
    }

    fn update_palette_hsl_override(
        &mut self,
        update: impl FnOnce(&mut ThemePaletteHslOverride),
        cx: &mut Context<Self>,
    ) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let mode = self.look.mode();
        let mut palette_hsl = self.overrides.palette_hsl(mode).clone();
        update(&mut palette_hsl);

        if self.overrides.palette_hsl(mode) == &palette_hsl {
            return;
        }

        self.overrides.set_palette_hsl_override(mode, palette_hsl);
        self.apply_theme_overrides(cx);
        self.syncing_sidebar_tokens = true;
        let overrides = self.overrides.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| sidebar.sync_global_overrides(&overrides, cx));
        self.syncing_sidebar_tokens = false;
        cx.notify();
    }

    fn update_palette_hs_override(&mut self, update: impl FnOnce(&mut ThemePaletteHsOverride), cx: &mut Context<Self>) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let mode = self.look.mode();
        let mut palette_hs = self.overrides.palette_hs(mode).clone();
        update(&mut palette_hs);

        if self.overrides.palette_hs(mode) == &palette_hs {
            return;
        }

        self.overrides.set_palette_hs_override(mode, palette_hs);
        self.apply_theme_overrides(cx);
        self.syncing_sidebar_tokens = true;
        let overrides = self.overrides.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| sidebar.sync_global_overrides(&overrides, cx));
        self.syncing_sidebar_tokens = false;
        cx.notify();
    }

    fn update_shadow_override(&mut self, update: impl FnOnce(&mut ThemeShadowOverride), cx: &mut Context<Self>) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let mut shadow =
            self.overrides.shadow_override().cloned().unwrap_or_else(|| default_shadow_override(&self.look));
        update(&mut shadow);

        if self.overrides.shadow_override() == Some(&shadow) {
            return;
        }

        self.overrides.set_shadow_override(shadow);
        self.apply_theme_overrides(cx);
        self.syncing_sidebar_tokens = true;
        let overrides = self.overrides.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| sidebar.sync_global_overrides(&overrides, cx));
        self.syncing_sidebar_tokens = false;
        cx.notify();
    }

    fn refresh_controls_layout(&self, cx: &mut Context<Self>) {
        let content_width = self.workbench.panels().read(cx).panel_sizes_px().get(CONTENT_PANEL_INDEX).copied().map(px);
        if let Some(content_width) = content_width {
            self.content_pane.update(cx, |pane, cx| pane.sync_controls_host_content_width(content_width, cx));
        } else {
            self.content_pane.update(cx, |pane, cx| pane.request_controls_layout_refresh(cx));
        }
    }

    fn handle_workbench_event(&mut self, event: &ResizablePanelsEvent, cx: &mut Context<Self>) {
        let mut dirty = false;

        if let ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden } = event
            && *panel_index == LEFT_SIDEBAR_PANEL_INDEX
        {
            self.sidebar_hidden = *hidden;
            self.sync_titlebar_controls(cx);
            dirty = true;
        }

        // Refresh only on settled events. SizesChanged also fires during live drag;
        // re-entering set_host_content_width there breaks the splitter grab.
        if matches!(event, ResizablePanelsEvent::PanelHiddenChanged { .. } | ResizablePanelsEvent::ResizeEnd { .. }) {
            self.refresh_controls_layout(cx);
            dirty = true;
        }

        // Avoid full-app re-renders on hover/size churn — those disrupt handle hit-testing.
        if dirty {
            cx.notify();
        }
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.workbench.panels().update(cx, |panels, cx| {
            panels.toggle_panel_hidden(LEFT_SIDEBAR_PANEL_INDEX, PanelHideMode::Completely, cx);
        });
    }

    pub(crate) fn toggle_mode(&mut self, cx: &mut Context<Self>) {
        let dark = match self.look.mode() {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
        self.set_mode(matches!(dark, ThemeMode::Dark), cx);
    }

    fn set_mode(&mut self, dark: bool, cx: &mut Context<Self>) {
        let mode = if dark { ThemeMode::Dark } else { ThemeMode::Light };
        if self.look.mode() == mode {
            return;
        }
        self.look.set_mode(mode);
        sync_color_control_theme(self.look.as_ref());
        self.sync_titlebar_controls(cx);
        cx.bump_luma_theme_revision();
        let theme = self.look.clone();
        let overrides = self.overrides.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| {
            sidebar.apply_theme_snapshot(theme, &overrides, cx);
        });
        self.sync_split_themes(cx);
        self.refresh_content_pane(cx);
        cx.notify();
    }

    fn sync_titlebar_controls(&self, cx: &mut Context<Self>) {
        let color = app_bar_icon_color(&self.look);
        let sidebar_icon = if self.sidebar_hidden {
            LucideIcon::PanelLeftOpen
        } else {
            LucideIcon::PanelLeft
        };
        for (button, icon) in [
            (&self.sidebar_toggle, ControlIcon::Lucide(sidebar_icon)),
            (&self.reset_theme_button, ControlIcon::Lucide(LucideIcon::RefreshCcw)),
        ] {
            button.update(cx, |button, cx| {
                button.set_presenter(titlebar_icon_presenter(icon.clone(), color), cx);
            });
        }
        self.mode_toggle.update(cx, |switch, cx| {
            switch.set_data(matches!(self.look.mode(), ThemeMode::Dark), cx);
            let icon_color = if self.look.mode() == ThemeMode::Dark {
                gpui::hsla(0.0, 0.0, 1.0, 1.0)
            } else {
                gpui::hsla(0.0, 0.0, 0.0, 1.0)
            };
            switch.set_switch_thumb_content(mode_switch_thumb_content(icon_color), cx);
        });
    }
}

fn mode_switch_thumb_content(
    color: gpui::Hsla,
) -> impl Fn(&ButtonContentContext<SwitchData>, &mut App) -> AnyElement + Send + Sync + 'static {
    move |model, _| {
        let icon = if model.data.checked {
            LucideIcon::Moon
        } else {
            LucideIcon::Sun
        };
        div()
            .size(px(16.0))
            .flex()
            .items_center()
            .justify_center()
            .child(xgpui_luma::infra::icon::lucide_icon(icon, color, 13.0))
            .into_any_element()
    }
}

fn titlebar_icon_presenter(icon: ControlIcon, color: gpui::Hsla) -> ControlPresenter<ButtonContentContext<()>> {
    Arc::new(move |_, _| match &icon {
        ControlIcon::Lucide(lucide) => {
            div().child(xgpui_luma::infra::icon::lucide_icon(*lucide, color, 14.0)).into_any_element()
        }
        ControlIcon::SvgPath(path) => {
            gpui::svg().size(px(14.0)).text_color(color).path(path.clone()).into_any_element()
        }
    })
}

fn app_bar_icon_color(look: &ShadcnLook) -> gpui::Hsla {
    look.resolve_content_only_button(ButtonFamilyRole::Icon, ControlSize::Sm, InteractionState::default())
        .foreground
}

use super::export::token_css_name;

impl Render for LumaStudioApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let sans = self.look.mode_tokens().typography.font.sans.family.clone();
        let title_style = self.look.typography_role(ShadcnTextRole::H4);
        let main_shell = self.workbench.render_body();

        let title_bar = TitleBar::new()
            .background_color(chrome.panel_background)
            .border_color(chrome.border)
            .text_color(chrome.title_text)
            .child(
                div()
                    .id("luma-studio-titlebar")
                    .h_full()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_2()
                    .text_color(chrome.title_text)
                    .font_family(sans.clone())
                    .child(div().typography_style(title_style).child("Luma Studio"))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(self.sidebar_toggle.clone())
                            .child(self.reset_theme_button.clone())
                            .child(self.mode_toggle.clone()),
                    ),
            );

        WorkbenchLayout::render_shell(
            &self.focus_scope,
            sans.into(),
            chrome.app_background,
            title_bar,
            main_shell,
            div()
                .id("luma-studio-bottom-app-bar")
                .h(px(32.0))
                .w_full()
                .flex_shrink_0()
                .flex()
                .items_center()
                .bg(chrome.panel_background)
                .border_t_1()
                .border_color(chrome.border),
        )
    }
}
