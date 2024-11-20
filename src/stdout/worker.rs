use iced::futures::{SinkExt, Stream, StreamExt};

use iced::stream::try_channel;
use iced::Subscription;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time;

use std::hash::Hash;
use std::process::Stdio;
use std::sync::Arc;


#[derive(Debug, Clone)]
pub struct StdCommand {
    command: String,
    target: String,
}

impl StdCommand {
    pub fn new(target: String, command: String) -> Self {
        Self {
            command: command.clone(),
            target: target.clone(),
        }
    }

    pub fn target(&self) -> String {
        self.target.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stdout {
    Prepare { output: Vec<String> },
    OutputUpdate { output: Vec<String> },
    Finished,
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
    command: StdCommand,
) -> Subscription<(I, Result<Stdout, Error>)> {
    Subscription::run_with_id(
        id,
        some_worker(command.clone()).map(move |output| (id, output)),
    )
}

pub fn some_worker(command: StdCommand) -> impl Stream<Item = Result<Stdout, Error>> {
    try_channel(1, |mut output| async move {
        let _ = output
            .send(Stdout::OutputUpdate {
                output: vec!["".to_string()],
            })
            .await;
        debug!("initialize worker: {:?}", command.target.clone());
        let mut cmd = Command::new(command.command.as_str());

        // Specify that we want the command's standard output piped back to us.
        // By default, standard input/output/error will be inherited from the
        // current process (for example, this means that standard input will
        // come from the keyboard and standard output/error will go directly to
        // the terminal if this process is invoked from the command line).
        cmd.stdout(Stdio::piped());
        cmd.stdin(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .arg(command.target.as_str())
            .spawn()
            .expect("failed to spawn command");

        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");
        let mut reader = BufReader::new(stdout).lines();
        let mut cache: Vec<String> = Vec::new();
        let interval = time::interval(time::Duration::from_millis(80));
        tokio::pin!(interval);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if cache.is_empty() {
                        continue;
                    }
                    let _ = output.send( Stdout::OutputUpdate {output: cache.clone() }).await;
                    cache.clear();
                }
                maybe_result = reader.next_line() => {
                    match maybe_result {
                    Ok(Some(line)) => {
                        cache.push(line);
                    }
                    _ => break,
                    }
                }
            }
        }
        if !cache.is_empty() {
            let _ = output
                .send(Stdout::OutputUpdate {
                    output: cache.clone(),
                })
                .await;
            cache.clear();
        }
        let _ = output.send(Stdout::Finished).await;
        debug!("leaving worker");
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: make this testable if possible
}
