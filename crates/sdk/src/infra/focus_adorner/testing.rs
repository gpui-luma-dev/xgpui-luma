use super::*;
use gpui::{AnyElement, App, Context, IntoElement, Render, TestAppContext, Window};
use crate::theme::InteractionState;

pub fn assert_stable_layout<F>(render: F)
where
    F: Fn(InteractionState, bool, &mut Window, &mut App) -> AnyElement + 'static,
{
    struct Harness<F> {
        render: F,
        bits: u8,
    }
    impl<F> Render for Harness<F>
    where
        F: Fn(InteractionState, bool, &mut Window, &mut App) -> AnyElement + 'static,
    {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let state = InteractionState {
                focused: self.bits & 1 != 0,
                hovered: self.bits & 2 != 0,
                pressed: self.bits & 4 != 0,
                disabled: self.bits & 8 != 0,
                invalid: self.bits & 16 != 0,
            };
            div()
                .flex()
                .flex_col()
                .items_start()
                .child(div().flex_none().debug_selector(|| "focus-target".into()).child((self.render)(
                    state,
                    self.bits & 32 != 0,
                    window,
                    cx,
                )))
                .child(div().h(px(10.)).w(px(10.)).flex_none().debug_selector(|| "following-row".into()))
        }
    }
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, _| Harness { render, bits: 0 });
    for scale_factor in [1.0, 1.5, 2.0] {
        cx.simulate_scale_factor_change(scale_factor);
        cx.run_until_parked();
        let target = cx.debug_bounds("focus-target").expect("target rendered");
        let following = cx.debug_bounds("following-row").expect("following row rendered");
        // End at the original state too: visible -> transparent must also be stable.
        for bits in (1..64).chain([0]) {
            view.update(cx, |view, cx| {
                view.bits = bits;
                cx.notify();
            });
            cx.run_until_parked();
            assert_eq!(cx.debug_bounds("focus-target").unwrap(), target, "control changed at state {bits}");
            assert_eq!(cx.debug_bounds("following-row").unwrap(), following, "sibling moved at state {bits}");
        }
    }
}

pub fn assert_textfield_layout(theme: std::sync::Arc<dyn crate::controls::textfield::TextFieldTheme>) {
    use crate::controls::textfield::*;
    use crate::theme::{ControlSize, StandardBoxScale};
    let template = ThemedTextFieldTemplate::new(theme.clone());
    assert_stable_layout(move |state, _, window, cx| {
        let id = "input".into();
        let empty = "".into();
        let enabled = !state.disabled;
        let state = TextFieldState {
            hovered: state.hovered,
            focused: state.focused,
            focus_visible: state.focused,
            invalid: state.invalid,
            ..Default::default()
        };
        let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), window.scale_factor());
        let model = TextFieldRenderModel {
            id: &id,
            placeholder: &empty,
            value: &empty,
            prefix_icon: None,
            size: ControlSize::Md,
            enabled,
            full_width: false,
            state,
            marked_range: None,
            caret_visible: false,
            horizontal_scroll: 0.,
            character_offsets: vec![0.],
            look: theme.resolve_look(state, enabled, ControlSize::Md, &scale),
        };
        let handlers = TextFieldTemplateHandlers {
            hover: Box::new(|_, _, _| {}),
            mouse_down: Box::new(|_, _, _| {}),
            mouse_move: Box::new(|_, _, _| {}),
            mouse_up: Box::new(|_, _, _| {}),
            mouse_up_out: Box::new(|_, _, _| {}),
            click: Box::new(|_, _, _| {}),
            key_down: Box::new(|_, _, _| {}),
            drag_move: Box::new(|_, _, _| {}),
        };
        template.render(&model, handlers, window, cx).into_any_element()
    });
}

pub fn assert_textarea_layout(theme: std::sync::Arc<dyn crate::controls::textarea::TextAreaTheme>) {
    use crate::controls::textarea::*;
    use crate::theme::ControlSize;
    let template = ThemedTextAreaTemplate::new(theme);
    assert_stable_layout(move |state, _, window, cx| {
        let id = "area".into();
        let empty = "".into();
        let model = TextAreaRenderModel {
            id: &id,
            placeholder: &empty,
            value: &empty,
            enabled: !state.disabled,
            full_width: false,
            size: ControlSize::Md,
            rows: 3,
            state: TextAreaState {
                hovered: state.hovered,
                focused: state.focused,
                focus_visible: state.focused,
                invalid: state.invalid,
                ..Default::default()
            },
            caret_visible: false,
            vertical_scroll: 0.,
            line_metrics: Vec::new(),
        };
        let handlers = TextAreaTemplateHandlers {
            hover: Box::new(|_, _, _| {}),
            mouse_down: Box::new(|_, _, _| {}),
            mouse_move: Box::new(|_, _, _| {}),
            mouse_up: Box::new(|_, _, _| {}),
            mouse_up_out: Box::new(|_, _, _| {}),
            click: Box::new(|_, _, _| {}),
            key_down: Box::new(|_, _, _| {}),
            drag_move: Box::new(|_, _, _| {}),
        };
        template.render(&model, handlers, window, cx).into_any_element()
    });
}

pub fn assert_selector_layout(theme: std::sync::Arc<dyn crate::controls::selector::SelectorTheme>) {
    use crate::controls::selector::*;
    use crate::theme::ControlSize;
    for trigger_style in [SelectorTriggerStyle::Outline, SelectorTriggerStyle::Ghost] {
        let template = ThemedSelectorTemplate::new(
            theme.clone(),
            crate::controls::selector_list::default_selector_items_template(),
        );
        assert_stable_layout(move |state, selected, window, cx| {
            let id = "selector".into();
            let label = "Choose".into();
            let icons = SelectorIcons::default();
            let model = SelectorRenderModel::<SelectorItem> {
                id: &id,
                label: &label,
                selected_index: None,
                items: &[],
                open: false,
                presence: Default::default(),
                popup_scroll: None,
                trigger_bounds: None,
                placement: SelectorPlacement::Smart,
                opening_mode: Default::default(),
                popup_geometry: None,
                initialize_popup_scroll: false,
                active_path: None,
                enabled: !state.disabled,
                size: ControlSize::Md,
                trigger_style,
                icons: &icons,
                without_elevation: true,
                item_template: None,
                panel_template: None,
                focus: Default::default(),
                state,
                visual_state: SelectorVisualState { interaction: state, selected, invalid: state.invalid, open: false },
            };
            template.render(&model, Default::default(), window, cx).into_any_element()
        });
    }
}

pub fn assert_popup_layout(theme: std::sync::Arc<dyn crate::controls::popup_menu::PopupMenuTheme>) {
    use crate::controls::popup_menu::*;
    use crate::theme::ControlSize;
    use std::sync::Arc;
    for trigger_style in [PopupMenuTriggerStyle::Outline, PopupMenuTriggerStyle::Ghost] {
        for split in [false, true] {
            let template = ThemedPopupMenuTemplate::new(theme.clone());
            assert_stable_layout(move |state, _, window, cx| {
                let id = "popup".into();
                let label = "Menu".into();
                let icons = crate::infra::icon::DisclosureIcons::default();
                let model = PopupMenuRenderModel {
                    id: &id,
                    label: &label,
                    content: Arc::new(|_, _| div().child("Menu").into_any_element()),
                    items: &[],
                    open: false,
                    disclosure_progress: 0.,
                    presence: Default::default(),
                    submenu_transition: None,
                    trigger_bounds: None,
                    placement: Default::default(),
                    trigger_style,
                    trigger_size: ControlSize::Md,
                    menu_size: ControlSize::Md,
                    icon_only: false,
                    icon: None,
                    end_icon: None,
                    disclosure_icons: &icons,
                    full_width: false,
                    without_elevation: true,
                    split,
                    split_state: Default::default(),
                    trigger_radius_override: None,
                    open_submenu: None,
                    active_path: None,
                    highlight: None,
                    enabled: !state.disabled,
                    focus: Default::default(),
                    state,
                };
                let handlers = PopupMenuTemplateHandlers {
                    split_faces: None,
                    trigger_bounds: Box::new(|_, _, _| {}),
                    action_click: Box::new(|_, _, _| {}),
                    trigger_click: Box::new(|_, _, _| {}),
                    trigger_hover: Box::new(|_, _, _| {}),
                    trigger_mouse_down: Box::new(|_, _, _| {}),
                    trigger_mouse_up: Box::new(|_, _, _| {}),
                    trigger_mouse_up_out: Box::new(|_, _, _| {}),
                    root_mouse_down_out: Box::new(|_, _, _| {}),
                    item_hovers: Vec::new(),
                    submenu_hovers: Vec::new(),
                    item_clicks: Vec::new(),
                };
                template.render(&model, handlers, window, cx).into_any_element()
            });
        }
    }
}
