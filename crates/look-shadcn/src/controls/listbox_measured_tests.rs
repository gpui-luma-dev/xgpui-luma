type TestLook = crate::ShadcnLook;
#[allow(clippy::too_many_arguments)]
fn render_test_list<E: IntoElement>(
    look: &TestLook,
    control: &mut ListBoxControl<Harness, u32, u32>,
    id: &'static str,
    flow: ListBoxFlow,
    padding: (f32, f32),
    template: impl Fn(&ListBoxItemRenderModel<'_, u32>, &mut App) -> E,
    window: &mut Window,
    cx: &mut Context<Harness>,
) -> Stateful<Div> {
    look.render_listbox(control, id, flow, padding, template, window, cx)
}
include!("../../../sdk/tests/support/listbox_measured.rs");
