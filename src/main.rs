#[macro_use]
extern crate log;

mod task_runners;
mod utils;

use iced::alignment::Horizontal::Left;
use iced::widget::{self, button, column, container, row, scrollable, text, tooltip};
use iced::widget::{horizontal_space, pick_list, Column};
use iced::Alignment::Center;
use iced::Length::Fill;
use iced::{Element, Font, Subscription, Task, Theme};
use once_cell::sync::Lazy;
use task_runners::makefile::*;
use utils::{async_read_lines, Error};

use std::fmt::Debug;
use std::sync::Arc;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

pub fn main() -> iced::Result {
    sensible_env_logger::init!();
    debug!("start ck");
    iced::application("Editor - Iced", Editor::update, Editor::view)
        .subscription(Editor::subscription)
        .theme(Editor::theme)
        .font(include_bytes!("../fonts/editor-icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .run_with(Editor::new)
}

#[derive(Debug)]
struct Editor {
    theme: Theme,
    targets: Vec<String>,
    tasks: Vec<StdOutput>,
    next_id: usize,

    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    current_scroll_offset: scrollable::RelativeOffset,
    anchor: scrollable::Anchor,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadMakeTargetsPEG,
    ParseMakeTargets(std::result::Result<Arc<String>, Error>),
    Reload,
    TaskMake(String),
    TaskStart(usize),
    TaskUpdate((usize, Result<worker::Stdout, worker::Error>)),
    TaskStop,
    ThemeSelected(Theme),

    ScrollToBeginning,
    ScrollToEnd,
    Scrolled(scrollable::Viewport),
}

impl Editor {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                theme: Theme::CatppuccinMocha,
                targets: Vec::new(),
                tasks: Vec::new(),
                next_id: 0,

                scrollbar_width: 15,
                scrollbar_margin: 0,
                scroller_width: 10,
                current_scroll_offset: scrollable::RelativeOffset::START,
                anchor: scrollable::Anchor::Start,
            },
            Task::batch([
                Task::done(Message::LoadMakeTargetsPEG),
                widget::focus_next(),
            ]),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeSelected(theme) => {
                self.theme = theme;

                Task::none()
            }
            Message::TaskMake(target) => {
                let id = self.next_id;
                self.next_id += 1;
                for task in self.tasks.iter_mut() {
                    task.stop();
                    debug!("task ({:?}) stopped", task.target());
                }
                self.tasks.push(StdOutput::new(id, target));

                Task::done(Message::TaskStart(id))
            }
            Message::TaskStart(id) => {
                if let Some(task) = self.tasks.get_mut(id) {
                    task.start();
                    debug!("task ({:?}) started", task.target());
                }

                Task::none()
            }
            Message::TaskUpdate((id, output)) => {
                if let Some(task) = self.tasks.get_mut(id) {
                    task.stream_update(output);
                }
                Task::none()
            }
            Message::TaskStop => Task::none(),
            Message::Reload => Task::done(Message::LoadMakeTargetsPEG),
            Message::LoadMakeTargetsPEG => {
                self.targets.clear();
                Task::perform(async_read_lines("Makefile"), Message::ParseMakeTargets)
            }
            Message::ParseMakeTargets(result) => {
                if let Ok(contents) = result {
                    for line in contents.lines() {
                        let target = parser::grammar::Targets(line);
                        if let Ok(t) = target {
                            self.targets.extend(t);
                        }
                    }
                    debug!("Found targets: {:?}", self.targets);
                }
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
    fn subscription(&self) -> Subscription<Message> {
        if self.tasks.is_empty() {
            return Subscription::none();
        }
        Subscription::batch(self.tasks.iter().map(StdOutput::subscription))
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
        let scroll_to_end_button =
            || action(down_icon(), "Scroll to end", Some(Message::ScrollToEnd));

        let scroll_to_beginning_button = || {
            action(
                up_icon(),
                "Scroll to beginning",
                Some(Message::ScrollToBeginning),
            )
        };

        let controls_output = row![
            horizontal_space(),
            scroll_to_end_button(),
            scroll_to_beginning_button(),
        ]
        .spacing(10)
        .padding(10);

        let status = row![].spacing(10);
        let mut targets = Vec::new();
        for target in self.targets.iter() {
            targets.push(target_card(
                action(
                    start_icon(),
                    target,
                    Some(Message::TaskMake(target.clone())),
                ),
                target,
            ));
        }
        let text_box: Column<Message> =
            Column::with_children(self.tasks.iter().map(StdOutput::view));
        let scrollable_stdout: Element<Message> = Element::from(
            scrollable(
                column![text_box,]
                    .align_x(Center)
                    .padding([40, 40])
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

        let scrollable_targets: Element<Message> = Element::from(
            scrollable(
                Column::from_vec(targets)
                    // column![text_box,]
                    .align_x(Left)
                    .padding([10, 0])
                    .spacing(10),
            )
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(0)
                    .margin(0)
                    .scroller_width(0)
                    .anchor(self.anchor),
            ))
            // .width(Fill)
            .height(Fill)
            .id(SCROLLABLE_ID.clone())
            .on_scroll(Message::Scrolled),
        );

        let row_of_scrollables = row![scrollable_targets, scrollable_stdout,];

        column![
            controls,
            controls_output,
            row_of_scrollables,
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

fn target_card<'a, Message: Clone + 'a>(
    action: Element<'a, Message>,
    label: &'a str,
) -> Element<'a, Message> {
    container(row![action, label].spacing(1))
        .style(container::bordered_box)
        .padding(1)
        .into()
}

fn action<'a, Message: Clone + 'a>(
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

fn reload_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

fn start_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e802}')
}

fn up_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e803}')
}

fn down_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}

// StdOutput
#[derive(Debug)]
struct StdOutput {
    id: usize,
    target: String,
    state: State,
    output: String,
    // sender: mpsc::Sender<Input>,
}

#[derive(Debug, Clone)]
enum State {
    Idle,
    Streaming { stream: String },
    Finished,
    Errored,
}

impl StdOutput {
    pub fn new(id: usize, target: String) -> Self {
        Self {
            id,
            target,
            state: State::Idle,
            output: String::new(),
        }
    }
    pub fn target(&self) -> String {
        self.target.clone()
    }

    pub fn start(&mut self) {
        match self.state {
            State::Idle { .. } | State::Finished { .. } | State::Errored { .. } => {
                self.state = State::Streaming {
                    stream: String::from("Stream started..."),
                };
            }
            State::Streaming { .. } => {}
        }
    }

    pub fn stop(&mut self) {
        self.state = State::Finished;
    }
    pub fn stream_update(&mut self, output_update: Result<worker::Stdout, worker::Error>) {
        if let State::Streaming { stream } = &mut self.state {
            match output_update {
                Ok(worker::Stdout::OutputUpdate { output }) => {
                    self.output.push_str(output.as_str());
                    // self.output.push('\n');
                    *stream = output
                }
                Ok(worker::Stdout::Finished) => {
                    self.state = State::Finished;
                }
                Ok(worker::Stdout::Prepare { output }) => *stream = output,
                Err(worker::Error::NoContent) => {
                    self.state = State::Errored;
                }
                Err(worker::Error::Failed(_)) => {
                    self.state = State::Errored;
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Streaming { .. } => {
                debug!("{:?} subscribed", self.target.clone());
                worker::subscription(self.id, self.target.clone()).map(Message::TaskUpdate)
            }
            _ => Subscription::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let _ = match &self.state {
            State::Idle { .. } => String::from("Press start..."),

            State::Streaming { stream } => (*stream).clone(),
            State::Finished { .. } => String::from("Finished..."),
            State::Errored { .. } => String::from("Errored..."),
        };

        let text_box: Column<Message> = column![text!("{}", self.output).font(Font::MONOSPACE)];

        text_box.into()
    }
}
