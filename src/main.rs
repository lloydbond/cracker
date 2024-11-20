extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod args;
mod stdout;
mod task_runners;
mod utils;

use args::parse_args;
use iced::alignment::Horizontal::Left;
use iced::widget::{
    self, button, column, container, horizontal_space, pick_list, row, scrollable, text, tooltip,
    Column,
};
use iced::Alignment::Center;
use iced::Length::Fill;
use iced::{Element, Font, Subscription, Task, Theme};
use itertools::Itertools;
use once_cell::sync::Lazy;
use std::env;
use stdout::worker::{self, StdCommand};
use task_runners::makefile::{self, parser};
use tokio::time::Instant;
use utils::{async_read_lines, Error};

use std::fmt::Debug;
use std::sync::Arc;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

pub fn main() -> iced::Result {
    pretty_env_logger::init();
    debug!("start ck");
    let filename: String = if let Ok(f) = parse_args(&env::args().collect_vec()) {
        f
    } else {
        return Ok(());
    };

    iced::application("Editor - Iced", Editor::update, Editor::view)
        .subscription(Editor::subscription)
        .theme(Editor::theme)
        .font(include_bytes!("../fonts/editor-icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .run_with(move || Editor::new(filename))
}

#[derive(Debug)]
struct Editor {
    filename: String,
    theme: Theme,
    targets: Vec<String>,
    tasks: Vec<StdOutput>,

    auto_scroll: bool,
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
    TaskMake(usize, String),
    TaskStart(usize, String),
    TaskUpdate((usize, Result<worker::Stdout, worker::Error>)),
    TaskStop(usize),
    ThemeSelected(Theme),

    ScrollToBeginning,
    ScrollToEnd,
    ScrollAutoToggle,
    Scrolled(scrollable::Viewport),
}

impl Editor {
    fn new(filename: String) -> (Self, Task<Message>) {
        (
            Self {
                filename,
                theme: Theme::CatppuccinMocha,
                targets: Vec::new(),
                tasks: Vec::new(),

                auto_scroll: true,
                scrollbar_width: 15,
                scrollbar_margin: 0,
                scroller_width: 10,
                current_scroll_offset: scrollable::RelativeOffset::END,
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
            Message::TaskMake(id, target) => {
                for task in self.tasks.iter_mut() {
                    task.stop();
                }
                let mut task = StdOutput::new(id, target);
                task.start();
                self.tasks.push(task);

                Task::none()
            }
            Message::TaskStart(id, target) => {
                let mut msg_next = Task::none();
                if let Some(task_id) = self.tasks.iter().position(|t| t.id == id) {
                    self.tasks[task_id].start();
                } else {
                    msg_next = Task::done(Message::TaskMake(id, target));
                }

                msg_next
            }
            Message::TaskStop(id) => {
                debug!("stop id: {id:?}");

                if let Some(task_id) = self.tasks.iter().position(|t| t.id == id) {
                    if let Some(task) = self.tasks.get_mut(task_id) {
                        task.stop();
                    }
                }
                debug!("task stop??");
                Task::none()
            }
            Message::TaskUpdate((id, output)) => {
                let mut next_task = Task::none();
                if let Some(task_id) = self.tasks.iter().position(|t| t.id == id) {
                    if let Some(task) = self.tasks.get_mut(task_id) {
                        task.stream_update(output);
                    }
                    if self.auto_scroll {
                        next_task = Task::done(Message::ScrollToEnd);
                    }
                }

                next_task
            }
            Message::ScrollAutoToggle => {
                self.auto_scroll = !self.auto_scroll;
                let offset = scrollable::AbsoluteOffset { x: 0.00, y: 59.0 };
                scrollable::scroll_by(SCROLLABLE_ID.clone(), offset)
            }
            Message::Reload => Task::done(Message::LoadMakeTargetsPEG),
            Message::LoadMakeTargetsPEG => {
                self.targets.clear();
                let filename = self.filename.clone();
                Task::perform(async_read_lines(filename), Message::ParseMakeTargets)
            }
            Message::ParseMakeTargets(result) => {
                if let Ok(contents) = result {
                    for line in contents.lines() {
                        let target = parser::Targets(line);
                        if let Ok(t) = target {
                            self.targets.extend(t);
                        }
                    }
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
        let scroll_auto_on_off_button = || {
            action(
                fast_forward_icon(),
                "auto scroll",
                Some(Message::ScrollAutoToggle),
            )
        };
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
            scroll_auto_on_off_button(),
            scroll_to_beginning_button(),
        ]
        .spacing(10)
        .padding(10);

        let status = row![].spacing(10);
        let mut targets = Vec::new();
        for (id, target) in self.targets.iter().enumerate() {
            targets.push(target_card(
                action(
                    start_icon(),
                    target,
                    Some(Message::TaskStart(id, target.clone())),
                ),
                target,
                action(stop_icon(), "stop", Some(Message::TaskStop(id))),
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
    other_action: Element<'a, Message>,
) -> Element<'a, Message> {
    container(row![action, other_action, label].spacing(1))
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
    icon('\u{0e801}')
}

fn up_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e802}')
}

fn down_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e803}')
}

fn fast_forward_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f103}')
}

fn stop_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e804}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}

// StdOutput
#[derive(Debug)]
struct StdOutput {
    id: usize,
    command: StdCommand,
    state: State,
    textbox_output: Vec<String>,
    tick: Instant,
    ms_200: core::time::Duration,
}

#[derive(Debug, Clone)]
enum State {
    Idle,
    Streaming { stream: Vec<String> },
    Finished,
    Errored,
}

impl StdOutput {
    pub fn new(id: usize, target: String) -> Self {
        let tick = Instant::now();
        Self {
            id,
            command: makefile::new(target),
            state: State::Idle,
            textbox_output: Vec::new(),
            tick,
            ms_200: core::time::Duration::from_millis(200),
        }
    }
    pub fn target(&self) -> String {
        self.command.target()
    }

    pub fn start(&mut self) {
        info!("start task {:?}", self.target());
        match self.state {
            State::Idle { .. } | State::Finished { .. } | State::Errored { .. } => {
                self.state = State::Streaming {
                    stream: vec!["Stream started...".to_string()],
                };
            }
            State::Streaming { .. } => {}
        }
    }

    pub fn stop(&mut self) {
        info!("stopping task {:?}", self.target());
        self.state = State::Finished;
        let end_stream = vec!["".to_string(), "stream ended...".to_string()];
        self.textbox_output.extend(end_stream);
        // self.textbox_output.clear();
    }
    pub fn stream_update(&mut self, output_update: Result<worker::Stdout, worker::Error>) {
        if let State::Streaming { .. } = &mut self.state {
            match output_update {
                Ok(worker::Stdout::OutputUpdate { output }) => {
                    self.textbox_output.extend(output);
                    // *stream = output
                }
                Ok(worker::Stdout::Finished) => {
                    self.state = State::Finished;
                }
                Ok(worker::Stdout::Prepare { output }) => {
                    self.textbox_output.extend(output);
                }

                Err(worker::Error::NoContent) => {
                    self.state = State::Errored;
                }
                Err(worker::Error::Failed(_)) => {
                    self.state = State::Errored;
                }
            }
        }
        if self.tick.elapsed() >= self.ms_200 && self.textbox_output.len() > 1_000_000 {
            let r = self.textbox_output.len() - 1_000_000;
            self.textbox_output.drain(..r);
            self.tick = Instant::now();
        }
    }
    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Streaming { .. } => {
                worker::subscription(self.id, self.command.clone()).map(Message::TaskUpdate)
            }
            _ => Subscription::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        fn get_window(len: usize, width: usize) -> usize {
            if len > width {
                return len - width;
            }
            0
        }
        let idx = get_window(
            self.textbox_output.len(),
            match self.state {
                State::Finished => 1_000,
                _ => 100,
            },
        );
        Column::with_children(
            self.textbox_output[idx..]
                .iter()
                .map(|o| text!("{}", o).font(Font::MONOSPACE))
                .map(Element::from),
        )
        .into()
    }
}
