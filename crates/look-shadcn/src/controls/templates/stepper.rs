use std::sync::Arc;

use xgpui_luma::controls::stepper::{StepperTheme, ThemedStepperTemplate};

use crate::controls::stepper::stepper_look;
use crate::look::ShadcnLook;

struct ShadcnStepperTheme {
    theme: ShadcnLook,
}

impl StepperTheme for ShadcnStepperTheme {
    fn resolve(
        &self,
        enabled: bool,
        size: xgpui_luma::theme::ControlSize,
    ) -> xgpui_luma::controls::stepper::StepperLook {
        let tokens = self.theme.mode_tokens();
        stepper_look(tokens.as_ref(), enabled, size)
    }
}

pub fn stepper_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::stepper::StepperTemplate> {
    Arc::new(ThemedStepperTemplate::new(stepper_theme(theme)))
}

pub fn stepper_theme(theme: ShadcnLook) -> Arc<dyn StepperTheme> {
    Arc::new(ShadcnStepperTheme { theme: theme.clone() })
}
