use std::sync::Arc;

use xgpui_luma_color::style::{ColorControlTheme, set_active_color_control_theme};
use xgpui_luma::theme::{LumaThemeSyncExt, ThemeMode};
use xgpui_luma_look_shadcn::ShadcnLook;
use luma_app_common::{built_in_look, built_in_theme, built_in_themes, fallback_look};

fn available_theme_names() -> Vec<String> {
    built_in_themes().iter().map(|theme| theme.id.to_string()).collect()
}

/// Startup theme: `default` or a built-in CSS theme ID (e.g. `retro-arcade`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellThemeChoice {
    Default,
    Named(String),
}

impl ShellThemeChoice {
    pub fn from_args() -> Self {
        match std::env::args().nth(1).as_deref() {
            None => Self::Default,
            Some(arg) => Self::parse(arg).unwrap_or_else(|usage| {
                eprintln!("{usage}");
                std::process::exit(1);
            }),
        }
    }

    fn parse(arg: &str) -> Result<Self, String> {
        let stem = arg.trim().to_ascii_lowercase();
        if stem == "default" {
            return Ok(Self::Default);
        }

        if built_in_theme(&stem).is_some() {
            return Ok(Self::Named(stem));
        }

        let available = available_theme_names();
        let hint = if available.is_empty() {
            "no embedded built-in themes found".to_string()
        } else {
            format!("available: {}", available.join(", "))
        };

        Err(format!("unknown shell theme {arg:?} ({hint})\n{}", Self::usage_line()))
    }

    fn usage_line() -> String {
        let program = std::env::args().next().unwrap_or_else(|| "luma-shell".into());
        let mut options = vec!["default".to_string()];
        options.extend(available_theme_names());
        format!("usage: {program} [{}]", options.join("|"))
    }

    pub fn shadcn_look(self) -> Arc<ShadcnLook> {
        match self {
            Self::Default => Arc::new(fallback_look()),
            Self::Named(stem) => {
                Arc::new(built_in_look(&stem).unwrap_or_else(|err| panic!("parse built-in shadcn theme {stem}: {err}")))
            }
        }
    }
}

pub fn sync_color_control_theme(look: &ShadcnLook) {
    let chrome = look.chrome();

    set_active_color_control_theme(ColorControlTheme::new(
        chrome.border,
        chrome.panel_background,
        matches!(look.mode(), ThemeMode::Dark),
    ));
}

pub fn toggle_shell_theme<C: LumaThemeSyncExt>(look: &ShadcnLook, cx: &mut C) {
    let mode = match look.mode() {
        ThemeMode::Light => ThemeMode::Dark,
        ThemeMode::Dark => ThemeMode::Light,
    };
    look.set_mode(mode);
    sync_color_control_theme(look);
    cx.bump_luma_theme_revision();
}
