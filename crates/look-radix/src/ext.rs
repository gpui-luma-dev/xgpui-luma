//! Look-local factory helpers. Apps use these instead of editing the SDK.

use std::sync::Arc;

use gpui::SharedString;
use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::checkbox::CheckboxData;
use xgpui_luma::controls::overlay_window::{self, OverlayWindowBuilder, ThemedOverlayWindowTemplate};
use xgpui_luma::controls::popup_menu::PopupMenuTemplate;
use xgpui_luma::controls::radio_button::RadioButtonData;
use xgpui_luma::controls::slider::SliderTemplate;
use xgpui_luma::controls::switch::SwitchData;
use xgpui_luma::controls::tabs::TabsTemplate;
use xgpui_luma::controls::textarea::TextAreaTemplate;
use xgpui_luma::controls::textfield::TextFieldTemplate;

use crate::button::{Paint, ButtonVariant};
use crate::button_layout::Radius;
use crate::checkbox::{CheckboxSize, CheckboxVariant};
use crate::look::Look;
use crate::overlay_window::overlay_window_theme;
use crate::popup_menu::PopupMenuVariant;
use crate::radio::{RadioSize, RadioVariant};
use crate::slider::SliderVariant;
use crate::switch::{SwitchSize, SwitchVariant};
use crate::tabs::TabsVariant;
use crate::textfield::TextFieldVariant;
use crate::tone::Tone;

/// Low-level template factories and the look-bound overlay-window builder.
///
/// Interactive controls spawn through look-owned builders
/// ([`crate::Button`], [`crate::Checkbox`], [`crate::Radio`], [`crate::Switch`],
/// [`crate::TextField`], [`crate::TextArea`], [`crate::Slider`], [`crate::Tabs`],
/// [`crate::Toggle`], [`crate::PopupMenu`], [`crate::Toolbar`], [`crate::TreeView`]).
/// Use this trait for direct SDK composition or forced-state previews.
/// See the crate-level customization guide for choosing the appropriate extension point.
pub trait LookControlExt {
    /// Bare button template for previews that force state / paint axes.
    fn button_template(&self, variant: ButtonVariant, paint: Paint) -> Arc<dyn ButtonTemplate<()>>;
    fn textfield_template(&self, variant: TextFieldVariant) -> Arc<dyn TextFieldTemplate>;
    fn textarea_template(&self, variant: TextFieldVariant) -> Arc<dyn TextAreaTemplate>;
    fn slider_template(&self, variant: SliderVariant) -> Arc<dyn SliderTemplate>;
    fn progress_template(
        &self,
        variant: crate::ProgressVariant,
        paint: Paint,
        radius: Radius,
    ) -> Arc<dyn xgpui_luma::controls::progress::ProgressTemplate>;
    /// Bare template for previews that render forced states instead of live controls.
    fn checkbox_template(&self, variant: CheckboxVariant) -> Arc<dyn ButtonTemplate<CheckboxData>>;
    fn checkbox_template_with(&self, variant: CheckboxVariant, paint: Paint) -> Arc<dyn ButtonTemplate<CheckboxData>>;
    fn checkbox_template_for(
        &self,
        variant: CheckboxVariant,
        paint: Paint,
        size: CheckboxSize,
        radius: Radius,
    ) -> Arc<dyn ButtonTemplate<CheckboxData>>;
    fn radio_template(&self, variant: RadioVariant) -> Arc<dyn ButtonTemplate<RadioButtonData>>;
    fn radio_template_with(&self, variant: RadioVariant, paint: Paint) -> Arc<dyn ButtonTemplate<RadioButtonData>>;
    fn radio_template_for(
        &self,
        variant: RadioVariant,
        paint: Paint,
        size: RadioSize,
    ) -> Arc<dyn ButtonTemplate<RadioButtonData>>;
    fn switch_template(&self, variant: SwitchVariant) -> Arc<dyn ButtonTemplate<SwitchData>>;
    fn switch_template_with(&self, variant: SwitchVariant, paint: Paint) -> Arc<dyn ButtonTemplate<SwitchData>>;
    fn switch_template_for(
        &self,
        variant: SwitchVariant,
        paint: Paint,
        size: SwitchSize,
        radius: Radius,
    ) -> Arc<dyn ButtonTemplate<SwitchData>>;
    fn tabs_template(&self) -> Arc<dyn TabsTemplate>;
    fn tabs_template_for(&self, variant: TabsVariant) -> Arc<dyn TabsTemplate>;
    /// Bare template for previews that render forced states instead of live controls.
    fn popup_menu_template(&self, variant: PopupMenuVariant, tone: Tone) -> Arc<dyn PopupMenuTemplate>;
    fn context_menu_template(
        &self,
        variant: crate::ContextMenuVariant,
        tone: Tone,
    ) -> Arc<dyn xgpui_luma::controls::context_menu::ContextMenuTemplate>;
    fn overlay_window(&self, id: impl Into<SharedString>) -> OverlayWindowBuilder;
}

impl LookControlExt for Look {
    fn button_template(&self, variant: ButtonVariant, paint: Paint) -> Arc<dyn ButtonTemplate<()>> {
        crate::button::button_template(self, variant, paint)
    }

    fn textfield_template(&self, variant: TextFieldVariant) -> Arc<dyn TextFieldTemplate> {
        crate::textfield::textfield_template(self, variant)
    }

    fn textarea_template(&self, variant: TextFieldVariant) -> Arc<dyn TextAreaTemplate> {
        crate::textarea::textarea_template(self, variant)
    }

    fn slider_template(&self, variant: SliderVariant) -> Arc<dyn SliderTemplate> {
        crate::slider::slider_template(self, variant)
    }

    fn progress_template(
        &self,
        variant: crate::ProgressVariant,
        paint: Paint,
        radius: Radius,
    ) -> Arc<dyn xgpui_luma::controls::progress::ProgressTemplate> {
        crate::progress_template(self, variant, paint, radius)
    }

    fn checkbox_template(&self, variant: CheckboxVariant) -> Arc<dyn ButtonTemplate<CheckboxData>> {
        self.checkbox_template_with(variant, Paint::accent())
    }

    fn checkbox_template_with(&self, variant: CheckboxVariant, paint: Paint) -> Arc<dyn ButtonTemplate<CheckboxData>> {
        crate::checkbox::checkbox_template(self, variant, paint)
    }

    fn checkbox_template_for(
        &self,
        variant: CheckboxVariant,
        paint: Paint,
        size: CheckboxSize,
        radius: Radius,
    ) -> Arc<dyn ButtonTemplate<CheckboxData>> {
        crate::checkbox::checkbox_template_for(self, variant, paint, size, radius)
    }

    fn radio_template(&self, variant: RadioVariant) -> Arc<dyn ButtonTemplate<RadioButtonData>> {
        self.radio_template_with(variant, Paint::accent())
    }

    fn radio_template_with(&self, variant: RadioVariant, paint: Paint) -> Arc<dyn ButtonTemplate<RadioButtonData>> {
        crate::radio::radio_template(self, variant, paint)
    }

    fn radio_template_for(
        &self,
        variant: RadioVariant,
        paint: Paint,
        size: RadioSize,
    ) -> Arc<dyn ButtonTemplate<RadioButtonData>> {
        crate::radio::radio_template_for(self, variant, paint, size)
    }

    fn switch_template(&self, variant: SwitchVariant) -> Arc<dyn ButtonTemplate<SwitchData>> {
        self.switch_template_with(variant, Paint::accent())
    }

    fn switch_template_with(&self, variant: SwitchVariant, paint: Paint) -> Arc<dyn ButtonTemplate<SwitchData>> {
        crate::switch::switch_template(self, variant, paint)
    }

    fn switch_template_for(
        &self,
        variant: SwitchVariant,
        paint: Paint,
        size: SwitchSize,
        radius: Radius,
    ) -> Arc<dyn ButtonTemplate<SwitchData>> {
        crate::switch::switch_template_for(self, variant, paint, size, radius)
    }

    fn tabs_template(&self) -> Arc<dyn TabsTemplate> {
        self.tabs_template_for(TabsVariant::Line)
    }

    fn tabs_template_for(&self, variant: TabsVariant) -> Arc<dyn TabsTemplate> {
        crate::tabs::tabs_template_for(self, variant)
    }

    fn popup_menu_template(&self, variant: PopupMenuVariant, tone: Tone) -> Arc<dyn PopupMenuTemplate> {
        crate::popup_menu::popup_menu_template(self, variant, tone)
    }

    fn context_menu_template(
        &self,
        variant: crate::ContextMenuVariant,
        tone: Tone,
    ) -> Arc<dyn xgpui_luma::controls::context_menu::ContextMenuTemplate> {
        crate::context_menu_template(self, variant, tone)
    }

    fn overlay_window(&self, id: impl Into<SharedString>) -> OverlayWindowBuilder {
        overlay_window::overlay_window(id)
            .template(Arc::new(ThemedOverlayWindowTemplate::new(overlay_window_theme(self))))
    }
}
