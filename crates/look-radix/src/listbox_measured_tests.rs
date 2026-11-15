type TestLook = crate::Look;
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
    let _ = padding; // Radix retains its own surface insets.
    super::ListBoxBuilder::new(id, control, Box::new(move |model, cx| template(model, cx).into_any_element()))
        .look(look)
        .flow(flow)
        .build(window, cx)
}
include!("../../sdk/tests/support/listbox_measured.rs");
