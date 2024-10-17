use iced::widget::{self, button, column, container, row, scrollable, text, tooltip};
use iced::widget::{horizontal_space, pick_list, Column};
use iced::Length::Fill;
use iced::{Center, Element, Font, Task, Theme};
use once_cell::sync::Lazy;

// use peg;
use makefile_lossless::Makefile;
use std::fs;

use std::io;
use std::process::Command;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

pub fn main() -> iced::Result {
    iced::application("Editor - Iced", Editor::update, Editor::view)
        .theme(Editor::theme)
        .font(include_bytes!("../fonts/editor-icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .run_with(Editor::new)
}

struct Editor {
    theme: Theme,
    targets: Vec<String>,
    output: String,

    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    current_scroll_offset: scrollable::RelativeOffset,
    anchor: scrollable::Anchor,
}

#[derive(Debug, Clone)]
enum Message {
    LoadMakeTargets,
    Reload,
    TaskMake(String),
    ThemeSelected(Theme),

    ScrollToBeginning,
    ScrollToEnd,
    Scrolled(scrollable::Viewport),
}

impl Editor {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                theme: Theme::Dracula,
                targets: Vec::new(),
                output: String::new(),

                scrollbar_width: 15,
                scrollbar_margin: 0,
                scroller_width: 10,
                current_scroll_offset: scrollable::RelativeOffset::START,
                anchor: scrollable::Anchor::Start,
            },
            Task::batch([Task::done(Message::LoadMakeTargets), widget::focus_next()]),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeSelected(theme) => {
                self.theme = theme;

                Task::none()
            }
            Message::TaskMake(target) => {
                let output = Command::new("make")
                    .arg(target)
                    .output()
                    .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
                if output.status.success() {
                    // let s = String::from_utf8_lossy(&output.stdout).into_owned();

                    self.output = String::from_utf8_lossy(&output.stdout).into_owned();
                } else {
                    self.output = String::from_utf8_lossy(&output.stderr).into_owned();
                }

                Task::none()
            }
            Message::Reload => Task::done(Message::LoadMakeTargets),
            Message::LoadMakeTargets => {
                let f = fs::File::open("Makefile").unwrap();
                let result = Makefile::read_relaxed(f);
                if result.is_ok() {
                    let makefile: makefile_lossless::Makefile = result.unwrap();

                    self.targets.clear();
                    for rule in makefile.rules() {
                        if rule.to_string().contains(" :") {
                            println!("multi target rules unsupported for now");
                            println!("{}", rule);
                            continue;
                        }
                        rule.targets()
                            .filter(|target| !target.starts_with("_"))
                            .for_each(|target| self.targets.push(target));
                        // for target in rule.targets() {
                        //     println!("{}", target);
                        //     self.targets.push(target);
                        // }
                    }
                }

                // peg::parser!{
                //     grammar list_parser() for str {
                //         rule number() -> u32
                //         = "[" l:(number() ** ",") "]" {l}
                //     }
                // }

                Task::none()
            }

            Message::ScrollToBeginning => {
                self.current_scroll_offset = scrollable::RelativeOffset::START;

                scrollable::snap_to(SCROLLABLE_ID.clone(), self.current_scroll_offset)
            }
            Message::ScrollToEnd => {
                self.current_scroll_offset = scrollable::RelativeOffset::END;

                scrollable::snap_to(SCROLLABLE_ID.clone(), self.current_scroll_offset)
            }
            Message::Scrolled(viewport) => {
                self.current_scroll_offset = viewport.relative_offset();

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let controls = row![
            action(reload_icon(), "reload", Some(Message::Reload)),
            horizontal_space(),
            pick_list(Theme::ALL, Some(self.theme.clone()), Message::ThemeSelected)
                .text_size(14)
                .padding([5, 10])
        ]
        .spacing(10)
        .align_y(Center);

        let status = row![].spacing(10);
        let scroll_to_end_button = || {
            button("Scroll to end")
                .padding(10)
                .on_press(Message::ScrollToEnd)
        };

        let scroll_to_beginning_button = || {
            button("Scroll to beginning")
                .padding(10)
                .on_press(Message::ScrollToBeginning)
        };

        let mut targets = Vec::new();
        for target in self.targets.iter() {
            targets.push(target_card(
                action(
                    start_icon(),
                    target,
                    Some(Message::TaskMake(target.to_string())),
                ),
                target,
            ));
        }
        let s = self.output.as_str();
        let text_box: Column<Message> = column![text!("{s}").font(Font::MONOSPACE)];
        let scrollable_content: Element<Message> = Element::from(
            scrollable(
                column![
                    scroll_to_end_button(),
                    text("Beginning!"),
                    // vertical_space().height(1200),
                    // text("Middle!"),
                    text_box,
                    // vertical_space().height(1200),
                    text("End!"),
                    scroll_to_beginning_button(),
                ]
                .align_x(Center)
                .padding([40, 0])
                .spacing(40),
            )
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(self.scrollbar_width)
                    .margin(self.scrollbar_margin)
                    .scroller_width(self.scroller_width)
                    .anchor(self.anchor),
            ))
            .width(Fill)
            .height(Fill)
            .id(SCROLLABLE_ID.clone())
            .on_scroll(Message::Scrolled),
        );

        column![
            controls,
            Column::from_vec(targets),
            scrollable_content,
            // text_box,
            status,
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

fn target_card<'a, Message: Clone + 'a>(
    action: Element<'a, Message>,
    label: &'a str,
) -> Element<'a, Message> {
    container(row![action, label].spacing(10))
        .style(container::bordered_box)
        .padding(10)
        .into()
}

fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(container(content).center_x(30));

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

fn reload_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

fn start_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e802}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}
