use std::sync::Arc;

use xgpui_luma::controls::stepper::{StepperTheme, ThemedStepperTemplate};

use crate::controls::stepper::stepper_style;
use crate::look::ShadcnLook;

struct ShadcnStepperTheme {
    theme: ShadcnLook,
}

impl StepperTheme for ShadcnStepperTheme {
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        stepper_style(self.theme.mode_tokens().as_ref(), self.theme.size())
    }
}

pub fn stepper_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::stepper::StepperTemplate> {
    Arc::new(ThemedStepperTemplate::new(stepper_theme(theme)))
}

pub fn stepper_theme(theme: ShadcnLook) -> Arc<dyn StepperTheme> {
    Arc::new(ShadcnStepperTheme { theme: theme.clone() })
}
