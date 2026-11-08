use std::sync::Arc;

use xgpui_luma::controls::textarea::{TextAreaTheme, ThemedTextAreaTemplate};
use xgpui_luma::theme::{ControlSize, StandardBoxScale};

use crate::look::ShadcnLook;

struct ShadcnTextAreaTheme {
    theme: ShadcnLook,
}

impl TextAreaTheme for ShadcnTextAreaTheme {
    fn resolve(
        &self,
        state: xgpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
    ) -> xgpui_luma::controls::textarea::TextAreaPalette {
        let tokens = self.theme.mode_tokens();
        crate::controls::textarea::textarea_palette(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Outline,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        state: xgpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> xgpui_luma::controls::textarea::TextAreaLook {
        let tokens = self.theme.mode_tokens();
        crate::controls::textarea::textarea_look(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Outline,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn textarea_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::textarea::TextAreaTemplate> {
    Arc::new(ThemedTextAreaTemplate::new(textarea_theme(theme.clone())))
}

pub fn textarea_theme(theme: ShadcnLook) -> Arc<dyn TextAreaTheme> {
    Arc::new(ShadcnTextAreaTheme { theme: theme.clone() })
}

struct ShadcnSurfaceTextAreaTheme {
    theme: ShadcnLook,
}

impl TextAreaTheme for ShadcnSurfaceTextAreaTheme {
    fn resolve(
        &self,
        state: xgpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
    ) -> xgpui_luma::controls::textarea::TextAreaPalette {
        let tokens = self.theme.mode_tokens();
        crate::controls::textarea::textarea_palette(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Surface,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        state: xgpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> xgpui_luma::controls::textarea::TextAreaLook {
        let tokens = self.theme.mode_tokens();
        crate::controls::textarea::textarea_look(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Surface,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn surface_textarea_theme(theme: ShadcnLook) -> Arc<dyn TextAreaTheme> {
    Arc::new(ShadcnSurfaceTextAreaTheme { theme: theme.clone() })
}

struct ShadcnPrimaryTextAreaTheme {
    theme: ShadcnLook,
}

struct ShadcnInputTextAreaTheme {
    theme: ShadcnLook,
}

impl TextAreaTheme for ShadcnInputTextAreaTheme {
    fn resolve(
        &self,
        state: xgpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
    ) -> xgpui_luma::controls::textarea::TextAreaPalette {
        let tokens = self.theme.mode_tokens();
        crate::controls::textarea::textarea_palette(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Input,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        state: xgpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> xgpui_luma::controls::textarea::TextAreaLook {
        let tokens = self.theme.mode_tokens();
        crate::controls::textarea::textarea_look(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Input,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn input_textarea_theme(theme: ShadcnLook) -> Arc<dyn TextAreaTheme> {
    Arc::new(ShadcnInputTextAreaTheme { theme: theme.clone() })
}

pub fn input_textarea_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::textarea::TextAreaTemplate> {
    Arc::new(ThemedTextAreaTemplate::new(input_textarea_theme(theme.clone())))
}

impl TextAreaTheme for ShadcnPrimaryTextAreaTheme {
    fn resolve(
        &self,
        state: xgpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
    ) -> xgpui_luma::controls::textarea::TextAreaPalette {
        let tokens = self.theme.mode_tokens();
        crate::controls::textarea::textarea_palette(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Primary,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        state: xgpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> xgpui_luma::controls::textarea::TextAreaLook {
        let tokens = self.theme.mode_tokens();
        crate::controls::textarea::textarea_look(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Primary,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn primary_textarea_theme(theme: ShadcnLook) -> Arc<dyn TextAreaTheme> {
    Arc::new(ShadcnPrimaryTextAreaTheme { theme: theme.clone() })
}

pub fn primary_textarea_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::textarea::TextAreaTemplate> {
    Arc::new(ThemedTextAreaTemplate::new(primary_textarea_theme(theme.clone())))
}
