pub mod stdoutput;

use iced::{
    widget::{button, container, row, tooltip},
    Element, Theme,
};

use crate::{icons, Message};

pub fn target_card<'a, Message: Clone + 'a>(
    action: Element<'a, Message>,
    label: &'a str,
    other_action: Element<'a, Message>,
) -> Element<'a, Message> {
    container(row![action, other_action, label].spacing(1))
        .style(container::bordered_box)
        .padding(1)
        .into()
}

pub fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(container(content).center_x(10));

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(container::transparent)
        .into()
    } else {
        action.style(button::secondary).into()
    }
}

pub fn target_list<'a>(
    targets: &'a [String],
    list: &mut Vec<
        iced_core::Element<
            'a,
            Message,
            Theme,
            iced_renderer::fallback::Renderer<iced_wgpu::Renderer, iced_tiny_skia::Renderer>,
        >,
    >,
) {
    for (id, target) in targets.iter().enumerate() {
        list.push(target_card(
            action(
                icons::start_icon(),
                target,
                Some(Message::TaskMake(id, target.clone())),
            ),
            target,
            action(icons::stop_icon(), "stop", Some(Message::TaskStop(id))),
        ));
    }
}
