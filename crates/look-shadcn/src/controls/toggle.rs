/// Inline padded toggle (shadcn default). Square geometry requires explicit `.round(true)` on the builder.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToggleLayout {
    #[default]
    Inline,
    Square,
}

impl ToggleLayout {
    pub fn from_round(round: bool) -> Self {
        if round { Self::Square } else { Self::Inline }
    }
}
