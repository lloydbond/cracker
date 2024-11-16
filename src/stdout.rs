use iced::futures::{SinkExt, Stream, StreamExt};
use iced::stream::try_channel;
use iced::Subscription;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use std::hash::Hash;
use std::process::Stdio;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Stdout {
    // Ready { sender: mpsc::Sender<Input> },
    Prepare { output: String },
    OutputUpdate { output: String },
    Finished,
    // ...
}

#[derive(Debug, Clone)]
pub enum Input {
    Cancel,
    // ...
}

#[derive(Debug, Clone)]
pub enum Error {
    Failed(Arc<std::io::Error>),
    NoContent,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Failed(Arc::new(error))
    }
}

pub fn subscription<I: 'static + Hash + Copy + Send + Sync>(
    id: I,
    target: String,
) -> Subscription<(I, Result<Stdout, Error>)> {
    Subscription::run_with_id(
        id,
        some_worker(target.clone()).map(move |output| (id, output)),
    )
}

pub fn some_worker(target: String) -> impl Stream<Item = Result<Stdout, Error>> {
    try_channel(1, |mut output| async move {
        let _ = output
            .send(Stdout::OutputUpdate {
                output: String::from(""),
            })
            .await;

        let mut cmd = Command::new("make");

        // Specify that we want the command's standard output piped back to us.
        // By default, standard input/output/error will be inherited from the
        // current process (for example, this means that standard input will
        // come from the keyboard and standard output/error will go directly to
        // the terminal if this process is invoked from the command line).
        cmd.stdout(Stdio::piped());
        cmd.stdin(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .arg(target.as_str())
            .spawn()
            .expect("failed to spawn command");

        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");
        let mut reader = BufReader::new(stdout).lines();
        while let result = reader.next_line().await {
            use iced::futures::StreamExt;
            match result {
                Ok(line) => match line {
                    Some(l) => {
                        let _ = output.send(Stdout::OutputUpdate { output: l }).await;
                    }
                    None => {
                        break;
                    }
                },
                Err(_) => {
                    println!("file stream error:");
                }
            }
        }
        let _ = output.send(Stdout::Finished).await;

        Ok(())
    })
}
