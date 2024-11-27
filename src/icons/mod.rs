use iced::{widget::text, Element, Font};

pub fn reload_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

pub fn start_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

pub fn up_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e802}')
}

pub fn down_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e803}')
}

pub fn fast_forward_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f103}')
}

pub fn stop_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e804}')
}

pub fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}
