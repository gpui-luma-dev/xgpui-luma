use xgpui_luma::controls::split_view::SplitViewEventExt;
use gpui::{Context, Entity};
use xgpui_luma::controls::sidebar::{SidebarControl, SidebarPresentation};
use xgpui_luma::controls::split_view::{SplitView, SplitViewEvent};

pub fn wire_split_nav_sync<T: 'static>(
    split_view: Entity<SplitView>,
    sidebar: Entity<SidebarControl>,
    cx: &mut Context<T>,
) {
    split_view.clone().on_event(cx, {
        let sidebar = sidebar.downgrade();
        move |_, event: &SplitViewEvent, cx| {
            if let SplitViewEvent::CollapsedChanged { collapsed } = event {
                let Some(sidebar) = sidebar.upgrade() else {
                    return;
                };
                sidebar.update(cx, |sidebar, cx| {
                    sidebar.set_presentation(
                        if *collapsed {
                            SidebarPresentation::Icons
                        } else {
                            SidebarPresentation::Expanded
                        },
                        cx,
                    );
                });
            }
        }
    });
}
