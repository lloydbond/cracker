use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::futures::Stream;
use iced::stream::try_channel;
use iced::Subscription;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use std::hash::Hash;
use std::process::Stdio;

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
    Failed,
    NoContent,
}

// impl From<std::io::Error> for Error {
//     fn from(error: std::io::Error) -> Self {
//         Error::Failed(Arc::new(error))
//     }
// }

pub fn subscription<I: 'static + Hash + Copy + Send + Sync>(
    id: I,
    target: String,
) -> Subscription<(I, Result<Stdout, Error>)> {
    println!("stdout::subscription");
    Subscription::run_with_id(id, some_worker(target)).map(move |output| (id, output))
}

pub fn some_worker(target: String) -> impl Stream<Item = Result<Stdout, Error>> {
    println!("try channel {:?}", target.clone());
    try_channel(1, |mut output| async move {
        // Create channel
        // let (sender, mut receiver) = mpsc::channel(1);

        // Send the sender back to the application
        // output.send(Stdout::Ready { sender }).await;
        println!("in try channel {:?}", target.clone());
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
        println!("stdout created");
        let mut reader = BufReader::new(stdout).lines();
        loop {
            use iced::futures::StreamExt;
            println!("in loop");
            let result = reader.next_line().await;
            match result {
                Ok(line) => match line {
                    Some(l) => {
                        println!("Line: {}", l.clone());
                        let _ = output.send(Stdout::OutputUpdate { output: l }).await;
                    }
                    None => {
                        println!("file output finished!");
                        break;
                    }
                },
                Err(e) => {
                    println!("file stream error: {:?}", e);
                    // output.send(Error::Failed(Arc::new(format!(
                    //     "file stream error: {:?}",
                    //     e
                    // ))));
                }
            }
            // Read next input sent from `Application`
            // let _ = receiver.select_next_some().await;

            // match input {
            //     Input::Cancel => {
            //         // Do some cleanup work...
            //         break;
            //     }
            // }
        }
        let _ = output.send(Stdout::Finished).await;

        Ok(())
    })
}
