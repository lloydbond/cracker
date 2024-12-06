use iced::widget::{text, Column};
use iced::{Element, Font, Subscription};
use tokio::time::Instant;

use crate::{
    stdout::worker::{self, StdCommand},
    task_runners::makefile,
    Message,
};

// StdOutput
#[derive(Debug)]
pub struct StdOutput {
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

    pub fn id(&self) -> usize {
        self.id
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
