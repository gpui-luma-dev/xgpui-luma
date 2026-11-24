use xgpui_luma::theme::InteractionLayer;

pub fn interaction_layer_key(layer: InteractionLayer) -> &'static str {
    match layer {
        InteractionLayer::Default => "default",
        InteractionLayer::Hovered => "hover",
        InteractionLayer::Pressed => "pressed",
        InteractionLayer::Disabled => "disabled",
    }
}
