use gpui::Hsla;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonFamilyRole {
    #[default]
    Text,
    Icon,
    Toggle {
        selected: bool,
    },
}

/// Normalizes absent face-border paint to transparent.
///
/// Face-border paint never determines focus geometry or focus color.
pub fn button_family_effective_border(border: Option<Hsla>) -> Hsla {
    border.unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.0, 0.0))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn effective_border_is_transparent_when_absent() {
        let effective = button_family_effective_border(None);
        assert_eq!(effective.a, 0.0);
    }
}
