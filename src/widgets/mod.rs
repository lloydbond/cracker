use iced::{
    widget::{button, container, row, tooltip},
    Element,
};

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
