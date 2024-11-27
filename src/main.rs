extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod args;
mod icons;
mod stdout;
mod task_runners;
mod utils;
mod widgets;

use crate::widgets::action;
use crate::widgets::target_card;
use args::parse_args;
use iced::alignment::Horizontal::Left;
use iced::widget::{self, column, horizontal_space, pick_list, row, scrollable, text, Column};
use iced::Alignment::Center;
use iced::Length::Fill;
use iced::{Element, Font, Subscription, Task, Theme};
use icons::{down_icon, fast_forward_icon, reload_icon, start_icon, stop_icon, up_icon};
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

    let result = parse_args(&env::args().collect_vec());

    let filename = match result {
        Ok(f) => {
            debug!("file returned: {}", f);
            f
        }
        Err(args::Error::CliExit) => {
            debug!("exit by cli");
            return Ok(());
        }
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
    task_history: Vec<StdOutput>,

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
                task_history: Vec::new(),

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
                fn trim_task_history(tasks: &mut Vec<StdOutput>) {
                    if tasks.len() >= 100 {
                        let r = tasks.len() - 100;
                        tasks.drain(..r);
                    }
                }
                if let Some(task) = self.task_history.last_mut() {
                    if task.id == id {
                        task.stop();
                    }
                };
                let mut task = StdOutput::new(id, target);
                task.start();
                self.task_history.push(task);
                trim_task_history(&mut self.task_history);

                Task::none()
            }
            Message::TaskStop(id) => {
                debug!("stop id: {id:?}");

                if let Some(task) = self.task_history.last_mut() {
                    if task.id == id {
                        task.stop();
                    }
                }

                debug!("task stop??");
                Task::none()
            }
            Message::TaskUpdate((id, output)) => {
                let mut next_task = Task::none();
                if let Some(task) = self.task_history.last_mut() {
                    if task.id == id {
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
        if self.task_history.is_empty() {
            return Subscription::none();
        }
        Subscription::batch(self.task_history.iter().map(StdOutput::subscription))
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
                    Some(Message::TaskMake(id, target.clone())),
                ),
                target,
                action(stop_icon(), "stop", Some(Message::TaskStop(id))),
            ));
        }
        let text_box: Column<Message> =
            Column::with_children(self.task_history.iter().map(StdOutput::view));
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
