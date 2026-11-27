//! The button face: what the button template paints, shared by every control whose trigger is a button
//! (the selector, the search selector, the popup menu and [`TriggerButton`](crate::controls::trigger_button)),
//! so the fill, border, shadow, focus ring and `:disabled { opacity }` fade are drawn in one place (#430).

use gpui::{AnyElement, BoxShadow, Div, ElementId, Hsla, SharedString, Stateful, div, prelude::*, px};

use crate::controls::button_family::button_family_effective_border;
use crate::infra::focus_adorner::FocusAdorner;
use crate::theme::LumaTextStyle;

/// A resolved face, in one interaction state.
#[derive(Clone, Debug)]
pub(crate) struct FaceLook {
    pub background: Hsla,
    pub foreground: Hsla,
    /// `None` is borderless.
    pub border: Option<Hsla>,
    pub border_width: f32,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    /// Empty keeps the inherited font.
    pub font_family: SharedString,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub height: f32,
    pub focus: FocusAdorner,
    pub focus_border: Hsla,
    /// The resolved `opacity` (`:disabled { opacity }`); 1 draws without a fade.
    pub opacity: f32,
}

impl FaceLook {
    /// The face a resolved root part describes (#433): the one reader every button-like trigger draws from, so
    /// border width and focus geometry are read in the face's own state.
    pub(crate) fn from_part(root: xgpui_luma_style::PartView<'_>) -> Self {
        use crate::style::PartRead;
        Self {
            background: root.color("background"),
            foreground: root.color("foreground"),
            border: root.color_opt("border"),
            border_width: root.number("border_width"),
            shadow: root.box_shadow_opt(),
            typography: root.typography(14.0, 20.0),
            font_family: root.font_family_name(),
            padding_x: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            gap: root.number("gap"),
            height: root.number("height"),
            focus: crate::controls::choice::styled::focus_adorner(root),
            focus_border: root.color("focus_border"),
            opacity: root.opacity(),
        }
    }
}

/// How the face box is sized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum FaceShape {
    /// Padded on both axes, with the face radius.
    Padded,
    /// A square `height` wide with no padding; a radius of half the height or more is a circle.
    Square,
    /// A circle `height` wide.
    Round,
    /// No padding, clipped to the radius: the content draws its own sub-faces (a split button).
    Bare,
}

/// One render of a face: its identity, shape and state.
pub(crate) struct Face {
    /// The root's id; the box is `(id, 0)` and its adorner frame `(id, 1)`.
    pub id: SharedString,
    pub shape: FaceShape,
    /// The corner radius of the box and its focus ring.
    pub radius: f32,
    /// Stretch to the parent's width. Content fills the box and a trailing element sits at the end.
    pub full_width: bool,
    /// Grow taller than `height` when the content needs it.
    pub min_height: bool,
    /// Show the focus ring.
    pub focused: bool,
    pub disabled: bool,
    /// Paint the look's shadow (never while disabled).
    pub elevation: bool,
    /// Dense: the focus ring paints inside the face and reserves no room around it.
    pub compact: bool,
}

impl Face {
    /// The face box: fill, text style, border and geometry around `content`, with `trailing` (a chevron)
    /// at the end. A caller adds its handlers or extra paint to the box before passing it to [`Face::root`].
    pub(crate) fn control(&self, look: &FaceLook, content: AnyElement, trailing: Option<AnyElement>) -> Stateful<Div> {
        let mut control = div()
            .id(ElementId::from((self.id.clone(), 0usize)))
            .relative()
            .flex()
            .items_center()
            .gap(px(look.gap))
            .bg(look.background)
            .text_color(look.foreground)
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .font_weight(look.typography.weight)
            .border(px(look.border_width))
            .border_color(button_family_effective_border(look.border));
        if !look.font_family.is_empty() {
            control = control.font_family(look.font_family.clone());
        }
        control = if self.min_height {
            control.min_h(px(look.height))
        } else {
            control.h(px(look.height))
        };
        control = match self.shape {
            FaceShape::Padded => control.px(px(look.padding_x)).py(px(look.padding_y)).rounded(px(self.radius)),
            FaceShape::Square => {
                let control = control.w(px(look.height)).flex_shrink_0().p_0();
                if self.radius >= look.height / 2.0 {
                    control.rounded_full()
                } else {
                    control.rounded(px(self.radius))
                }
            }
            FaceShape::Round => control.w(px(look.height)).flex_shrink_0().p_0().rounded_full(),
            FaceShape::Bare => control.p_0().overflow_hidden().rounded(px(self.radius)),
        };
        if self.full_width {
            control = control.w_full();
        }
        let fills = self.full_width || trailing.is_some();
        control = if fills {
            control.justify_between()
        } else {
            control.justify_center()
        };
        let content = div().text_color(look.foreground).child(content);
        let content = if fills {
            content.flex().items_center().gap(px(look.gap)).flex_1().min_w(px(0.0))
        } else if self.shape == FaceShape::Bare {
            content.flex().size_full()
        } else {
            content
        };
        control = control.child(content);
        if let Some(trailing) = trailing {
            control = control.child(div().flex().items_center().justify_center().flex_shrink_0().child(trailing));
        }
        #[cfg(any(test, feature = "test-support"))]
        {
            let id = self.id.clone();
            control = control.debug_selector(move || format!("{id}-face"));
        }
        control
    }

    /// The face's root around a `control` from [`Face::control`]: its shadow, the focus ring and the fade.
    pub(crate) fn root(&self, mut control: Stateful<Div>, look: &FaceLook) -> Stateful<Div> {
        let shadow = look.shadow.clone().filter(|layers| !layers.is_empty());
        if let Some(shadow) = shadow
            && crate::infra::shadow_layout::should_paint_shadow(self.elevation, self.disabled, true)
        {
            control = control.shadow(shadow);
        }
        let extent = if self.compact { 0.0 } else { look.focus.extent() };
        let mut adorned = div().id(ElementId::from((self.id.clone(), 1usize))).relative().child(control);
        let mut root = div().id(self.id.clone()).relative();
        if self.full_width {
            adorned = adorned.w_full();
            root = root.w_full();
        }
        root = if extent > 0.0 {
            root.p(px(extent)).child(adorned)
        } else {
            root.child(adorned)
        };
        root = root.child(look.focus.overlay(look.focus_border, self.focused, self.radius, self.compact));
        root = crate::style::fade(root, look.opacity);
        if !self.disabled {
            root = root.cursor_pointer();
        }
        root
    }
}
